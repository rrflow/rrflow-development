# The storage crates stay in this repository as standalone crates

**Status:** active planning evidence; no code changed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/monorepo-standalone-crates-20261125`
**Owner:** [Code structure and conventions](../../../architecture/code-structure-and-conventions.md), sections 5 and 6.12.

## Change brief

Baseline `058f459a8`. Earlier on 2026-11-25 the owner described the storage engines and foundation primitives living in separate repositories pulled in through Cargo. The same day the owner decided: "we will keep it a monorepo for now however have clear workspaces defined for each".

Current behavior: section 6 of the structure record names every crate and tier but says nothing about which crates must be able to stand alone. Target: the decision is recorded and section 6.12 gives the four rules a standalone crate obeys, the crates it applies to, and where each one stands today. Scope: `code-structure-and-conventions.md` and this journal. No code. Stop if an objective, roadmap or POA&M status would have to change.

## Reading of the decision

"Clear workspaces defined for each" is read as one Cargo workspace in which each storage engine and foundation crate is a bounded member that depends on no product crate. It is not read as a separate Cargo workspace and lock file for each crate. The owner has not confirmed this reading.

## Measurements

Read at `058f459a8` from each crate's `Cargo.toml` and a text search of its sources:

- `rrflow-kernel` and `rrflow-arrow` have no first-party dependency.
- `rrd-lsm` depends on `rrflow-kernel` only; its 7 kernel references are all to `digest` (`digest.rs`, 249 lines).
- `rrflow-cld` depends on `rrflow-arrow`, and its test `tests/stamped_history.rs` uses `rrd_store::{RrflowKvStore, StorageEngine}`.
- `rrd-store` uses kernel product types, most often `RuntimeLogicalModel` (18) and `ScopeId` (14).
- `rrd-vector` references `rrd_store` 8 times, 7 of them `Error`.
- 84 first-party dependency lines under `crates/` name a `path`; 18 use `workspace = true`; the root manifest declares one first-party crate.

## Verification

- `python scripts/knowledge/render_navigation.py` regenerated the index.
- `cargo test -p rrd-engine --locked --test workspace_architecture`: 34 passed. `check_documentation.py`: OK.
- Not run: any other build or test; no code changed. Rule 3 of section 6.12 has not been run for any crate.

No objective, roadmap or POA&M status changes.
