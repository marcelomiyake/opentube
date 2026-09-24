#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
scanner_image="sonarsource/sonar-scanner-cli:latest"
sonar_host="${SONAR_HOST_URL:-http://127.0.0.1:9000}"

if [[ -z "${SONAR_TOKEN:-}" ]]; then
  echo "Set SONAR_TOKEN in the environment; the token is never written to the repo." >&2
  exit 1
fi

python3 - "$repo_root" "$repo_root/.local/coverage/rust.lcov" "$repo_root/.local/coverage" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1]).resolve()
source_report = Path(sys.argv[2])
scanner_report_dir = Path(sys.argv[3])
if not source_report.is_file():
    raise SystemExit(f"Rust LCOV report not found: {source_report}")

lines = []
for line in source_report.read_text().splitlines():
    if line.startswith("SF:"):
        source = Path(line[3:]).resolve()
        try:
            relative = source.relative_to(root)
        except ValueError as error:
            raise SystemExit(f"LCOV source is outside the repository: {source}") from error
        line = f"SF:/usr/src/{relative.as_posix()}"
    lines.append(line)

prefixes = {
    "identity-service": "identity-service/",
    "video-service": "video-service/",
    "transcoding-worker": "transcoding-worker/",
    "platform-core": "crates/platform-core/",
}
reports = {name: [] for name in prefixes}
record = []
for line in lines:
    record.append(line)
    if line == "end_of_record":
        source = next((item[3:] for item in record if item.startswith("SF:")), "")
        for name, prefix in prefixes.items():
            if source.startswith(f"/usr/src/{prefix}"):
                reports[name].extend(record)
                break
        record = []

for name, report in reports.items():
    (scanner_report_dir / f"rust-sonar-{name}.lcov").write_text(
        "\n".join(report) + ("\n" if report else "")
    )
PY

projects_to_scan=("$@")
should_scan() {
  local candidate
  if ((${#projects_to_scan[@]} == 0)); then return 0; fi
  for candidate in "${projects_to_scan[@]}"; do
    if [[ "$candidate" == "$1" ]]; then return 0; fi
  done
  return 1
}

scan_project() {
  local key="$1" name="$2" sources="$3"
  shift 3
  docker run --rm --network host \
    -e SONAR_HOST_URL="$sonar_host" -e SONAR_TOKEN \
    -v "$repo_root:/usr/src" -w /usr/src \
    "$scanner_image" \
    "-Dsonar.projectKey=$key" \
    "-Dsonar.projectName=$name" \
    "-Dsonar.sources=$sources" \
    "$@"
}

if should_scan opentube-identity-service; then
  scan_project opentube-identity-service "OpenTube identity-service" identity-service/src \
    -Dsonar.rust.lcov.reportPaths=.local/coverage/rust-sonar-identity-service.lcov \
    -Dsonar.rust.clippy.enabled=false
fi
if should_scan opentube-video-service; then
  scan_project opentube-video-service "OpenTube video-service" video-service/src \
    -Dsonar.rust.lcov.reportPaths=.local/coverage/rust-sonar-video-service.lcov \
    -Dsonar.rust.clippy.enabled=false
fi
if should_scan opentube-transcoding-worker; then
  scan_project opentube-transcoding-worker "OpenTube transcoding-worker" transcoding-worker/src \
    -Dsonar.rust.lcov.reportPaths=.local/coverage/rust-sonar-transcoding-worker.lcov \
    -Dsonar.rust.clippy.enabled=false
fi
if should_scan opentube-platform-core; then
  scan_project opentube-platform-core "OpenTube platform-core" crates/platform-core/src \
    -Dsonar.rust.lcov.reportPaths=.local/coverage/rust-sonar-platform-core.lcov \
    -Dsonar.rust.clippy.enabled=false
fi
if should_scan opentube-web-frontend; then
  scan_project opentube-web-frontend "OpenTube web-frontend" web-frontend/src \
    -Dsonar.tests=web-frontend/src,web-frontend/e2e \
    -Dsonar.test.inclusions='web-frontend/src/**/*.test.ts,web-frontend/src/**/*.test.vue,web-frontend/e2e/**/*.spec.ts' \
    -Dsonar.javascript.lcov.reportPaths=web-frontend/coverage/lcov.info
fi

for client in android-mobile-frontend android-tv-frontend; do
  if should_scan "opentube-$client"; then
    scan_project "opentube-$client" "OpenTube $client" "$client/app/src/main/java" \
      "-Dsonar.tests=$client/app/src/test/java,$client/app/src/androidTest/java" \
      "-Dsonar.kotlin.binaries=$client/app/build/intermediates/built_in_kotlinc/debug/compileDebugKotlin/classes" \
      "-Dsonar.coverage.jacoco.xmlReportPaths=$client/app/build/reports/coverage/test/debug/report.xml,$client/app/build/reports/coverage/androidTest/debug/connected/report.xml"
  fi
done
