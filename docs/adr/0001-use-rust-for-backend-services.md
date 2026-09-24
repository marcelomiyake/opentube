# ADR-0001: Use Rust for Backend Services

- **Status:** Implemented (retrospective)
- **Recorded:** 2026-09-25
- **Original decision date:** Unknown from repository evidence
- **Decision owner:** Project owner
- **Confirmation:** Current implementation is documented at the project owner's request; historical team approval is not recorded.

> This record captures the Rust backend already present in the repository. The options and rationale below are a retrospective comparison, not a claim that the original project formally evaluated them.

## Contents

- [Context and problem statement](#context-and-problem-statement)
- [Decision drivers](#decision-drivers)
- [Options considered](#options-considered)
- [Decision outcome](#decision-outcome)
- [Consequences](#consequences)
- [Evidence and realization](#evidence-and-realization)
- [Review triggers](#review-triggers)
- [References](#references)

## Context and problem statement

OpenTube uses Rust services for identity, video metadata, uploads, and video-processing callbacks. Transcoding is a separate worker that consumes RabbitMQ jobs; Vue and Kotlin applications provide the web, phone, and TV clients.

The project owner's rationale is that Rust is safer, fast, and can use less CPU and memory. The owner recognizes Rust's deeper learning curve and considers it feasible with AI-assisted code authoring. These are design expectations and the owner's experience, not results from a comparative benchmark in this repository.

## Decision drivers

- Catch memory-safety and concurrency errors through Rust's type and ownership checks.
- Keep API and worker runtime overhead appropriate for the local CPU/memory budgets, including resource-intensive media processing.
- Support concurrent network and storage work in independently deployable services.
- Make Rust development manageable for this proof of concept with AI assistance, compiler feedback, focused tests, and human review.

## Options considered

### Rust for backend services

- **Benefits:** Compiled native services and compile-time ownership checks suit concurrent APIs and workers; resource use can be measured and tuned per container.
- **Costs and risks:** Ownership, lifetimes, and async Rust have a steeper learning curve; transcoding libraries and build environments require toolchain coordination.

### Go for backend services

- **Benefits:** Straightforward service development, static types, built-in concurrency, and a broad cloud-service ecosystem.
- **Costs and risks:** Garbage collection and a different safety model; migration would add cost without measured evidence that the current Rust choice is insufficient.

### TypeScript/Node.js for backend services

- **Benefits:** Could align the web backend language with the Vue client.
- **Costs and risks:** Would use a managed runtime and would not provide Rust's ownership checks; suitability for video processing would need separate performance evidence.

No cross-language performance benchmark is available, so these trade-offs are qualitative.

## Decision outcome

Use Rust for the identity, video, and transcoding backend services. AI-assisted authoring makes the learning curve acceptable for this proof of concept when paired with compiler checks, tests, documented contracts, and human review. The decision does not claim that AI-generated code is correct without verification.

## Consequences

### Positive

- Rust's type and ownership checks help prevent memory-safety errors in API and worker code.
- Native binaries and low runtime overhead are expected to support resource-conscious deployments; comparative consumption has not been measured.
- The asynchronous worker can scale independently from user-facing services.

### Negative and risks

- Contributors must learn Rust ownership, async execution, and service-specific contracts.
- Toolchains and native media dependencies increase local build setup complexity.
- HLS correctness, upload authorization, idempotency, and failure handling still require explicit tests and review.

## Evidence and realization

- Rust crates are defined by [identity-service](../../identity-service/Cargo.toml), [video-service](../../video-service/Cargo.toml), and [transcoding-worker](../../transcoding-worker/Cargo.toml).
- The [System Design](../system-design.md) and [contract catalog](../contracts/README.md) describe service and event boundaries.
- The [Kubernetes resource budgets](../kubernetes-resources.md) are local chart settings, not language-comparison benchmarks.

## Review triggers

- Reconsider if representative profiling shows a service misses measured CPU, memory, or latency targets and a controlled alternative-language comparison indicates a better fit.
- Reconsider if Rust's learning or maintenance burden becomes a material risk despite AI assistance, compiler tooling, tests, and review.
- Create a superseding ADR before changing the backend language or runtime.

## References

- [OpenTube README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
