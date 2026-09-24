# OpenTube Helm chart

This chart deploys OpenTube into the local `opentube` namespace: identity, video, transcoding, web, PostgreSQL, RabbitMQ, and MinIO workloads. The checked-in chart owns the Kubernetes templates; `scripts/kind-up.sh` builds and loads the application images, initializes local-only credentials, installs the release, and opens loopback-only port-forwards.

> Documentation: [project index](../../../docs/README.md) · [repository overview](../../../README.md)

## Contents

- [Build and use prerequisites](#build-and-use-prerequisites)
- [Prerequisites and deploy](#prerequisites-and-deploy)
- [Resource budgets and persistence](#resource-budgets-and-persistence)
- [Undeploy and use](#undeploy-and-use)
- [AI development disclaimer](#ai-development-disclaimer)

## Build and use prerequisites

- **Build/deploy:** use the root project README for source-image build commands and the chart instructions below for the Helm release. Required tools are Docker, the supported local Kubernetes cluster/context, kubectl, and Helm.
- **Use:** the cluster release must be ready; use the loopback browser/port-forward instructions in the root README. See the resource table for each pod container budget.


## Prerequisites and deploy

Use the existing Kind cluster with context `kind-kind`, Docker, Kind, kubectl, and Helm. Do not create or switch clusters. Follow the root [README](../../../README.md#local-development) and run:

```sh
./scripts/kind-up.sh
```

For subsequent chart-only upgrades after images and `.local/helm-values.yaml` exist:

```sh
helm upgrade --install opentube deploy/helm/opentube \
  --values .local/helm-values.yaml --namespace opentube --create-namespace --wait --timeout 12m
```

## Resource budgets and persistence

Every workload container has CPU and memory requests and limits. Exact per-container values are listed in the [Kubernetes resource budget](../../../docs/kubernetes-resources.md) and configurable in [`values.yaml`](values.yaml). PostgreSQL, RabbitMQ, and MinIO PVCs use `Retain` on StatefulSet deletion/scale-down. Removing the Helm release preserves claims; deleting the namespace or claims deletes their local data. This is not a backup/restore guarantee.

## Undeploy and use

```sh
helm uninstall opentube --namespace opentube
```

The command removes Helm-managed workloads and services while retaining the dependency PVCs. The deployment script also creates local port-forward processes; stop them with `scripts/kind-port-forward-stop.sh`. Open the web UI at `http://127.0.0.1:8080`. See the [System Design](../../../docs/system-design.md), [contracts](../../../docs/contracts/README.md), and [documentation index](../../../docs/README.md).


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
