# The storage crates sit in `crates/storage/`, and rrflowLSM's crate is `rrflow-lsm`

**Status:** active landing evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/storage-folder-20261126`
**Owner:** [Code structure and conventions](../../../architecture/code-structure-and-conventions.md), sections 6.2 and 6.8 package 5; kernel package K13.

## Change brief

Baseline: the architecture manifest change of pull request 518, on `main` at `aa8bdc907`. Current behavior: rrflowLSM's crate is `rrd-lsm`, and it, `rrflow-cld` and `rrd-store` sit in `crates/persistence/`, a folder name that matches no tier of the plan. Target: `crates/storage/rrflow-lsm`, `crates/storage/rrflow-cld`, `crates/storage/rrd-store`; the package is `rrflow-lsm` and its library `rrflow_lsm`; `crates/persistence/` no longer exists. Scope: the three folders, every reference to the package, library or folder in code, manifests, CI workflows, scripts and current documents, and this journal. No behavior, format or public operation changes. Failure oracle: a test count for a moved crate differs from before, a stored format constant changes, or `Cargo.lock` differs in anything but the package name. Stop if a stored or wire value contains the old name.

Order: this is package 5, taken before packages 3 and 4. Package 3 removes `rrd-maintenance`, whose draft validation ideas the context-maintenance record still points at, and package 4 needs one identifier type chosen for the reasoning tree; both are recorded as unmeasured in section 6.9. Package 5 depends on neither.

Research decision: none. Trace and debugging decision: none; no runtime path changes.

## Traceability

| Was | Is | Tests that characterize it |
|---|---|---|
| `crates/persistence/rrd-lsm` (package `rrd-lsm`, library `rrd_lsm`) | `crates/storage/rrflow-lsm` (`rrflow-lsm`, `rrflow_lsm`) | the crate's own suite, unchanged in content |
| `crates/persistence/rrflow-cld` | `crates/storage/rrflow-cld` | the crate's own suite |
| `crates/persistence/rrd-store` | `crates/storage/rrd-store`; its name waits for the split into rrflowKV, rrflowTX and rrflowDB | the crate's own suite |

Every file moved with `git mv`; no file was deleted or merged.

## What changed

- The three folders moved. `rrd-store` moves now, under its present name, so that `crates/persistence/` is gone in one step.
- `rrd-lsm`, `rrd_lsm` and the folder path were replaced in code, manifests, workflows, scripts and current documents. `Cargo.lock` changed only in the package name.
- `crates/architecture.toml`: the three crates sit in their tier's folder; the pending list fell from 12 renames and 17 moves to 11 and 14.
- `.github/workflows/rrd-lsm-benchmark.yml` is now `rrflow-lsm-benchmark.yml`.
- Left as they were: the text of evidence, history and research records, the files under `eval/inputs` and `eval/results`, and the frozen `rrflow-1.0-active-change.json`. Only the 14 links in two evidence records that pointed at a moved file were repointed. The history files `rrd-lsm-migration.md` and `rrd-lsm-promotion-benchmark.md` keep their names.
- Not changed: any stored format constant. The segment and index magic values (`RRDSEG06`, `RRDIX006`) do not contain the crate name and the test that pins them passes.

## Verification (Windows, local)

- `cargo check --workspace --all-targets`: clean. `Cargo.lock` differs only in the package name `rrd-lsm` → `rrflow-lsm` (24 lines each way).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo fmt --all`: clean.
- `cargo test --locked -p rrd-engine --test workspace_architecture --test direct_read_source_closure`: 37 passed.
- `cargo test --locked -p rrd-contract`: 209 passed. `-p rrflow-qualification-storage`: 54 passed. `-p rrd-cluster`: 25 passed.
- `cargo test --locked -p rrflow-cld`: 12 passed, the same count as before the move.
- `cargo test --locked -p rrd-store`: 299 passed. The first run failed one test, `historical_storage_artifacts_remain_parseable_and_retain_failed_runs`, which found 0 of 35 retained files: the rewrite had changed its file-name filter from `rrd-lsm` to `rrflow-lsm`, and the August result files keep their names. The filter was restored and the suite passed.
- `cargo test --locked -p rrflow-lsm`: 276 passed, the same count as `rrd-lsm` gave before the move.
- `check_architecture.py`, `check_code_shape.py`, `check_selected_tests.py`, `check_workflow.py`, `check_documentation.py`, `build_execution_inventory.py` and the script tests `test_architecture`, `test_code_shape`, `test_selected_tests`, `test_select_ci_lanes`, `test_execution_inventory`, `test_ci_lanes_policy`, `test_cache_warm_policy`: OK. `test_select_ci_lanes` first failed two tests whose fixture still placed the crate under `persistence`; the fixture was corrected.
- Not run: `cargo test --workspace` (the engine, server, client and CLI suites), Linux, macOS. The candidate gate and the nightly matrix run them.

## Observable differences

- Trace targets follow the library name: `rrd_lsm::open`, `::transaction`, `::projected_read` and `::segment_write` are now `rrflow_lsm::...`. A log filter written against the old target no longer matches.
- The storage qualification report names option owners as `rrflow_lsm::...`; its fixture changed with it.

No objective, roadmap or POA&M status changes.
