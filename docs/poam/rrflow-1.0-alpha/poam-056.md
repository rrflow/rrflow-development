# POAM-056 — HNSW search reads the vector-version source before traversal

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-056`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

The parent ledger owns priority and lifecycle status; this detail cannot change
those independently.

## Observed deficiency

On main `1fd7f163de3448c694e09c916b328540303eab1c`,
`search_vectors_at` in `crates/authority/rrd-engine/src/engine/vector/search.rs`
tries the specialized TurboQuant generation path first. When that path does not
serve a request, the named approximate route invokes `read_versioned` with a
`Vectors { kind: None }` source and builds candidates from those changes before
loading and searching HNSW. The read is subject to the request's storage-key
budget. This is distinct from [POAM-055](poam-055.md), which describes an
unservable request that is refused only after the vector read.

In a single release-build Linux ARM64 diagnostic of 20,000 generated 384-dimensional
vectors with `ef=32`, the installed RRFlow engine reported the HNSW path, served
all 100 queries in each of the unfiltered and filtered cases, and reported median
client latencies of 1,765.8 ms and 1,743.8 ms respectively. The oracle reported
mean recall of 0.982 and 1.0. The [retained report and samples](../../../eval/workbench/lab/evidence/arm-20k-20261106/report.json)
and [journal](../../evidence/change-journals/gate-j/comparison-lab-public-vector-paths.md)
state the source, identities, conditions, and limits. The host was shared and
other work ran during the measurement. No profile attributes the latency to a
specific function, and this run is not a release qualification or a general
performance comparison.

## Impact

A successful approximate HNSW search still performs a potentially corpus-sized
versioned read, putting work and storage-budget pressure ahead of graph traversal.

## Owning gates

E-04 and OBJ-13 (bounded vector search with an immutable generation and exact delta overlay).

Roadmap owner: [Gate E](../../roadmap/rrflow-1.0/gate-e.md).

## Closure evidence

An installed public-engine test proves that a selected HNSW generation and its
exact delta overlay return the correct visible results for filtered and unfiltered
queries, updates, deletes, moved vectors, retained read stamps, and reopen. Read
evidence and multiple corpus sizes show the generation path does not require a
full vector-version source read on each search. Profile retained requests to
attribute costs; repeat a pinned, comparable, matched-quality benchmark with
host interference controlled before claiming improvement. Missing generations,
stale overlays and exhausted budgets must fail or use an explicitly authorized
correct fallback without presenting approximate candidates as canonical results.
