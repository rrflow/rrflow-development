#!/usr/bin/env python3
"""Check lance_system.py against a real Lance, which the Rust tests do not build against.

The index it serves has to be the one a manifest asked for, and its replies must not stall.
Both went wrong once without a visible failure, so a result looked good and was not.

Run from a venv holding lance-requirements.txt. A stalled reply only shows on Linux, so the
stall check proves nothing elsewhere. It exits non-zero and names each check that failed.

Usage: check_lance_shim.py
"""
import http.client
import sys
import tempfile
import threading
import time
from http.server import ThreadingHTTPServer

import numpy

import lance_system as shim

POINTS = 20_000
DIMENSIONS = 32
PARTITIONS = 64
CLIENTS = 8
REQUESTS_PER_CLIENT = 50
# Far above a local round trip and below the 40 ms a delayed ACK costs.
MAX_P95_MS = 20.0


def build(index):
    shim.prepare({"name": "check", "dimensions": DIMENSIONS, "metric": "cosine", "categories": 10, "index": index})
    vectors = numpy.random.default_rng(20261103).normal(size=(POINTS, DIMENSIONS)).astype(numpy.float32)
    for start in range(0, POINTS, 500):
        ids = list(range(start, start + 500))
        shim.ingest({"ids": ids, "vectors": vectors[start:start + 500].tolist(), "categories": [i % 10 for i in ids]})
    while not shim.finish({})["ready"]:
        if shim.Store.build_error:
            raise RuntimeError(shim.Store.build_error)
        time.sleep(0.1)
    return shim.evidence()["evidence"]


def request_times_ms():
    server = ThreadingHTTPServer(("127.0.0.1", 0), shim.Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    samples = []

    def client():
        connection = http.client.HTTPConnection("127.0.0.1", server.server_address[1])
        for _ in range(REQUESTS_PER_CLIENT):
            started = time.perf_counter()
            connection.request("GET", "/health")
            connection.getresponse().read()
            samples.append((time.perf_counter() - started) * 1000)
        connection.close()

    threads = [threading.Thread(target=client) for _ in range(CLIENTS)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    server.shutdown()
    return sorted(samples)


def main():
    failed = []

    def check(name, passed, detail):
        print(f"{'ok  ' if passed else 'FAIL'} {name}: {detail}")
        if not passed:
            failed.append(name)

    with tempfile.TemporaryDirectory(prefix="lance_shim_check_") as directory:
        shim.Store.directory = directory
        evidence = build({"index": "ivf_rq", "partitions": PARTITIONS})
        check("ivf keeps the partitions a manifest asked for", f"in {PARTITIONS} partitions" in evidence, evidence)
        evidence = build({"index": "hnsw", "m": 16, "ef_construction": 100})
        check("hnsw keeps its one partition", "in 1 partitions" in evidence, evidence)
        shim.teardown({})
        shim.Store.table = None
        shim.Store.connection = None

    samples = request_times_ms()
    p50, p95 = samples[len(samples) // 2], samples[len(samples) * 95 // 100]
    check(
        f"replies do not stall at {CLIENTS} clients",
        p95 < MAX_P95_MS,
        f"empty request p50 {p50:.2f} ms, p95 {p95:.2f} ms",
    )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
