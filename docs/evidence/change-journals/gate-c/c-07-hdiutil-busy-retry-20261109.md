# C-07: retry a finished, busy `hdiutil create`

**Status:** active change-journal evidence; a CI harness repair, not capacity or release evidence

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-hdiutil-busy-retry-20261109`

**Owner:** C-07 candidate CI, [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md); pull request #301, which folds the duplicate #300

## Change

`pipeline / installed domain crash (macos-latest)` failed intermittently
before its native oracle ran: `hdiutil create` returned `Resource busy` and
the harness treated the first failure as terminal. `run_native_capacity.py`
now retries that one command up to five times with a 1, 2, 4, 8 second
backoff, and only when the failed command finished and closed both streams.
A create that timed out or left stdout or stderr open is never retried,
because an escaped process may still hold the image and a later success
would hide it. Every other native failure keeps its existing semantics.

## Evidence

- Starting revision: `fc0ca9f1` (#301 on current main `a9f1b0fb`).
- `python -m unittest test_native_capacity` in `scripts/ci`: 44 tests, OK,
  5 skipped on Windows. New tests: retries exhaust and propagate; a success
  after retries continues; an unfinished busy create is not retried.
- Not run here: macOS. Candidate CI's macOS jobs are the acceptance.
- Review: Codex found the unfinished-command gap in the first classifier;
  repaired before merge.
