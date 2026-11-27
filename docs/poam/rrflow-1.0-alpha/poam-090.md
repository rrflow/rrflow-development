# POAM-090 — cold history bypasses the rrflowCLD backend boundary

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-090`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Read on 2026-11-20 at `33159f8e2`. The original [reference-module direction](../../roadmap/reference-modules-and-reasoning-fabric-owner-direction-20261120.md) explains the initial comparison harness. At that revision:

- `parquet` is a normal dependency of `rrflow-ql` (`rrflow/ql/Cargo.toml`), reached by every product binary through `rrd-engine`'s default `full` feature.
- The cold tier is that writer. `encode_cold_history_page` and `decode_cold_history_page` (`rrflow-ql/src/cold_history.rs`) call the Parquet crate directly, the file extension `.parquet` is in `rrd-engine/src/engine/cold_history/maintain.rs`, and the bundled install configuration turns it on (`[storage.cold_history]`).
- No trait or versioned reader stands between the cold tier and the format.
- rrflowCLD has no storage-tier home and no format of its own, so there is nothing native to compare Parquet or Vortex against.
- Vortex is named nowhere.

The November 22 ownership change moves the codec, columnar port, physical adapters and their workloads into `rrflow/cld`. Publication and receipt-directed readback use its `ColdStorageBackend` trait. [Candidate evidence](../../evidence/change-journals/gate-f/cld-storage-owner.md) records acceptance and remaining gaps.

The [database semantics audit](../../reference/data/database-semantics.md) at
`b7def0fc7` confirms authenticated, bounded Parquet generation readback with
artifact/source verification. A stamped query/context read spanning the eligible tiers remains outstanding.

## Impact

The installed history reader can reopen either supported format when its
codec is included in the build. Ordinary QL queries still lack a stamped
DB scan across hot and cold history. The codec benchmark measures encoding
and decoding in memory. Installed historical point-read costs, focus prefetch
and recovery following hot-space reclamation remain unqualified.

## Owning gates

F-06 owns the cold tier. Kernel contract K10 owns the versioned reader.

Roadmap owner: [Gate F](../../roadmap/rrflow-1.0/gate-f.md).

## Closure evidence

Close when production publication and stamped readback both use the RRFlow-owned
CLD port, QL and TX are format-independent, and the Vortex and Parquet feature
builds pass the same installed correctness, recovery and workload tests in
[F-06](../../roadmap/rrflow-1.0/gate-f.md). Publish comparative point and scan
costs at matched quality. Keep this deficiency open until both product paths
have that evidence.
