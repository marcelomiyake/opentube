# ADR-0002: Use Vue for the Web Frontend

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

The browser client is a Vue 3 and TypeScript application for account access, uploads, video discovery, search, and playback. Separate native Android phone and TV clients are maintained. The original web framework selection record was not found.

The scope of this decision is Vue 3 and TypeScript for the OpenTube browser client. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Build a responsive browser client around reusable upload, catalog, and playback views.
- Keep browser behavior and API access testable independently of backend services.
- Retain a distinct web client while Android clients use native platform interfaces.

## Options considered

### Vue 3 with TypeScript

- **Benefits:** The current component UI, Vite build, and tests support the implemented browser flows.
- **Costs and risks:** Vue-specific conventions and frontend tooling must be maintained alongside native Android and Rust toolchains.

### React with TypeScript

- **Benefits:** Could support the same component-based workflows and ecosystem integrations.
- **Costs and risks:** Would require rewriting the existing browser client and tests without evidence of a Vue limitation.

### Web-only server-rendered client

- **Benefits:** Could reduce browser framework code for read-heavy pages.
- **Costs and risks:** Upload progress, playback, and interactive catalog state are already implemented in the client.

## Decision outcome

Retain Vue 3 and TypeScript for the web client. This ADR describes the current implementation and a retrospective comparison, not documented original selection criteria.

## Consequences

### Positive

- Browser UI remains a separately deployable API consumer.
- The web-specific presentation is separate from native Android phone and TV experiences.

### Negative and risks

- The repository maintains web and native-client implementations for overlapping user journeys.
- No comparative framework benchmark or migration case is documented.

## Evidence and realization

- [package.json](../../web-frontend/package.json)
- [Dockerfile](../../web-frontend/Dockerfile)
- [system-design.md](../system-design.md)
- [README.md](../../README.md)

## Review triggers

- Reconsider if web-client maintenance or accessibility evidence shows the current framework blocks required workflows.
- Create a superseding ADR before replacing Vue or consolidating the web and native clients.

## References

- [opentube README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
