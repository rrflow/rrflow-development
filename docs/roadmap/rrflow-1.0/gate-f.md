# Gate F — connect rrflowKV to Arrow/DataFusion correctly

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/gate-f`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | F-01 | Replace pre-materialized `Vec<QueryRow>` snapshots with a stamped `RrflowKvTableProvider` streaming bounded Arrow `RecordBatch` values from rrflowKV memtables and immutable segment pages. DataFusion receives borrowed buffers only when the physical encoding is eligible and receives pool-owned decoded buffers otherwise. | `rrd-query`, `rrd-store` | Provider tests prove batch streaming, buffer lifetime safety, fixed memory bounds on data larger than query memory, and exact results across borrowed and decoded paths. |
| [ ] | F-02 | Push supported projection, predicate, limit, and ordering requirements into rrflowKV key/page scans; report unsupported predicates honestly and account for physical I/O, decoded, copied, and allocated bytes. | `rrd-query`, `rrd-store` | Explain and physical-counter tests show less I/O and decoding for selective queries while results equal the unoptimized oracle; no test equates memory mapping with universal zero-copy. |
| [ ] | F-03 | Implement graph expansion, BM25 candidate generation, HNSW candidate generation, and `math::rrf()` as native physical operators that exchange stamped Arrow batches with DataFusion. | `rrd-query`, `rrd-vector` | Mixed query tests prove one stamp, deterministic ordering, exact reranking, and no external database round trip. |
| [ ] | F-04 | Enforce query memory, spill, elapsed-time, scanned-key, graph-step, candidate, and result-byte budgets across native and DataFusion operators. | `rrd-query`, `rrd-engine` | Each limit has a deterministic truncation or denial fixture with measured resource evidence. |
| [ ] | F-05 | Add read-stamp/query/projection caches with byte accounting and cursor/schema invalidation. | `rrd-engine`, `rrd-query` | Repeated-query benchmark shows bounded reuse; mutation and schema tests prove stale batches are never returned. |

## Exit condition

Gate F exits only when DataFusion consumes streamed authoritative access paths
instead of hiding an eager whole-estate materialization.
