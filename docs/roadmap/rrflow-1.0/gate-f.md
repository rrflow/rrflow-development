# Gate F — connect rrflowKV to Arrow/DataFusion correctly

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/gate-f`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | F-01 | Replace pre-materialized `Vec<QueryRow>` snapshots with a stamped `RrflowKvTableProvider` streaming bounded Arrow `RecordBatch` values from rrflowKV memtables and immutable segment pages, including relationship adjacency selected for a query. Relationship expansion must use that governed storage stream, not raw-pointer chasing or an unmetered path around the configured physical page cache. DataFusion receives borrowed buffers only when the physical encoding is eligible and receives pool-owned decoded buffers otherwise. | `rrd-query`, `rrd-store`, `rrd-lsm` | Provider tests prove batch streaming, buffer lifetime safety, fixed memory bounds on data larger than query memory, exact results across borrowed and decoded paths, and cache/accounting parity for relationship work. |
| [ ] | F-02 | Push supported projection, predicate, limit, and ordering requirements into rrflowKV key/page scans; report unsupported predicates honestly and account for physical I/O, decoded, copied, and allocated bytes. | `rrd-query`, `rrd-store` | Explain and physical-counter tests show less I/O and decoding for selective queries while results equal the unoptimized oracle; no test equates memory mapping with universal zero-copy. |
| [ ] | F-03 | Implement internal relationship expansion, BM25 candidate generation, HNSW candidate generation, and `math::rrf()` as native physical operators that exchange stamped Arrow batches with DataFusion. Relationship expansion consumes the E-01/F-01 stream and respects the same authorization, cache, stamp and budgets. | `rrd-query`, `rrd-vector` | Mixed query tests prove one stamp, deterministic ordering, exact reranking, configured cache use and no external database round trip or bypass of the storage stream. |
| [ ] | F-04 | Enforce query memory, spill, elapsed-time, scanned-key, graph-step, candidate, and result-byte budgets across native and DataFusion operators. | `rrd-query`, `rrd-engine` | Each limit has a deterministic truncation or denial fixture with measured resource evidence. |
| [ ] | F-05 | Add read-stamp/query/projection caches with byte accounting and cursor/schema invalidation. | `rrd-engine`, `rrd-query` | Repeated-query benchmark shows bounded reuse; mutation and schema tests prove stale batches are never returned. |
| [ ] | F-06 | rrflowDB coordinates publication of selected hot history through rrflowKV to a format-independent rrflowCLD backend. rrflowQL requests a stamped scan from DB. Keep each artifact's role, versions, tombstones, provenance, authorization and read stamps explicit before reclaiming its source. | DB, KV and CLD own placement, storage access and encoding. TX owns atomic visibility. QL consumes the scan. | Run the identical historical point/range, projection and recovery corpus against Vortex and Parquet. Answers and both temporal coordinates must agree through correction, interruption, full disk, corruption, concurrent reads, restart and reclamation. Record selected format/path, publication and read stamps, bytes read, decoding work, latency and peak memory. Compare pruning with an unpruned exact read. |

The installed path currently writes Parquet cold history. The columnar comparison port
has separate reference adapters. The first change moves the existing generation writer
and authenticated reader behind the same storage contract, preserving source digests
and stamped answers. Then connect the DB scan to QL, with format-specific dependencies
confined to CLD. Vortex is the primary candidate and Parquet the required reference
backend. Select a deployment default from measured point-read, scan, encoding, size
and recovery workloads. Reclamation requires proven coverage of retained stamps.

Both planned product features, `cld-vortex` and `cld-parquet`, must select the
same integration and benchmark targets. The existing `vortex-reference` and
`parquet-reference` comparison features establish no installed-path parity. Record
buffer ownership, bytes copied and bytes decoded when promoting a slice to MX.

The backend implementation package must provide the following verification
commands. The feature names and benchmark target below are planned interfaces.
Both runs must exercise the same data and assertions. Keep the process-level
correction, focus change and reopen test in the installed engine suite as well.

```sh
cargo test --locked -p rrd-store --features cld-vortex
cargo test --locked -p rrd-store --features cld-parquet
cargo bench --locked -p rrd-store --features cld-vortex --bench cold_history
cargo bench --locked -p rrd-store --features cld-parquet --bench cold_history
```

## Consumption by the real-project build

D-05 uses F-01/F-02/F-04 when a phase actually consumes analytical streaming;
F-03 owns mixed graph/lexical/vector analytical operators and F-05 cache reuse.
Native source commits and exact native queries retain their existing engine
path. Do not postpone all source ingestion behind unrelated analytical cache
optimization, or call eager whole-engram materialization the finished streaming
implementation. Each consuming package names its missing provider/operator
prerequisite and measured allocation/batch-lifetime oracle.

## Exit condition

Gate F exits only when DataFusion consumes streamed authoritative access paths
with bounded materialization, and both F-06 backends pass the same cold-storage
publication, query and recovery cases.
