# JEV Local MVP Readiness

> **Historical snapshot:** this score was obtained before the 2026-09-24 Sonar and coverage fixes. Its level-2 evidence describes that earlier state; current measurements are in [verification results](results.md). The JEV assessment has not been rerun for the updated working tree.

> Project documentation index: [Documentation index](../README.md)

**Requested model alias:** `jev-latest`
**Returned model:** `jev-1.13.0`
**Assessment date:** 2026-09-24 UTC
**Question:** How ready is this project to be accepted as a verified local learning MVP, based on reproducibility of the core upload-to-playback flow and completeness of verification evidence? Production readiness is a separate question.

## Result

| Score | Confidence | Probability distribution |
| ---: | ---: | --- |
| 2.0 | 1.0 | Level 0: 0.0; level 1: 0.0; level 2: 1.0; level 3: 0.0; level 4: 0.0 |

The returned score places OpenTube at level 2: the local Kind flow and major test suites are verified, while important verification gaps remain. This is an advisory judgment, not a replacement for the test report or SonarQube gate.

## Rubric sent to JEV

| Level | Description |
| ---: | --- |
| 0 | No coherent design or reproducible local video flow is demonstrated. |
| 1 | Design and implementation exist, but the core upload, processing, publication, and playback flow is not verified end to end. |
| 2 | The core local flow and major test suites are verified, but significant verification gaps remain, such as unmet coverage targets or missing client playback E2E evidence. |
| 3 | The full local acceptance suite and all stated quality thresholds are verified, including coverage, duplicate code, security issue, and hotspot criteria. |
| 4 | Production readiness is independently supported by security, backup and recovery, capacity, scale, and operational evidence in addition to the complete local acceptance suite. |

## Evidence considered

- Kind E2E passed seeded creator login, upload validation, synthetic MP4 upload to MinIO, RabbitMQ/FFmpeg processing, publication after processing, repeat-safe callback, PostgreSQL ready state, HLS master/variant/segment requests, and two cursor pages with 26 distinct videos.
- Rust unit tests: 8 passed. Web unit tests: 9 passed. Browser E2E: 4 passed. Android phone and TV each passed 1 unit test and 1 API 37 emulator browse/search launch smoke test. Helm lint passed.
- Seven SonarQube project analyses found zero blocker/critical issues, zero hotspots, and zero duplication. Overall coverage was below 80% in six projects; phone and TV new-code coverage were 39.8% and 49.1%, and their gates failed the 80% condition. Five other projects did not report new-code coverage.
- Android emulator tests do not verify playback. The local OpenDesign interface was unavailable, so generated prototypes are absent. Production security and capacity are outside this local profile.

The API returned `confidence=1.0`, meaning all probability was assigned to one level in this response. It does not guarantee that the judgment is correct. The API reported 957 input tokens and 22 output tokens. No numeric JEV pass threshold was defined; the score is not a release gate.

The score request used TypeSafe's `POST /v1/systemone` endpoint with model alias `jev-latest` and a single Score question. The API response was recorded without sending credentials or source code. See the [TypeSafe API reference](https://docs.typesafe.ai/api) and [Score primitive documentation](https://docs.typesafe.ai/primitives/score) for response semantics.
