# C-07: integrate shared mutable accounting and joined engine flow

**Status:** active integration evidence; dependencies, candidate CI and native qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-integrated-memtable-flow`
**Owner:** OBJ-02 / C-07g,h / POAM-035 / [PR166](https://github.com/rrflow/rrflow-development/pull/166)

## Scope and traceability

[PR166](https://github.com/rrflow/rrflow-development/pull/166) recorded this bounded integration before merging source. Public development main `68c06059fbba600af5f06a6de30667c07e35ae6b` is the base; empty opening commit `f1c7a124` ordinarily merged PR160 `5a3c6140edc04957dc7f3a60079333b6ac71f784` at `b02dc8663755552333f1e920e81001f044836934`, then merged PR165 `67dd886f`. The latter merge conflicted only in generated execution inventory. Driver is this session. Both parent implementations and their earlier PR152/153/158/162 dependencies remain independently owned and unmerged.

Current and canonical source: the existing sustained harness's real exact vector, rrflowQL and rrflowKV flow, forced restart, replay and numeric-only denial evidence from PR160/162; the existing `ActiveReadViews` shared mutable-generation admission and projected lifetime tests from PR165. The integration carries both into the same engine/storage and test tree. No implementation is moved, deleted or replaced by a parallel authority; no test, budget, configuration, public operation or native qualification threshold changes. The parent PR165 journal records its deliberate change to the aggregate pinned-byte diagnostic meaning. The POAM retains both the reproduced 4,126/8,252 component denial and the earlier 100,000-record failure with unknown numeric charge.

Read README, C-07/POAM-035 owners, both parent PR briefs and journals, the merged source/test and generated diff, the existing full installed smoke and reader-lifetime oracles, and repository CI procedure. The parents cite Rust allocation and database fault-testing research; this integration copies no upstream code and introduces no new trace or resource policy. Existing native-process, cursor, independent score, error and receipt evidence remain the proof boundary.

## Checks and limits

On Windows in isolated `target/c07-integrated-memtable-build` with four build jobs and test debug disabled, `cargo test --offline --locked -p rrd-lsm --test projected_lifetime` passed all 11 tests. `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::installed_capacity_workload_smoke -- --exact --ignored` passed on the combined source: actual installed engine writes, independently checked exact vector result and score, rrflowQL record and relationship reads at the search cursor, process kill/reopen, repeat, and idempotent receipt replay. `cargo test --offline --locked -p rrflow-cli --test installed_reopen --quiet` passed 17 with seven long/native tests ignored. The parent PR165 independently passed the full rrd-lsm suite, five reader-lifetime tests, selected cross-store admission/atomicity/inspection tests, strict Clippy and a separate installed smoke.

`cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings` and `cargo fmt --all -- --check` passed on the combined tree. Navigation/inventory, documentation, workflow, staged diff and post-commit inventory checks are required before push. This eight-record exact-search fixture does not qualify HNSW, TurboQuant, 100,000-record readback, the full native capacity matrix or the 24-hour soak. It cannot establish the numerical denial or cause of the earlier `90bb821e` run. No source ingestion, provider observation, project build-plan execution or alpha completion is claimed. Dependencies, review and exact candidate CI remain required; C-07 and POAM-035 remain open.

Checklist: parent heads, baseline, scope, behavior, traceability, independent real-process oracle, resource/diagnostic decisions and stop conditions were recorded in the PR; the generated conflict and actual checks are recorded; candidate CI and merge remain pending.
