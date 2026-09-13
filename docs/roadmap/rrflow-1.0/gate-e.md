# Gate E — make graph and indexes native incremental access paths

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/gate-e`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | E-01 | Replace graph reconstruction and linear relation scans with temporal outgoing/incoming adjacency prefix scans. | `rrd-store`, `rrd-query` | Directed/typed/depth-bounded traversal matches the exact graph oracle and physical evidence scales with visited edges, not estate size. |
| [ ] | E-02 | Persist scalar and unique indexes transactionally with record mutations. | `rrd-store`, `rrd-query` | Insert/update/retire/conflict/reopen differential proves index and authoritative record cannot drift. |
| [ ] | E-03 | Persist incremental BM25 dictionary, document statistics, postings, positions, and tombstones at a declared source cursor. Select posting-partition codecs from measured raw/bit-packed, partitioned Elias-Fano, and PFOR candidates; codec choice is authenticated metadata and cannot change lexical semantics. | `rrd-query`, `rrd-store` | Incremental results equal a full exact rebuild across update/delete/reopen/corruption fixtures; adversarial sparse/dense/clustered postings prove exact seek/iteration while bytes, decode work, latency, and update/compaction amplification justify each retained codec. |
| [ ] | E-04 | Commit canonical vectors with an atomic index delta; search an immutable HNSW generation plus exact delta overlay and exact-rerank final candidates. Evaluate TurboQuant_prod (MSE quantizer plus one-bit QJL residual) as an authenticated candidate representation and evaluate an LSM-VEC-style disk graph only after the exact/HNSW baseline exists; neither is an assumed default. | `rrd-vector`, `rrd-store`, `rrd-engine` | Exact oracle, recall@k, estimator error/bias, filtered search, update/delete, stale generation, reopen, interrupted-build, build/compaction amplification, RSS, and latency tests pass on declared RRFlow corpora. Approximate candidates never become authoritative results and exact reranking/fallback remains available. |
| [ ] | E-05 | Add cost/selectivity estimates choosing point, range, scalar, BM25, exact-vector, or HNSW access without caller-selected internals. | `rrd-query` | Stable explain plans and adversarial fixtures prove correctness fallback when statistics or projections are absent/stale. |

## Exit condition

Gate E exits only when graph, lexical, scalar, and vector routes are real
bounded storage access paths with exact fallbacks.
