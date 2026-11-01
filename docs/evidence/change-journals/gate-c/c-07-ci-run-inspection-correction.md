# C-07 CI inspection starting-tree correction and live report

**Status:** active correction and local report verification; revised candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-ci-run-inspection-correction`
**Owner:** OBJ-02 / C-07 CI prerequisite / [PR216](https://github.com/rrflow/rrflow-development/pull/216)

The [first inspection journal](c-07-ci-run-inspection.md) incorrectly gives
`2f232cb39dbac82e83371a5689f65ca501b72fb8` as the starting tree. That is a
commit, not the tree of its correctly recorded starting commit
`6fe8b7590d913beda9b009551e5d2d2644e406d9`. The unquoted PowerShell Git
revision expression was parsed incorrectly and its failing exit was overlooked.
The corrected command `git rev-parse '6fe8b7590d913beda9b009551e5d2d2644e406d9^{tree}'`
returned `3bd842a6132a2815fab0133988898ec2483b6293`. The original record is
retained. Its live candidate/merge-group source/tree data came from GitHub API
commit records and is a separate observation, not this incorrect baseline tree.

Live inspection of this PR's candidate exposed a separate usability problem:
future pending steps made the active-job report hundreds of lines long. The
operator projection now suppresses only pending, queued, waiting or requested
steps without a conclusion; parent job status, active steps, failures, unknown
status and completed steps with missing conclusions remain visible. The added
fixture exercises all of those cases. `python scripts/ci/test_inspect_candidate_runs.py`
passed seven tests. The live command `python scripts/ci/inspect_candidate_runs.py
41001003407 --pr 216` reported one active Ubuntu 100-cycle step and retained
the exact run/attempt and source identity. This snapshot cannot prove the final
CI outcome. The original candidate ran on head
`526bc8e7136466bcbdde72bef6a9f2d7f89016db`; a revised head requires a fresh
candidate and protected merge group. No acceptance or product authority changes.

Checklist: brief recorded before changes; reviewed original journal, Git tree,
report code, its tests and actual API result; preserved the error; ran focused
tests and a live read-only report; `python scripts/ci/check_workflow.py` and
`python scripts/ci/check_documentation.py` passed; regenerated navigation and
ran `python scripts/ci/build_execution_inventory.py --check` successfully;
reviewed the diff. No upstream
source adapted, native process touched, test removed, gate weakened or roadmap
status changed. Time savings and original native failure causes remain unproved.
