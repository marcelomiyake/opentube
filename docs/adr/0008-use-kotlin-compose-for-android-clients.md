# ADR-0008: Use Kotlin and Compose for Android Clients

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

OpenTube provides a phone client for sign-in, browse, upload, and playback, and a TV client for browse and remote-controlled playback. Both use native Android project structures and Kotlin/Compose rather than being Kubernetes workloads. No original mobile framework selection record was found.

The scope of this decision is Native Kotlin and Jetpack Compose clients for Android phone and Android TV. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Support Android device and emulator workflows, including media playback.
- Use native interaction patterns for phone touch input and TV remote focus.
- Keep client presentations separate from the Rust APIs and Vue web client.

## Options considered

### Kotlin with Jetpack Compose

- **Benefits:** The current native clients use Android UI and media integrations, with distinct phone and TV interaction models.
- **Costs and risks:** Requires Android Studio/SDK and Kotlin build tooling, and duplicates some web-client flows.

### Flutter or another cross-platform toolkit

- **Benefits:** Could share UI code across mobile targets.
- **Costs and risks:** Would introduce a framework migration and would still need platform-specific playback and TV input work.

### Vue web app in a WebView

- **Benefits:** Could reuse browser UI code.
- **Costs and risks:** Would not directly retain the current native Android navigation and media integration model.

## Decision outcome

Retain native Kotlin/Compose clients for phone and TV, while Vue remains the web client. This records the implemented split and a retrospective trade-off analysis.

## Consequences

### Positive

- Phone and TV clients can tailor navigation, focus, and media controls to their device class.
- Android clients share the backend contracts without being deployed as server workloads.

### Negative and risks

- The project must maintain three client experiences across web, phone, and TV.
- Android SDK, emulator, and Gradle setup adds contributor prerequisites.

## Evidence and realization

- [README.md](../../android-mobile-frontend/README.md)
- [build.gradle.kts](../../android-mobile-frontend/app/build.gradle.kts)
- [README.md](../../android-tv-frontend/README.md)
- [build.gradle.kts](../../android-tv-frontend/app/build.gradle.kts)
- [system-design.md](../system-design.md)

## Review triggers

- Reconsider if client duplication becomes a sustained maintenance issue or if supported device platforms change.
- Keep phone and TV accessibility, input, and playback needs in the evaluation before consolidating clients.

## References

- [opentube README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
