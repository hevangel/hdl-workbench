#!/usr/bin/env python3
"""Serve the staged IDE bundle from loopback with module-friendly headers."""

import os
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer


class StaticHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Cross-Origin-Resource-Policy", "cross-origin")
        super().end_headers()


directory = os.environ.get("XEZIM_STAGED_SITE", "/tmp/luna06-local")
port = int(os.environ.get("XEZIM_STAGED_PORT", "4203"))
server = ThreadingHTTPServer(("127.0.0.1", port), partial(StaticHandler, directory=directory))
server.serve_forever()
