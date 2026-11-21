# Archived native preflight integration

**Status:** active integration evidence; native capacity qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-native-preflight-integration-20261121`
**Owner:** C-07 bounded storage diagnostics and native capacity; alpha recovery prerequisite

## Before-edit brief

Baseline 3ef272c48b68de9b5f92ef4c2f7f6d2428e347cd. Ordinary-merge the two saved
h04-c07-merge-preflight sources, 5936e895eaca3194dea09192eb12347d9250f0d2 and
e8d5ac07dab50ae580090832749b0654141e1352. Both have two parents and identical
tree 0a9cf6a33441bfc9594049495464b9638c41c1c0. Preserve current native-worker,
sampler, bounded storage-cursor and shared-memtable behavior in their canonical
modules. Source history is not yet joined; no new throughput capability claimed.

Read the source diff and current implementation/test owners before resolving:
rrd-engine engine/storage_cursor and tests, rrd-contract storage_cursor and
public_contract/storage_cursor, rrd-store rrflow_kv/inspection_cursor and its
storage_inspection/direct_read_paths tests, rrd-lsm memtable, CLI sustained
campaign/reader/native_worker/sample_task/sample_trace, qualification-storage
sustained, native capacity supervisor and selected_tests. Existing archive
ledger names these destinations; it is only a navigation aid. The current
five-attempt finished-busy retry and held-output policy supersede the old
single-attempt fixture. Current selected-case preflight supersedes the old
manual pairs. Do not restore old generated SDK digests or the retired eval
crate boundary. No unrelated source or submodule pin is in scope.

Oracle: original cursor range/kind and lifetime/refusal tests must remain;
run focused cursor/store/LSM tests, relevant Python native sampler/supervision
suites and architecture, then candidate and merge queue. Read complete changed
behavior and assertions, not only names. Reuse unchanged full-suite evidence
only with explicit identity and scope. No external research or new performance
trace is needed for this integration; native long-run capacity and real-world
reasoning optimization remain unqualified. Stop on unmapped source, discarded
assertion, new platform behavior or red CI. Driver: this session; PR472 is an
independent history package. Continue remaining archives after this lands.

## Source resolution

Ordinary source merges 16d5038d and 31edd225, followed by green PR472 main
in 0c11cfa8. Both source trees are identical; the second resolution used the
same bounded map. No application/test/CI delta remains versus the joined main.
Initial resolution stopped on a moved test entry point before committing;
read the old restart-range assertion and confirmed its current owner in
CLI support/running_engine.rs, then resumed the same merge. No archive or
worktree was created and no current file was deleted.

The cursor contract, engine and KV owners retain OpenRange/OpenRecordKind,
ordered half-open key validation, finite page/lifetime budgets, scoped kind
selection, session authorization, idempotent opening, expiry and release.
The physical profile counters and clone-capacity recalculation remain in LSM
memtable; its independent physical-traversal test remains. Store tests retain
bounded pin admission and pages across maintenance. Public-contract tests now
live in public_contract/storage_cursor.rs; the old monolithic test entry is
removed because its assertions live in that existing test target.

The source sustained.rs additions were read across sampling, correlation,
point scheduling, held-sample progress, interruption, replay and shutdown.
Current owners: sustained.rs composes campaign.rs, point_run.rs,
capacity_workload.rs, workload/run.rs, sample_task.rs, sample_trace.rs and
native_worker.rs. Current PointRun additionally uses start_sample_at and
observed sample deadlines. The same held-helper ACK, interruption/reaping,
readback and idempotent-replay assertions remain in capacity_workload.rs.
ReaderSelection is owned by rrflow-qualification-storage; the old eval path
is removed. Native worker image, process allocation and correlation owners
remain. Contract catalogue/SDK output retains today's generated identity.

Current CI selects the split test names, adds the later stale-snapshot conflict
oracle, uses the selected-case macOS preflight, and preserves held-output
metadata, bounded native-command capture and finished-busy retries. Those
later fixes supersede the trial's earlier one-create expectation. Current
process trace observers and redaction stay intact. Historical evidence and
POAM-035 retain their later state; this integration does not reopen old run
results as new capacity evidence or close any qualification gate.

## Local validation

- Engine storage_cursor: 5 passed; contract storage_cursor: 6 passed.
- Store storage_inspection and direct_read_paths: 4 + 4 passed.
- LSM memtable-filtered cases: 13 passed across selected targets; qualification
  reader-selection identity: 1 passed. Zero-selected targets are not counted.
- CLI installed_reopen: 39 passed, 14 explicitly ignored qualification/helpers.
- Explicit installed sampler/server/store correlation and reopen: 1 passed
  (`sustained::campaign::installed_sampler_trace_matches_server_and_store`,
  --exact --ignored --nocapture). No full soak or loaded 4k claim.
- workspace_architecture: 35 passed; workflow/documentation/navigation and
  git diff --check passed. An initial mistyped check_docs.py command did not
  run a check; corrected to scripts/ci/check_documentation.py, exit 0.
- Python native-capacity suite: 49 run, 6 platform-specific skips, exit 0.
  Native Windows membership/death/timeout and remaining script checks pending;
  release harness compilation is still running. Candidate remains draft.

Cargo subprocess exits were 0, logs native-preflight-*.log under the session
external evidence directory. No source behavior changed in this package.

Final local checks: Windows native-sample membership, abrupt parent death and
timeout cleanup each passed against the release harness; policy suite 7 passed;
Windows harness image identity 6 passed; macOS failure-capture policy 3 passed;
macOS preflight selection 5 passed. No APFS or Linux cgroup execution is claimed
on this Windows host. The batched Linux command exited 2 for missing required
--cgroup-parent before executing; it was not an applicable native check here.

A controller-history merge was briefly opened while the first release harness
was still compiling, then immediately aborted to restore stable source inputs.
No controller source change was committed. The following Windows image check
rebuilt the stable source and passed. Both release check commands exited 0;
source remains identical to main, apart from this package's evidence. The
ignored long-running capacity experiments were not enabled as qualification.
Candidate and queue must still pass before merge.
