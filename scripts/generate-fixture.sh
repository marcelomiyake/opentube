#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixture_dir="$repo_root/.local-media"
mkdir -p "$fixture_dir"

if command -v ffmpeg >/dev/null 2>&1; then
  ffmpeg -hide_banner -loglevel error -y \
    -f lavfi -i 'testsrc=size=640x360:rate=24' \
    -f lavfi -i 'sine=frequency=440:sample_rate=48000' \
    -t 6 -c:v libx264 -preset ultrafast -pix_fmt yuv420p -c:a aac -b:a 96k \
    -movflags +faststart "$fixture_dir/opentube-fixture.mp4"
else
  docker run --rm --user "$(id -u):$(id -g)" --entrypoint ffmpeg -v "$fixture_dir:/out" jrottenberg/ffmpeg:7.1-ubuntu \
    -hide_banner -loglevel error -y \
    -f lavfi -i 'testsrc=size=640x360:rate=24' \
    -f lavfi -i 'sine=frequency=440:sample_rate=48000' \
    -t 6 -c:v libx264 -preset ultrafast -pix_fmt yuv420p -c:a aac -b:a 96k \
    -movflags +faststart /out/opentube-fixture.mp4
fi

chmod 0644 "$fixture_dir/opentube-fixture.mp4"
printf 'Created synthetic MP4 fixture: %s (%s bytes)\n' "$fixture_dir/opentube-fixture.mp4" "$(stat -c %s "$fixture_dir/opentube-fixture.mp4")"
