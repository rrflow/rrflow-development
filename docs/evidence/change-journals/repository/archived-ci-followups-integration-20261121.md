# Archived CI follow-up integration

**Status:** active integration; candidate and merge queue pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-ci-followups-integration-20261121`
**Owner:** repository candidate gate and C-07 harness; alpha recovery prerequisite

## Before-edit brief

Baseline b9f197726ba5d3cacfd7b954445ac93aadc7115b. Ordinary merges of saved
03018e6d8f827003b1ade05346e7fe455ae68b8f (staged lifecycle bytecode),
33601784f17397ef5a2eb4f7b0b9448660d210e8 (scheduled/manual main alarms),
58d70748f7d717ddcfceeca6b0fc50c9db5bdf28 (workspace-owned package lookup),
and bcac6931195ee6c02058b51665b2b6fe496bbe3c (native teardown trial).
Read their original diffs and current run_offline_lifecycle, merge_alarm,
workspace_packages/select_ci_lanes/build_execution_inventory owners. Current
code already carries the first three changes; tests must establish that their
failure semantics remain. POAM-074 records why the native teardown trial was
superseded; retain current owned-process/output cleanup and bounded busy-create
handling, not a retired device-node wait. Its historical journal remains history.

Scope: those canonical scripts/tests, journal and generated index. Failure
oracle: nested members select their own package, nested nonmembers select the
containing member, main alarms include schedule/manual events, staged lifecycle
imports do not create root-owned bytecode, native cleanup refuses uncertain
ownership. Run matching Python suites, workflow/docs/navigation and diff checks.
No new native runtime qualification, external research, UI action or new tracing
is needed for this source integration. Preserve newer selectors and qualification
policy. Stop on unmapped source, lost assertions, oversized history or red gate.
No roadmap status is promoted. Continue remaining archives after this package.
