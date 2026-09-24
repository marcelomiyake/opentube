# web-frontend

Vue 3 and TypeScript web client served by Nginx. It provides public browsing, full-text search, an infinite-scroll feed with a load-more fallback, HLS playback, and authenticated MP4 upload for the seeded creator.

> Documentation: [project index](../docs/README.md) · [repository overview](../README.md)

The browser uses same-origin `/api` routes. The local frontend proxy routes API traffic to in-cluster services; browser uploads use the short-lived MinIO URL returned by video-service.

Run `pnpm install`, `pnpm dev`, `pnpm test`, and `pnpm e2e` from this directory.

## Browser agent support (WebMCP)

When the browser exposes the experimental WebMCP API, the page registers `search_public_videos`. It accepts a bounded text query and returns up to eight ready, public video records using the existing video API. Results include only the video ID, title, creator, and publication time; descriptions and playback URLs stay out of the tool result. The tool does not sign in, upload, start playback, or change data. User-supplied titles and creator names are marked as untrusted content. Registration is feature-detected and removed on component teardown; the feed remains usable in unsupported browsers. The implementation uses `webmcp-types` 0.1.9 for development-time types only; there is no runtime SDK or polyfill. OpenTube's native Android phone and TV apps do not use WebMCP.

WebMCP remains browser-dependent and is a progressive enhancement. For local manual checks, follow Chrome's current WebMCP guide to enable the testing flag, relaunch the browser, and inspect the registered tool; production availability follows the current Chrome origin-trial/support requirements. Native browser-agent interoperability has not been verified for this app. Follow the current [Chrome WebMCP setup](https://developer.chrome.com/docs/ai/webmcp), [imperative API](https://developer.chrome.com/docs/ai/webmcp/imperative-api), [best practices](https://developer.chrome.com/docs/ai/webmcp/best-practices), and [security guidance](https://developer.chrome.com/docs/ai/webmcp/secure-tools) before changing the tool. See the [contract catalog](../docs/contracts/README.md#contracts) and [verification record](../docs/verification/lighthouse.md) for ownership and checks.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `opentube` / `web-frontend`.
- **API, event, and data contract owners/producers/consumers:** see the [contract catalog](../docs/contracts/README.md) for each authoritative interface.
- **Parent architecture:** [System Design](../docs/system-design.md).

### Build prerequisites

Use this component’s pinned toolchain and lockfile/wrapper. The supported versions and complete local build environment are listed in the [root README](../README.md).

### Use prerequisites

This component is used as part of the parent system. Start its required local dependencies and use the supported local access path described in the [root README](../README.md).

### Build, verify, deploy, undeploy, and use

Build, run, and verification commands for this component are documented above. There is no independent release lifecycle for this component.
The parent Helm release owns deployment and removal; follow the [chart guide](../deploy/helm/opentube/README.md) and [root deployment lifecycle](../README.md). Uninstall removes application workloads while retained PVCs keep their data; deleting the namespace or claims purges persistent data.

[Documentation index](../docs/README.md)


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
