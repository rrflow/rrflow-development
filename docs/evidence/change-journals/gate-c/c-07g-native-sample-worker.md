# C-07g: isolate native resource acquisition in an owned worker

**Status:** active draft implementation; native cleanup and qualification remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-native-sample-worker`
**Owner:** OBJ-02 / C-07g-h / POAM-035 / [PR195](https://github.com/rrflow/rrflow-development/pull/195)

## Change brief, baseline and implementation traceability

The public development starting main was `e73ba9287dafba5d8fb8fb6a990fd145ffd01ab3`.
The branch fast-forwarded the complete prior [PR171](https://github.com/rrflow/rrflow-development/pull/171)
draft `6c15edacd5b484709275dcfe2a2de6f4ba4ca09b` after its
[PR193](https://github.com/rrflow/rrflow-development/pull/193) dependency.
PR193 subsequently passed candidate and protected merge-group CI and landed
as `291fee511887932572ecc9935e9786e098727677`. This draft is a proposed
replacement for PR171 only after actual integration and CI; the earlier PR,
source, failure records, and immutable journals remain intact.

PR171's async ownership preserved scheduling while an *async* sample was held,
but its `sample_with_identity` still called native process/host refresh, memory
scanning and allocation verification synchronously on a current-thread runtime.
This draft moves those calls to the same test executable as an owned child.
The existing `Process` monitor owns its child, bounded capture, cancellation and
deadline; a per-workload owner admits one acquisition and joins completed
monitors, and segment/idle cleanup explicitly waits for the worker. The parent
records the actual completion time after the helper response. The authenticated
diagnostics RPC, resource-window validator, caps, workload, engine and public
contract remain. No sample is synthesized when the worker fails. Native
allocation verification checks the engine PID, the *original harness PID*,
and the worker PID in the same cgroup or Windows Job, preserving the prior
harness and engine checks. The helper's output must name the spawned PID.

Changed `tests/support/process.rs`, `sustained.rs`, `sustained/workload.rs`,
`native.rs`, `native_allocation.rs`, and the constructor-only `fleet.rs` and
`damage.rs`; added `sustained/native_worker.rs`, this journal, a POAM-035
observation and generated navigation. The previous async hold and durable
reopen/replay checks remain and the installed smoke also holds and interrupts
the native helper. No production source, public model, original evidence,
threshold or roadmap status is removed or changed. This is a test harness
implementation, not the project's source-bound engine workflow.

Reviewed README, C-07 and POAM-035, prior PR171 and inventory journals, the
complete changed helper, `process.rs` owned-child lifecycle, `sustained.rs`
sampling/scheduling and installed fixtures, `native.rs`, `native_allocation.rs`,
workload/fleet/damage constructors, and `scripts/ci/run_sustained_capacity.py`
and `owned_windows_process.py`. Tokio's documented [select behavior](https://docs.rs/tokio/latest/tokio/macro.select.html)
and [blocking-task cancellation limit](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
informed the decision; no upstream code is copied. Existing bounded timing
traces are retained.

## Local Windows evidence

The first exploratory focused run compiled while the source was changing; it
passed but is not the final-source oracle. The later eight-file source manifest
`target/c07-native-worker-195-sources-v4.json` SHA-256
`c2c336a2c19b9e55e664cf205d58a9195b53695ab5ccb9dd14b73818de9a650d`
identifies the tested working-tree files over PR171 head. The isolated test
harness SHA-256 was `a4b763a4b23b0f6b25e3f045980f75454a179a55ba22562aa2940b7e8fc3f79e`
(20,743,680 bytes). No clean commit or candidate CI existed at test time.

With `CARGO_TARGET_DIR=target/c07-native-worker-195-build`, two Cargo build
jobs and offline locked inputs, on Windows x64:

- `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::native_worker::tests::held_native_worker_is_cancelled_reaped_and_owner_reused -- --exact --nocapture`: 1 passed;
  the held helper was cancelled, observed absent, a nonexistent target failed,
  and subsequent real sampling succeeded. Log SHA-256
  `f60f0137659df890f4d3d2c7b5bb877aedb9cddf7667c01cd90433988f486dc7`.
- The same target's `sustained::installed_capacity_workload_smoke -- --exact --ignored --nocapture`: 1 passed; sequence 2 acknowledged during the original async hold and a *later* sequence 3 after the helper entered its native hold; 20 attempts and 36 observed resources matched progress. The interrupted native helper was observed gone before engine kill/reopen and acknowledged-write replay. Log SHA-256
  `99d104c565c6e93166d21b79fb1f6e4615dd69b4dbab779f6365635c7d59d834`.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen`: 26 passed, 11 ignored. `cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings` passed. Log SHA-256 respectively
  `110eb7621b840274081f0cef4ed98240a12674cb8919c0866bf532bd75d4013e` and
  `40e64ff59b50102519f9eece3e2cadd7b2bfe9eba294b976c4e1156c8af08261`.
