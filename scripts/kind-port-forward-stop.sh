#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pid_dir="$repo_root/.local/port-forwards"
for pid_file in "$pid_dir"/*.pid; do
  [[ -f "$pid_file" ]] || continue
  pid="$(cat "$pid_file")"
  if kill -0 "$pid" 2>/dev/null && ps -p "$pid" -o args= | rg -q 'kubectl port-forward.*service/(web-frontend|identity-service|video-service|minio)'; then
    kill "$pid"
  fi
  rm -f "$pid_file"
done
printf 'Stopped OpenTube loopback port-forwards. The opentube namespace remains available.\n'
