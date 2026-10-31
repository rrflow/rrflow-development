# Passing Inventory candidate checks and rejected queue admission

**Status:** active candidate evidence; protected landing incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-candidate-check-admission`
**Owner:** OBJ-14 / D-03 / POAM-006; PR224 driver: active SWAG session

## Qualified source and observed rejection

The [completion-relative correction](d-03-completion-relative-tests.md) at
`168e93c649fe6548187b01db22bd47eb5552e077` now has two completed full CI runs:

- [Pull-request run 41161519177](https://github.com/rrflow/rrflow-development/actions/runs/41161519177): all 23 jobs passed.
- [Normal dispatch 41165175925](https://github.com/rrflow/rrflow-development/actions/runs/41165175925): all 23 jobs passed at the unchanged head.

These include the required gate and the native Windows, Linux and macOS
100-cycle reopen partitions. They do not establish protected landing or the
complete project-to-worker journey.

Normal automatic merging was enabled, but explicit queue admission before and
after the second run returned `Required status check "pipeline / ci-gate" is
expected.` REST reports successful exact-head gate checks from GitHub Actions
app 15368, matching the repository ruleset. GraphQL reports the successful gate
as required for PR224. The PR remains open with no queue entry or review threads;
the ruleset requires zero approving reviews. No legacy branch-protection check
or checks on the current test-merge commit explained the discrepancy.

The successful pull-request suite is `111579446761`; the successful dispatch
suite is `111589031418`. A later skipped same-head suite `111580828121` belongs
to draft PR226. This co-occurrence is recorded, not established as the cause.
No third identical manual run was started, no status was fabricated, and no
required check, merge rule or administrator bypass was used.

## Bounded evidence increment

The PR description received the evidence brief before this edit. Starting from
the clean source above, add only this journal and generated navigation. That
gives the ordinary ready-PR pipeline a unique evidence head and retains the
observed admission failure rather than treating green checks as a merge.
This is not a production correction or a claim to fix GitHub's behavior.

Reviewed the current PR, both run/job lists, check runs and suites, required
ruleset, legacy-protection response, test-merge checks, required-context GraphQL
response, queue rejection, `AGENTS.md`, CI owner and prior correction record.
Raw observations remain under
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/` as
`completion-relative-ci-success.json`, `completion-relative-refresh-success.json`,
the two enqueue-result records and `gate-required-context-result.json`.
Local logs from prior candidate failures remain unchanged.

`python scripts/knowledge/render_navigation.py` passes with 827 nodes and one
updated parent index. `python scripts/ci/build_execution_inventory.py` and its
`--check` pass with 2,098 records. `python scripts/ci/check_documentation.py`
passes with 828 document statuses and 826 classified coordinates; whitespace
passes `git diff --check`. The final diff contains only this journal and index.
No implementation tests need repeating locally for this documentation-only
increment. New-head candidate CI and protected merge-group qualification remain
mandatory. PR221's retained behavior must have a verified landed destination
before its dependency can close. No roadmap status closes in this journal.
