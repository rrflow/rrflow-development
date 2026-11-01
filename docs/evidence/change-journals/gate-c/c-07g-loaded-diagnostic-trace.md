# C-07 loaded diagnostic trace characterization

**Status:** active draft characterization; original failure and native qualification open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-loaded-diagnostic-trace`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR171](https://github.com/rrflow/rrflow-development/pull/171)

## Bound change and traceability

Starting PR171 head `6bf837206bd30c13554624a48bd9f91d44900dee`, tree
`e7ce596908f115ac5a8d9d692d950a313c9a6764`. Its
[brief](https://github.com/rrflow/rrflow-development/pull/171) was extended
before implementation to authorize one bounded installed load trace and the
private return of already measured client phase durations. The existing
`sample` still returns the same `ResourceSample`; no campaign format,
threshold, engine behavior, public API or acceptance status changes. The
existing nested `check_trace` helper moved within the same test module to
`check_sample_trace`, preserving its digest match, redaction and timing checks
and adding uniqueness and native count readback. The original real installed
trace/restart smoke passed after the move. The new separate ignored test
interleaves writes and diagnostics against an owned installed process, then
checks representative values and exact replay after restart.

Read the README identity and C-07/POAM-035 owners, prior sampler failure and
trace journals, this PR's brief and stop conditions, the complete modified
`sustained.rs`, `sample_task.rs`, native sampling and allocation sections,
workload prepare/execute/verify, the installed child-process fixture, and the
store diagnostic timing fields. No third-party implementation was adapted.
Tokio's locked-version [blocking-task](https://docs.rs/tokio/1.53.1/tokio/task/fn.spawn_blocking.html)
and [process cleanup](https://docs.rs/tokio/1.53.1/tokio/process/) documentation
confirms that aborting already-started blocking work and dropping a process
handle do not establish bounded, fully reaped interruption. This test does not
resolve that separate sampler ownership gap.

Changed `crates/adapters/rrflow-cli/tests/support/sustained.rs`; created this
journal; regenerated `docs/evidence/change-journals/gate-c/README.md` and
`docs/roadmap/rrflow-1.0-file-plan.jsonl`. The complete Rust file and source
diff were reread. The generated index and inventory were checked by their
repository generators, not manually inspected line by line. No paths were
deleted or moved. The unresolved sampler ownership and C-07 qualification
remain assigned to this draft package; its full candidate and merge evidence
do not yet exist.

## Reproducible local identity and outcomes

Native Windows x64 developer host; no isolated native allocation profile.
The test was run from the starting head plus a working-tree patch SHA-256
`c6cbb096e8511f40ed4596b0eb95d82202c2b7e77ac87539d9b4598ddace0eeb`;
the changed Rust file SHA-256 was
`1484605267f9107d4a43875bd2af3524bada09790a2561bd8f08dd7924324e9c`.
An isolated `CARGO_TARGET_DIR=target/c07-loaded-trace-6bf`,
`CARGO_BUILD_JOBS=2`, `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0` and `CARGO_INCREMENTAL=0` built with
`cargo test --offline --locked -p rrflow-cli --test installed_reopen --no-run`.
The actual test harness SHA-256 was
`3e29e7d89203a43b252cab1956be00c1dc21ea06149b34bb8cea70b26d8e249e`,
and primary executable SHA-256 was
`5686a9323ab6aef1b821bd13e8b5dcb40016b6a08890ec9aaae6f80b7517c5ce`.
These hashes name local build artifacts, not a signed release.

`cargo test --offline --locked -p rrflow-cli --test installed_reopen
sustained::installed_sampler_trace_matches_server_and_store -- --exact
--ignored --nocapture` passed one real installed idle trace/restart case.
The new `sustained::installed_sampler_trace_during_write_load` selected by the
same command shape passed in 128.44 seconds: 4,000 acknowledged synthetic
records, 32 diagnostics, 32 client-observed overlapping write/diagnostic
intervals, four representative record/vector/relation readbacks before and
after restart and four idempotent replays. No failed write or sample was
reported. The log is `target/c07-loaded-trace-6bf-results.log`, SHA-256
`2993248d71f0b0d637e555501544afeb0499eda8a7991ae06ab28416bb86adcc`;
the independently parsed local numeric summary is
`target/c07-loaded-trace-6bf-summary.json`. These artifacts are local, not
retained GitHub qualification artifacts. Failures carry stage, ordinal and
error digest rather than raw sensitive server data; bounded progress is
reported every 500 acknowledgements.

For the 32 samples, client diagnostic RPC median was 10.925 ms, nearest-rank
p95 14.239 ms and maximum 14.324 ms. Server service maximum was 7.204 ms,
storage mutex-wait maximum 1.700 ms, in-lock assembly maximum 4.759 ms and
memtable profile maximum 4.329 ms. The maximum observed host acquisition was
52.590 ms. The samples observed up to 56,377 memtable keys, 75,972 versions
and 40 segments; maxima may belong to different samples. Client interval
overlap does not prove simultaneous storage-lock contention.

The prior retained 100,000-record run had a 1,268.551 ms client RPC and about
1,980 segments at the slow observation, but had no correlated server/store
timing. This 4,000-record uncontrolled-host run did not reproduce that RPC
delay and did not reproduce its segment shape, concurrency or allocation.
The measurements cannot identify the old failure's cause or justify a
production optimization. A comparable loaded attribution and native sampling
interruption/ownership remain required before qualifying the integrated
candidate; no prior run is overwritten.

`cargo test --offline --locked -p rrflow-cli` passed 52 tests, with ten ignored
tests (including native long qualification) remaining separate. The initial
`cargo fmt --all -- --check` failed on this new test's formatting; formatting
the modified file and rerunning passed. The initial strict Clippy rejected the
new progress counter's manual multiple-of expression; changing that expression
to `is_multiple_of(500)` made `cargo clippy --offline --locked -p rrflow-cli
--test installed_reopen -- -D warnings` pass. The earlier package suite and
measurements above remain bound to the earlier patch; they are not silently
relabeled as results for the corrected source.

The corrected source was rebuilt and the same 4,000-record loaded test rerun.
It passed in 127.25 seconds with 4,000 acknowledged writes, 32 correlated
samples, 32 overlapping client intervals, the same four pre/post-restart
readbacks and four replays. The corrected Rust file SHA-256 was
`25c52d952d76d576c83c31c06e624afb37ca711d99af31a8afd6f8055f5d4dbe`, patch
SHA-256 `a72a3235f20d856057ab13ecf0108902cac7abce6f2358e1af998c5fcb093982`,
harness SHA-256 `5a5cfc0dd7c0ed5a8af68650986e55bd0d7e5ba00c4ae84a745a1ee86f724752`;
primary executable digest was unchanged. Retained local log
`target/c07-loaded-trace-6bf-final.log` SHA-256
`8a2af2bc5fa804de75a7932ecdfc2e67d513f088522f9c0b2a69b89f9c2285c5`, with
numeric `target/c07-loaded-trace-6bf-final-summary.json`. Across its 32
samples, client RPC median/p95/max was 10.714/14.147/15.113 ms, service
maximum 6.572 ms, mutex wait maximum 0.342 ms, held assembly maximum
4.260 ms, profile maximum 3.806 ms and host collection maximum 49.497 ms;
up to 40 segments were observed. This second run also does not reproduce or
attribute the original slow RPC.

On the corrected source, `cargo fmt --all -- --check`,
`python scripts/ci/check_documentation.py`, `python scripts/ci/check_workflow.py`,
`python scripts/knowledge/render_navigation.py --check`,
`python scripts/ci/build_execution_inventory.py --check`, and `git diff --check`
passed. Navigation was regenerated with `python scripts/knowledge/render_navigation.py`
and inventory with `python scripts/ci/build_execution_inventory.py`.
None of these results is candidate or protected merge-group CI, a
100,000-record capacity pass or a 24-hour soak.

Checklist: brief and exact stop conditions before editing; exact source and
binary identities; old and new evidence separated; representative independent
readback/replay and redacted trace checks; complete source/diff review;
generated navigation/inventory before commit. PR171 remains draft and conflicts
with current main; PR193 integration, macOS/Linux native characterization,
full candidate CI, native capacity matrix and soak are unrun. No roadmap,
objective or POAM status changed.
