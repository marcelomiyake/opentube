# ADR-0006: Use Kubernetes for the Local OpenTube Runtime

- **Status:** Implemented (retrospective)
- **Recorded:** 2026-09-25
- **Original decision date:** Unknown from repository evidence
- **Decision owner:** Project owner
- **Confirmation:** Current implementation is documented at the project owner’s request; historical team approval is not recorded.

> This record documents the technology in the current implementation. The options and rationale below are a retrospective comparison, not a claim that the original project formally evaluated them.

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

The chart deploys the web client, identity service, video service, transcoding worker, PostgreSQL, RabbitMQ, and MinIO. Application services use separate Deployments; local stateful dependencies use persistent storage. The Kind cluster is an educational single-node failure domain.

The scope of this decision is Kubernetes on local Kind for web, backend, worker, and stateful dependencies. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Exercise service discovery, health checks, rollout, and independent application replicas.
- Keep the web/API/worker and stateful dependency roles separately deployed.
- Declare CPU and memory requests and limits for pod containers.

## Options considered

### Kubernetes on local Kind

- **Benefits:** Matches the implemented workload resources and allows local deployment workflow verification.
- **Costs and risks:** Requires a local cluster and storage; a single-node cluster does not demonstrate high availability.

### Docker Compose

- **Benefits:** Simpler startup for a local multi-container stack.
- **Costs and risks:** Does not exercise the current Kubernetes service, rollout, replica, or PVC configuration.

### Direct host processes

- **Benefits:** Avoids container orchestration overhead.
- **Costs and risks:** Does not reproduce container networking and stateful workload declarations.

## Decision outcome

Retain Kubernetes as the local deployment model and Kind as its workstation runtime. Treat local results as functional evidence only, not production readiness or resilience evidence.

## Consequences

### Positive

- The application roles have separate Services and Deployments, while persistent dependencies have explicit storage.
- Local CPU and memory budgets are visible and reviewable.

### Negative and risks

- Kind shares host resources and its single node is one failure domain.
- Cluster setup and storage behavior add maintenance overhead for developers.

## Evidence and realization

- [templates](../../deploy/helm/opentube/templates)
- [kubernetes-resources.md](../kubernetes-resources.md)
- [system-design.md](../system-design.md)
- [README.md](../../README.md)

## Review triggers

- Reconsider if the supported target changes or the project no longer needs Kubernetes-specific behavior.
- Before sharing a remote cluster, define network access, authentication, secrets, backups, and production capacity.

## References

- [opentube README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
