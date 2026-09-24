# Verification Results

**Collected:** 2026-09-25 (America/Sao_Paulo)
**Repository revision:** `e5c1b578a742a03d24e5e2fa83540a537cdeea3c` plus uncommitted implementation and verification changes
**Environment:** Linux x86_64; SonarQube Community Build `26.9.0.129388`; existing Kind context `kind-kind`, namespace `opentube`; Android API 37 x86_64 emulator.

> Project documentation index: [Documentation index](../README.md)

The Android and web test/coverage refresh and SonarQube re-analysis completed on 2026-09-25. The Kind end-to-end run and service-backed Rust coverage run are earlier successful local runs recorded below. All reported results use synthetic media and local services.

## Test and deployment results

| Area | Command or environment | Observed result |
| --- | --- | --- |
| Rust formatting | `cargo fmt --all --check` | Passed. |
| Rust workspace tests | `cargo test --workspace` | Passed with local PostgreSQL, RabbitMQ, and MinIO integration dependencies. |
| Rust coverage imported by the latest Sonar analysis | `cargo llvm-cov --workspace --lcov --output-path .local/coverage/rust.lcov -- --test-threads=1` with disposable PostgreSQL, RabbitMQ, and MinIO services configured | 23 tests passed across identity, platform-core, video, and worker. Sonar measures: identity 81.4%, video 84.5%, worker 86.0%, platform-core 100.0%. Identity and video used separate test databases for their separate migration histories. |
| Web unit/component tests | `cd web-frontend && pnpm test` | 24 passed across 3 files, including 2 WebMCP tests. Latest Vitest line coverage: 90.84% total (`App.vue` 88.97%, `api.ts` 100%). |
| Web production build | `cd web-frontend && pnpm build` | Passed. Vite warned that the minified JavaScript bundle is 672.86 kB. |
| Web browser E2E | `cd web-frontend && pnpm e2e` | 4 passed: multi-page feed without duplicates, HLS playlist request, search, and creator upload flow. |
| Android phone unit tests | `cd android-mobile-frontend && ./gradlew testDebugUnitTest` | 6 passed: 5 API-client tests and 1 video-model test. |
| Android phone unit coverage | `cd android-mobile-frontend && ./gradlew createDebugUnitTestCoverageReport` | JaCoCo unit report generated. |
| Android phone emulator | `cd android-mobile-frontend && ./gradlew createDebugCoverageReport -PidentityBaseUrl=http://127.0.0.1:18083 -PvideoBaseUrl=http://127.0.0.1:19082` | 6 passed on API 37: populated fixture feed and playback view, empty search results, search API error, sign-in, and upload flows. The test identity/video stubs are loopback-only. |
| Android TV unit tests | `cd android-tv-frontend && ./gradlew testDebugUnitTest` | 3 passed: 2 API-client tests and 1 video-model test. |
| Android TV emulator | `cd android-tv-frontend && ./gradlew createDebugCoverageReport -PidentityBaseUrl=http://127.0.0.1:18083 -PvideoBaseUrl=http://127.0.0.1:19082` | 4 passed on API 37: populated fixture browsing/playback, empty search results, and search API error. The test identity/video stubs are loopback-only. |
| Helm | `helm lint deploy/helm/opentube` | 1 chart linted, 0 failures. |
| Kind flow | `scripts/kind-e2e.sh` on `kind-kind`, namespace `opentube` | Passed validation, seeded login, MinIO upload, RabbitMQ/FFmpeg processing, ready publication, repeat-safe callback, HLS master/variant/segment requests, PostgreSQL ready state, and cursor feed checks. Pagination returned 2 pages with 26 distinct seeded videos. |

The mobile HLS test opens the playback view with a synthetic HLS fixture; it does not assert rendered frames or playback quality. The Kind test verifies generated HLS files and HTTP delivery, not production playback, throughput, or availability. The latest Kind check used a generated synthetic MP4 fixture.

## SonarQube

The latest analysis of all seven OpenTube Sonar projects is recorded in [SonarQube verification](sonarqube.md). All seven overall coverage measures exceed 80%, all seven server quality gates pass, and all seven have zero duplicated lines and open issues. The Android refresh added deterministic fixture-driven browse, playback, and empty/error search coverage; exact aggregate, line, and branch measures are listed in the linked report.

## Runtime and tool versions

| Tool | Version observed |
| --- | --- |
| `rustc` / `cargo` | 1.98.1 / 1.98.1 |
| Node.js / pnpm | 24.21.0 / 11.19.0 |
| Java | 21.0.12.1 |
| Gradle wrapper | 9.4.1 |
| Android emulator / image | Emulator 37.1.11.0; Android API 37 Google APIs x86_64 |
| Kind | v0.33.0 |
| Helm | v4.3.0+gbec5b06 |
| SonarQube | Community Build 26.9.0.129388 |

At observation time, the `identity-service`, `video-service`, `transcoding-worker`, and `web-frontend` Deployments each had 2/2 available replicas. PostgreSQL, RabbitMQ, and MinIO each had one running replica. This is the local Kind profile only.

## Remaining evidence gaps

- Android UI tests open the fixture playback view but do not assert decoded frames or sustained playback.
- The local OpenDesign URL `http://127.0.0.1:17758/` refused connections during the design run, so generated prototypes are not included.
- The local profile does not establish production TLS, encryption at rest, backup and recovery, traffic capacity, or scale behavior.
- Sonar did not provide Git blame/new-code coverage for uncommitted files; the reported coverage is overall project coverage.

The ByteByteGo scenario figures in the system design remain scenario assumptions, not measured OpenTube data. The GPT-6 Luna (max) statement is a requested workflow disclaimer; model use is not independently verified.
