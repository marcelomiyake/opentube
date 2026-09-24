# Kubernetes CPU and memory budgets

The Helm chart's configured CPU and memory requests and limits are listed below. Values are per pod container. These are local chart defaults, not measured consumption or production sizing claims.


> Project documentation index: [Documentation index](README.md)


## Workload budgets

| Pod/workload and container | CPU request | Memory request | CPU limit | Memory limit | Source |
| --- | ---: | ---: | ---: | ---: | --- |
| `identity-service` / `identity-service` | 50m | 128Mi | 500m | 512Mi | [Chart values](../deploy/helm/opentube/values.yaml) |
| `video-service` / `video-service` | 100m | 192Mi | 1 | 768Mi | [Chart values](../deploy/helm/opentube/values.yaml) |
| `transcoding-worker` / `transcoding-worker` | 200m | 256Mi | 2 | 2Gi | [Chart values](../deploy/helm/opentube/values.yaml) |
| `web-frontend` / `nginx` | 25m | 32Mi | 250m | 128Mi | [Chart values](../deploy/helm/opentube/values.yaml) |
| `postgres` / `postgres` | 100m | 256Mi | 1 | 1Gi | [Chart values](../deploy/helm/opentube/values.yaml) |
| `rabbitmq` / `rabbitmq` | 100m | 256Mi | 1 | 1Gi | [Chart values](../deploy/helm/opentube/values.yaml) |
| `minio` / `minio` | 100m | 256Mi | 1 | 1Gi | [Chart values](../deploy/helm/opentube/values.yaml) |

All seven workload containers have CPU and memory requests and limits. PostgreSQL, RabbitMQ, and MinIO each run as a single StatefulSet pod in this chart. PVC storage requests (PostgreSQL 2Gi, RabbitMQ 1Gi, MinIO 4Gi) are separate. There are no init containers in these pod templates.

## Kubernetes resource management practice

Set CPU and memory `requests` and `limits` on every container in every Pod, including init containers and sidecars. Requests guide scheduling and reserve baseline capacity; CPU limits may throttle, and memory limits can trigger OOM termination. Measure representative usage, leave startup/burst headroom, monitor throttling and restarts, and right-size deliberately. Treat the listed values as local development defaults, not production sizing guidance. PVC storage requests are separate from container budgets.

## Build, use, and persistence

The [root README](../README.md) documents build, deploy, use, and undeploy commands with their persistent-data effects. The [Helm chart guide](../deploy/helm/opentube/README.md) is the deployment owner. This resource inventory is configuration guidance and does not itself deploy workloads.
