# Dependency and Node 26.13.1 update verification

**Collected:** 2026-11-15 (America/Sao_Paulo)
**Repository revision:** `8eb9e3a` (branch `chore/node-26.13.1-security-updates`) plus the follow-up commit that adds the Gradle verification metadata and this record
**Environment:** Linux x86_64; Node.js 26.13.1; pnpm 12.6.0; `rustc`/`cargo` 1.100.0; OpenJDK 25.0.5; Gradle 9.4.1 (checked-in wrappers)

> Project documentation index: [Documentation index](../README.md) · [Verification record](README.md)

This record covers the Node 26.13.1 pin and the dependency updates for known advisories. It does not refresh the Kind, emulator, Lighthouse or SonarQube results in [the observed results](results.md).

## Changes verified

- Web: Node 26.13.1 build image; Vitest and `@vitest/coverage-v8` 3 → 4.1.12; Happy DOM 20 → 21.0.4; Vite 7.3.8; `glob`/`minimatch` overrides in `pnpm-workspace.yaml`.
- Rust: `rustls` 0.23.46, `rustls-webpki` 0.103.17, `webpki-roots` 1.0.12, `h2` 0.4.20, `ctutils` 0.4.8, `tokio` 1.54.1. `aws-sdk-s3` now omits its legacy `rustls` (hyper 0.14) and `sigv4a` default features, which removes `rustls` 0.21, `h2` 0.3 and `crypto-bigint` 0.5 from the tree.
- Android (phone and TV): a Guava 33.7.2-android constraint, because `media3` resolved the vulnerable Guava 33.3.1-android (GHSA-xxph-c9ww-hj94). The lockfiles were rewritten, and the 4 new artifacts were added to `gradle/verification-metadata.xml`. Their JAR SHA-1 values match Maven Central.

## Observed results

| Area | Command | Observed result |
| --- | --- | --- |
| Rust advisories | `cargo deny check advisories` | 0 vulnerabilities. Remaining notices are unmaintained crates only: `instant`, `pin-utils`, `rustls-pemfile`. |
| Rust workspace tests | `SQLX_OFFLINE=true cargo test --workspace --locked` | Did not complete. The `transcoding-worker` test binary fails to load `libssl.so.4` on this host. The same failure occurs on `main` at the pre-change revision, so this run is not evidence for the Rust changes. Service-backed integration tests were not run. |
| Web dependency audit | `cd web-frontend && pnpm audit` | No known vulnerabilities. |
| Web unit/component tests | `cd web-frontend && pnpm test` | 24 passed across 3 files. |
| Web production build | `cd web-frontend && pnpm build` | Passed. |
| Android phone unit tests | `cd android-mobile-frontend && ./gradlew testDebugUnitTest` | 6 passed, 0 failed. |
| Android TV unit tests | `cd android-tv-frontend && ./gradlew testDebugUnitTest` | 3 passed, 0 failed. |
| Android release builds | `./gradlew assembleRelease` in each Android app | Passed with dependency verification enabled. |
| Android shipped dependencies | OSV query of every `releaseRuntimeClasspath` entry in `app/gradle.lockfile` | 0 known vulnerabilities in either app. |

Not run for this change: web browser E2E, Android emulator suites, Helm lint, and the Kind flow. Build-tool configurations in the Android lockfiles (Android lint, the Unified Test Platform) still resolve components with published advisories. Those components are not packaged in the APKs.
