# C-07 permanent manual macOS preflight

**Status:** active draft workflow; default-branch dispatch and candidate acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-preflight-workflow`
**Owner:** OBJ-02 / C-07 / POAM-035; [PR204](https://github.com/rrflow/rrflow-development/pull/204)

The [temporary experiment](c-07-native-preflight-outcome.md) ran the repeated macOS capacity cases in 12m52s including setup, instead of waiting for unrelated candidate partitions. Starting source `81e2c00cb242fd2d16e53c205d6618d1bfa92b85` has the original benchmark workflow restored. Before implementation, PR204's brief bound a separate manual diagnostic workflow and stated that it cannot replace candidate or merge-group CI. Driver: this session.

Adds `.github/workflows/c07-native-preflight.yml`, `workflow_dispatch` only, with a pinned recursive checkout, exact `GITHUB_SHA`/tree logging, focused capacity-runner tests, the same three fresh/existing APFS repetitions and bounded native-create artifact path. It preserves the main benchmark workflow and the existing required CI caller, job dependencies and gate. The existing `scripts/ci/check_workflow.py` validates the manual-only trigger, exact dispatched checkout, macOS platform, repetitions and failure behavior; `scripts/ci/test_native_capacity.py` rejects simulated missing repetitions, ignored failures, branch substitution and PR-event triggers. `docs/operations/ci.md` states how to dispatch it and that it is characterization only.

Inspected both CI workflows, the native capacity runner and policy tests, the temporary successful preflight and the official GitHub [manual-workflow documentation](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow), which requires a workflow to exist on the default branch before manual dispatch. No upstream code was adapted. The same runner and limits are reused, and no RRFlow engine or lifecycle state is added. A focused workflow cannot satisfy, skip, or weaken the mandatory candidate and protected merge-group tests.

`python scripts/ci/test_native_capacity.py` passed 30 tests including the new policy mutation cases. `python scripts/ci/test_candidate_admission.py` passed 11. `python scripts/ci/check_workflow.py` passed with four workflows and all nine substantive candidate jobs. The temporary preflight is real macOS evidence for its exact experimental head, not a run of this new workflow: GitHub cannot dispatch the new path until it exists on default main. After normal landing, dispatch the permanent workflow and retain its exact revision and result. The earlier native EBUSY cause, failure-artifact-on-macOS proof, C-07 full native matrix and soak, PR194 integration, and full candidate and protected merge-group CI remain open. No objective, roadmap or POAM status changes.

- [x] Bind the manual-only scope and unchanged candidate gate in the PR before editing.
- [x] Reuse the proven native runner; test policy against trigger, revision, failure and coverage substitutions.
- [x] Review the authored workflow and policy diff; regenerate navigation and run documentation and staged inventory checks before commit.
- [ ] After protected landing, run the new workflow on an exact ref; retain its result without calling it candidate or alpha acceptance.
