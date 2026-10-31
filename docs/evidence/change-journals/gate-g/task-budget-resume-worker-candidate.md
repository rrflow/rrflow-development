# Budget recovery reconciliation with worker candidate

**Status:** active prerequisite composition; worker landing and recovery CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-budget-resume-worker-candidate`
**Owner:** G-01b/G-02a / POAM-009; PR230; active SWAG session

## Source and change scope

The pre-edit [PR230 brief](https://github.com/rrflow/rrflow-development/pull/230)
binds clean `67c197ea2dd60d597e18cc37e1b57d33c45d67cb` and prepared worker
`cb42e8a96ad14892690c7e46b222eda93df25762`. PR206 has landed; PR222's full
candidate run is active and PR229 remains draft. This preparation does not
claim their acceptance or start duplicate intermediate CI.

Ordinary merge `e7500b65de57ca2e17c83383cfdb2b5df88cddde` produces tree
`fe46ab002b7f10cd3c8518f1b709a09fe7ef41e5`, equal to the read-only preview.
There are no conflicts. The complete HEAD-to-preview diff adds only the
existing controller, native-client and worker reconciliation journals and
their Gate G/H indexes. All non-documentation paths equal the starting source.
Runtime resume, public/native/SDK contracts, worker transport, proof harness,
tests and prior receipts stay byte-identical at their existing owners.

An exploratory triple-dot diff reported multiple merge bases and selected one.
The acceptance oracle instead compares the complete actual merge tree directly
with starting HEAD and verifies both parent ancestries. No arbitrary-base diff
is used to infer runtime preservation. No implementation is moved, removed,
replaced or newly qualified by the composition.

Review covered matching AGENTS and shared objective/roadmap/POAM/change-authoring
and CI ownership, complete incoming journals/diffs, the complete
[recovery implementation](task-budget-resume.md) and
[installed history](task-budget-resume-live.md). Scope adds this journal and
regenerates the nearest Gate G index beyond the declared ordinary merge.
The [worker composition](task-worker-native-candidate.md) retains its own exact
source and prerequisite status.

## Acceptance and limitations

`git diff --exit-code 67c197ea HEAD -- . ':!docs'` passes at `e7500b65`.
Preview/actual tree and both parent ancestries are verified separately.
Commands, exit codes and log digests are retained in SWAG
`.swag/evidence/20261031/resume-worker-candidate/`.
All local acceptance checks pass:

- Both `git merge-base --is-ancestor` parent checks and non-documentation equality.
- `python scripts/knowledge/render_navigation.py` and `--check`: 873 nodes,
  3,148 edges; only the nearest Gate G index regenerated.
- `python scripts/ci/build_execution_inventory.py` and `--check`: 2,382 records.
- `python scripts/ci/check_documentation.py`: 874 statuses, 872 coordinates.
- `git diff --check`: no whitespace errors.

No acceptance command failed. Full-file and complete diff review confirms
only this journal and its generated index beyond the declared merge. AGENTS
and all earlier receipts remain unchanged.

Existing engine/contract/server/client and installed BF16 results retain
their original attribution. No unchanged Rust suite or provider trial is
repeated for the documentation-only delta; no new binary is produced.
Connectome PR29 separately demonstrates bounded desktop Stop/resume and worker
readback against its frozen `0f75c015` UI and `6a4cbb04` engine. That evidence
does not qualify a new build, full project effects or public distribution.

Research, upstream adaptation, new tracing and resource experiments are not
applicable to source-preserving reconciliation. Stop conditions prohibit
unexpected code changes, conflicts, weakened admission/checks and lost evidence.
No new deficiency, roadmap/POAM closure or product-version change is asserted.

Keep PR230 draft until the actual worker landing is reconciled, then complete
its own candidate and protected acceptance. Unknown-outcome reconciliation,
ordinary-edit UI replanning, authorized project effects, correlated RROFLOW,
independently reviewed learning and model/native/container qualification remain
open. The complete required candidate/protected pipeline is not replaced by
these local source-preservation checks.
