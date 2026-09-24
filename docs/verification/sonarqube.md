# SonarQube Cloud verification

`OpenTube Rust` analyzes the whole Cargo workspace, while `identity-service`, `video-service`, and `transcoding-worker` each have a separate SonarQube Cloud project. The [GitHub Actions workflow](https://github.com/marcelomiyake/opentube/actions/workflows/sonarcloud-microservices.yml) runs one workspace job and one job per service on pushes to `main`; `workflow_dispatch` supports a manual rerun. Organization-level auto-import of newly created GitHub repositories is disabled, so analysis is managed by this workflow.

Each job provisions PostgreSQL, RabbitMQ, and RustFS so database, queue, and object-store integration tests execute instead of returning early. The workspace job runs `cargo llvm-cov --workspace --lcov --output-path target/coverage/lcov.info -- --test-threads=1` from the repository root; service jobs run the same command from their service directory without `--workspace`. Each job imports its LCOV report with `cargo sonar-scanner` and reads the GitHub repository secret named `SONAR_TOKEN`. All service code, including `src/main.rs`, remains in the coverage denominator.

## Projects and coverage

Coverage below is SonarCloud's overall line coverage for `main`, not new-code or local coverage. The baseline is the latest Cloud result before the coverage-test updates; the current column is the latest result after them. Values are from the 2026-09-27 snapshot.

| Microservice | SonarCloud project | Before | Current | Change |
| --- | --- | ---: | ---: | ---: |
| `identity-service` | [project](https://sonarcloud.io/project/overview?id=marcelomiyake_opentube_identity-service) | 81.4% | 84.3% | +2.9 pp |
| `video-service` | [project](https://sonarcloud.io/project/overview?id=marcelomiyake_opentube_video-service) | 84.6% | 85.8% | +1.2 pp |
| `transcoding-worker` | [project](https://sonarcloud.io/project/overview?id=marcelomiyake_opentube_transcoding-worker) | 87.6% | 88.0% | +0.4 pp |

The aggregate [OpenTube Rust project](https://sonarcloud.io/project/overview?id=marcelomiyake_opentube-rust) previously reported 28.6% coverage (2,074 lines to cover, 1,480 uncovered) on revision `50cbfbfb762e7341ca2b5ff6dd7fed486d9efc71`. The first workspace analysis on `270dd88e931c750c98dc432c43edc071342c1336` raised the dashboard to 59.1% (3,251 lines to cover, 1,329 uncovered) and passed its Quality Gate, but its file list included 78 Rust, Kotlin, web, and configuration files. The workspace job now sets `sonar.inclusions=**/*.rs`; the corrected analysis on `e757cebc58a8fb73bd7741acb339f1e35f58b46f` lists only five Rust files and reports 86.8% overall coverage (2,215 lines to cover, 293 uncovered). Its new-code coverage is 96.6% across 175 lines (6 uncovered), and the Quality Gate passed. GitHub Actions run [36317691532](https://github.com/marcelomiyake/opentube/actions/runs/36317691532) completed successfully for that revision.

The three service projects have passing Quality Gates, zero open or confirmed issues, zero hotspots awaiting review, zero bugs, zero vulnerabilities, zero code smells, and 0.0% duplicated lines.

## Aggregate workspace verification

At repository revision `0ea69024c6157d37de4d45f3936eaa031ce84f71`, the workspace coverage command passed all 24 tests and reported 85.85% line coverage across 2,367 lines:

```sh
cargo llvm-cov --workspace --summary-only -- --test-threads=1
```

The LCOV generation command used by the new workspace job also passed and wrote `target/coverage/lcov.info`:

```sh
cargo llvm-cov --workspace --lcov --output-path target/coverage/lcov.info -- --test-threads=1
```

Both runs used Rust 1.98.1, Cargo 1.98.1, and `cargo-llvm-cov` 0.9.1. PostgreSQL 17, RabbitMQ 4.1, and RustFS 1.0 test containers were available on loopback at ports 15432, 15672, and 19000. The test variables pointed to separate identity and video databases, the RabbitMQ test vhost, and the RustFS bucket `opentube-ci`; AWS credentials were synthetic test values and metadata lookup was disabled.

The scanner dry run resolved the repository root, requested project key, Rust-only inclusion pattern, and `target/coverage/lcov.info`. A direct upload using the available local `SONAR_TOKEN` was rejected because that token cannot execute analysis in the organization. The GitHub Actions repository secret authorized the aggregate upload, and run [36317691532](https://github.com/marcelomiyake/opentube/actions/runs/36317691532) submitted the Rust-only report on revision `e757cebc58a8fb73bd7741acb339f1e35f58b46f`. SonarCloud's file tree for that analysis contains only `*.rs` sources.

## Verification

Confirm the latest workflow completed for the pushed commit and each project's `main` analysis matches that revision. Review coverage, active issues, security hotspots, duplication, and the Quality Gate. Project links above open the live dashboards; the workflow link shows the run history. Local coverage does not replace a completed Cloud analysis. Earlier local Sonar results are historical and are kept separately in [`results.md`](results.md).
