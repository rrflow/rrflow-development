# C-07g: account shared mutable read generations once

**Status:** active implementation evidence; dependency, candidate CI and native qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-shared-memtable-pins`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR165](https://github.com/rrflow/rrflow-development/pull/165)

## Baseline, failure and change

[PR165](https://github.com/rrflow/rrflow-development/pull/165) bound this package before source edits and recorded the public-accounting clarification before its documentation was changed. Public development main `68c06059fbba600af5f06a6de30667c07e35ae6b` is the baseline; opening commit `6f6ca69b` is empty and the branch ordinarily merged PR158 `38e2b306` as `a1f22d7dddc539b634e066e182e81b71f7ba4f66`. Driver is this session; PR152/153/158 and exact candidate CI remain dependencies.

The original implementation cloned the same immutable `Arc<MutableGeneration>` into overlapping read streams, but charged its full logical size separately for every view. The new real-database regression failed before repair with `ProjectedReadLimit { resource: PinnedBytes, limit: 4126, observed: 8252 }` for two readers of the same 4,126-byte generation. This is a measured component failure, not the retained 100,000-record run's missing numeric denial.

`ActiveReadViews` now maintains reference counts for nonempty mutable generations by live allocation identity and charges each once. Its strong owner prevents identity/address reuse until the last lease releases. The registry still separately charges distinct mutable allocations, including copies created by a write while an old view remains active; it validates the allocation and size before admitting, and mutates accounting only after all checks pass. Zero-byte generations are not retained by the registry, preserving writes after an empty view without forced copy-on-write. Manifest accounting, active-view and byte limits, overflow checks, reclamation roots, retained output pool, and each stream's logical `pinned_bytes` evidence remain. No limit is raised. The aggregate `read_view_ownership` and derived engine `pinned_read_bytes` counter intentionally change from a per-view mutable sum to one logical charge per live mutable allocation; they remain conservative logical accounting, not RSS or full process memory.

## Traceability and review

The previous per-view behavior was explicitly documented in `ProjectedReadBudget::max_pinned_bytes` and `Database::read_view_ownership` and asserted by `shared_segments_do_not_hide_memtable_pins_or_new_generations`. This package deliberately replaces that same-allocation assertion and updates the Rust documentation; the test still exercises distinct manifests, denied admission, old snapshots and release. Canonical destination is the existing `ActiveReadViews` registry; no fallback, alternate pool or source tree is created. The existing mutable-generation pool independently accounts copy-on-write generations. The externally projected diagnostic says conservative aggregate, not a mandated per-view sum. No wire shape or HTTP operation changes, but this public diagnostic and admission behavior is observably different and needs review.

Read README, alpha/C-07/POAM-035 and CI owners; PR158, PR162 and original retained failure; complete projected-lifetime and reader-lifetime tests, full changed source diff, `MutableMemory::prepare` and reservation/drop, `Database::begin_projected_read`, stream construction/terminal release, downstream `rrd-store` and engine diagnostic projection. Longer files were inspected at the relevant symbols. [Rust's Arc documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html#method.ptr_eq) establishes allocation identity and shared ownership; no upstream implementation or dependency was copied. Existing ownership counters and native failures are the trace; no additional resource ceiling or logging authority was added.

## Checks and limits

On Windows with isolated `target/c07-shared-memtable-pins-build`, four build jobs and test debug disabled:

- The initial real-database `cargo test --offline --locked -p rrd-lsm --test projected_lifetime shared_mutable_generation_is_charged_once_and_survives_owner_release` failed with the exact 4,126/8,252 denial above, then passed after the fix.
- `cargo test --offline --locked -p rrd-lsm --test projected_lifetime` passed 11 tests: shared owner admission, four concurrent views and independent values, separate copy-on-write accounting/denial and reservation release, empty-view no-copy behavior, distinct manifest retention, cancellation and drop. `cargo test --offline --locked -p rrd-lsm --lib segment::reader::lifetime_tests` passed five tests, including no retained mutable owners after completion, cancellation and failure.
- `cargo test --offline --locked -p rrd-lsm --quiet` passed its owning suite with explicitly ignored fixtures. `cargo test --offline --locked -p rrd-store --test mutable_admission --test semantic_commit_atomicity --test storage_inspection --quiet` passed all reported tests. `cargo clippy --offline --locked -p rrd-lsm --all-targets -- -D warnings` and `cargo fmt --all -- --check` passed.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::installed_capacity_workload_smoke -- --exact --ignored` passed one small real installed-process smoke with kill/reopen and replay. It is not the 100,000-record profile.

Navigation/inventory, documentation, workflow, staged diff and post-commit inventory must pass before push. Exact candidate CI, dependency merges, review, integrated 100,000-record characterization, full native matrix and 24-hour soak are not complete. The old run's pinned-byte values were not captured, so the component overcharge is a plausible mechanism, not an established cause or a proven repair of that run. C-07 and POAM-035 remain open.

Checklist: objective, starting revision, current/target behavior, public semantic change, implementation traceability, research, failure oracle, limits and stop conditions were recorded in the PR; the old regression, replacement evidence and remaining native/CI requirements are retained here.
