#!/usr/bin/env python3
"""Loopback-only, deterministic video API fixture for Android UI coverage."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import sys
from urllib.parse import parse_qs, urlsplit


VIDEOS = (
    {
        "id": "coverage-video-1",
        "title": "Coverage fixture video",
        "description": "Synthetic pagination and playback fixture.",
        "creator": "Coverage Studio",
        "published_at": "2026-09-25T12:00:00Z",
        "playback_url": "http://127.0.0.1:19082/fixture.m3u8",
    },
    {
        "id": "coverage-video-2",
        "title": "Second coverage video",
        "description": "Second page fixture.",
        "creator": "Coverage Studio",
        "published_at": "2026-09-25T11:00:00Z",
        "playback_url": "http://127.0.0.1:19082/fixture.m3u8",
    },
)


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        request = urlsplit(self.path)
        if request.path == "/fixture.m3u8":
            body = b"#EXTM3U\n#EXT-X-ENDLIST\n"
            self.send_response(200)
            self.send_header("Content-Type", "application/vnd.apple.mpegurl")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if request.path != "/v1/videos":
            self._respond(404, {"error": "not found"})
            return

        params = parse_qs(request.query)
        query = params.get("query", [""])[0].casefold()
        cursor = params.get("cursor", [""])[0]
        if query == "failure":
            self._respond(503, {"error": "synthetic unavailable"})
        elif query == "empty":
            self._respond(200, {"items": [], "next_cursor": None})
        elif cursor == "page-2":
            self._respond(200, {"items": [VIDEOS[1]], "next_cursor": None})
        else:
            self._respond(200, {"items": [VIDEOS[0]], "next_cursor": "page-2"})

    def _respond(self, status, payload):
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, _format, *_args):
        return


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 19082
    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    print(f"Android instrumentation video stub listening on 127.0.0.1:{port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
