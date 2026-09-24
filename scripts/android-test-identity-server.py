#!/usr/bin/env python3
"""Loopback-only identity and media stubs for Android instrumentation tests."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import sys


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path != "/v1/login":
            self.send_error(404)
            return
        length = int(self.headers.get("Content-Length", "0"))
        try:
            payload = json.loads(self.rfile.read(length))
        except (json.JSONDecodeError, UnicodeDecodeError):
            self._respond(400, {"error": "invalid request"})
            return

        if payload.get("email") != "creator@opentube.local" or not payload.get("password"):
            self._respond(401, {"error": "invalid credentials"})
            return
        self._respond(200, {"access_token": "android-test-token", "expires_in": 3600})

    def do_PUT(self):
        if self.path not in ("/upload", "/reject"):
            self.send_error(404)
            return
        length = int(self.headers.get("Content-Length", "0"))
        self.rfile.read(length)
        if self.path == "/reject":
            self._respond(403, {"error": "synthetic storage rejection"})
            return
        self._respond(200, {"stored": True})

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
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 18083
    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    print(f"Android instrumentation identity stub listening on 127.0.0.1:{port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
