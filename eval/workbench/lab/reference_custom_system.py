#!/usr/bin/env python3
"""A complete custom system for the comparison lab, as small as the protocol allows.

It proves the custom path end to end and is the starting point for a shim in front
of a real system. It scans exactly in plain Python, so it is slow by design, and its
speed says nothing about anything else.
"""
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

MAX_BODY_BYTES = 256 * 1024 * 1024


class Store:
    metric = "cosine"
    points = []


def score(left, right):
    if Store.metric == "euclidean":
        return -sum((a - b) ** 2 for a, b in zip(left, right))
    return sum(a * b for a, b in zip(left, right))


def search(request):
    query, wanted = request["vector"], request.get("category")
    scored = [
        (score(query, vector), point_id)
        for point_id, vector, category in Store.points
        if wanted is None or category == wanted
    ]
    scored.sort(key=lambda entry: (-entry[0], entry[1]))
    return {"ids": [point_id for _, point_id in scored[: request["k"]]]}


def prepare(request):
    Store.metric, Store.points = request["metric"], []
    return {}


def ingest(request):
    Store.points.extend(zip(request["ids"], request["vectors"], request["categories"]))
    return {}


ROUTES = {
    "/prepare": prepare,
    "/ingest": ingest,
    "/finish": lambda _: {"ready": True},
    "/search": search,
    "/teardown": lambda _: {},
}


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    # Headers and body are two writes. With Nagle on, the body waits for a delayed ACK.
    disable_nagle_algorithm = True

    def reply(self, status, value):
        body = json.dumps(value).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/health":
            self.reply(200, {"protocol_version": 1, "name": "reference-custom-system", "version": "1"})
        else:
            self.reply(404, {"error": "unknown route"})

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        if length > MAX_BODY_BYTES:
            self.reply(413, {"error": "request too large"})
            return
        request = json.loads(self.rfile.read(length) or b"{}")
        route = ROUTES.get(self.path)
        if route is None:
            self.reply(404, {"error": "unknown route"})
        else:
            self.reply(200, route(request))

    def log_message(self, *_):
        pass


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 18089
    ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
