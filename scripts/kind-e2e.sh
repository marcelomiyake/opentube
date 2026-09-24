#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
namespace=opentube
identity_url=http://127.0.0.1:18081
video_url=http://127.0.0.1:18082
fixture="$repo_root/.local-media/opentube-fixture.mp4"

"$repo_root/scripts/init-local-env.sh"
set -a
source .env
set +a
: "${OPENTUBE_SEED_EMAIL:?OPENTUBE_SEED_EMAIL is required}"
: "${OPENTUBE_SEED_PASSWORD:?OPENTUBE_SEED_PASSWORD is required}"
: "${OPENTUBE_WORKER_TOKEN:?OPENTUBE_WORKER_TOKEN is required}"

if [[ ! -f "$fixture" ]]; then "$repo_root/scripts/generate-fixture.sh"; fi

kubectl -n "$namespace" exec statefulset/postgres -- psql -U "$OPENTUBE_DATABASE_ADMIN_USER" -d opentube -v ON_ERROR_STOP=1 -c \
  "INSERT INTO video.videos (id,owner_id,owner_email,title,description,content_type,size_bytes,source_object_key,hls_prefix,status,published_at) SELECT gen_random_uuid(),'00000000-0000-0000-0000-000000000001','creator@opentube.local','pagination-check '||lpad(n::text,3,'0'),'seeded local cursor pagination test','video/mp4',1,'e2e-pagination-'||n,'videos/fixtures/hls','ready',now()-(n||' seconds')::interval FROM generate_series(1,26) AS n ON CONFLICT (source_object_key) DO NOTHING" >/dev/null

login_body="$(python3 -c 'import json,sys; print(json.dumps({"email":sys.argv[1],"password":sys.argv[2]}))' "$OPENTUBE_SEED_EMAIL" "$OPENTUBE_SEED_PASSWORD")"
access_token="$(curl --silent --show-error --fail "$identity_url/v1/login" -H 'Content-Type: application/json' -d "$login_body" | python3 -c 'import json,sys; print(json.load(sys.stdin)["access_token"])')"

invalid_type="$(curl --silent -o /dev/null -w '%{http_code}' "$video_url/v1/videos" -H "Authorization: Bearer $access_token" -H 'Content-Type: application/json' -d '{"title":"invalid","description":"","content_type":"text/plain","size_bytes":12}')"
oversized="$(curl --silent -o /dev/null -w '%{http_code}' "$video_url/v1/videos" -H "Authorization: Bearer $access_token" -H 'Content-Type: application/json' -d '{"title":"invalid","description":"","content_type":"video/mp4","size_bytes":1073741825}')"
[[ "$invalid_type" == 400 && "$oversized" == 400 ]] || { echo "Upload validation failed: content-type=$invalid_type size=$oversized" >&2; exit 1; }

title="OpenTube E2E $(date +%s)"
create_json="$(curl --silent --show-error --fail "$video_url/v1/videos" -H "Authorization: Bearer $access_token" -H 'Content-Type: application/json' -d "$(python3 -c 'import json,sys; print(json.dumps({"title":sys.argv[1],"description":"Synthetic Kind end-to-end fixture","content_type":"video/mp4","size_bytes":int(sys.argv[2])}))' "$title" "$(stat -c %s "$fixture")")")"
video_id="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])' <<<"$create_json")"
upload_url="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["upload_url"])' <<<"$create_json")"
curl --silent --show-error --fail -X PUT -H 'Content-Type: video/mp4' --data-binary "@$fixture" "$upload_url" >/dev/null
curl --silent --show-error --fail -X POST "$video_url/v1/videos/$video_id/complete" -H "Authorization: Bearer $access_token" >/dev/null

encoded_query="$(python3 -c 'import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1]))' "$title")"
published_json=''
for attempt in $(seq 1 120); do
  published_json="$(curl --silent --show-error --fail "$video_url/v1/videos?query=$encoded_query&limit=24")"
  if python3 -c 'import json,sys; sys.exit(0 if json.load(sys.stdin)["items"] else 1)' <<<"$published_json"; then break; fi
  sleep 2
done
playback_url="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["items"][0]["playback_url"])' <<<"$published_json")"
master="$(curl --silent --show-error --fail "$playback_url")"
grep -q '^#EXTM3U' <<<"$master"
variant_path="$(awk '/\.m3u8$/ {print; exit}' <<<"$master")"
variant_url="${playback_url%/*}/$variant_path"
playlist="$(curl --silent --show-error --fail "$variant_url")"
segment_path="$(awk '!/^#/ && /\.ts$/ {print; exit}' <<<"$playlist")"
curl --silent --show-error --fail --head "${variant_url%/*}/$segment_path" >/dev/null

event_id="$(kubectl -n "$namespace" exec statefulset/postgres -- psql -U "$OPENTUBE_DATABASE_ADMIN_USER" -d opentube -Atqc "SELECT payload->>'event_id' FROM video.outbox WHERE payload->>'video_id'='$video_id' ORDER BY created_at DESC LIMIT 1")"
callback_body="{\"result\":\"ready\",\"hls_prefix\":\"videos/$video_id/hls\",\"failure_code\":null}"
for _ in 1 2; do
  code="$(curl --silent --output /dev/null --write-out '%{http_code}' -X POST "$video_url/internal/videos/$video_id/processing-result" -H "Authorization: Bearer $OPENTUBE_WORKER_TOKEN" -H "Idempotency-Key: $event_id" -H 'Content-Type: application/json' -d "$callback_body")"
  [[ "$code" == 200 ]] || { echo "Repeated processing callback returned $code" >&2; exit 1; }
done

ready_count="$(kubectl -n "$namespace" exec statefulset/postgres -- psql -U "$OPENTUBE_DATABASE_ADMIN_USER" -d opentube -Atqc "SELECT count(*) FROM video.videos WHERE id='$video_id' AND status='ready'")"
[[ "$ready_count" == 1 ]] || { echo "Published video was not persisted as ready." >&2; exit 1; }

node scripts/verify-feed-pages.mjs
printf 'Kind E2E passed: validation, creator login, MinIO upload, RabbitMQ processing, ready publication, repeat-safe callback, HLS master/variant/segment, PostgreSQL persistence, and cursor pages. Video ID: %s\n' "$video_id"
