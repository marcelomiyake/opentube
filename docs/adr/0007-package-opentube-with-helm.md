# ADR-0007: Package OpenTube with Helm

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

One chart renders application Deployments and Services, PostgreSQL, RabbitMQ, and MinIO resources with configurable local values. The deployment scripts install the chart into the existing Kind cluster. Persistent claims require cleanup decisions separate from release removal.

The scope of this decision is The Helm chart as the local installation and upgrade interface for OpenTube. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Keep service and dependency Kubernetes objects in a reviewable package.
- Configure local images, replicas, and resource budgets through chart values.
- Use a reproducible release lifecycle for install, upgrade, rollback, and uninstall.

## Options considered

### Helm

- **Benefits:** The implemented chart packages templates and values under one named release.
- **Costs and risks:** Template logic and values require validation; release removal and namespace deletion have distinct data-retention effects.

### Raw manifests or Kustomize

- **Benefits:** Could reduce template complexity and make environment overlays explicit.
- **Costs and risks:** Would require replacing the current chart-based scripts and release workflow.

### Compose files

- **Benefits:** Could provide a simpler local container lifecycle.
- **Costs and risks:** Would diverge from the implemented Kubernetes workload model.

## Decision outcome

Keep Helm as the packaging and release interface for the local Kubernetes deployment. Follow the documented uninstall procedure that preserves claims unless data deletion is intentional.

## Consequences

### Positive

- The application and local dependencies share one chart release and documented values.
- Operators can review resource and replica changes alongside templates.

### Negative and risks

- A chart can render valid YAML while still expressing incorrect workload behavior; source review and deployment verification remain necessary.
- Deleting the namespace can remove persistent data even when the chart describes a retained claim policy.

## Evidence and realization

- [Chart.yaml](../../deploy/helm/opentube/Chart.yaml)
- [values.yaml](../../deploy/helm/opentube/values.yaml)
- [templates](../../deploy/helm/opentube/templates)
- [README.md](../../README.md)

## Review triggers

- Reassess if dependency lifecycles need independent releases or if the deployment target changes.
- Update the ADR when chart ownership, persistence retention, or release commands change.

## References

- [opentube README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
