# transcoding-worker guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

- Treat RabbitMQ delivery as at-least-once. Derive output keys from video ID and make processing idempotent.
- Ack only after outputs are durable and the video-service callback succeeds.
- Retry transient errors up to the configured bound; dead-letter terminal failures.
- Never publish or expose a video from the worker. Only the video service commits the `ready` state.
- Do not log signed URLs, storage credentials, tokens, or file contents.
- Test valid synthetic media, malformed input, retry exhaustion, duplicate delivery, and partial object-store writes.
