# Architectural Decision Records

This index records the significant architectural choices for OpenTube. Each ADR covers one decision and links to its implementation evidence.

## Records

- [ADR-0001: Use Rust for Backend Services](0001-use-rust-for-backend-services.md)
- [ADR-0002: Use Vue for the Web Frontend](0002-use-vue-for-web-frontend.md)
- [ADR-0003: Use PostgreSQL for Identity and Video State](0003-use-postgresql-for-identity-and-video-state.md)
- [ADR-0004: Use RabbitMQ for Transcoding Jobs](0004-use-rabbitmq-for-transcoding-jobs.md)
- [ADR-0005: Use MinIO for Video Objects](0005-use-minio-for-video-objects.md)
- [ADR-0006: Use Kubernetes for the Local OpenTube Runtime](0006-use-kubernetes-for-local-opentube-runtime.md)
- [ADR-0007: Package OpenTube with Helm](0007-package-opentube-with-helm.md)
- [ADR-0008: Use Kotlin and Compose for Android Clients](0008-use-kotlin-compose-for-android-clients.md)

## Maintaining this log

New records use the [ADR template](../templates/adr.template.md). Keep one decision per file, number records sequentially, and add a new ADR when an accepted or implemented decision changes. Do not rewrite historical outcomes. Retrospective records distinguish known implementation evidence from unknown original decision dates or approval history.

This Markdown index has no separate software build, deploy, use, or undeploy lifecycle; see the [project README](../../README.md) for application operations.

## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
