# POAM-090 — the product writes Parquet, and rrflowCLD has no format of its own

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-090`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Read on 2026-11-20 at `33159f8e2`. The owner's direction of 2026-11-19 and 2026-11-20 is that Parquet is a reference module for testing and comparison, never the product ([direction](../../roadmap/reference-modules-and-reasoning-fabric-owner-direction-20261120.md)). On `main`:

- `parquet` is a normal dependency of `rrd-query` (`crates/compute/rrd-query/Cargo.toml`), reached by every product binary through `rrd-engine`'s default `full` feature.
- The cold tier is that writer. `encode_cold_history_page` and `decode_cold_history_page` (`rrd-query/src/cold_history.rs`) call the Parquet crate directly, the file extension `.parquet` is in `rrd-engine/src/engine/cold_history/maintain.rs`, and the bundled install configuration turns it on (`[storage.cold_history]`).
- No trait or versioned reader stands between the cold tier and the format.
- rrflowCLD has no storage-tier home and no format of its own, so there is nothing native to compare Parquet or Vortex against.
- Vortex is named nowhere.

Since 2026-11-20 a columnar port exists in `rrd-store` with Parquet and Vortex as optional reference modules behind it. The cold tier does not use the port yet, and no native format is behind it.

The [database semantics audit](../../reference/data/database-semantics.md) at
`b7def0fc7` confirms authenticated, bounded Parquet generation readback with
artifact/source verification. The outstanding deficiency is production use of
the columnar port and native format plus merged query/context access, not the
absence of all cold-history reads. Existing readback does not close this item.

## Impact

The cold tier cannot be compared with a reference, because it is the reference. Every product build carries the Parquet and Arrow writer stack. What rrflowCLD is meant to be for reasoning, columnar history read cold at one stamp, has no implementation to tune.

## Owning gates

F-06 owns the cold tier. Kernel contract K10 owns the versioned reader.

Roadmap owner: [Gate F](../../roadmap/rrflow-1.0/gate-f.md).

## Closure evidence

Closed when a columnar port in the storage tier has a native rrflowCLD implementation that the product uses; Parquet and Vortex are optional reference modules behind the same port that no product package enables, enforced by the workspace architecture test; and a comparison of the three on the cold-history workload is recorded.
