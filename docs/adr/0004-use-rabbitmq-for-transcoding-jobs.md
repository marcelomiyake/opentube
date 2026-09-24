# ADR-0004: Use RabbitMQ for Transcoding Jobs

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

After the video service commits upload completion and a PostgreSQL outbox row, it publishes a versioned processing message to RabbitMQ. The transcoding worker consumes jobs, stores HLS output, and reports completion through an internal API callback. The original broker selection record was not found.

The scope of this decision is RabbitMQ as the asynchronous job transport between video upload and transcoding. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Keep video upload acceptance independent of transcoding duration.
- Buffer and retry asynchronous media-processing jobs.
- Preserve a database-backed handoff record if broker publication is interrupted.

## Options considered

### RabbitMQ

- **Benefits:** The current queue workflow supports job delivery, acknowledgments, and a worker callback.
- **Costs and risks:** Adds broker lifecycle, queue monitoring, and duplicate-delivery handling.

### Poll PostgreSQL for pending jobs

- **Benefits:** Uses durable database state directly and removes broker operations.
- **Costs and risks:** Couples worker polling to the application database and changes the current job isolation and backlog behavior.

### Kafka or another event log

- **Benefits:** Could support retained streams and several independent consumers.
- **Costs and risks:** Adds operational and retention concepts not required by the current single transcoding workflow.

## Decision outcome

Retain RabbitMQ for transcoding job transport. PostgreSQL outbox and video state remain authoritative; the system does not promise exactly-once execution.

## Consequences

### Positive

- Upload completion can return before transcoding finishes.
- The broker separates web/API request load from media-processing throughput.

### Negative and risks

- Messages may be redelivered, so output writes and callbacks need idempotent behavior.
- A broker outage delays processing until pending outbox work can be retried.

## Evidence and realization

- [Cargo.toml](../../video-service/Cargo.toml)
- [Cargo.toml](../../transcoding-worker/Cargo.toml)
- [rabbitmq.yaml](../../deploy/helm/opentube/templates/rabbitmq.yaml)
- [system-design.md](../system-design.md)

## Review triggers

- Revisit if job retention, multiple independent consumers, or measured throughput changes the required messaging model.
- Preserve the durable outbox and idempotent completion semantics if replacing RabbitMQ.

## References

- [opentube README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
