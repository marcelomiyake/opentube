CREATE SCHEMA IF NOT EXISTS video;

CREATE TABLE IF NOT EXISTS video.videos (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    owner_email TEXT NOT NULL,
    title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 120),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 2000),
    content_type TEXT NOT NULL CHECK (content_type = 'video/mp4'),
    size_bytes BIGINT NOT NULL CHECK (size_bytes BETWEEN 1 AND 1073741824),
    source_object_key TEXT NOT NULL UNIQUE,
    hls_prefix TEXT,
    status TEXT NOT NULL CHECK (status IN ('pending_upload', 'uploaded', 'processing', 'ready', 'failed')),
    failure_code TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at TIMESTAMPTZ,
    search_vector TSVECTOR GENERATED ALWAYS AS (
        to_tsvector('simple', coalesce(title, '') || ' ' || coalesce(description, ''))
    ) STORED,
    CHECK ((status = 'ready') = (published_at IS NOT NULL))
);

CREATE INDEX IF NOT EXISTS videos_publication_cursor_idx
    ON video.videos (published_at DESC, id DESC) WHERE status = 'ready';
CREATE INDEX IF NOT EXISTS videos_search_idx ON video.videos USING GIN (search_vector);

CREATE TABLE IF NOT EXISTS video.outbox (
    id UUID PRIMARY KEY,
    aggregate_id UUID NOT NULL REFERENCES video.videos(id),
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    delivered_at TIMESTAMPTZ,
    attempts INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS outbox_pending_idx ON video.outbox (created_at) WHERE delivered_at IS NULL;

CREATE TABLE IF NOT EXISTS video.processing_callbacks (
    idempotency_key TEXT PRIMARY KEY,
    video_id UUID NOT NULL REFERENCES video.videos(id),
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
