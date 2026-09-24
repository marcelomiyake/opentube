use std::{env, net::SocketAddr, time::Duration};

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
    Client as S3Client, config::Builder as S3ConfigBuilder, presigning::PresigningConfig,
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use lapin::{
    BasicProperties, Connection, ConnectionProperties,
    options::{BasicPublishOptions, ConfirmSelectOptions, QueueDeclareOptions},
    types::FieldTable,
};
use platform_core::{Claims, verify_token};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::time::sleep;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

const MAX_UPLOAD_BYTES: i64 = 1_073_741_824;
const UPLOAD_TTL_SECONDS: u64 = 900;
const UPLOAD_QUEUE: &str = "video.uploaded.v1";

#[derive(Clone)]
struct AppState {
    db: PgPool,
    jwt_secret: Vec<u8>,
    worker_token: String,
    internal_s3: S3Client,
    public_s3: S3Client,
    bucket: String,
    media_public_base: String,
}

#[derive(Deserialize)]
struct CreateVideo {
    title: String,
    #[serde(default)]
    description: String,
    content_type: String,
    size_bytes: i64,
}

#[derive(Serialize)]
struct CreateVideoResponse {
    id: Uuid,
    status: &'static str,
    upload_url: String,
    upload_expires_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct ListQuery {
    query: Option<String>,
    cursor: Option<String>,
    limit: Option<i64>,
}

#[derive(Deserialize, Serialize, Clone)]
struct VideoSummary {
    id: Uuid,
    title: String,
    description: String,
    creator: String,
    published_at: DateTime<Utc>,
    playback_url: String,
}

#[derive(Deserialize, Serialize)]
struct VideoPage {
    items: Vec<VideoSummary>,
    next_cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct UploadEvent {
    event_id: Uuid,
    video_id: Uuid,
    source_object_key: String,
    content_type: String,
    size_bytes: i64,
    schema_version: u8,
    occurred_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
struct ProcessingResult {
    result: String,
    hls_prefix: Option<String>,
    failure_code: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Cursor {
    published_at: DateTime<Utc>,
    id: Uuid,
    query: String,
}

#[derive(sqlx::FromRow)]
struct VideoRow {
    id: Uuid,
    title: String,
    description: String,
    owner_email: String,
    published_at: DateTime<Utc>,
    hls_prefix: String,
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: Uuid,
    payload: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let secret = env::var("JWT_SECRET")?;
    if secret.len() < 32 {
        return Err("JWT_SECRET must contain at least 32 bytes".into());
    }
    let region_name = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".into());
    let shared = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(region_name))
        .load()
        .await;
    let internal_s3 = s3_client(&shared, &env::var("S3_ENDPOINT")?);
    let public_endpoint = env::var("S3_PUBLIC_ENDPOINT")
        .unwrap_or_else(|_| env::var("S3_ENDPOINT").unwrap_or_default());
    let public_s3 = s3_client(&shared, &public_endpoint);
    let bucket = env::var("S3_BUCKET").unwrap_or_else(|_| "opentube".into());
    ensure_bucket(&internal_s3, &bucket).await?;

    let state = AppState {
        db: pool.clone(),
        jwt_secret: secret.into_bytes(),
        worker_token: env::var("WORKER_TOKEN")?,
        internal_s3,
        public_s3,
        bucket,
        media_public_base: env::var("MEDIA_PUBLIC_BASE")
            .unwrap_or_else(|_| "http://127.0.0.1:19000".into()),
    };
    let app = app_router(state);

    if let Ok(rabbit_url) = env::var("RABBITMQ_URL") {
        tokio::spawn(outbox_relay(pool, rabbit_url));
    }
    let address: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8082".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "video service listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn app_router(state: AppState) -> Router {
    let internal = Router::new()
        .route(
            "/internal/videos/{video_id}/processing-result",
            post(processing_result),
        )
        .layer(middleware::from_fn_with_state(state.clone(), worker_auth));
    Router::new()
        .route("/v1/videos", get(list_videos).post(create_video))
        .route("/v1/videos/{video_id}", get(get_video))
        .route("/v1/videos/{video_id}/complete", post(complete_upload))
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .merge(internal)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

fn s3_client(shared: &aws_config::SdkConfig, endpoint: &str) -> S3Client {
    let config = S3ConfigBuilder::from(shared)
        .endpoint_url(endpoint)
        .force_path_style(true)
        .build();
    S3Client::from_conf(config)
}

async fn ensure_bucket(client: &S3Client, bucket: &str) -> Result<(), Box<dyn std::error::Error>> {
    if client.head_bucket().bucket(bucket).send().await.is_err() {
        client.create_bucket().bucket(bucket).send().await?;
    }
    let playback_policy = json!({
        "Version": "2012-10-17",
        "Statement": [{
            "Sid": "AllowAnonymousHlsPlayback",
            "Effect": "Allow",
            "Principal": "*",
            "Action": "s3:GetObject",
            "Resource": format!("arn:aws:s3:::{bucket}/videos/*/hls/*")
        }]
    });
    client
        .put_bucket_policy()
        .bucket(bucket)
        .policy(playback_policy.to_string())
        .send()
        .await?;
    Ok(())
}

fn authenticated(headers: &HeaderMap, secret: &[u8]) -> Result<Claims, StatusCode> {
    let raw = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token = raw
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    verify_token(token, secret).map_err(|_| StatusCode::UNAUTHORIZED)
}

async fn create_video(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateVideo>,
) -> Result<(StatusCode, Json<CreateVideoResponse>), StatusCode> {
    let claims = authenticated(&headers, &state.jwt_secret)?;
    let title = input.title.trim();
    if title.is_empty()
        || title.len() > 120
        || input.description.len() > 2000
        || input.content_type != "video/mp4"
        || !(1..=MAX_UPLOAD_BYTES).contains(&input.size_bytes)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let id = Uuid::new_v4();
    let object_key = format!("videos/{id}/source.mp4");
    let expires = DateTime::from_timestamp(Utc::now().timestamp() + UPLOAD_TTL_SECONDS as i64, 0)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let signing = PresigningConfig::expires_in(Duration::from_secs(UPLOAD_TTL_SECONDS))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let signed = state
        .public_s3
        .put_object()
        .bucket(&state.bucket)
        .key(&object_key)
        .content_type("video/mp4")
        .presigned(signing)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("INSERT INTO video.videos (id, owner_id, owner_email, title, description, content_type, size_bytes, source_object_key, status) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'pending_upload')")
        .bind(id)
        .bind(claims.sub)
        .bind(claims.email)
        .bind(title)
        .bind(input.description.trim())
        .bind(input.content_type)
        .bind(input.size_bytes)
        .bind(object_key)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok((
        StatusCode::CREATED,
        Json(CreateVideoResponse {
            id,
            status: "pending_upload",
            upload_url: signed.uri().to_string(),
            upload_expires_at: expires,
        }),
    ))
}

async fn complete_upload(
    State(state): State<AppState>,
    Path(video_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let claims = authenticated(&headers, &state.jwt_secret)?;
    let row: Option<(String, i64, String)> = sqlx::query_as("SELECT source_object_key, size_bytes, status FROM video.videos WHERE id=$1 AND owner_id=$2")
        .bind(video_id).bind(claims.sub).fetch_optional(&state.db).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let Some((object_key, expected_size, status)) = row else {
        return Err(StatusCode::NOT_FOUND);
    };
    if status != "pending_upload" {
        return Ok((
            StatusCode::ACCEPTED,
            Json(json!({"id":video_id,"status":status})),
        ));
    }
    let object = state
        .internal_s3
        .head_object()
        .bucket(&state.bucket)
        .key(&object_key)
        .send()
        .await
        .map_err(|_| StatusCode::CONFLICT)?;
    if object.content_length().unwrap_or_default() != expected_size {
        return Err(StatusCode::CONFLICT);
    }
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let updated = sqlx::query("UPDATE video.videos SET status='uploaded' WHERE id=$1 AND owner_id=$2 AND status='pending_upload'")
        .bind(video_id).bind(claims.sub).execute(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if updated.rows_affected() == 1 {
        let event = UploadEvent {
            event_id: Uuid::new_v4(),
            video_id,
            source_object_key: object_key,
            content_type: "video/mp4".into(),
            size_bytes: expected_size,
            schema_version: 1,
            occurred_at: Utc::now(),
        };
        let payload = serde_json::to_value(event).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        sqlx::query("INSERT INTO video.outbox (id, aggregate_id, event_type, payload) VALUES ($1,$2,'video.uploaded.v1',$3)")
            .bind(Uuid::new_v4()).bind(video_id).bind(payload).execute(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    }
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id":video_id,"status":"uploaded"})),
    ))
}

async fn list_videos(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<VideoPage>, StatusCode> {
    let search = query.query.unwrap_or_default().trim().to_owned();
    if search.len() > 200 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let limit = query.limit.unwrap_or(24).clamp(1, 50);
    let cursor = query
        .cursor
        .as_deref()
        .map(decode_cursor)
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    if cursor.as_ref().is_some_and(|c| c.query != search) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let cursor_time = cursor.as_ref().map(|c| c.published_at);
    let cursor_id = cursor.as_ref().map(|c| c.id);
    let mut rows = sqlx::query_as::<_, VideoRow>(
        "SELECT id,title,description,owner_email,published_at,hls_prefix FROM video.videos \
         WHERE status='ready' AND ($1 = '' OR search_vector @@ plainto_tsquery('simple',$1)) \
         AND ($2::timestamptz IS NULL OR (published_at,id) < ($2,$3)) \
         ORDER BY published_at DESC,id DESC LIMIT $4",
    )
    .bind(&search)
    .bind(cursor_time)
    .bind(cursor_id)
    .bind(limit + 1)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let has_more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let items: Vec<VideoSummary> = rows
        .into_iter()
        .map(|r| VideoSummary {
            id: r.id,
            title: r.title,
            description: r.description,
            creator: r.owner_email,
            published_at: r.published_at,
            playback_url: format!(
                "{}/{}/{}/master.m3u8",
                state.media_public_base.trim_end_matches('/'),
                state.bucket,
                r.hls_prefix.trim_start_matches('/')
            ),
        })
        .collect();
    let next_cursor = if has_more {
        items.last().map(|v| {
            encode_cursor(&Cursor {
                published_at: v.published_at,
                id: v.id,
                query: search,
            })
        })
    } else {
        None
    };
    Ok(Json(VideoPage { items, next_cursor }))
}

async fn get_video(
    State(state): State<AppState>,
    Path(video_id): Path<Uuid>,
) -> Result<Json<VideoSummary>, StatusCode> {
    let row = sqlx::query_as::<_, VideoRow>("SELECT id,title,description,owner_email,published_at,hls_prefix FROM video.videos WHERE id=$1 AND status='ready'")
        .bind(video_id).fetch_optional(&state.db).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let Some(r) = row else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(VideoSummary {
        id: r.id,
        title: r.title,
        description: r.description,
        creator: r.owner_email,
        published_at: r.published_at,
        playback_url: format!(
            "{}/{}/{}/master.m3u8",
            state.media_public_base.trim_end_matches('/'),
            state.bucket,
            r.hls_prefix.trim_start_matches('/')
        ),
    }))
}

async fn worker_auth(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: middleware::Next,
) -> impl IntoResponse {
    let valid = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|v| v == state.worker_token);
    if valid {
        next.run(request).await
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn processing_result(
    State(state): State<AppState>,
    Path(video_id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty())
        .ok_or(StatusCode::BAD_REQUEST)?;
    if idempotency_key.len() > 200 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let result: ProcessingResult =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let payload = serde_json::to_value(&result).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let claimed: Option<(String,)> = sqlx::query_as(
        "INSERT INTO video.processing_callbacks (idempotency_key, video_id, payload) VALUES ($1,$2,$3) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING idempotency_key",
    )
    .bind(idempotency_key)
    .bind(video_id)
    .bind(&payload)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if claimed.is_none() {
        let existing: Option<(Uuid, serde_json::Value)> = sqlx::query_as(
            "SELECT video_id,payload FROM video.processing_callbacks WHERE idempotency_key=$1",
        )
        .bind(idempotency_key)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        tx.commit()
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        return match existing {
            Some((existing_video, existing_payload))
                if existing_video == video_id && existing_payload == payload =>
            {
                Ok(StatusCode::OK)
            }
            _ => Err(StatusCode::CONFLICT),
        };
    }
    match result.result.as_str() {
        "ready" => {
            let prefix = result
                .hls_prefix
                .filter(|v| !v.is_empty())
                .ok_or(StatusCode::BAD_REQUEST)?;
            if !prefix.starts_with(&format!("videos/{video_id}/hls")) {
                return Err(StatusCode::BAD_REQUEST);
            }
            let update = sqlx::query("UPDATE video.videos SET status='ready', hls_prefix=$2, failure_code=NULL, published_at=COALESCE(published_at,now()) WHERE id=$1 AND status IN ('uploaded','processing','ready')")
                .bind(video_id).bind(prefix).execute(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            if update.rows_affected() != 1 {
                return Err(StatusCode::NOT_FOUND);
            }
        }
        "failed" => {
            let code = result
                .failure_code
                .unwrap_or_else(|| "processing_failed".into());
            let safe_code = code
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                .take(64)
                .collect::<String>();
            let update = sqlx::query("UPDATE video.videos SET status='failed', failure_code=$2 WHERE id=$1 AND status IN ('uploaded','processing')")
                .bind(video_id).bind(safe_code).execute(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            if update.rows_affected() != 1 {
                return Err(StatusCode::NOT_FOUND);
            }
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    }
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(StatusCode::OK)
}

async fn outbox_relay(pool: PgPool, rabbit_url: String) {
    loop {
        match relay_once(&pool, &rabbit_url).await {
            Ok(()) => sleep(Duration::from_millis(700)).await,
            Err(error) => {
                tracing::warn!(%error, "outbox relay unavailable; will retry");
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn relay_once(
    pool: &PgPool,
    rabbit_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let connection = Connection::connect(rabbit_url, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;
    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await?;
    channel
        .queue_declare(
            UPLOAD_QUEUE,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    let mut tx = pool.begin().await?;
    let rows = sqlx::query_as::<_, OutboxRow>("SELECT id,payload FROM video.outbox WHERE delivered_at IS NULL ORDER BY created_at LIMIT 20 FOR UPDATE SKIP LOCKED")
        .fetch_all(&mut *tx).await?;
    for row in rows {
        let bytes = serde_json::to_vec(&row.payload)?;
        channel
            .basic_publish(
                "",
                UPLOAD_QUEUE,
                BasicPublishOptions::default(),
                &bytes,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await?
            .await?;
        sqlx::query("UPDATE video.outbox SET delivered_at=now(),attempts=attempts+1 WHERE id=$1")
            .bind(row.id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    connection.close(200, "outbox iteration complete").await?;
    Ok(())
}

fn encode_cursor(cursor: &Cursor) -> String {
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(cursor).expect("cursor serialization is infallible"))
}

fn decode_cursor(value: &str) -> Result<Cursor, ()> {
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| ())?;
    serde_json::from_slice(&bytes).map_err(|_| ())
}

async fn live() -> StatusCode {
    StatusCode::OK
}
async fn ready(State(state): State<AppState>) -> StatusCode {
    if sqlx::query("SELECT 1").execute(&state.db).await.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use platform_core::issue_token;
    use tower::ServiceExt;

    const TEST_JWT_SECRET: &[u8] = b"video-service-test-secret-at-least-32-bytes";

    async fn test_pool() -> Option<PgPool> {
        let database_url = env::var("OPENTUBE_TEST_VIDEO_DATABASE_URL").ok()?;
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .connect(&database_url)
            .await
            .expect("test Postgres is reachable");
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        Some(pool)
    }

    async fn test_state(db: PgPool) -> AppState {
        let shared = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .load()
            .await;
        let endpoint = env::var("OPENTUBE_TEST_S3_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:19000".into());
        let s3 = s3_client(&shared, &endpoint);
        let bucket = env::var("OPENTUBE_TEST_S3_BUCKET").unwrap_or_else(|_| "opentube".into());
        if env::var("OPENTUBE_TEST_S3_ENDPOINT").is_ok() {
            ensure_bucket(&s3, &bucket)
                .await
                .expect("test object-store bucket is ready");
        }
        AppState {
            db,
            jwt_secret: TEST_JWT_SECRET.to_vec(),
            worker_token: "test-worker-token".into(),
            internal_s3: s3.clone(),
            public_s3: s3,
            bucket,
            media_public_base: "http://media.test".into(),
        }
    }

    async fn insert_video(
        pool: &PgPool,
        id: Uuid,
        status: &str,
        published_at: Option<DateTime<Utc>>,
        title: &str,
        description: &str,
    ) {
        sqlx::query("INSERT INTO video.videos (id,owner_id,owner_email,title,description,content_type,size_bytes,source_object_key,hls_prefix,status,published_at) VALUES ($1,$2,$3,$4,$5,'video/mp4',1,$6,$7,$8,$9)")
            .bind(id)
            .bind(Uuid::new_v4())
            .bind("coverage@example.test")
            .bind(title)
            .bind(description)
            .bind(format!("coverage/{id}/source.mp4"))
            .bind((status == "ready").then(|| format!("videos/{id}/hls")))
            .bind(status)
            .bind(published_at)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn insert_owned_video(pool: &PgPool, id: Uuid, owner_id: Uuid, status: &str) {
        sqlx::query("INSERT INTO video.videos (id,owner_id,owner_email,title,content_type,size_bytes,source_object_key,status) VALUES ($1,$2,$3,'Upload completion test','video/mp4',42,$4,$5)")
            .bind(id)
            .bind(owner_id)
            .bind("coverage-owner@example.test")
            .bind(format!("coverage/{id}/missing.mp4"))
            .bind(status)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn cleanup_videos(pool: &PgPool, ids: &[Uuid]) {
        for id in ids {
            sqlx::query("DELETE FROM video.processing_callbacks WHERE video_id=$1")
                .bind(id)
                .execute(pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM video.outbox WHERE aggregate_id=$1")
                .bind(id)
                .execute(pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM video.videos WHERE id=$1")
                .bind(id)
                .execute(pool)
                .await
                .unwrap();
        }
    }

    fn request(method: &str, uri: &str, body: String) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    fn creator_token() -> String {
        issue_token(Uuid::new_v4(), "creator@example.test", TEST_JWT_SECRET, 10).unwrap()
    }

    #[test]
    fn cursor_round_trips_and_preserves_search() {
        let cursor = Cursor {
            published_at: Utc::now(),
            id: Uuid::new_v4(),
            query: "quiet lake".into(),
        };
        let decoded = decode_cursor(&encode_cursor(&cursor)).unwrap();
        assert_eq!(decoded.id, cursor.id);
        assert_eq!(decoded.query, "quiet lake");
        assert_eq!(decoded.published_at, cursor.published_at);
    }

    #[test]
    fn malformed_cursor_is_rejected() {
        assert!(decode_cursor("not a cursor").is_err());
    }

    #[test]
    fn upload_size_boundaries_are_inclusive() {
        assert!((1..=MAX_UPLOAD_BYTES).contains(&1));
        assert!((1..=MAX_UPLOAD_BYTES).contains(&MAX_UPLOAD_BYTES));
        assert!(!(1..=MAX_UPLOAD_BYTES).contains(&0));
        assert!(!(1..=MAX_UPLOAD_BYTES).contains(&(MAX_UPLOAD_BYTES + 1)));
    }

    #[tokio::test]
    async fn create_video_rejects_unauthenticated_and_invalid_upload_metadata() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let app = app_router(test_state(pool).await);
        let path = "/v1/videos";
        let unauthenticated = app
            .clone()
            .oneshot(request(
                "POST",
                path,
                json!({"title":"Title","description":"","content_type":"video/mp4","size_bytes":1})
                    .to_string(),
            ))
            .await
            .unwrap();
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

        let token = creator_token();
        let cases = [
            json!({"title":"  ","description":"","content_type":"video/mp4","size_bytes":1}),
            json!({"title":"x".repeat(121),"description":"","content_type":"video/mp4","size_bytes":1}),
            json!({"title":"Title","description":"x".repeat(2001),"content_type":"video/mp4","size_bytes":1}),
            json!({"title":"Title","description":"","content_type":"video/quicktime","size_bytes":1}),
            json!({"title":"Title","description":"","content_type":"video/mp4","size_bytes":0}),
            json!({"title":"Title","description":"","content_type":"video/mp4","size_bytes":MAX_UPLOAD_BYTES + 1}),
        ];
        for body in cases {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(path)
                        .header("content-type", "application/json")
                        .header("authorization", format!("Bearer {token}"))
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn valid_video_creation_returns_a_presigned_upload_and_persists_pending_state() {
        if env::var("OPENTUBE_TEST_S3_ENDPOINT").is_err()
            || env::var("AWS_ACCESS_KEY_ID").is_err()
            || env::var("AWS_SECRET_ACCESS_KEY").is_err()
        {
            return;
        }
        let Some(pool) = test_pool().await else {
            return;
        };
        let app = app_router(test_state(pool.clone()).await);
        let owner_id = Uuid::new_v4();
        let email = "coverage-creator@example.test";
        let token = issue_token(owner_id, email, TEST_JWT_SECRET, 10).unwrap();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/videos")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({"title":"Synthetic upload","description":"Test fixture","content_type":"video/mp4","size_bytes":42}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let created: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(created["status"], "pending_upload");
        let upload_url = created["upload_url"].as_str().unwrap();
        assert!(upload_url.contains("/opentube/videos/"));
        assert!(upload_url.contains("/source.mp4?"));
        let created_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        assert!(
            DateTime::parse_from_rfc3339(created["upload_expires_at"].as_str().unwrap())
                .unwrap()
                .with_timezone(&Utc)
                > Utc::now()
        );

        let stored: (String, String, i64) =
            sqlx::query_as("SELECT status,owner_email,size_bytes FROM video.videos WHERE id=$1")
                .bind(created_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored, ("pending_upload".into(), email.into(), 42));
        cleanup_videos(&pool, &[created_id]).await;
    }

    #[tokio::test]
    async fn upload_completion_checks_owner_object_size_and_idempotent_state() {
        if env::var("OPENTUBE_TEST_S3_ENDPOINT").is_err()
            || env::var("AWS_ACCESS_KEY_ID").is_err()
            || env::var("AWS_SECRET_ACCESS_KEY").is_err()
        {
            return;
        }
        let Some(pool) = test_pool().await else {
            return;
        };
        let owner_id = Uuid::new_v4();
        let pending_id = Uuid::new_v4();
        let uploaded_id = Uuid::new_v4();
        let ids = [pending_id, uploaded_id];
        insert_owned_video(&pool, pending_id, owner_id, "pending_upload").await;
        insert_owned_video(&pool, uploaded_id, owner_id, "uploaded").await;
        let app = app_router(test_state(pool.clone()).await);
        let token =
            issue_token(owner_id, "coverage-owner@example.test", TEST_JWT_SECRET, 10).unwrap();
        let call = |id: Uuid, bearer: &str| {
            Request::builder()
                .method("POST")
                .uri(format!("/v1/videos/{id}/complete"))
                .header("authorization", format!("Bearer {bearer}"))
                .body(Body::empty())
                .unwrap()
        };

        let missing = app
            .clone()
            .oneshot(call(Uuid::new_v4(), &token))
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
        let wrong_owner = app
            .clone()
            .oneshot(call(pending_id, &creator_token()))
            .await
            .unwrap();
        assert_eq!(wrong_owner.status(), StatusCode::NOT_FOUND);
        let already_completed = app
            .clone()
            .oneshot(call(uploaded_id, &token))
            .await
            .unwrap();
        assert_eq!(already_completed.status(), StatusCode::ACCEPTED);
        let missing_object = app.clone().oneshot(call(pending_id, &token)).await.unwrap();
        assert_eq!(missing_object.status(), StatusCode::CONFLICT);

        let state: (String,) = sqlx::query_as("SELECT status FROM video.videos WHERE id=$1")
            .bind(pending_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(state.0, "pending_upload");
        cleanup_videos(&pool, &ids).await;
    }

    #[tokio::test]
    async fn public_feed_search_and_cursor_pagination_use_ready_videos_only() {
        let Some(pool) = test_pool().await else {
            return;
        };
        sqlx::query("DELETE FROM video.videos WHERE owner_email='coverage@example.test' AND title IN ('Quiet lake','Night observatory','Cloud atlas','Unpublished observatory') AND created_at > now() - interval '30 minutes'")
            .execute(&pool)
            .await
            .unwrap();
        let ids = [
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let marker = format!("coverage{}", ids[0].simple());
        let now = Utc::now();
        insert_video(
            &pool,
            ids[0],
            "ready",
            Some(now - chrono::Duration::seconds(3)),
            &format!("{marker} quiet lake"),
            "Morning light",
        )
        .await;
        insert_video(
            &pool,
            ids[1],
            "ready",
            Some(now - chrono::Duration::seconds(2)),
            &format!("{marker} night observatory"),
            "A telescope",
        )
        .await;
        insert_video(
            &pool,
            ids[2],
            "ready",
            Some(now - chrono::Duration::seconds(1)),
            &format!("{marker} cloud atlas"),
            "Weather maps",
        )
        .await;
        insert_video(
            &pool,
            ids[3],
            "uploaded",
            None,
            &format!("{marker} unpublished observatory"),
            "Still processing",
        )
        .await;
        let app = app_router(test_state(pool.clone()).await);

        let first = app
            .clone()
            .oneshot(request(
                "GET",
                &format!("/v1/videos?query={marker}&limit=2"),
                String::new(),
            ))
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let page: VideoPage =
            serde_json::from_slice(&to_bytes(first.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].id, ids[2]);
        assert_eq!(page.items[1].id, ids[1]);
        assert!(page.items.iter().all(|v| !v.title.contains("Unpublished")));
        assert!(
            page.items[0]
                .playback_url
                .ends_with(&format!("{}/hls/master.m3u8", ids[2]))
        );
        let cursor = page.next_cursor.expect("second page is available");

        let second = app
            .clone()
            .oneshot(request(
                "GET",
                &format!("/v1/videos?query={marker}&limit=2&cursor={cursor}"),
                String::new(),
            ))
            .await
            .unwrap();
        let page_two: VideoPage =
            serde_json::from_slice(&to_bytes(second.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(
            page_two.items.iter().map(|v| v.id).collect::<Vec<_>>(),
            [ids[0]]
        );
        assert!(page_two.next_cursor.is_none());

        let search = app
            .clone()
            .oneshot(request(
                "GET",
                &format!("/v1/videos?query={marker}%20observatory"),
                String::new(),
            ))
            .await
            .unwrap();
        let search_page: VideoPage =
            serde_json::from_slice(&to_bytes(search.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(search_page.items.len(), 1);
        assert_eq!(search_page.items[0].id, ids[1]);

        let invalid_cursor = app
            .clone()
            .oneshot(request(
                "GET",
                &format!("/v1/videos?query=other&cursor={cursor}"),
                String::new(),
            ))
            .await
            .unwrap();
        assert_eq!(invalid_cursor.status(), StatusCode::BAD_REQUEST);
        let malformed_cursor = app
            .clone()
            .oneshot(request("GET", "/v1/videos?cursor=bad", String::new()))
            .await
            .unwrap();
        assert_eq!(malformed_cursor.status(), StatusCode::BAD_REQUEST);

        let detail = app
            .clone()
            .oneshot(request(
                "GET",
                &format!("/v1/videos/{}", ids[1]),
                String::new(),
            ))
            .await
            .unwrap();
        assert_eq!(detail.status(), StatusCode::OK);
        let unpublished = app
            .clone()
            .oneshot(request(
                "GET",
                &format!("/v1/videos/{}", ids[3]),
                String::new(),
            ))
            .await
            .unwrap();
        assert_eq!(unpublished.status(), StatusCode::NOT_FOUND);

        cleanup_videos(&pool, &ids).await;
    }

    #[tokio::test]
    async fn processing_callbacks_are_authenticated_idempotent_and_publish_ready_videos() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let ready_id = Uuid::new_v4();
        let failed_id = Uuid::new_v4();
        let ids = [ready_id, failed_id];
        insert_video(&pool, ready_id, "uploaded", None, "Ready callback", "").await;
        insert_video(&pool, failed_id, "processing", None, "Failed callback", "").await;
        let app = app_router(test_state(pool.clone()).await);
        let callback_path = format!("/internal/videos/{ready_id}/processing-result");

        let unauthorized = app
            .clone()
            .oneshot(request("POST", &callback_path, "{}".into()))
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let tokenized = |id: Uuid, idempotency_key: Option<&str>, payload: &str| {
            let mut builder = Request::builder()
                .method("POST")
                .uri(format!("/internal/videos/{id}/processing-result"))
                .header("content-type", "application/json")
                .header("authorization", "Bearer test-worker-token");
            if let Some(key) = idempotency_key {
                builder = builder.header("idempotency-key", key);
            }
            builder.body(Body::from(payload.to_owned())).unwrap()
        };
        let missing_key = app
            .clone()
            .oneshot(tokenized(ready_id, None, "{}"))
            .await
            .unwrap();
        assert_eq!(missing_key.status(), StatusCode::BAD_REQUEST);
        let invalid_json = app
            .clone()
            .oneshot(tokenized(ready_id, Some("ready-event"), "{"))
            .await
            .unwrap();
        assert_eq!(invalid_json.status(), StatusCode::BAD_REQUEST);
        let invalid_prefix = app
            .clone()
            .oneshot(tokenized(
                ready_id,
                Some("invalid-prefix"),
                &json!({"result":"ready","hls_prefix":"videos/other/hls"}).to_string(),
            ))
            .await
            .unwrap();
        assert_eq!(invalid_prefix.status(), StatusCode::BAD_REQUEST);

        let ready_payload = json!({"result":"ready","hls_prefix":format!("videos/{ready_id}/hls"),"failure_code":null}).to_string();
        let ready = app
            .clone()
            .oneshot(tokenized(ready_id, Some("ready-event"), &ready_payload))
            .await
            .unwrap();
        assert_eq!(ready.status(), StatusCode::OK);
        let duplicate = app
            .clone()
            .oneshot(tokenized(ready_id, Some("ready-event"), &ready_payload))
            .await
            .unwrap();
        assert_eq!(duplicate.status(), StatusCode::OK);
        let conflict = app
            .clone()
            .oneshot(tokenized(
                failed_id,
                Some("ready-event"),
                &json!({"result":"failed","failure_code":"conflict"}).to_string(),
            ))
            .await
            .unwrap();
        assert_eq!(conflict.status(), StatusCode::CONFLICT);

        let failed = app
            .clone()
            .oneshot(tokenized(
                failed_id,
                Some("failed-event"),
                &json!({"result":"failed","failure_code":"bad input!"}).to_string(),
            ))
            .await
            .unwrap();
        assert_eq!(failed.status(), StatusCode::OK);
        let stored: (String, Option<String>) =
            sqlx::query_as("SELECT status,failure_code FROM video.videos WHERE id=$1")
                .bind(failed_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored, ("failed".into(), Some("badinput".into())));

        let invalid_result = app
            .clone()
            .oneshot(tokenized(
                failed_id,
                Some("unknown-event"),
                &json!({"result":"unknown"}).to_string(),
            ))
            .await
            .unwrap();
        assert_eq!(invalid_result.status(), StatusCode::BAD_REQUEST);

        cleanup_videos(&pool, &ids).await;
    }
}
