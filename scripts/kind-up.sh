#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
namespace=opentube
cluster=kind

if ! kind get clusters | rg -qx "$cluster"; then
  echo "Expected the existing Kind cluster '$cluster'; this script does not create or alter other clusters." >&2
  exit 1
fi
if [[ "$(kubectl config current-context)" != "kind-kind" ]]; then
  echo "Current kubectl context is not kind-kind." >&2
  exit 1
fi

"$repo_root/scripts/init-local-env.sh"
# .env is an ignored, user-owned local configuration file.
set -a
source .env
set +a

required_vars=(
  OPENTUBE_DATABASE_ADMIN_USER OPENTUBE_DATABASE_ADMIN_PASSWORD
  OPENTUBE_IDENTITY_DB_PASSWORD OPENTUBE_VIDEO_DB_PASSWORD
  OPENTUBE_MINIO_ROOT_USER OPENTUBE_MINIO_ROOT_PASSWORD
  OPENTUBE_RABBIT_USER OPENTUBE_RABBIT_PASSWORD
  OPENTUBE_JWT_SECRET OPENTUBE_WORKER_TOKEN OPENTUBE_SEED_EMAIL OPENTUBE_SEED_PASSWORD
)
for name in "${required_vars[@]}"; do
  value="${!name:-}"
  if [[ ! "$value" =~ ^[A-Za-z0-9._@-]+$ ]]; then
    echo "$name must be set and use only letters, numbers, dot, underscore, @, or hyphen." >&2
    exit 1
  fi
done

mkdir -p .local
cat > .local/helm-values.yaml <<EOF
localCredentials:
  databaseAdminUser: "$OPENTUBE_DATABASE_ADMIN_USER"
  databaseAdminPassword: "$OPENTUBE_DATABASE_ADMIN_PASSWORD"
  identityPassword: "$OPENTUBE_IDENTITY_DB_PASSWORD"
  videoPassword: "$OPENTUBE_VIDEO_DB_PASSWORD"
  minioRootUser: "$OPENTUBE_MINIO_ROOT_USER"
  minioRootPassword: "$OPENTUBE_MINIO_ROOT_PASSWORD"
  rabbitUser: "$OPENTUBE_RABBIT_USER"
  rabbitPassword: "$OPENTUBE_RABBIT_PASSWORD"
  jwtSecret: "$OPENTUBE_JWT_SECRET"
  workerToken: "$OPENTUBE_WORKER_TOKEN"
  seedEmail: "$OPENTUBE_SEED_EMAIL"
  seedPassword: "$OPENTUBE_SEED_PASSWORD"
EOF

docker build --target identity-service -t opentube-identity:dev -f Dockerfile.services .
docker build --target video-service -t opentube-video:dev -f Dockerfile.services .
docker build --target transcoding-worker -t opentube-worker:dev -f Dockerfile.services .
docker build -t opentube-web:dev -f web-frontend/Dockerfile web-frontend

kind load docker-image opentube-identity:dev opentube-video:dev opentube-worker:dev opentube-web:dev --name "$cluster"
minio_image="$(awk '/^minio:$/ { in_minio=1; next } in_minio && /^[^ ]/ { exit } in_minio && /^  image:/ { print $2; exit }' deploy/helm/opentube/values.yaml)"
case "$(uname -m)" in
  x86_64) image_platform=linux/amd64 ;;
  aarch64|arm64) image_platform=linux/arm64 ;;
  *) echo "Unsupported local Kind image architecture: $(uname -m)" >&2; exit 1 ;;
esac
docker pull --platform="$image_platform" "$minio_image"
mkdir -p .local
docker save --platform="$image_platform" -o .local/opentube-minio.tar "$minio_image"
kind load image-archive .local/opentube-minio.tar --name "$cluster"
helm upgrade --install opentube deploy/helm/opentube --values .local/helm-values.yaml --namespace "$namespace" --create-namespace --wait --timeout 12m

# Kind sees local images with mutable :dev tags. Force Pods to use the newly loaded image contents.
for app in identity-service video-service transcoding-worker web-frontend; do
  kubectl -n "$namespace" rollout restart "deployment/$app"
done
for app in identity-service video-service transcoding-worker web-frontend; do
  kubectl -n "$namespace" rollout status "deployment/$app" --timeout=180s
done

mkdir -p .local/port-forwards
start_forward() {
  local name="$1" target="$2" mapping="$3"
  local pid_file=".local/port-forwards/${name}.pid"
  local log_file=".local/port-forwards/${name}.log"
  if [[ -f "$pid_file" ]]; then
    local old_pid
    old_pid="$(cat "$pid_file")"
    if kill -0 "$old_pid" 2>/dev/null && ps -p "$old_pid" -o args= | rg -q "kubectl port-forward.*${target}"; then
      kill "$old_pid"
    fi
  fi
  setsid kubectl port-forward --address 127.0.0.1 -n "$namespace" "$target" "$mapping" >"$log_file" 2>&1 </dev/null &
  echo "$!" > "$pid_file"
}

start_forward web service/web-frontend 8080:80
start_forward identity service/identity-service 18081:8081
start_forward video service/video-service 18082:8082
start_forward media service/minio 19000:9000

for url in http://127.0.0.1:8080/ http://127.0.0.1:18081/health/ready http://127.0.0.1:18082/health/ready http://127.0.0.1:19000/minio/health/ready; do
  ready=false
  for attempt in $(seq 1 30); do
    if curl --silent --fail "$url" >/dev/null; then ready=true; break; fi
    sleep 1
  done
  if [[ "$ready" != true ]]; then echo "Local endpoint did not become ready: $url" >&2; exit 1; fi
done

if command -v adb >/dev/null 2>&1 && adb devices | rg -q 'emulator-[0-9]+[[:space:]]+device'; then
  for port in 18081 18082 19000; do adb reverse "tcp:$port" "tcp:$port"; done
fi

printf 'OpenTube is ready in namespace %s. Web UI: http://127.0.0.1:8080\n' "$namespace"
