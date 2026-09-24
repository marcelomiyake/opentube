# SonarQube verification

## Scope

- **Server:** SonarQube Community Build 26.9.0.129388 at `http://127.0.0.1:9000` (server status `UP`).
- **Latest analysis:** 2026-09-25, submitted 18:08–18:10 America/Sao_Paulo after the architecture-fit Markdown updates, based on `e5c1b578a742a03d24e5e2fa83540a537cdeea3c` plus the uncommitted worktree; no commit was created.
- **Command:** `./scripts/sonar-scan-all.sh` with `SONAR_TOKEN` supplied through the environment.
- **Reports:** service-backed Rust LCOV, web Vitest LCOV, and Android unit/instrumented JaCoCo reports generated from the current source.

> Project documentation index: [Documentation index](../README.md)

## Results

The latest scan submitted all seven OpenTube analyses. SonarQube reports every quality gate as **OK**, with zero duplicated lines, bugs, vulnerabilities, code smells, or security hotspots for each project.

| Sonar project | Coverage | Line coverage | Branch coverage | Quality gate |
|---|---:|---:|---:|---|
| [`opentube-identity-service`](http://127.0.0.1:9000/dashboard?id=opentube-identity-service) | **81.4%** | 81.4% | Not reported | **OK** |
| [`opentube-video-service`](http://127.0.0.1:9000/dashboard?id=opentube-video-service) | **84.5%** | 84.5% | Not reported | **OK** |
| [`opentube-transcoding-worker`](http://127.0.0.1:9000/dashboard?id=opentube-transcoding-worker) | **86.0%** | 86.0% | Not reported | **OK** |
| [`opentube-platform-core`](http://127.0.0.1:9000/dashboard?id=opentube-platform-core) | **100.0%** | 100.0% | Not reported | **OK** |
| [`opentube-web-frontend`](http://127.0.0.1:9000/dashboard?id=opentube-web-frontend) | **88.4%** | 87.4% | 90.3% | **OK** |
| [`opentube-android-mobile-frontend`](http://127.0.0.1:9000/dashboard?id=opentube-android-mobile-frontend) | **87.2%** | 94.0% | 72.7% | **OK** |
| [`opentube-android-tv-frontend`](http://127.0.0.1:9000/dashboard?id=opentube-android-tv-frontend) | **93.6%** | 100.0% | 81.5% | **OK** |

The aggregate Sonar **Coverage** metric is above 80% for all seven projects. The mobile app's branch coverage is 72.7%; branch coverage is a separate measure and does not change the reported aggregate metric or server gate result.

## Coverage inputs and tests

- **Rust:** `cargo llvm-cov --workspace --lcov --output-path .local/coverage/rust.lcov -- --test-threads=1` ran with disposable PostgreSQL, RabbitMQ, and MinIO test services configured. Identity (2), platform core (2), video (8), and worker (11) tests passed. Identity and video use separate test databases because their migrations are independently versioned. The test setup now creates the configured MinIO bucket when absent.
- **Web:** the current Vitest LCOV report was imported; all 24 frontend unit/component tests passed, including WebMCP checks.
- **Android mobile:** 6 unit tests and 6 API 37 emulator tests passed. Instrumented cases exercise a populated fixture feed and playback view, search empty/error states, login, and upload flows.
- **Android TV:** 3 unit tests and 4 API 37 emulator tests passed. Instrumented cases exercise fixture browsing/playback and search empty/error states.
- **Analysis:** `./scripts/sonar-scan-all.sh` submitted all seven current project reports successfully. Quality-gate and measure APIs were queried after server processing.

## Limits

Some Android JaCoCo imports warn about Compose dependency files (`Effects.kt` and `LazyDsl.kt`) outside the configured project sources; application sources were analyzed and the reports imported. The mobile branch measure remains below 80%, though its overall coverage is above 80%. SonarQube may have incomplete new-code attribution for uncommitted files. Community Build results cover enabled analyzers and are not a comprehensive security audit or production runtime measurement.
