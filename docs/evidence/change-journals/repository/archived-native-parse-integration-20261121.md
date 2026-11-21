# Archived native Parse integration

**Status:** active integration evidence; full source-learning qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-native-parse-integration-20261121`
**Owner:** OBJ-07 / OBJ-14, Gate D D-02/P3 source recovery

## Before-edit brief

Baseline d4a5ca068ac5d35de2a0b13cd8d1adb72e93302b. PR481 is in CI;
join its green merge before this package is made ready.

Integrate the original native Parse worker, executor, executable-plan and fact-publication sources through ordinary merges.

Outcome: OBJ-07 / OBJ-14, Gate D D-02/P3 and POAM-006/059. Driver: this recovery session, after PR481. Sources: 27ba5615b0a5f295f48b5641eb60796ac95917d1 (including third-parent journal tree 276e298e903bf6a43c12471a0edfabb52619adcc), 7dba8d2774379f5cc3d37d3f2ea8b09a49c6fd29, f32ef87c83484de12b8faaa689b18cbd99218f71, 29f6618f165097f753ef23fa1f1d8a675b594926. The broad parser-candidate snapshot is later reconciliation work.

Current behavior: native worker launch already supplies pinned executable identity, empty environment, Windows job/handle-list or Linux containment, and serialized child launches. Current Parse publishes source-derived facts and checkpoints, has bounded physical batches, indexed reads, retirement, restart and replay tests. Target: incorporate each original source without returning to the older unbounded deletion batch, repeating a lost worker forever, losing current resource admission, or advertising Parse on a platform without a supervisor.

Before-edit source map: original supervision six-file diff and its untracked journal; all eight original executor file changes; executable-plan and fact-publication patches. Current owners: engine/attunement/{windows_supervisor,linux_supervisor,evidence}, parse/{step,publish,worker,records} and records/{facts,mutations,tests}; core child_launch_lock; engram local_process/{driver,preparation}; split installed_lifecycle/replan and plans; attunement/parse_step tests. The source monolithic record/test owners map to these existing modules. Keep current engram names, public operation catalogue, native worker wire protocol, policy-controlled source selection and the immutable engine installation identity.

Scope: merge these sources, resolve mapped conflicts in canonical owners, carry missing tests or behavior if found; linked recovery journal and navigation. No new parser, independent state machine, compatibility lane or provider activation. Inspect conflict regions and auto-applied changes before each source merge commit.

Acceptance: engine --lib parse::records; engine --lib parse_step; engine --lib windows_supervisor; engine --lib derivation; rrd-attunement --test worker; CLI --test parser_worker; the installed_engine_plans_applies_serves_discovers_authenticates_and_reopens test; architecture/docs/diff, then candidate and merge queue. Dedicated CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target, four jobs. No Linux/macOS claim from Windows tests; no model, vector or visual comparisons.

No external research or new trace mechanism: existing source verification, frame validation, transaction receipts and fault hooks supply the oracle. This is source recovery with existing installed behavior verification, not a new capability or alpha closure. Stop for an unmapped source effect, dropped assertion, red CI or unrelated edits. Record commands, failures and remaining gaps before marking ready. Continue other archives after landing and branch cleanup.
