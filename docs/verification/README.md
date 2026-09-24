# Verification record

Verification reports distinguish planned checks from observed results. The synthetic test fixture is generated on demand with FFmpeg; no third-party or personal video is bundled. See [the observed results](results.md) and the separate [JEV readiness assessment](jev-readiness.md).

Browser quality and search metadata are reported in [Lighthouse and SEO META verification](lighthouse.md), separately from unit/KinD, SonarQube, and JEV evidence.

The current frontend scan and server quality-gate status are recorded in [SonarQube verification](sonarqube.md).

> Documentation: [project index](../README.md) · [repository overview](../../README.md)

## Commands

| Area | Command |
| --- | --- |
| Rust unit tests | `cargo test --workspace` |
| Web unit/component tests | `cd web-frontend && pnpm test` |
| Web production build | `cd web-frontend && pnpm build` |
| Web browser E2E | `cd web-frontend && pnpm e2e` |
| Android phone unit tests | `cd android-mobile-frontend && ./gradlew testDebugUnitTest` |
| Android TV unit tests | `cd android-tv-frontend && ./gradlew testDebugUnitTest` |
| Android phone unit and emulator coverage | `cd android-mobile-frontend && ./gradlew createDebugUnitTestCoverageReport createDebugCoverageReport` |
| Android TV unit and emulator coverage | `cd android-tv-frontend && ./gradlew createDebugUnitTestCoverageReport createDebugCoverageReport` |
| Helm validation | `helm lint deploy/helm/opentube` |
| Start local Kind profile | `scripts/kind-up.sh` |
| Kind integration/E2E | `scripts/kind-e2e.sh` |

The Kind flow deploys into namespace `opentube` on the existing `kind-kind` context. It does not create a cluster or modify the `notification-system` namespace. Port-forwards bind to loopback; Android emulator access uses `adb reverse`. The phone instrumentation suite uses the loopback identity/storage stub in `scripts/android-test-identity-server.py`; its feed/playback checks use a synthetic HLS fixture endpoint. Emulator UI tests exercise the playback view but do not assert rendered frames or playback quality.

## Recording results

For each verification run, record the command, commit/revision (or working-tree state), UTC timestamp, OS/tool versions, environment (local unit, emulator, or Kind), outcome, and relevant test output. Keep source interview sizing figures separate from observed data. SonarQube gate results and JEV readiness judgments are recorded separately from test results.

## Document lifecycle

This file documents verification commands and evidence. It has no independent software build, deployment, or undeployment lifecycle; run only the verification procedures that apply to the stated project and environment.


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
