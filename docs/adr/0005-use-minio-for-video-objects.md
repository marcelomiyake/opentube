# ADR-0005: Use MinIO for Video Objects

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

The browser and Android upload flows send media to a presigned object-storage URL. The transcoding worker reads source objects and writes HLS outputs; PostgreSQL stores metadata and lifecycle state. The original object-store selection record was not found.

The scope of this decision is S3-compatible MinIO object storage for uploaded source media and generated HLS output in the local profile. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Keep large media bytes outside relational metadata tables.
- Exercise direct upload and object-based transcoding locally.
- Use an S3-compatible interface for application storage calls.

## Options considered

### MinIO in the local cluster

- **Benefits:** Provides a local S3-compatible endpoint and persistent object storage for the current workflow.
- **Costs and risks:** The local deployment and image maintenance are project responsibilities; this setup is not a backup or production durability claim.

### Store media in PostgreSQL

- **Benefits:** Would keep metadata and blobs under one database service.
- **Costs and risks:** Would mix large media I/O with transactional metadata and change the current upload/transcoding flow.

### External cloud object storage

- **Benefits:** Could move media storage outside the local cluster.
- **Costs and risks:** Requires a selected provider, credentials, networking, and cost/retention policy not established by this local proof of concept.

## Decision outcome

Retain MinIO as the S3-compatible object store in the local profile. Keep PostgreSQL authoritative for metadata and do not infer production suitability from local operation.

## Consequences

### Positive

- Upload, source retrieval, and HLS output use a clear object-storage boundary.
- The same S3-oriented client path can be exercised without a cloud account.

### Negative and risks

- Persistent local volumes do not provide off-host backup, replication, or recovery.
- The pinned MinIO image and production replacement strategy require maintenance review.

## Evidence and realization

- [Cargo.toml](../../video-service/Cargo.toml)
- [Cargo.toml](../../transcoding-worker/Cargo.toml)
- [minio.yaml](../../deploy/helm/opentube/templates/minio.yaml)
- [values.yaml](../../deploy/helm/opentube/values.yaml)
- [system-design.md](../system-design.md)

## Review triggers

- Reconsider if the system adopts a managed object store or defines production durability, encryption, lifecycle, and cost requirements.
- Review the maintained image source and compatibility before updating the local object-store image.

## References

- [opentube README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
