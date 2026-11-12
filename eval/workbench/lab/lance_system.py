#!/usr/bin/env python3
"""Lance in the comparison lab: LanceDB's embedded engine behind the custom protocol.

Lance has no server, so this process is its server. JSON handling and Python's lock
count against Lance here, which each binding states as a limit of its results.

Usage: lance_system.py PORT DIRECTORY
"""
import json
import sys
import threading
from datetime import timedelta
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from importlib.metadata import version

import lancedb
import numpy
import pyarrow
from lancedb.index import Bitmap, HnswSq, IvfRq

MAX_BODY_BYTES = 256 * 1024 * 1024
METRICS = {"cosine": "cosine", "dot": "dot", "euclidean": "l2"}
BUILD_TIMEOUT = timedelta(hours=6)


class Store:
    directory = None
    connection = None
    table = None
    name = None
    dimensions = None
    metric = None
    index = None
    build = None
    build_error = None
    ready = False


def prepare(request):
    Store.connection = lancedb.connect(Store.directory)
    Store.name = request["name"]
    Store.dimensions = request["dimensions"]
    Store.metric = METRICS[request["metric"]]
    Store.index = request["index"]
    Store.build, Store.build_error, Store.ready = None, None, False
    schema = pyarrow.schema(
        [
            ("id", pyarrow.int64()),
            ("vector", pyarrow.list_(pyarrow.float32(), Store.dimensions)),
            ("category", pyarrow.int32()),
        ]
    )
    Store.table = Store.connection.create_table(Store.name, schema=schema, mode="overwrite")
    return {}


def ingest(request):
    values = numpy.asarray(request["vectors"], dtype=numpy.float32).reshape(-1)
    vectors = pyarrow.FixedSizeListArray.from_arrays(pyarrow.array(values), Store.dimensions)
    Store.table.add(
        pyarrow.table(
            {
                "id": pyarrow.array(request["ids"], type=pyarrow.int64()),
                "vector": vectors,
                "category": pyarrow.array(request["categories"], type=pyarrow.int32()),
            }
        )
    )
    return {}


def index_config():
    kind = Store.index["index"]
    if kind == "hnsw":
        # Lance recommends one partition for HNSW unless a table is very large.
        return HnswSq(
            distance_type=Store.metric,
            num_partitions=1,
            m=Store.index["m"],
            ef_construction=Store.index["ef_construction"],
        )
    if kind == "ivf_rq":
        return IvfRq(distance_type=Store.metric, num_partitions=Store.index["partitions"])
    raise ValueError(f"this shim does not build a {kind} index")


def build():
    try:
        Store.table.optimize()
        if Store.index["index"] != "exact":
            Store.table.create_index("vector", config=index_config(), wait_timeout=BUILD_TIMEOUT)
        Store.table.create_index("category", config=Bitmap(), wait_timeout=BUILD_TIMEOUT)
        # No optimize here: it merges small IVF partitions, and took 336 down to 7, so the
        # searches would no longer measure the index the manifest declares.
        Store.ready = True
    except Exception as error:  # A failed build is the result, so it reaches the harness.
        Store.build_error = str(error)


def finish(_):
    if Store.build_error:
        raise RuntimeError(f"index build failed: {Store.build_error}")
    if Store.build is None:
        Store.build = threading.Thread(target=build, daemon=True)
        Store.build.start()
    return {"ready": Store.ready}


def search(request):
    k, setting = request["k"], request["setting"]
    query = (
        Store.table.search(
            numpy.asarray(request["vector"], dtype=numpy.float32), vector_column_name="vector"
        )
        .distance_type(Store.metric)
        .limit(k)
        .select(["id", "_distance"])
    )
    if Store.index["index"] == "exact":
        query = query.bypass_vector_index()
    if setting.get("ef") is not None:
        query = query.ef(setting["ef"])
    if setting.get("nprobes") is not None:
        query = query.nprobes(setting["nprobes"])
    if setting.get("exact_rerank") is not None:
        # Lance reranks k times this factor and refuses more candidates than ef, so round down.
        query = query.refine_factor(max(1, setting["exact_rerank"] // k))
    if request.get("category") is not None:
        query = query.where(f"category = {int(request['category'])}", prefilter=True)
    return {"ids": query.to_arrow()["id"].to_pylist()}


def evidence():
    if Store.index["index"] == "exact":
        return {"evidence": "exact scan, with no vector index"}
    stats = Store.table.index_stats("vector_idx")
    if stats is None:
        return {"evidence": "no vector index exists, so searches scanned every row"}
    plan = (
        Store.table.search(numpy.ones(Store.dimensions, dtype=numpy.float32), vector_column_name="vector")
        .limit(1)
        .explain_plan()
    )
    used = "uses" if "ANN" in plan else "does not use"
    details = Store.table.to_lance().stats.index_stats("vector_idx")
    partitions = sum(len(segment["partitions"]) for segment in details["indices"])
    return {
        "evidence": f"{stats.index_type}: {stats.num_indexed_rows} rows indexed, "
        f"{stats.num_unindexed_rows} not, in {partitions} partitions; the query plan {used} it"
    }


def teardown(_):
    if Store.connection is not None and Store.name is not None:
        Store.connection.drop_table(Store.name, ignore_missing=True)
    return {}


ROUTES = {
    "/prepare": prepare,
    "/ingest": ingest,
    "/finish": finish,
    "/search": search,
    "/teardown": teardown,
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
            self.reply(
                200,
                {
                    "protocol_version": 1,
                    "name": "lancedb",
                    "version": f"{version('lancedb')}, lance {version('pylance')}",
                },
            )
        elif self.path == "/evidence":
            self.reply(200, evidence())
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
            return
        try:
            self.reply(200, route(request))
        except Exception as error:  # A refusal stays visible to the harness as its text.
            self.reply(500, {"error": str(error)})

    def log_message(self, *_):
        pass


if __name__ == "__main__":
    Store.directory = sys.argv[2]
    ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), Handler).serve_forever()
