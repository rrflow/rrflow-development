# C-07 CI draft-lane candidate refreshed onto current main

**Status:** active integrated candidate; exact candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-ci-draft-lanes-main-refresh`
**Owner:** OBJ-02 / C-07 / POAM-035 / [PR164](https://github.com/rrflow/rrflow-development/pull/164)

Driver: this session. The [PR164](https://github.com/rrflow/rrflow-development/pull/164) brief bound the ordinary merge from source parent `c56e6a48f7e634f6b573f3c847b54cd8ff30a632` and current public main `dac051d96924ddcc818c3dfd84d7e035e9f835e1` before changes. The prior journal retains the canceled/skip CI failure and local test-first repair. This integration preserves both the ready/draft concurrency groups and merged PR157's two-worker Windows reopen build, all required jobs and the fail-closed gate. It changes no product runtime or completion status. Stop if either CI policy or required coverage must be weakened.

The two conflicts were generated Gate C journal navigation and execution inventory. Both were regenerated from the combined tree, preserving all incoming journals and documentation. Relevant sections of README, C-07/POAM-035, CI operations, workflow and policy tests were reviewed; the full change-authoring procedure and PR brief were read. No new runtime trace, upstream code or research is introduced: the original PR cites GitHub concurrency and skipped-required-check documentation. This refresh and its local checks ran on the host of an active C-07 characterization; that campaign's timing is not isolated.

`python scripts/ci/test_candidate_admission.py` passed all 11 tests. `python scripts/ci/check_workflow.py`, `python scripts/ci/check_documentation.py`, `python scripts/knowledge/render_navigation.py --check`, staged `git diff --check` and `python scripts/ci/build_execution_inventory.py --check` passed. The first `python -m unittest scripts.ci.test_candidate_admission -v` invocation failed because the script-local import `check_workflow` was not on that module invocation path; invoking the checked-in test script directly resolved it without changing the test.

Fresh candidate CI, merge queue and integration are pending; the previous green CI run applies only to the earlier head. C-07 native matrix and 24-hour qualification remain open. No objective, roadmap or POAM completion status changes.

Checklist: owner, parent heads and stop conditions bound before effects; generated conflicts reconstructed from both inputs; CI policy and required checks preserved; tests and errors recorded; complete diff reviewed; candidate CI and merge pending.
