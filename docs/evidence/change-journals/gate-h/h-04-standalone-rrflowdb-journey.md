# H-04 standalone rrflowDB journey and source-build staging

**Status:** active candidate evidence; native CI and integration pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-standalone-rrflowdb-journey`
**Owner:** OBJ-01 / OBJ-04, H-04, D-01, POAM-006, [PR194](https://github.com/rrflow/rrflow-development/pull/194)

## Change brief and starting identity

Public development main was `1f8fa02cf9df74ad5dab5b7481c49f68eeca96af`,
tree `aaacbbccbc208a5558a1e00e824bbf3a5c9638a1`; the draft's opening commit
was `a6b57a9362f916f94fb4c5ecd6c0652b0996bee4`. PR193 must land first and
its inventory/workflow changes must be integrated before this candidate is
ready. Existing native admission rejects multiply linked executables. The
guide directed Windows users to a Cargo output with two links, causing install
planning to fail. The guide now stages a separately copied digest-matched
executable; an added real-process corpus tests the example's exact data and
relationship before and after reopen in the existing native candidate lanes.
No production code, security rule, existing CI partition, version or roadmap
status changes. Stop for unsafe process ownership, unbounded capture, secret
output, changed executable bytes, weakened native link checks, or lost CI.

Read in full: the local rrflowDB guide, example entry point, this package's
runner and test, ADR-0002, and repository change-authoring procedure. Inspected
the README identity and guide warp, OBJ-14 and POAM-006, H-04 and D-01, CI
candidate chain and native lifecycle commands, source example operations and
`DatabaseView` checks, `rrflowdb_local` installed test, the native installed
lifecycle's staged executable and child-process helpers, workflow-policy
parser and existing hard-link admission, as well as the prior C-07 launch
journal which recorded the same Cargo-output issue. The exact changed and
generated files are in the candidate diff. No behavior was removed or moved.

## Oracle and actual local results

First failure: on Windows, the direct Cargo build output failed install plan
with `file link count denied: expected=1 observed=2`. `fsutil hardlink list`
identified the built `rrflow.exe` and `deps/rrflow.exe`. Failure output was
retained under `target/rrflowdb-standalone-proof-6a5b52b3e0de4be3986610d28d96367c/plan.log`.
This failure and an earlier C-07 observation are not security bypass evidence.

The initial combined `cargo build --locked -p rrflow-cli --bin rrflow -p
rrd-client --example rrflowdb` using the shared root Cargo target failed with
missing `RrdEngine` serving APIs despite those APIs being in the checked-out
source. A new `CARGO_TARGET_DIR=target/verify-standalone-193` and
`CARGO_BUILD_JOBS=2`, `CARGO_PROFILE_DEV_DEBUG=0` built the same command in
3m54s. This evidence does not establish the precise stale-cache mechanism.
The standalone binaries were built from clean source head `81025f79` of PR193;
`git diff 1f8fa02c 81025f79 -- crates Cargo.lock Cargo.toml rust-toolchain.toml`
reported no source/build-input differences. They were not built from this
candidate's eventual commit.

With the isolated target and `CARGO_PROFILE_TEST_DEBUG=0`,
`cargo test --locked -p rrd-client --test rrflowdb_local` passed four tests,
including installed database relations/atomicity/reopen, attunement read and
cancel/replay, and Parquet rejection. The test uses real HTTP and native
storage inside the test process; it is not the standalone executable.

`python scripts/ci/test_rrflowdb_standalone.py` passed six tests: exact
independent values and instance identity, missing/truncated/mixed results,
failure and deadline cleanup, bounded output, rejection of unrelated errors
as the negative oracle, and rejection of missing/skipped CI steps.
`python scripts/ci/run_rrflowdb_standalone.py <isolated rrflow.exe> <isolated
rrflowdb.exe>` passed on Windows using byte-identical staged copies. Primary
SHA-256 `40a2785c9fd3d643d03aaa0a8cced85d409d060540a507293a1bcc2c7958bb2b`;
example SHA-256 `fdc7a1f3f034fa963107964ba8de72918b7a10704de6c7bdc266964ccee1e30f`;
reopened runtime cursor 7. It checks authenticated initialization, exact
`open` and `done` task and relation values at equal stamps, abort readback,
the specific old-state rejection, offline verify, same instance and cursor
after restart. The independent Windows script and retained result are at
`target/verify-standalone-193-flow.ps1` and
`target/rrflowdb-standalone-proof-8561a081b5e943e1961c29c608169313/result.json`;
these are operator-local artifacts, not public CI artifacts.

`python scripts/ci/check_workflow.py` and
`python scripts/ci/test_candidate_admission.py` passed (11 candidate-admission
tests). The workflow policy enforces both native steps and the runner tests;
the runner limits each stdout/stderr to 16 MiB, bounds each process and kills
only a child it started. It does not claim graceful Windows shutdown, a
process-tree sandbox, or a 100,000-record workload. Documentation, navigation,
and inventory checks passed: `python scripts/ci/check_documentation.py`,
`python scripts/knowledge/render_navigation.py --check`, and
`python scripts/ci/build_execution_inventory.py --check`; four execution
inventory unit tests also passed. `git diff --check` passed. Final generated
inventory was refreshed again after this evidence was recorded. Candidate
and merge-group checks remain pending.

## Research, trace and checklist

Existing first-party lifecycle and filesystem contracts were the source;
no third-party implementation or runtime dependency was added. The bounded
runner emits only operation outcome, hashes and cursor rather than credentials
or raw process output. Native Windows timing is characterization, not a
benchmark. Authors reviewed the complete newly authored runner, test, guide
and tracked diff; no generated or historical evidence was discarded.

Checklist: PR brief before implementation; exact failure captured; engine
authority and link denial preserved; named scope and stop conditions; bounded
child and independent result oracles; required local commands and surfaced
failures recorded; generated navigation/inventory and final diff review
required before commit; candidate and protected merge-group checks pending.
Linux/macOS actual runs, platform-specific failure handling, PR193 integration,
full candidate CI and protected merge are unrun. No C-07, D-01, H-04 or alpha
closure, no product status change, and no general provider/SDK qualification.
