# identity-service guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

- Own only the `identity` PostgreSQL schema. Never read video-service tables.
- Hash passwords with Argon2id; never log credentials or return password hashes.
- Keep access tokens short-lived and validate configuration on startup.
- Add unit tests for password verification and endpoint tests for valid/invalid login.
- Seed only the explicitly configured local demo creator; do not add registration or account recovery.
