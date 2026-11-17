# Pull-request-per-package process restored: 2026-11-16

**Status:** active change journal; records one bounded package and claims nothing beyond the commands listed
**Coordinate:** `rrflow://rrflow-instance/data/change-journal/pr-flow-restoration-20261116`
**Owner:** the record of the 2026-11-16 process change; [AGENTS.md](../../../../AGENTS.md) owns the rule and [CI and runner operations](../../../operations/ci.md) owns the enforcement

## Change brief

- **Owner direction (2026-11-16):** the one-branch rule of 2026-11-14 is "not my rules" and not best practice for a prototype; do "the real world correct approach"; pre-release means no compatibility and no legacy; the owner needs a development build; failures must be loud.
- **Current behaviour:** `main` carried only a deletion and force-push ruleset ("main: pre-release, one branch"); work landed by direct push; alarm issues had no assignee. Commit `2b14c3ff1` (2026-11-14) had removed the pull-request requirement, the required `pipeline / ci-gate` check and the merge queue that [ci.md](../../../operations/ci.md#repository-enforcement) specifies.
- **Target behaviour:** one short-lived branch and one pull request per package, merged by the queue on a green gate; a direct push to `main` is denied; a failure mentions the author and assigns the owner.
- **Files:** `AGENTS.md`, `docs/operations/ci.md`, `docs/operations/agent-sessions.md`, `scripts/ci/check_session_end.py`, `scripts/ci/test_session_end.py`, `scripts/ci/merge_alarm.py`, `scripts/ci/test_merge_alarm.py`, this journal. Not changed: `README.md` (front-page wording needs the owner's approval), the `ci` workflow triggers.
- **Oracle:** `scripts/ci/test_session_end.py` (14 tests against throwaway repositories); the documentation and navigation checks; the pull request merging through the queue after the ruleset changes.
- **Stop conditions:** the ruleset names a check that never reports, so no pull request can merge; the owner cannot land an emergency fix.

## Evidence

Starting revision `e94a06009` on `main`. Files read in full: `AGENTS.md` (top block, landing and session-end paragraphs), `scripts/ci/check_session_end.py`, `scripts/ci/test_session_end.py`; read in part: `docs/operations/ci.md` (landing, alarm, enforcement and session-end sections), `scripts/ci/merge_alarm.py` (issue creation).

| Command | Result |
|---|---|
| `python test_session_end.py` (in `scripts/ci`) | 14 tests, OK |
| `python test_merge_alarm.py` (in `scripts/ci`) | 10 tests, OK, including the new assignee assertion |
| `gh api repos/rrflow/rrflow-development/assignees/EonsofStupid` | 204: the owner can be assigned, so the issue create does not fail |
| `check_code_shape.py`, `check_workflow.py`, `check_selected_tests.py` | OK |
| `render_navigation.py`, `check_documentation.py` | run on an export of the committed tree; see the pull request |

Finding: while this package ran, another session wrote `joined-checkout-baseline-20261116.md` and a `Microsoft/` PowerShell cache into this checkout. The journal assumes direct-main delivery. Neither is part of this change, and neither was committed.

## Not run

- The Connectome development build (`scripts/dev/connectome_dev.py`) was not run.
- The assignee on a live alarm issue: the unit test and the `assignees` API call passed, but no real alarm was raised.
- The ruleset change on GitHub is not part of this commit. It was applied on 2026-11-17 through `gh api` (ruleset 25439032 on `rrflow/rrflow-development`: pull request, `pipeline / ci-gate`, merge queue, no deletion, no force push) before this pull request merged, so this pull request is the first to go through the queue. Whether the queue merges it is not yet proven; if the queue strands it, the ruleset is reverted.
