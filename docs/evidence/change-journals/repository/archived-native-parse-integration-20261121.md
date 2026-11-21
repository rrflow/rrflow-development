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


## Source resolution

Original sources were merged sequentially: 27ba5615 in 8197c713,
7dba8d27 in 643febb5, f32ef87c in 7b591b7c, and 29f6618f in 75ad96e9.
The supervision stash's third-parent tree contains only the native-supervision
journal, which already exists under its canonical evidence owner. That tree
was inspected, not omitted from the stash review.

No implementation or test delta remains against the starting main. Native
child launch locking remains in rrd-core and both current engram local-process
launch sites. The superseded process-driver monolith was not reintroduced.
Windows still pins the primary image through launch and limits inherited
handles and job memory; Linux still launches /proc/self/exe with an empty
environment and owns child termination/reaping. The original executable-plan
patch advertised Parse everywhere; current evidence advertises it only on
Windows/Linux, where a native supervisor exists. The installed CLI oracle
uses that same platform condition.

The original executor's records functions map to current parse/records and
its facts, mutations and tests modules. Current step and publish retain later
physical-batch measurement, bounded retirement across Steps, current-job lost
worker suppression, commit indexes and source-authorized reads. The original
first-parse, edit/refusal/deletion retirement, fact-identity, summary and restart
oracles remain, alongside later replay, indexed-context and followed-project
oracles. The original fact-publication journal paragraph already exists in the
more complete P3 series owner and was not duplicated.

Read the original six-file supervision diff and third-parent journal; all
eight executor changes; executable-plan and fact-publication patches. Current
reads covered worker and driver call sites, platform executable-phase policy,
step selection and schema/index preparation, physical publication measurement,
record/test inventory, the original fact-publication paragraph, and installed
replan dispatch. This is bounded change review, not a complete reread of the
whole engine. POAM-006/059 and source-learning acceptance remain open.

## Verification

At 75ad96e9, dedicated CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target,
CARGO_BUILD_JOBS=4.

- cargo test -p rrd-engine --lib parse::records --locked: 9 passed.
- cargo test -p rrd-engine --lib parse_step --locked: 9 passed in 345.89 seconds,
  including the 40-large-file physical-batch regression, fact retirement,
  indexed context, replay and followed-project relearning.
- cargo test -p rrd-engine --lib windows_supervisor --locked: 8 passed,
  5 ignored (four subprocess fixtures and the explicitly selected binary test).
- cargo test -p rrd-engine --lib derivation --locked: 2 passed.
- cargo test -p rrd-attunement --test worker --locked: 9 passed.
- cargo test -p rrflow-cli --test parser_worker --locked: 3 passed using the
  primary executable, including changed-digest and partial-frame refusal.
- Documentation/navigation and diff checks passed.

- cargo test -p rrflow-cli --test installed_lifecycle
  installed_engine_plans_applies_serves_discovers_authenticates_and_reopens
  --locked: 1 passed in 18.91 seconds, 16 filtered. This runs the real installed
  process, derived Parse work and authorized receipt readback after restart.
- cargo test -p rrd-engine --test workspace_architecture --locked: 35 passed.

These checks ran on Windows. Linux/macOS were not run locally; the merge gate
selects its required lanes. No test failure was hidden or retried to obtain green. No new native API, provider,
vector comparison, UI, hosted synchronization or capacity claim is introduced.
