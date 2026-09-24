use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client as S3Client, config::Builder as S3ConfigBuilder, primitives::ByteStream};
use chrono::Utc;
use futures_util::StreamExt;
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ConfirmSelectOptions,
        QueueDeclareOptions,
    },
    types::{AMQPValue, FieldTable, LongString, ShortString},
};
use serde::{Deserialize, Serialize};
use tokio::{fs, fs::File, io::AsyncWriteExt, process::Command, time::sleep};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

const JOB_QUEUE: &str = "video.uploaded.v1";
const RETRY_QUEUE: &str = "video.uploaded.retry";
const MAX_DELIVERIES: u64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UploadEvent {
    event_id: Uuid,
    video_id: Uuid,
    source_object_key: String,
    content_type: String,
    size_bytes: i64,
    schema_version: u8,
    occurred_at: chrono::DateTime<Utc>,
}

#[derive(Serialize)]
struct ProcessingResult {
    result: &'static str,
    hls_prefix: Option<String>,
    failure_code: Option<String>,
}

#[derive(Debug)]
enum WorkError {
    InvalidMedia,
    Transient(String),
}

#[derive(Deserialize)]
struct ProbeOutput {
    streams: Vec<ProbeStream>,
}

#[derive(Deserialize)]
struct ProbeStream {
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    let rabbit_url = env::var("RABBITMQ_URL")?;
    let s3_endpoint = env::var("S3_ENDPOINT")?;
    let region_name = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".into());
    let shared = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(region_name))
        .load()
        .await;
    let s3_config = S3ConfigBuilder::from(&shared)
        .endpoint_url(s3_endpoint)
        .force_path_style(true)
        .build();
    let s3 = S3Client::from_conf(s3_config);
    let bucket = env::var("S3_BUCKET").unwrap_or_else(|_| "opentube".into());
    let service_url = env::var("VIDEO_SERVICE_URL")?;
    let worker_token = env::var("WORKER_TOKEN")?;
    let (connection, channel) = connect(&rabbit_url).await?;
    declare_queues(&channel, JOB_QUEUE, RETRY_QUEUE).await?;
    let mut consumer = channel
        .basic_consume(
            JOB_QUEUE,
            "opentube-transcoder",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    tracing::info!("transcoding worker consuming jobs");
    while let Some(delivery) = consumer.next().await {
        let delivery = match delivery {
            Ok(d) => d,
            Err(error) => {
                tracing::warn!(%error, "consumer delivery failed");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        let event: UploadEvent = match serde_json::from_slice(&delivery.data) {
            Ok(event) => event,
            Err(error) => {
                tracing::warn!(%error, "discarding malformed job");
                delivery.ack(BasicAckOptions::default()).await?;
                continue;
            }
        };
        let attempt = retry_count(&delivery.properties);
        match process_video(&s3, &bucket, &event).await {
            Ok(prefix) => {
                if report_result(
                    &service_url,
                    &worker_token,
                    event.video_id,
                    &event.event_id.to_string(),
                    ProcessingResult {
                        result: "ready",
                        hls_prefix: Some(prefix),
                        failure_code: None,
                    },
                )
                .await
                .is_ok()
                {
                    delivery.ack(BasicAckOptions::default()).await?;
                } else {
                    retry_or_fail(
                        &channel,
                        RETRY_QUEUE,
                        delivery,
                        &service_url,
                        &worker_token,
                        &event,
                        attempt,
                        "callback_unavailable",
                    )
                    .await?;
                }
            }
            Err(WorkError::InvalidMedia) => {
                let result = ProcessingResult {
                    result: "failed",
                    hls_prefix: None,
                    failure_code: Some("invalid_media".into()),
                };
                if report_result(
                    &service_url,
                    &worker_token,
                    event.video_id,
                    &event.event_id.to_string(),
                    result,
                )
                .await
                .is_ok()
                {
                    delivery.ack(BasicAckOptions::default()).await?;
                } else {
                    retry_or_fail(
                        &channel,
                        RETRY_QUEUE,
                        delivery,
                        &service_url,
                        &worker_token,
                        &event,
                        attempt,
                        "callback_unavailable",
                    )
                    .await?;
                }
            }
            Err(WorkError::Transient(reason)) => {
                tracing::warn!(video_id=%event.video_id, %reason, attempt, "transcoding attempt failed");
                retry_or_fail(
                    &channel,
                    RETRY_QUEUE,
                    delivery,
                    &service_url,
                    &worker_token,
                    &event,
                    attempt,
                    "processing_unavailable",
                )
                .await?;
            }
        }
    }
    connection.close(200, "worker stopping").await?;
    Ok(())
}

async fn connect(
    url: &str,
) -> Result<(Connection, Channel), Box<dyn std::error::Error + Send + Sync>> {
    let connection = Connection::connect(url, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;
    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await?;
    Ok((connection, channel))
}

async fn declare_queues(
    channel: &Channel,
    job_queue: &str,
    retry_queue: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut retry_args = FieldTable::default();
    retry_args.insert(ShortString::from("x-message-ttl"), AMQPValue::LongInt(5000));
    retry_args.insert(
        ShortString::from("x-dead-letter-exchange"),
        AMQPValue::LongString(LongString::from("")),
    );
    retry_args.insert(
        ShortString::from("x-dead-letter-routing-key"),
        AMQPValue::LongString(LongString::from(job_queue)),
    );
    channel
        .queue_declare(
            job_queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    channel
        .queue_declare(
            retry_queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            retry_args,
        )
        .await?;
    Ok(())
}

async fn process_video(
    s3: &S3Client,
    bucket: &str,
    event: &UploadEvent,
) -> Result<String, WorkError> {
    process_video_with_tools(s3, bucket, event, Path::new("ffprobe"), Path::new("ffmpeg")).await
}

async fn process_video_with_tools(
    s3: &S3Client,
    bucket: &str,
    event: &UploadEvent,
    ffprobe: &Path,
    ffmpeg: &Path,
) -> Result<String, WorkError> {
    if event.content_type != "video/mp4" || !(1..=1_073_741_824).contains(&event.size_bytes) {
        return Err(WorkError::InvalidMedia);
    }
    let work_dir = PathBuf::from("/tmp/opentube-transcode").join(event.video_id.to_string());
    let _ = fs::remove_dir_all(&work_dir).await;
    fs::create_dir_all(&work_dir)
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    let source_path = work_dir.join("source.mp4");
    let object = s3
        .get_object()
        .bucket(bucket)
        .key(&event.source_object_key)
        .send()
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    let mut input = object.body.into_async_read();
    let mut output = File::create(&source_path)
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    tokio::io::copy(&mut input, &mut output)
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    output
        .flush()
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    if fs::metadata(&source_path)
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?
        .len() as i64
        != event.size_bytes
    {
        return Err(WorkError::InvalidMedia);
    }
    let probe = probe_media(ffprobe, &source_path).await?;
    let video_stream = probe
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video"))
        .ok_or(WorkError::InvalidMedia)?;
    let width = video_stream
        .width
        .filter(|v| *v > 0)
        .ok_or(WorkError::InvalidMedia)?;
    let height = video_stream
        .height
        .filter(|v| *v > 0)
        .ok_or(WorkError::InvalidMedia)?;
    let prefix = format!("videos/{}/hls", event.video_id);
    let output_dir = work_dir.join("hls");
    fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    let mut variants = Vec::new();
    for target in rendition_heights(height) {
        variants.push(
            transcode_variant(ffmpeg, &source_path, &output_dir, width, height, target).await?,
        );
    }
    let master = render_master(&variants);
    fs::write(output_dir.join("master.m3u8"), master.as_bytes())
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    upload_tree(s3, bucket, &output_dir, &prefix).await?;
    Ok(prefix)
}

#[derive(Debug)]
struct Variant {
    height: u32,
    width: u32,
    bandwidth: u32,
    directory: String,
}

async fn probe_media(program: &Path, path: &Path) -> Result<ProbeOutput, WorkError> {
    let output = Command::new(program)
        .args(["-v", "error", "-show_streams", "-of", "json"])
        .arg(path)
        .output()
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    if !output.status.success() {
        return Err(WorkError::InvalidMedia);
    }
    serde_json::from_slice(&output.stdout).map_err(|_| WorkError::InvalidMedia)
}

async fn transcode_variant(
    program: &Path,
    source: &Path,
    output_dir: &Path,
    source_width: u32,
    source_height: u32,
    target_height: u32,
) -> Result<Variant, WorkError> {
    let height = target_height.min(source_height).max(2) / 2 * 2;
    let width =
        ((source_width as f64 * height as f64 / source_height as f64).round() as u32).max(2) / 2
            * 2;
    let label = format!("{height}p");
    let variant_dir = output_dir.join(&label);
    fs::create_dir_all(&variant_dir)
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    let playlist = variant_dir.join("index.m3u8");
    let segment_pattern = variant_dir.join("segment_%05d.ts");
    let scale = format!(
        "scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2"
    );
    let output = Command::new(program)
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(source)
        .args([
            "-vf",
            &scale,
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "24",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-hls_time",
            "4",
            "-hls_playlist_type",
            "vod",
            "-hls_segment_filename",
        ])
        .arg(segment_pattern)
        .arg(playlist)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| WorkError::Transient(e.to_string()))?;
    if !output.status.success() {
        return Err(WorkError::InvalidMedia);
    }
    Ok(Variant {
        height,
        width,
        bandwidth: match height {
            0..=360 => 700_000,
            361..=720 => 2_000_000,
            _ => 4_500_000,
        },
        directory: label,
    })
}

fn render_master(variants: &[Variant]) -> String {
    let mut master = String::from("#EXTM3U\n#EXT-X-VERSION:3\n");
    for variant in variants {
        master.push_str(&format!(
            "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}x{}\n{}/index.m3u8\n",
            variant.bandwidth, variant.width, variant.height, variant.directory
        ));
    }
    master
}

fn rendition_heights(source_height: u32) -> Vec<u32> {
    let maximum = source_height.min(1080);
    let mut heights = [360_u32, 720, 1080]
        .into_iter()
        .filter(|height| *height <= source_height)
        .collect::<Vec<_>>();
    if heights.last().copied().unwrap_or_default() < maximum {
        heights.push(maximum);
    }
    if heights.is_empty() {
        heights.push(maximum);
    }
    heights
}

async fn upload_tree(
    s3: &S3Client,
    bucket: &str,
    root: &Path,
    prefix: &str,
) -> Result<(), WorkError> {
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        let mut entries = fs::read_dir(&directory)
            .await
            .map_err(|e| WorkError::Transient(e.to_string()))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| WorkError::Transient(e.to_string()))?
        {
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|e| WorkError::Transient(e.to_string()))?;
            let relative = relative.to_string_lossy().replace('\\', "/");
            let key = format!("{prefix}/{relative}");
            let body = ByteStream::from_path(&path)
                .await
                .map_err(|e| WorkError::Transient(e.to_string()))?;
            let content_type = if path.extension().is_some_and(|ext| ext == "m3u8") {
                "application/vnd.apple.mpegurl"
            } else {
                "video/mp2t"
            };
            s3.put_object()
                .bucket(bucket)
                .key(key)
                .content_type(content_type)
                .body(body)
                .send()
                .await
                .map_err(|e| WorkError::Transient(e.to_string()))?;
        }
    }
    Ok(())
}

