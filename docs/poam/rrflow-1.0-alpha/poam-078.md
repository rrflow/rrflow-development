# POAM-078 — the estate-to-engram rename is half applied

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-078`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

A code read at `de435238` on 2026-11-11 found the following:

- **Two planners for one concept.** `crates/authority/rrd-engine/src/engine/memory_engram.rs` and `memory_estate.rs` are both public planners. `memory_estate.rs:216` delegates to the engram planner, and their tests are duplicated in `crates/authority/rrd-engine/tests/memory_engram.rs` and `memory_estate.rs`.
- **Names with a lowercase "engram" mid-word.** Thirty occurrences, among them `MemoryengramPlan` and `PersistMemoryengram` (`crates/transport/rrd-contract/src/memory_engram.rs`), `DataengramOrganisation` and `InstalledengramRecord`.
- **Two resource kinds.** `ResourceKind` has both `Estate` and `Engram` (`crates/transport/rrd-contract/src/lib.rs:5656-5657`), and nothing uses `ResourceKind::Engram`.
- **Other inconsistent names:**
  - The `rrd-estate` crate and its routes keep "estate".
  - "Organisation" and "Organization" are both used.
  - "CLD" names both the process-local hit counters (`crates/authority/rrd-engine/src/runtime/coldlight.rs`) and the cold tier.

## Impact

One concept has several names in the public contract and generated SDKs. A reader can't tell which name is canonical, and AI clients learn both.

## Owning gates

A-01 and A-07 own frozen names and public vocabulary.

Roadmap owner: [Gate A](../../roadmap/rrflow-1.0/gate-a.md).

## Closure evidence

Closed when each concept has one name in the code, contract, catalogue, SDKs and docs, recorded in the vocabulary, with no duplicate planner, unused variant or mixed-case identifier left.
