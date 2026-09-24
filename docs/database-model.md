# OpenTube database model

This is the human-readable model of the service-owned PostgreSQL schemas. The migrations are authoritative; update this inventory when either migration changes.

- **Owner:** `opentube`; identity tables are owned by `opentube` / `identity-service`, and video tables are owned by `opentube` / `video-service`.
- **Known consumers:** `opentube` / `identity-service` owns and reads/writes `identity.users`; `opentube` / `video-service` owns video metadata and outbox records; `opentube` / `transcoding-worker` uses the video processing callback contract rather than direct database access. No cross-repository database consumer is identified.
- **Database:** PostgreSQL with service-specific `identity` and `video` schemas.
- **Sources:** [`identity-service/migrations/0001_identity.sql`](../identity-service/migrations/0001_identity.sql), [`video-service/migrations/0001_video.sql`](../video-service/migrations/0001_video.sql).

## Tables

### `identity.users`

| Column | Type and rules | Description |
| --- | --- | --- |
| `id` | UUID, primary key | User identifier and owner ID referenced by video metadata. No cross-schema foreign key is declared. |
| `email` | TEXT, required, unique | Login email. |
| `password_hash` | TEXT, required | Password hash; plaintext credentials are not stored. |
| `display_name` | TEXT, required | Creator name displayed by the system. |
| `created_at` | TIMESTAMPTZ, required, default `now()` | User creation time. |

### `video.videos`

| Column | Type and rules | Description |
| --- | --- | --- |
| `id` | UUID, primary key | Video identifier. |
| `owner_id` | UUID, required | Identity user ID stored as a cross-service reference. |
| `owner_email` | TEXT, required | Owner email snapshot used by video records. |
| `title` | TEXT, required, 1–120 characters | Public video title. |
| `description` | TEXT, required, default empty, at most 2,000 characters | Public video description. |
| `content_type` | TEXT, required, must equal `video/mp4` | Uploaded source media type. |
| `size_bytes` | BIGINT, required, 1–1,073,741,824 | Source file size; maximum is 1 GiB. |
| `source_object_key` | TEXT, required, unique | MinIO object key for uploaded source media. |
| `hls_prefix` | TEXT, nullable | MinIO prefix containing the processed HLS rendition. |
| `status` | TEXT, required; `pending_upload`, `uploaded`, `processing`, `ready`, or `failed` | Upload and transcoding lifecycle state. |
| `failure_code` | TEXT, nullable | Processing failure reason code. |
| `created_at` | TIMESTAMPTZ, required, default `now()` | Video record creation time. |
| `published_at` | TIMESTAMPTZ, nullable | Publication time; present only when the video is ready. |
| `search_vector` | TSVECTOR, generated and stored | Full-text index source from title and description using the `simple` configuration. |
| Status check | `ready` iff `published_at` is non-null | Keeps publication state consistent with readiness. |

A partial publication cursor index covers ready videos by `(published_at DESC, id DESC)`. A GIN index supports `search_vector` queries.

### `video.outbox`

| Column | Type and rules | Description |
| --- | --- | --- |
| `id` | UUID, primary key | Event record identifier. |
| `aggregate_id` | UUID, required, FK to `video.videos.id` | Video that produced the event. |
| `event_type` | TEXT, required | Event name, such as an upload or processing lifecycle event. |
| `payload` | JSONB, required | Versioned event payload. |
| `created_at` | TIMESTAMPTZ, required, default `now()` | Event creation time. |
| `delivered_at` | TIMESTAMPTZ, nullable | Broker publication time. |
| `attempts` | INTEGER, required, default `0` | Publication attempt count. |

### `video.processing_callbacks`

| Column | Type and rules | Description |
| --- | --- | --- |
| `idempotency_key` | TEXT, primary key | Deduplicates callback processing. |
| `video_id` | UUID, required, FK to `video.videos.id` | Video whose processing result is reported. |
| `payload` | JSONB, required | Callback result, including processing status and output location. |
| `created_at` | TIMESTAMPTZ, required, default `now()` | Callback receipt time. |

## Related documentation

- [System Design](system-design.md)
- [Contract catalog](contracts/README.md)
- [Documentation index](README.md)