async fn report_result(
    base: &str,
    token: &str,
    video_id: Uuid,
    idempotency_key: &str,
    result: ProcessingResult,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/internal/videos/{video_id}/processing-result",
            base.trim_end_matches('/')
        ))
        .bearer_auth(token)
        .header("idempotency-key", idempotency_key)
        .json(&result)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(format!("video service returned {}", response.status()).into());
    }
    Ok(())
}

fn exhausted_retry_budget(attempt: u64) -> bool {
    attempt >= MAX_DELIVERIES - 1
}

async fn retry_or_fail(
    channel: &Channel,
    retry_queue: &str,
    delivery: lapin::message::Delivery,
    service_url: &str,
    token: &str,
    event: &UploadEvent,
    attempt: u64,
    failure_code: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if exhausted_retry_budget(attempt) {
        report_result(
            service_url,
            token,
            event.video_id,
            &event.event_id.to_string(),
            ProcessingResult {
                result: "failed",
                hls_prefix: None,
                failure_code: Some(failure_code.into()),
            },
        )
        .await?;
        delivery.ack(BasicAckOptions::default()).await?;
        return Ok(());
    }
    let mut headers = FieldTable::default();
    headers.insert(
        ShortString::from("x-retry-count"),
        AMQPValue::LongLongInt((attempt + 1) as i64),
    );
    let payload = serde_json::to_vec(event)?;
    channel
        .basic_publish(
            "",
            retry_queue,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default()
                .with_delivery_mode(2)
                .with_headers(headers),
        )
        .await?
        .await?;
    delivery.ack(BasicAckOptions::default()).await?;
    Ok(())
}

