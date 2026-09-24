# OpenTube repository guidance

> Human guide: [README.md](README.md) · [Documentation index](docs/README.md)

## Product and boundaries

- This is a local educational MVP, not a public video platform. Use synthetic test media and local-only secrets.
- Preserve the bounded contexts: identity owns account credentials; video owns upload and public-video lifecycle; the worker owns transcoding execution.
- Each application directory owns its README.md and AGENTS.md. Do not add shared database reads across service schemas.
- Keep the implementation small. Use DDD concepts where they clarify ownership; do not add event sourcing, a service mesh, Redis, a recommendation engine, or full CQRS without a requirement.
- Only ready videos are public. Feed pages use stable keyset cursors and never repeat an item across pages.

## Engineering rules

- Backend services and the worker are Rust. Web is Vue 3/TypeScript. Android phone and TV are Kotlin/Compose.
- Add focused unit, integration, and end-to-end tests for requested behaviors. Run the relevant checks and record the actual command, revision, environment, and result under `docs/verification/`.
- Keep dependencies and tool versions explicit. Never commit `.env`, tokens, passwords, signing keys, local Android paths, generated media, or build outputs.
- Use conventional commit messages in the form required by Conventional Commits 1.0.0.
- Do not claim production-scale capacity, encryption, availability, or test results from a local Kind run.

## Local workflow

- `cargo test --workspace`
- `cd web-frontend && pnpm test`
- Build each Android app with its checked-in Gradle wrapper and run instrumented tests on the matching emulator.
- Deploy through `deploy/helm/opentube` into the `opentube` namespace only. Do not alter the `notification-system` namespace or its resources.
- Expose local services on loopback. Use `adb reverse` for Android emulator access.
