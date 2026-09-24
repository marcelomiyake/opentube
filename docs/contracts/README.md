# API and event contract catalog

This catalog identifies contracts implemented in OpenTube and the repository components that produce and consume them. [OpenAPI](../../contracts/openapi.yaml) and the service implementations are authoritative.

> Documentation: [project index](../README.md) · [repository overview](../../README.md)

## Contents

- [Contracts](#contracts)
- [Consumer map](#consumer-map)
- [Compatibility and security](#compatibility-and-security)
- [Related documentation](#related-documentation)
- [Document lifecycle](#document-lifecycle)
- [AI development disclaimer](#ai-development-disclaimer)

## Contracts

| Contract | Type and authority | Owner | Producer | Consumers |
| --- | --- | --- | --- | --- |
| Identity HTTP API | REST/JSON; [OpenAPI](../../contracts/openapi.yaml), `video-service/` | `opentube` / `identity-service` | `opentube` / `identity-service` | `opentube` / `web-frontend`, `opentube` / `android-mobile-frontend`, `opentube` / `android-tv-frontend` where authentication is required. |
| Video HTTP API | REST/JSON; [OpenAPI](../../contracts/openapi.yaml), `video-service/` | `opentube` / `video-service` | `opentube` / `video-service` | `opentube` / `web-frontend`, `opentube` / `android-mobile-frontend`, `opentube` / `android-tv-frontend`; `opentube` / `transcoding-worker` calls the internal processing-result callback. |
| WebMCP `search_public_videos` tool | Browser tool contract; [`web-frontend/src/webmcp.ts`](../../web-frontend/src/webmcp.ts) plus the public Video HTTP API | `opentube` / `web-frontend` | `opentube` / `web-frontend` registers the tool in the active document | A browser agent invoking the same-page `document.modelContext`; specific agent products are unknown. No cross-origin exposure is configured. |
| `video.uploaded.v1` | RabbitMQ event; [System Design](../system-design.md#6-api-and-data-contracts), `video-service/` and `transcoding-worker/` | `opentube` / `video-service` owns the outbox event; `transcoding-worker` owns processing behavior | `opentube` / `video-service` | `opentube` / `transcoding-worker`. |
| Processing-result callback | Internal REST/JSON; [OpenAPI](../../contracts/openapi.yaml) | `opentube` / `video-service` owns the state contract | `opentube` / `transcoding-worker` | `opentube` / `video-service`. |
| PostgreSQL schemas | SQL migrations in `identity-service/migrations/` and `video-service/migrations/` | Each service owns its schema; `video-service` owns its outbox | `opentube` / `identity-service` and `opentube` / `video-service` | The corresponding `opentube` service only; cross-schema reads are not permitted. |

## Consumer map

All known HTTP clients are components in this repository. The WebMCP tool is exposed only to an agent using the active web page; concrete browser-agent consumers are unknown. Other-repository API consumers are unknown; none is recorded in the tracked repositories. Android TV is a read/watch client and does not consume upload operations.

## Compatibility and security

Clients use REST/JSON. Creator operations require short-lived JWTs; the worker callback uses a separate local service token. Event delivery is at-least-once and processing-result callbacks are idempotent. Keep schemas service-owned and do not expose internal callback routes publicly.

## Related documentation

- [System Design](../system-design.md)
- [Project README](../../README.md)
- [Documentation index](../README.md)

## Document lifecycle

This contract catalog is maintained as Markdown and links to the implementation-owned interface definitions. It has no independent software build, deployment, or undeployment lifecycle. Review it when its linked contracts or consumers change.


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