fn retry_count(properties: &BasicProperties) -> u64 {
    properties
        .headers()
        .as_ref()
        .and_then(|h| h.inner().get("x-retry-count"))
        .and_then(|v| match v {
            AMQPValue::LongLongInt(n) if *n >= 0 => Some(*n as u64),
            AMQPValue::LongInt(n) if *n >= 0 => Some(*n as u64),
            _ => None,
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs as std_fs, os::unix::fs::PermissionsExt};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    fn executable_script(contents: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("opentube-test-tool-{}", Uuid::new_v4()));
        std_fs::write(&path, contents).unwrap();
        let mut permissions = std_fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o700);
        std_fs::set_permissions(&path, permissions).unwrap();
        path
    }

    fn cleanup(paths: &[PathBuf]) {
        for path in paths {
            let _ = std_fs::remove_file(path);
        }
    }

    #[test]
    fn master_playlist_lists_all_adaptive_variants() {
        let text = render_master(&[
            Variant {
                height: 360,
                width: 640,
                bandwidth: 700_000,
                directory: "360p".into(),
            },
            Variant {
                height: 720,
                width: 1280,
                bandwidth: 2_000_000,
                directory: "720p".into(),
            },
        ]);
        assert!(text.starts_with("#EXTM3U\n"));
        assert!(text.contains("360p/index.m3u8"));
        assert!(text.contains("1280x720"));
    }

    #[test]
    fn master_playlist_without_variants_still_has_a_valid_header() {
        assert_eq!(render_master(&[]), "#EXTM3U\n#EXT-X-VERSION:3\n");
    }

    #[test]
    fn rendition_heights_never_upscale_and_stop_at_1080p() {
        assert_eq!(rendition_heights(240), [240]);
        assert_eq!(rendition_heights(360), [360]);
        assert_eq!(rendition_heights(480), [360, 480]);
        assert_eq!(rendition_heights(720), [360, 720]);
        assert_eq!(rendition_heights(1440), [360, 720, 1080]);
    }

    #[test]
    fn retry_count_accepts_supported_nonnegative_amqp_numbers_only() {
        let without_headers = BasicProperties::default();
        assert_eq!(retry_count(&without_headers), 0);

        for (value, expected) in [
            (AMQPValue::LongLongInt(3), 3),
            (AMQPValue::LongInt(4), 4),
            (AMQPValue::LongLongInt(-1), 0),
            (AMQPValue::ShortInt(8), 0),
        ] {
            let mut headers = FieldTable::default();
            headers.insert(ShortString::from("x-retry-count"), value);
            let properties = BasicProperties::default().with_headers(headers);
            assert_eq!(retry_count(&properties), expected);
        }
    }

    #[test]
    fn retry_budget_fails_only_after_the_final_delivery() {
        assert!(!exhausted_retry_budget(0));
        assert!(!exhausted_retry_budget(MAX_DELIVERIES - 2));
        assert!(exhausted_retry_budget(MAX_DELIVERIES - 1));
        assert!(exhausted_retry_budget(MAX_DELIVERIES));
    }

    #[tokio::test]
    async fn media_probe_maps_success_process_failure_malformed_json_and_missing_tool() {
        let success = executable_script(
            "#!/bin/sh\nprintf '%s' '{\"streams\":[{\"codec_type\":\"video\",\"width\":640,\"height\":360}]}'\n",
        );
        let failure = executable_script("#!/bin/sh\nexit 1\n");
        let malformed = executable_script("#!/bin/sh\nprintf '%s' 'not json'\n");
        let input = env::temp_dir().join(format!("opentube-probe-{}.mp4", Uuid::new_v4()));
        std_fs::write(&input, b"synthetic fixture").unwrap();

        let probe = probe_media(&success, &input).await.unwrap();
        assert_eq!(probe.streams.len(), 1);
        assert_eq!(probe.streams[0].width, Some(640));
        assert!(matches!(
            probe_media(&failure, &input).await,
            Err(WorkError::InvalidMedia)
        ));
        assert!(matches!(
            probe_media(&malformed, &input).await,
            Err(WorkError::InvalidMedia)
        ));
        assert!(matches!(
            probe_media(Path::new("/missing/opentube-ffprobe"), &input).await,
            Err(WorkError::Transient(_))
        ));

        cleanup(&[success, failure, malformed, input]);
    }

    #[tokio::test]
    async fn transcode_dimensions_are_even_and_process_errors_are_classified() {
        let success = executable_script("#!/bin/sh\nexit 0\n");
        let failure = executable_script("#!/bin/sh\nexit 1\n");
        let root = env::temp_dir().join(format!("opentube-transcode-test-{}", Uuid::new_v4()));
        let input = root.join("source.mp4");
        let output = root.join("hls");
        tokio::fs::create_dir_all(&root).await.unwrap();
        tokio::fs::write(&input, b"synthetic fixture")
            .await
            .unwrap();

        let variant = transcode_variant(&success, &input, &output, 853, 479, 360)
            .await
            .unwrap();
        assert_eq!(variant.height, 360);
        assert_eq!(variant.width % 2, 0);
        assert_eq!(variant.directory, "360p");
        assert_eq!(variant.bandwidth, 700_000);
        assert!(matches!(
            transcode_variant(&failure, &input, &output, 640, 360, 360).await,
            Err(WorkError::InvalidMedia)
        ));
        assert!(matches!(
            transcode_variant(
                Path::new("/missing/opentube-ffmpeg"),
                &input,
                &output,
                640,
                360,
                360
            )
            .await,
            Err(WorkError::Transient(_))
        ));

        cleanup(&[success, failure]);
        let _ = tokio::fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn invalid_media_metadata_is_rejected_before_storage_access() {
        let shared = aws_config::SdkConfig::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .build();
        let s3 = S3Client::new(&shared);
        let event = UploadEvent {
            event_id: Uuid::new_v4(),
            video_id: Uuid::new_v4(),
            source_object_key: "unused/source.mp4".into(),
            content_type: "video/quicktime".into(),
            size_bytes: 1,
            schema_version: 1,
            occurred_at: Utc::now(),
        };
        assert!(matches!(
            process_video(&s3, "unused", &event).await,
            Err(WorkError::InvalidMedia)
        ));
        let too_large = UploadEvent {
            content_type: "video/mp4".into(),
            size_bytes: 1_073_741_825,
            ..event
        };
        assert!(matches!(
            process_video(&s3, "unused", &too_large).await,
            Err(WorkError::InvalidMedia)
        ));
    }

    #[tokio::test]
    async fn processing_callback_sends_worker_auth_and_surfaces_service_errors() {
        async fn serve_once(status: u16) -> (String, tokio::task::JoinHandle<String>) {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let task = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut buffer = vec![0; 8192];
                let size = stream.read(&mut buffer).await.unwrap();
                let request = String::from_utf8_lossy(&buffer[..size]).into_owned();
                let status_text = if status == 200 { "OK" } else { "Unavailable" };
                stream
                    .write_all(
                        format!(
                            "HTTP/1.1 {status} {status_text}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        )
                        .as_bytes(),
                    )
                    .await
                    .unwrap();
                request
            });
            (format!("http://{address}"), task)
        }

        let video_id = Uuid::new_v4();
        let (base, received) = serve_once(200).await;
        report_result(
            &base,
            "unit-worker-token",
            video_id,
            "event-1",
            ProcessingResult {
                result: "ready",
                hls_prefix: Some(format!("videos/{video_id}/hls")),
                failure_code: None,
            },
        )
        .await
        .unwrap();
        let request = received.await.unwrap();
        assert!(request.contains(&format!(
            "POST /internal/videos/{video_id}/processing-result"
        )));
        assert!(request.contains("authorization: Bearer unit-worker-token"));
        assert!(request.contains("idempotency-key: event-1"));

        let (base, _) = serve_once(503).await;
        let error = report_result(
            &base,
            "unit-worker-token",
            video_id,
            "event-2",
            ProcessingResult {
                result: "failed",
                hls_prefix: None,
                failure_code: Some("callback_unavailable".into()),
            },
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("503"));
    }

    #[tokio::test]
    async fn rabbit_delivery_retries_once_and_reports_failure_after_the_budget() {
        let Ok(rabbit_url) = env::var("OPENTUBE_TEST_RABBITMQ_URL") else {
            return;
        };
        let (connection, channel) = connect(&rabbit_url).await.unwrap();
        let suffix = Uuid::new_v4().simple().to_string();
        let job_queue = format!("opentube.coverage.jobs.{suffix}");
        let retry_queue = format!("opentube.coverage.retries.{suffix}");
        declare_queues(&channel, &job_queue, &retry_queue)
            .await
            .unwrap();
        let mut jobs = channel
            .basic_consume(
                &job_queue,
                "coverage-worker",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
        let mut retries = channel
            .basic_consume(
                &retry_queue,
                "coverage-retry-check",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
        let make_event = || UploadEvent {
            event_id: Uuid::new_v4(),
            video_id: Uuid::new_v4(),
            source_object_key: format!("coverage/{suffix}/source.mp4"),
            content_type: "video/mp4".into(),
            size_bytes: 42,
            schema_version: 1,
            occurred_at: Utc::now(),
        };
        let retry_event = make_event();
        channel
            .basic_publish(
                "",
                &job_queue,
                BasicPublishOptions::default(),
                &serde_json::to_vec(&retry_event).unwrap(),
                BasicProperties::default(),
            )
            .await
            .unwrap()
            .await
            .unwrap();
        let delivery = tokio::time::timeout(Duration::from_secs(3), jobs.next())
            .await
            .expect("job delivery arrives")
            .unwrap()
            .unwrap();
        retry_or_fail(
            &channel,
            &retry_queue,
            delivery,
            "http://127.0.0.1:1",
            "unit-token",
            &retry_event,
            0,
            "transient",
        )
        .await
        .unwrap();
        let retry = tokio::time::timeout(Duration::from_secs(3), retries.next())
            .await
            .expect("retry delivery arrives")
            .unwrap()
            .unwrap();
        assert_eq!(retry_count(&retry.properties), 1);
        assert_eq!(
            serde_json::from_slice::<UploadEvent>(&retry.data)
                .unwrap()
                .event_id,
            retry_event.event_id
        );
        retry.ack(BasicAckOptions::default()).await.unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let callback = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0; 4096];
            let size = stream.read(&mut buffer).await.unwrap();
            let request = String::from_utf8_lossy(&buffer[..size]).into_owned();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .await
                .unwrap();
            request
        });
        let final_event = make_event();
        channel
            .basic_publish(
                "",
                &job_queue,
                BasicPublishOptions::default(),
                &serde_json::to_vec(&final_event).unwrap(),
                BasicProperties::default(),
            )
            .await
            .unwrap()
            .await
            .unwrap();
        let final_delivery = tokio::time::timeout(Duration::from_secs(3), jobs.next())
            .await
            .expect("final delivery arrives")
            .unwrap()
            .unwrap();
        retry_or_fail(
            &channel,
            &retry_queue,
            final_delivery,
            &format!("http://{address}"),
            "unit-token",
            &final_event,
            MAX_DELIVERIES - 1,
            "retry_limit_reached",
        )
        .await
        .unwrap();
        let callback_request = callback.await.unwrap();
        assert!(callback_request.contains("retry_limit_reached"));
        let retry_state = channel
            .queue_declare(
                &retry_queue,
                QueueDeclareOptions {
                    passive: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .unwrap();
        assert_eq!(retry_state.message_count(), 0);
        channel
            .queue_delete(&job_queue, Default::default())
            .await
            .unwrap();
        channel
            .queue_delete(&retry_queue, Default::default())
            .await
            .unwrap();
        connection.close(200, "test complete").await.unwrap();
    }

    #[tokio::test]
    async fn minio_transcoding_pipeline_writes_stable_hls_outputs() {
        if env::var("OPENTUBE_TEST_S3_ENDPOINT").is_err()
            || env::var("AWS_ACCESS_KEY_ID").is_err()
            || env::var("AWS_SECRET_ACCESS_KEY").is_err()
        {
            return;
        }
        let endpoint = env::var("OPENTUBE_TEST_S3_ENDPOINT").unwrap();
        let bucket = env::var("OPENTUBE_TEST_S3_BUCKET").unwrap_or_else(|_| "opentube".into());
        let shared = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .load()
            .await;
        let config = S3ConfigBuilder::from(&shared)
            .endpoint_url(endpoint)
            .force_path_style(true)
            .build();
        let s3 = S3Client::from_conf(config);
        if s3.head_bucket().bucket(&bucket).send().await.is_err() {
            s3.create_bucket()
                .bucket(&bucket)
                .send()
                .await
                .expect("create temporary test bucket");
        }
        let id = Uuid::new_v4();
        let source_key = format!("coverage/{id}/source.mp4");
        let prefix = format!("videos/{id}/hls");
        let source = b"synthetic MP4 fixture";
        s3.put_object()
            .bucket(&bucket)
            .key(&source_key)
            .body(ByteStream::from(source.to_vec()))
            .send()
            .await
            .expect("write synthetic source to MinIO");

        let ffprobe = executable_script(
            "#!/bin/sh\nprintf '%s' '{\"streams\":[{\"codec_type\":\"video\",\"width\":854,\"height\":480}]}'\n",
        );
        let ffmpeg = executable_script(
            "#!/bin/sh\nfor output do last=\"$output\"; done\nmkdir -p \"$(dirname \"$last\")\"\nprintf '%s\\n' '#EXTM3U' '#EXT-X-ENDLIST' > \"$last\"\nprintf segment > \"$(dirname \"$last\")/segment_00000.ts\"\n",
        );
        let event = UploadEvent {
            event_id: Uuid::new_v4(),
            video_id: id,
            source_object_key: source_key.clone(),
            content_type: "video/mp4".into(),
            size_bytes: source.len() as i64,
            schema_version: 1,
            occurred_at: Utc::now(),
        };

        let result = process_video_with_tools(&s3, &bucket, &event, &ffprobe, &ffmpeg)
            .await
            .unwrap();
        assert_eq!(result, prefix);
        let keys = [
            source_key.clone(),
            format!("{prefix}/master.m3u8"),
            format!("{prefix}/360p/index.m3u8"),
            format!("{prefix}/360p/segment_00000.ts"),
            format!("{prefix}/480p/index.m3u8"),
            format!("{prefix}/480p/segment_00000.ts"),
        ];
        for key in keys.iter().skip(1) {
            s3.head_object()
                .bucket(&bucket)
                .key(key)
                .send()
                .await
                .unwrap_or_else(|_| panic!("expected transcoded object {key}"));
        }
        let master = s3
            .get_object()
            .bucket(&bucket)
            .key(format!("{prefix}/master.m3u8"))
            .send()
            .await
            .unwrap()
            .body
            .collect()
            .await
            .unwrap()
            .into_bytes();
        let master_text = String::from_utf8(master.to_vec()).unwrap();
        assert!(master_text.contains("360p/index.m3u8"));
        assert!(master_text.contains("480p/index.m3u8"));

        for key in keys {
            s3.delete_object()
                .bucket(&bucket)
                .key(key)
                .send()
                .await
                .unwrap();
        }
        cleanup(&[ffprobe, ffmpeg]);
        let work_dir = PathBuf::from("/tmp/opentube-transcode").join(id.to_string());
        let _ = tokio::fs::remove_dir_all(work_dir).await;
    }
}
