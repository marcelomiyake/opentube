# ADR-0003: Use PostgreSQL for Identity and Video State

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

Identity and video services use owned PostgreSQL schemas and migrations. Video upload completion commits state and an outbox row before asynchronous transcoding begins. Media bytes and HLS segments live in object storage, not relational rows. The original database selection rationale was not found.

The scope of this decision is PostgreSQL as the durable relational store for identity and video metadata. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Keep account and video lifecycle records durable and queryable.
- Commit upload-state transitions and event handoff atomically.
- Separate relational metadata from large media objects and asynchronous processing.

## Options considered

### PostgreSQL

- **Benefits:** Transactions, constraints, service-owned schemas, and SQL support the current lifecycle and outbox model.
- **Costs and risks:** The local profile runs a single database instance; availability and scaling are limited by that deployment.

### A distributed key-value store

- **Benefits:** Could scale particular high-volume lookup patterns.
- **Costs and risks:** Would require a separate consistency design for upload lifecycle and outbox transitions.

### Separate database engines per service

- **Benefits:** Could increase datastore autonomy.
- **Costs and risks:** Adds local operational and backup complexity without evidence that the current service-owned schemas need separate engines.

## Decision outcome

Retain PostgreSQL for identity and video metadata, with service-owned schemas and migrations. Store media objects in MinIO and use the outbox for asynchronous work handoff.

## Consequences

### Positive

- Relational transactions guard upload acceptance and processing eligibility.
- Metadata ownership remains distinguishable by schema and service migrations.

### Negative and risks

- The database is a local single-replica dependency without a production recovery guarantee.
- Schema boundaries are service-owned within one shared local PostgreSQL instance, not separate database failure domains.

## Evidence and realization

- [0001_identity.sql](../../identity-service/migrations/0001_identity.sql)
- [0001_video.sql](../../video-service/migrations/0001_video.sql)
- [values.yaml](../../deploy/helm/opentube/values.yaml)
- [database-model.md](../database-model.md)
- [system-design.md](../system-design.md)

## Review triggers

- Reconsider after measured database bottlenecks, changed durability requirements, or a need to isolate service failures.
- If splitting the database, retain the upload/outbox atomicity or document a replacement consistency protocol.

## References

- [opentube README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
