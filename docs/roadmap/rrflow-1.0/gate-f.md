# Gate F — connect rrflowKV to Arrow/DataFusion correctly

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/gate-f`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | F-01 | Replace pre-materialized `Vec<QueryRow>` snapshots with a stamped `RrflowKvTableProvider` streaming bounded Arrow `RecordBatch` values from rrflowKV memtables and immutable segment pages, including relationship adjacency selected for a query. Relationship expansion must use that governed storage stream, not raw-pointer chasing or an unmetered path around the configured physical page cache. DataFusion receives borrowed buffers only when the physical encoding is eligible and receives pool-owned decoded buffers otherwise. | `rrd-query`, `rrd-store`, `rrflow-lsm` | Provider tests prove batch streaming, buffer lifetime safety, fixed memory bounds on data larger than query memory, exact results across borrowed and decoded paths, and cache/accounting parity for relationship work. |
| [ ] | F-02 | Push supported projection, predicate, limit, and ordering requirements into rrflowKV key/page scans; report unsupported predicates honestly and account for physical I/O, decoded, copied, and allocated bytes. | `rrd-query`, `rrd-store` | Explain and physical-counter tests show less I/O and decoding for selective queries while results equal the unoptimized oracle; no test equates memory mapping with universal zero-copy. |
| [ ] | F-03 | Implement internal relationship expansion, BM25 candidate generation, HNSW candidate generation, and `math::rrf()` as native physical operators that exchange stamped Arrow batches with DataFusion. Relationship expansion consumes the E-01/F-01 stream and respects the same authorization, cache, stamp and budgets. | `rrd-query`, `rrd-vector` | Mixed query tests prove one stamp, deterministic ordering, exact reranking, configured cache use and no external database round trip or bypass of the storage stream. |
| [ ] | F-04 | Enforce query memory, spill, elapsed-time, scanned-key, graph-step, candidate, and result-byte budgets across native and DataFusion operators. | `rrd-query`, `rrd-engine` | Each limit has a deterministic truncation or denial fixture with measured resource evidence. |
| [ ] | F-05 | Add read-stamp/query/projection caches with byte accounting and cursor/schema invalidation. | `rrd-engine`, `rrd-query` | Repeated-query benchmark shows bounded reuse; mutation and schema tests prove stale batches are never returned. |
| [ ] | F-06 | rrflowDB coordinates publication of selected hot history through rrflowKV to a format-independent rrflowCLD backend. rrflowQL requests a stamped scan from DB. Keep each artifact's role, versions, tombstones, provenance, authorization and read stamps explicit before reclaiming its source. | DB, KV and CLD own placement, storage access and encoding. TX owns atomic visibility. QL consumes the scan. | Run the identical historical point/range, projection and recovery corpus against Vortex and Parquet. Answers and both temporal coordinates must agree through correction, interruption, full disk, corruption, concurrent reads, restart and reclamation. Record selected format/path, publication and read stamps, bytes read, decoding work, latency and peak memory. Compare pruning with an unpruned exact read. |

The installed engine publishes immutable change generations through
`rrflow-cld`. The writer comes from the installation policy and each reader
comes from the generation's receipt. The CLI bundles Parquet and can include
Vortex. Both formats preserve the original and corrected mutations across
process restart. The [CLD change journal](../../evidence/change-journals/gate-f/cld-storage-owner.md)
records the source paths, verification commands and measured comparison.

F-06 remains open. The next read boundary is one DB scan that gives QL
the correct result across hot and cold data at the requested valid time
and knowledge cursor. Before reclaiming an LSM range, prove coverage and
recovery for every retained stamp. Record buffer ownership, bytes copied and
decoded when promoting a slice to MX. Vortex remains the primary candidate
for workload evaluation. Format selection requires installed point-read,
range-scan, encoding, size and recovery measurements.

The [workspace storage checks](../../../crates/authority/rrd-engine/tests/workspace_architecture/storage.rs)
enforce ownership of the format dependencies. Use the same corpus for both
feature-isolated runs. The columnar benchmark measures encoding, full decode
and projected decode in memory. The installed lifecycle check runs the
publication, correction and restart sequence with both writers.

```sh
cargo test --locked -p rrflow-cld --no-default-features --features cld-vortex
cargo test --locked -p rrflow-cld --no-default-features --features cld-parquet
cargo bench --locked -p rrflow-cld --profile dev --no-default-features --features cld-vortex --bench columnar -- --rows 5000 --trials 3
cargo bench --locked -p rrflow-cld --profile dev --no-default-features --features cld-parquet --bench columnar -- --rows 5000 --trials 3
cargo test --locked -p rrflow-cli --features cld-vortex --test installed_lifecycle -- engrams_and_history::configured_cold_history_publishes_and_reads_after_process_restart --exact
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
