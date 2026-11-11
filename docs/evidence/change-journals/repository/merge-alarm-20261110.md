# Merge alarm: a pull request that cannot land is announced and tracked

**Status:** active implementation evidence

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/merge-alarm-20261110`

**Owner:** [CI and runner operations](../../../operations/ci.md#merge-alarms)

**Starting revision:** `main` at `2d2fba85b0f6d5f57aea8a7ad88a875622fddb7c`.

**Owner decision, 2026-11-10.** The repository owner asked for auto-merge, automatic branch deletion after a successful merge, and failure that is "loud" so an AI session follows up.

**Current behavior.** Auto-merge and branch deletion on merge were already enabled on this repository and on `rrflow/connectome-development`, confirmed through the repository API. A failed candidate or a removal from the merge queue left only a red check on the pull request, with no durable record for a later session to find.

**Target behavior.** A failed candidate, a removal from the queue, or a red `main` opens one `merge-blocked` issue, comments on the pull request and mentions its author. Further failures add to that issue. It closes when the pull request merges or closes, or, for `main`, when `main` next passes.

**Changed paths.** `.github/workflows/merge-alarm.yml`, `.github/workflows/ci-reusable.yml` (runs the new test), `scripts/ci/merge_alarm.py`, `scripts/ci/test_merge_alarm.py`, `docs/operations/ci.md`, `AGENTS.md`, this record and its generated index.

**Design.** `scripts/ci/check_workflow.py` forbids `pull_request_target`, so queue removals use `pull_request` and a fork's event is skipped. Candidate results use `workflow_run`. The job checks out the default branch, so the script that holds write permission is never one proposed by the pull request. A removal is an alarm only after the pull request's state has had 45 seconds to settle, because a merge also removes it from the queue.

**Failure oracles.** `scripts/ci/test_merge_alarm.py` holds 14 cases against a fake `gh`: a failed queue run, a repeat failure, cancelled and successful runs, a superseded commit, a draft, a run with no linked pull request, an unattributable queue branch, a merge leaving the queue, re-queueing, a removal that settles open, closing, red and green `main`, and a bot author. The tests were written with the implementation, so no red run was recorded.

**Acceptance.** On Windows: `python scripts/ci/test_merge_alarm.py` ran 14 tests and passed. `python scripts/ci/check_workflow.py` reported policy OK for 8 workflows.

**Not established.** GitHub only runs `workflow_run` from the default branch, so the workflow cannot run before it lands. The `reason` values GitHub sends with a removal have not been observed; the code reports the value it receives and does not branch on it.

**Research decision:** GitHub's event documentation was relied on from prior knowledge and is tested only by the first live failure. **Roadmap/POA&M change:** none.
