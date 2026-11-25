# Component map corrected to the owner's 2026-11-24 definitions

**Status:** active documentation correction; no code changed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/component-map-owner-definitions-20261124`
**Owner:** [RRFlow system overview](../../../architecture/system-overview.md), canonical component terminology.

## Change brief

Baseline `68e34e83c`. On 2026-11-24 the owner restated the components and ordered that no new code is written until existing code is aligned to them. The system overview defined rrflowMX as volatile working memory, did not name rrflowCX, and described rrflowKV only as an access boundary. Scope: `docs/architecture/system-overview.md`. Stop if a status in the objectives, roadmap or POA&M would have to change.

Owner statements recorded: rrflowKV is "a low-level, versioned, embedded, ACID-compliant, key-value database for Rust"; "MX was never folded into KV", it is "a memory and knowledge layer for AI agents that runs on the same underlying engine as the rrflowDB database platform"; "CX is connections distributed services so we can scale" on the hosted offering.

Files read: `system-overview.md`, the rrflowMX passages of `README.md`, `kernel-contract.md`, `database-semantics.md`, `instance-topology.md`, `engine-data-flow.md` and `rrflow-1.0-alpha.md`, and the workspace manifests through `cargo metadata`.

## Changes

- Platform map: rrflowMX and rrflowKV are no longer drawn inside rrflowDB, and rrflowCX is added as an optional layer.
- Terminology: rrflowKV and rrflowMX carry the owner's definitions, the code previously called rrflowMX is listed as the volatile storage profile, and rrflowCX is new.
- A dated section names the seven records that still use rrflowMX in the old sense.

## Evidence for the "current" column

`cargo metadata --no-deps --locked` on the baseline: no workspace crate is named for KV, MX, TX, DB or CX. `rrd-store/src/rrflow_kv/` is a module. It references `crate::access` 11 times, and `access`, `archive` and `backup` reference it. `RrflowMxStore` is defined in `rrd-store/src/engine.rs`. `rrd-kubernetes` has no consumer in the workspace. These are searches of import paths, not trial builds.

## Verification

- `python scripts/knowledge/render_navigation.py`, then `python scripts/ci/check_documentation.py`: passed.
- Not verified: time-series and geospatial capability in code, and which parts of `rrd-cluster` are working.

## Open conflict for the owner

The overview states that Rust packages are named `rrd-*` for internal boundaries and `rrflow-*` for outward adapters. `rrflow-kernel`, `rrflow-cld` and `rrflow-arrow` already break that rule, and naming crates after the components would replace it. That sentence is unchanged here.

No objective, roadmap or POA&M status changes.
