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
- SonarQube Cloud analyzes `identity-service`, `video-service`, and `transcoding-worker` as separate monorepo projects through `.github/workflows/sonarcloud-microservices.yml`. Pushes to `main` start one test-and-coverage analysis job per service using this repository’s `SONAR_TOKEN` secret. Verify each analyzed revision, findings, coverage, and Quality Gate in [the verification guide](docs/verification/sonarqube.md).
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

## Releases

`.github/workflows/release.yml` runs [release-please](https://github.com/googleapis/release-please) on every push to `main`. From the Conventional Commits merged since the last tag, it keeps one release pull request open, titled `chore: release X.Y.Z`:

- `fix`, `perf` and `revert` make a patch release, `feat` a minor release, and a breaking change (`!`) a major release. The other types release nothing on their own and stay out of the changelog.
- It writes the version to `version.txt` and adds the release to `CHANGELOG.md`.
- Merging it tags `vX.Y.Z` and publishes the GitHub release.
- GitHub runs no workflow on that pull request, because the workflow's own token opened it. The other workflows run on `main` once it merges.

Never change the version or `CHANGELOG.md` by hand. `.release-please-manifest.json` holds the last released version, and `release-please-config.json` the settings. To force a version, add a `Release-As: X.Y.Z` footer to a commit.
