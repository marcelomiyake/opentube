# video-service guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

- Own only the `video` schema. The only public status is `ready`.
- Use an outbox row in the same transaction as the `uploaded` transition.
- Use keyset pagination ordered by `(published_at DESC, id DESC)`; never offset-page the feed.
- Enforce the MP4 and 1 GiB upload boundary on the server. Do not proxy media bytes through the API.
- Keep object-store credentials, signed URLs, JWTs, and private error details out of logs.
- Add unit, PostgreSQL/S3 integration, and API tests for state changes, duplicate completion, and cursor stability.
