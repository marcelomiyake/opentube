# Documentation index

Use this index to find the repository overview, agent guidance, architecture, contracts, operational instructions, verification records, and reusable templates. This static Markdown index has no application build, deployment, or undeployment lifecycle.

## Contents

- [Document responsibilities](#document-responsibilities)
- [Project and component guides](#project-and-component-guides)
- [Agent instructions](#agent-instructions)
- [Architecture and contracts](#architecture-and-contracts)
- [Build, deployment, and operations](#build-deployment-and-operations)
- [Verification and evaluation](#verification-and-evaluation)
- [Standards and reusable templates](#standards-and-reusable-templates)
- [Applying the standard](#applying-the-standard)
- [AI development disclaimer](#ai-development-disclaimer)

## Document responsibilities

- This index ([docs/README.md](README.md)) links every project Markdown document by purpose.
- Root and component `README.md` files help people build, deploy, use, and contribute.
- `AGENTS.md` files give scoped instructions to coding agents.
- Architecture and contract records identify system boundaries, owners, producers, consumers, and authoritative sources.
- Operational and verification documents explain applicable lifecycle commands and recorded evidence.

## Project and component guides

- [README.md](../README.md) — Root or component README
- [android-mobile-frontend/README.md](../android-mobile-frontend/README.md) — Root or component README
- [android-tv-frontend/README.md](../android-tv-frontend/README.md) — Root or component README
- [docs/README.md](README.md) — Documentation index
- [identity-service/README.md](../identity-service/README.md) — Root or component README
- [transcoding-worker/README.md](../transcoding-worker/README.md) — Root or component README
- [video-service/README.md](../video-service/README.md) — Root or component README
- [web-frontend/README.md](../web-frontend/README.md) — Root or component README

## Agent instructions

- [docs/agent-documentation-hooks.md](agent-documentation-hooks.md) — Codex hook recommendation for post-agent documentation sync

- [AGENTS.md](../AGENTS.md) — Scoped agent guidance
- [android-mobile-frontend/AGENTS.md](../android-mobile-frontend/AGENTS.md) — Scoped agent guidance
- [android-tv-frontend/AGENTS.md](../android-tv-frontend/AGENTS.md) — Scoped agent guidance
- [identity-service/AGENTS.md](../identity-service/AGENTS.md) — Scoped agent guidance
- [transcoding-worker/AGENTS.md](../transcoding-worker/AGENTS.md) — Scoped agent guidance
- [video-service/AGENTS.md](../video-service/AGENTS.md) — Scoped agent guidance
- [web-frontend/AGENTS.md](../web-frontend/AGENTS.md) — Scoped agent guidance


- [CLAUDE.md](../CLAUDE.md) — Single-line import of scoped `AGENTS.md` guidance
- [android-mobile-frontend/CLAUDE.md](../android-mobile-frontend/CLAUDE.md) — Single-line import of scoped `AGENTS.md` guidance
- [android-tv-frontend/CLAUDE.md](../android-tv-frontend/CLAUDE.md) — Single-line import of scoped `AGENTS.md` guidance
- [identity-service/CLAUDE.md](../identity-service/CLAUDE.md) — Single-line import of scoped `AGENTS.md` guidance
- [transcoding-worker/CLAUDE.md](../transcoding-worker/CLAUDE.md) — Single-line import of scoped `AGENTS.md` guidance
- [video-service/CLAUDE.md](../video-service/CLAUDE.md) — Single-line import of scoped `AGENTS.md` guidance
- [web-frontend/CLAUDE.md](../web-frontend/CLAUDE.md) — Single-line import of scoped `AGENTS.md` guidance

## Architecture and contracts

- [docs/contracts/README.md](contracts/README.md) — API/data contract catalog
- [docs/database-model.md](database-model.md) — database tables, columns, ownership, and descriptions
- [docs/system-design.md](system-design.md) — System Design or architecture decision

- [docs/adr/README.md](adr/README.md) — Architectural Decision Records index and maintenance guidance
- [docs/adr/0001-use-rust-for-backend-services.md](adr/0001-use-rust-for-backend-services.md) — ADR-0001: Use Rust for Backend Services
- [docs/adr/0002-use-vue-for-web-frontend.md](adr/0002-use-vue-for-web-frontend.md) — ADR-0002: Use Vue for the Web Frontend
- [docs/adr/0003-use-postgresql-for-identity-and-video-state.md](adr/0003-use-postgresql-for-identity-and-video-state.md) — ADR-0003: Use PostgreSQL for Identity and Video State
- [docs/adr/0004-use-rabbitmq-for-transcoding-jobs.md](adr/0004-use-rabbitmq-for-transcoding-jobs.md) — ADR-0004: Use RabbitMQ for Transcoding Jobs
- [docs/adr/0005-use-minio-for-video-objects.md](adr/0005-use-minio-for-video-objects.md) — ADR-0005: Use MinIO for Video Objects
- [docs/adr/0006-use-kubernetes-for-local-opentube-runtime.md](adr/0006-use-kubernetes-for-local-opentube-runtime.md) — ADR-0006: Use Kubernetes for the Local OpenTube Runtime
- [docs/adr/0007-package-opentube-with-helm.md](adr/0007-package-opentube-with-helm.md) — ADR-0007: Package OpenTube with Helm
- [docs/adr/0008-use-kotlin-compose-for-android-clients.md](adr/0008-use-kotlin-compose-for-android-clients.md) — ADR-0008: Use Kotlin and Compose for Android Clients

## Build, deployment, and operations

- [deploy/helm/opentube/README.md](../deploy/helm/opentube/README.md) — Helm chart deployment guide
- [docs/kubernetes-resources.md](kubernetes-resources.md) — Kubernetes resource budget and good practice

## Verification and evaluation

- [docs/verification/README.md](verification/README.md) — Verification procedure or evidence index
- [docs/verification/jev-readiness.md](verification/jev-readiness.md) — Verification record
- [docs/verification/lighthouse.md](verification/lighthouse.md) — Lighthouse and SEO metadata check
- [docs/verification/sonarqube.md](verification/sonarqube.md) — latest frontend scan and project quality gates
- [docs/verification/results.md](verification/results.md) — Verification record

## Standards and reusable templates

- [docs/documentation-standard.md](documentation-standard.md) — Documentation standard
- [docs/templates/adr.template.md](templates/adr.template.md) — Reusable Markdown ADR template
- [docs/templates/agents.template.md](templates/agents.template.md) — Reusable Markdown template
- [docs/templates/api-contract.template.md](templates/api-contract.template.md) — Reusable Markdown template
- [docs/templates/component-readme.template.md](templates/component-readme.template.md) — Reusable Markdown template
- [docs/templates/design-notes.template.md](templates/design-notes.template.md) — Reusable Markdown template
- [docs/templates/design-report.template.md](templates/design-report.template.md) — Reusable Markdown template
- [docs/templates/general-document.template.md](templates/general-document.template.md) — Reusable Markdown template
- [docs/templates/operations-guide.template.md](templates/operations-guide.template.md) — Reusable Markdown template
- [docs/templates/root-readme.template.md](templates/root-readme.template.md) — Reusable Markdown template
- [docs/templates/system-design.template.md](templates/system-design.template.md) — Reusable Markdown template
- [docs/templates/verification-record.template.md](templates/verification-record.template.md) — Reusable Markdown template

## Applying the standard

- [Documentation standard](documentation-standard.md)
- [Component README template](templates/component-readme.template.md)


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
