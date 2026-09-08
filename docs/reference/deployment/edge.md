# RRFlow offline edge artifact profile

**Status:** active narrow deployment proof; not an rrflowDB profile
**Coordinate:** `rrflow://rrflow-instance/data/reference/deployment/edge`
**Owner:** no-network packaging of local feature-hash generation and mmap exact search

`rrflow-edge` is a thin executable around `OfflineEdgeIndex`. It proves that a
small binary can build and query the compact dense artifact without an HTTP
client, async runtime, model downloader, cluster service, object-store SDK,
GPU runtime, or TLS dependency. It is not rrflowMX, rrflowKV, rrflowDB,
rrflowQL, a project installer, an attunement runtime, or a Connectome server.

The current CLI supports only:

```text
rrflow-edge build <documents.json> <artifact> [dimensions] [seed]
rrflow-edge query <artifact> <text> [top_k] [dimensions] [seed]
```

Build hashes the supplied document corpus, creates deterministic FeatureHash
embeddings, constructs one model-bound compact dense artifact, and publishes
it atomically. Query verifies and mmaps the artifact, generates one
FeatureHash query embedding under `NetworkPolicy::Deny`, and runs exact cosine
search. A different feature-hash seed cannot open the artifact as the same
model.

## What this profile proves

`crates/adapters/rrflow-edge/tests/offline.rs` proves deterministic build,
model mismatch rejection, mmap search, no-network policy, and shared-corpus
results. CI rejects common networking dependencies from the edge tree and from
the optional local FastEmbed tree, and currently enforces a 2 MiB release
binary regression ceiling.

The retained fixed-seed observation is
[`m6-edge-local-10000x128.json`](../../evidence/m6-edge-local-10000x128.json).
It records the exact host, toolchain, corpus, commands, artifact size, memory,
latency, and socket counters for one 10,000-by-128 run. That record is local
regression evidence only. Its feature-hash ranking is not semantic-quality
evidence, its hardcoded CI ceiling is not a production sizing promise, and it
does not establish superiority over another database.

## Missing product behavior

This binary does not open persistent engine state, authenticate a project
session, run rrflowQL/DataFusion, traverse graph relationships, fuse BM25 and
vector results, execute reasoning trees, install adapters, perform attunement,
or expose HTTP/WebSocket/MCP/SDK operations. It cannot be cited as the first
usable alpha or as evidence that Connectome can attach to rrflowDB.

Gate D owns reproducible installation and attunement. Gates C, E, and F own the
persistent data, index, graph, and analytical paths. Gates H and J own external
delivery and clean-machine proof. Any retained edge distribution must package
those same engine capabilities rather than grow this artifact helper into a
parallel runtime.
