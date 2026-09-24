# transcoding-worker

Rust RabbitMQ consumer for `video.uploaded.v1`. It validates source objects with `ffprobe`, writes deterministic HLS output with FFmpeg, then reports an idempotent terminal result to the video service. It runs as a two-replica Kubernetes Deployment.

> Documentation: [project index](../docs/README.md) · [repository overview](../README.md)

Configuration: RabbitMQ URL, internal S3 endpoint and credentials, bucket, video-service internal URL, and `WORKER_TOKEN`. The container image includes FFmpeg. Local development uses synthetic MP4 fixtures only.

Run `cargo test -p transcoding-worker`; run locally with `cargo run -p transcoding-worker`.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `opentube` / `transcoding-worker`.
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
