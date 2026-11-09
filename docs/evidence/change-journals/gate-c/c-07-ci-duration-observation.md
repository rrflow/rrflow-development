# Candidate CI duration observation

**Status:** active draft evidence; C-07 and alpha remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-ci-duration-observation`
**Owner:** PR #216, OBJ-02 / C-07 tooling prerequisite.

Starting revision `6dcec8da790e229e86f0f0ae0bfea68ee9356ab5`, tree
`6a5af5812ed20999ae4d07af21ad04fcd8353c2a`; the PR brief preceded edits.
The existing inspector reported failures and active steps but did not expose
the successful build durations that dominate observed CI. `summarize` now
reports the ten longest completed, validly timed steps, job links and the eligible count;
it keeps deterministic ties and bounds the retained ranking. Incomplete or
invalid timestamps are not assigned durations. Existing failure reporting and
run/source validation remain unchanged. These are elapsed step observations,
not a diagnosis, critical-path calculation, cache qualification or acceptance
receipt. `docs/operations/ci.md` now refers to the runner's exact enforced case
set instead of the stale literal five; current main had six cases and the #224
candidate had seven. No test or gate was removed or relaxed.

Read in full: `scripts/ci/inspect_candidate_runs.py` and its focused test file;
reviewed the relevant CI operations sections, runner `REQUIRED_CASES`, GitHub
candidate/job metadata, root instructions and prior PR216 evidence. Changed
paths are those files, the guide, this journal and the generated parent index.
No upstream code, runtime trace, secrets, raw log content or resource claim is
introduced. GitHub job timestamps are existing run metadata; retain original
job logs and receipts for diagnosis and qualification.

The two new fixture tests first failed with `KeyError: timed_step_count` before
implementation. `python scripts/ci/test_inspect_candidate_runs.py` then passed
all nine, including completed success/failure ranking, unknown timestamps,
deterministic ties and the ten-row bound. The read-only command
`python scripts/ci/inspect_candidate_runs.py 41161519177 --pr 224` reported
attempt 2, event head `168e93c649fe6548187b01db22bd47eb5552e077`, tested source
`c02bb795a406cb659a5943b7195cf2aab720df8f`, tree
`f0ab72ffba68fad1b94f0b3f1f64c7010927def0`, 385 timed steps, and a 2,327-second
Linux reopen build as longest. Its PR-head comparison was false because #224
had advanced. This matches the retained GitHub job timestamps but does not
establish cause, equivalent machines, the current candidate result or C-07
capacity and soak. The prior failed macOS candidate remains unresolved.

`python scripts/ci/test_offline_lifecycle.py` passed all 17 policy tests;
`python scripts/ci/check_workflow.py` passed; navigation regeneration and
`python scripts/ci/check_documentation.py` passed (798 document statuses,
796 classified coordinates). Execution inventory regeneration and `--check`
passed for 2,045 records. Complete changed-file reread, current-main and #224 integration,
the package candidate and protected merge-group CI remain outstanding. No
roadmap or POA&M completion status changed.
