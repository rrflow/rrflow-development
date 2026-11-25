# The kernel crate moves out from under the engine

**Status:** active structural move; no behavior changed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/kernel-own-folder-20261125`
**Owner:** [Code structure and conventions](../../../architecture/code-structure-and-conventions.md), tier 0, and kernel package K13 in the [kernel contract](../../../roadmap/kernel-contract.md).

## Change brief

Baseline `68e34e83c`. The owner ordered on 2026-11-24 that existing code is aligned before any new code, and approved starting the moves ("go"). `rrflow-kernel` lived at `crates/authority/rrd-engine/kernel/`, inside the engine's directory, although 21 workspace crates depend on it and the storage crates below the engine are among them. Target: the same crate, unchanged, at `crates/foundation/rrflow-kernel/`, the tier-0 group the structure record names. Scope: the directory move and every path literal that named the old location. Stop if any Rust source line other than a path literal has to change, or if a test has to be removed.

## Traceability

| Item | Before | After |
|---|---|---|
| Crate | `rrflow-kernel` at `crates/authority/rrd-engine/kernel` | `rrflow-kernel` at `crates/foundation/rrflow-kernel` |
| Package name, features, dependencies, public API | unchanged | unchanged |
| `rrd_engine::kernel` re-export | present | present, untouched |
| Tracked files moved | 61 | 61, all as renames |

Changed besides the renames: the workspace member and dependency path in `Cargo.toml`; fixture paths in two `rrd-contract` tests; path tables in `direct_read_source_closure.rs` and `workspace_architecture/storage.rs`; the source comment in the generated `signal_catalogue.rs` and its generator; path tables in four CI scripts and two of their tests; `code_shape_baseline.json`; and the path text in eleven documentation records. The system overview no longer calls the library nested.

Not changed: `scripts/ci/test_select_ci_lanes.py`. Its fixture builds a synthetic workspace with a member nested inside another member to test that the lane selector handles nesting. That capability stays tested even though the real workspace no longer has a nested member. Change journals keep the old path as history.

## Verification

On Windows, baseline plus this change:

- `cargo check --workspace --all-targets --locked`: finished, no errors. `Cargo.lock` unchanged.
- `cargo test -p rrflow-kernel --locked`: every target passed (75 unit tests, 1 ignored, plus the integration targets).
- `cargo test -p rrd-contract --locked --test reasoning_tree_contract --test router_contract`: 3 and 6 passed.
- `cargo test -p rrd-engine --locked --test direct_read_source_closure`: 1 passed.
- `cargo test -p rrd-engine --locked --test workspace_architecture`: 34 passed.
- Every `scripts/ci/check_*.py`: passed, including documentation policy, code shape, generated surfaces, selected tests and workflow policy.
- `python -m unittest discover -s scripts/ci`: 293 run, 6 errors, all in `test_windows_harness_image` because no Windows harness binary is configured on this host. The same 6 errored before the change.
- Not run: the full workspace test suite, any Linux or macOS build, clippy.

## Surfaced during the work

A blanket path replacement first broke 20 lane-selection tests by rewriting their synthetic fixture. The file was restored and the count returned to the baseline.

No objective, roadmap or POA&M status changes.
