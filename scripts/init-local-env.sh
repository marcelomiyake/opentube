#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
env_file="$repo_root/.env"
if [[ -f "$env_file" ]]; then
  exit 0
fi

umask 077
temporary_file="$(mktemp "$repo_root/.env.XXXXXX")"
trap 'rm -f "$temporary_file"' EXIT

if command -v kubectl >/dev/null 2>&1 \
  && [[ "$(kubectl config current-context 2>/dev/null || true)" == "kind-kind" ]] \
  && kubectl -n opentube get secret opentube-secrets >/dev/null 2>&1; then
  kubectl -n opentube get secret opentube-secrets -o json | python3 -c '
import base64, json, shlex, sys
data = json.load(sys.stdin)["data"]
mapping = {
    "OPENTUBE_DATABASE_ADMIN_USER": "database-admin-user",
    "OPENTUBE_DATABASE_ADMIN_PASSWORD": "database-admin-password",
    "OPENTUBE_IDENTITY_DB_PASSWORD": "identity-password",
    "OPENTUBE_VIDEO_DB_PASSWORD": "video-password",
    "OPENTUBE_MINIO_ROOT_USER": "minio-root-user",
    "OPENTUBE_MINIO_ROOT_PASSWORD": "minio-root-password",
    "OPENTUBE_RABBIT_USER": "rabbit-user",
    "OPENTUBE_RABBIT_PASSWORD": "rabbit-password",
    "OPENTUBE_JWT_SECRET": "jwt-secret",
    "OPENTUBE_WORKER_TOKEN": "worker-token",
    "OPENTUBE_SEED_EMAIL": "seed-email",
    "OPENTUBE_SEED_PASSWORD": "seed-password",
}
for name, key in mapping.items():
    value = base64.b64decode(data[key]).decode("utf-8")
    print(f"{name}={shlex.quote(value)}")
' > "$temporary_file"
  chmod 600 "$temporary_file"
  mv "$temporary_file" "$env_file"
  trap - EXIT
  echo "Recovered the existing opentube Kind credentials into the ignored .env file."
  exit 0
fi

random_hex() { openssl rand -hex "$1"; }
{
  printf 'OPENTUBE_DATABASE_ADMIN_USER=opentube_admin\n'
  printf 'OPENTUBE_DATABASE_ADMIN_PASSWORD=%s\n' "$(random_hex 32)"
  printf 'OPENTUBE_IDENTITY_DB_PASSWORD=%s\n' "$(random_hex 32)"
  printf 'OPENTUBE_VIDEO_DB_PASSWORD=%s\n' "$(random_hex 32)"
  printf 'OPENTUBE_MINIO_ROOT_USER=opentube\n'
  printf 'OPENTUBE_MINIO_ROOT_PASSWORD=%s\n' "$(random_hex 32)"
  printf 'OPENTUBE_RABBIT_USER=opentube\n'
  printf 'OPENTUBE_RABBIT_PASSWORD=%s\n' "$(random_hex 32)"
  printf 'OPENTUBE_JWT_SECRET=%s\n' "$(random_hex 32)"
  printf 'OPENTUBE_WORKER_TOKEN=%s\n' "$(random_hex 32)"
  printf 'OPENTUBE_SEED_EMAIL=creator@opentube.local\n'
  printf 'OPENTUBE_SEED_PASSWORD=%s\n' "$(random_hex 24)"
} > "$temporary_file"
chmod 600 "$temporary_file"
mv "$temporary_file" "$env_file"
trap - EXIT
echo "Generated unique local credentials in the ignored .env file."
echo "The seeded creator password is available there as OPENTUBE_SEED_PASSWORD."