- An explicit test-owned Windows Job run of `sustained::native_worker::tests::native_job_contains_engine_harness_and_worker` passed. It launched a real installed engine and the helper under the existing launcher implementation with a 2 GiB process/aggregate test limit and 24 host vCPUs. The test verified engine, harness and helper membership and limits, cancelled a held helper and observed it absent, rejected a false harness PID, then successfully sampled again. The receipt and result log SHA-256 is
  `fb4033a1dcbd2d156f7b15be4c393232beb0c38c8af7bc74aebcea0b372167d0`;
  the local, non-product probe script under `target/c07-native-worker-195-windows-job-probe.py` has SHA-256
  `203a450b5443bffc1329fdfe1e94c081d7e5348580a77c0980db7a1822197ec7`.
  These are synthetic bounds, not the frozen profile.

A separate prior v2 working-tree characterization (`target/c07-native-worker-195-sources-v2.json`, SHA-256
`2271b73326e748f4505bbfbc07142a1604328510d96c1e436ee751d2cabb7c52`)
ran `sustained::installed_sampler_trace_during_write_load -- --exact --ignored --nocapture`: 4,000 acknowledged writes, 32 correlated samples and 32 overlapping client intervals, no failures, at most 40 segments. It is not final-head CI or the 100k profile. Its 32 sample completion times had median 66.69 ms and maximum 95.36 ms; a separate prior PR171 run on the host had median 45.71 ms and maximum 63.97 ms. These runs were not controlled or matched for host interference, so neither the difference's cause nor a reliable performance regression is established. Log SHA-256 values are
`0fb53b4d29ce2c15e9c83632f58af04f392a712c267c97a8d68012565b9b7a91` and
`3e02b64f6413f8496e21b40d5e98d691c1207c3386f93127eb6104a3548249a7`.

## Stops and unrun evidence

The earlier 100,000-record sample-gap failure's root cause remains unknown.
One process per acquisition adds measurable work; sustained cost under actual
resource caps remains unmeasured. The original `Process` monitor calls
blocking `wait` after requesting termination. The async owner reports a cleanup
timeout, but source review does not prove the manager thread and child are
bounded if native termination or reaping itself fails. Abrupt harness death,
helper executable identity across replacement, Linux cgroup, macOS behavior,
process/handle leak fault injection and the full native capacity matrix are
not proven. The test-only worker uses an explicitly requested private hold;
no product environment or remote capability can activate it.

Keep PR195 draft and do not claim its stop conditions resolved until these
remaining cases are addressed or explicitly rejected with evidence. The
pre-existing failed C-07 sample and resource limits remain unchanged; no
100k-profile pass, 24-hour soak, gate closure, alpha or release is claimed.
Documentation/navigation/inventory/workflow checks and any later fixes are
recorded separately below.

Checklist: PR brief and traceability map before code; one existing harness
boundary; expected old tests and evidence retained; actual observations and
allocation identity checked; focused test then owning target and lint;
failures/omissions and open stops stated; linked journal and generated index;
no POAM lifecycle status change; candidate and merge-group CI not run.

## Generated and policy checks

After the source tests, independently rehashed all eight tested files against
the v4 manifest; all matched. `cargo fmt --all -- --check`,
`python scripts/ci/check_documentation.py` (740 statuses / 738 classified
coordinates), `python scripts/knowledge/render_navigation.py --check` (18
indexes), `python scripts/ci/build_execution_inventory.py --check` (1897
records), `python scripts/ci/check_workflow.py`, and `git diff --cached --check`
passed locally. The landed PR193 merge tree and its candidate tree are equal;
merging actual `development/main` into the PR171 parent produces the same
parent source tree. These checks do not transfer runtime or candidate evidence
to a future head.
