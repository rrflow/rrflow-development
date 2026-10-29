# C-07 candidate CI: isolate draft and ready scheduling

**Status:** active CI repair evidence; exact candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-ci-draft-ready-isolation`
**Owner:** OBJ-02 / C-07 / POAM-035 / [PR164](https://github.com/rrflow/rrflow-development/pull/164)

## Baseline and failure

[PR164](https://github.com/rrflow/rrflow-development/pull/164) recorded the bounded brief before implementation. Public development main `68c06059fbba600af5f06a6de30667c07e35ae6b`, tree `0d5e54a1d1c5126de2cbffce1fc08a194aa78278`, was the starting source; opening branch commit `3956f600f559abe20cabffa394fb8a61b9750416` contains no product edits. Driver is this session. PR157 also changes the workflow policy; ordinary integration and its own candidate CI remain required.

[PR154 run 40574700979](https://github.com/rrflow/rrflow-development/actions/runs/40574700979/job/122437887381) was cancelled. Its always-evaluated `ci-gate` failed with every partition result `cancelled`; a later draft event produced a skipped run on the same candidate. These are observable facts, not an underlying test failure. The prior workflow grouped draft and ready events together and allowed a delayed draft event to cancel an active ready candidate. The recorded sequence is consistent with this race, but the exact delivery/order cause of the cancellation is not independently established. The separate replacement run must finish before PR154 is accepted.

## Change and research

The workflow's existing same-PR/ref concurrency key now adds a draft or candidate lane. Draft events cannot cancel ready candidates; ready runs of the same PR still supersede one another. Returning to draft no longer cancels an already-running ready candidate, which may consume CI resources until it completes or a later ready candidate supersedes it. Other PR and ref identities remain separate. The `ci-gate`, required checks, full platform matrix, timeouts, branch protection and engine behavior are unchanged. `scripts/ci/check_workflow.py` enforces the revised shape; `scripts/ci/test_candidate_admission.py` evaluates the checked-in expression over representative boolean/string contexts and rejects removed or reversed isolation. This local model does not emulate GitHub event delivery.

Primary research: [GitHub concurrency documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency) states ordering is not guaranteed. [GitHub required-check troubleshooting](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks#handling-skipped-but-required-checks) says conditionally skipped jobs report success, so simply skipping the required gate on cancellation would weaken the proof. No upstream implementation or dependency was added. Existing GitHub job logs, exact run IDs and gate results are the diagnostic evidence; no new trace, benchmark or runtime resource policy is introduced.

Read in full: README bootstrap sections, CI candidate-chain owner, workflow caller, `verify_candidate_admission` and the candidate admission regression file, the cancelled gate log, original PR154 candidate brief, relevant POAM-035 section and full changed diff. Longer roadmap and workflow files were inspected at relevant owners and symbols. Changed paths: the workflow caller, two policy/test scripts, CI operations record, POAM-035, this journal and generated navigation/inventory. No source is deleted or moved. The 100,000-record native failure remains open.

## Checks and remaining work

- Before the workflow change, `python scripts/ci/test_candidate_admission.py` failed the new draft-lane regression against the old shared group.
- After implementation, the same command passed 11 tests, including expression truth-table inputs and malformed/reversed group rejection. `python scripts/ci/check_workflow.py` and `python -m py_compile scripts/ci/check_workflow.py scripts/ci/test_candidate_admission.py` passed.
- The initial combined PR-body/write-and-create shell request was blocked by the automatic command policy; writing the brief and creating the draft PR as separate commands succeeded before any source edits. No approval or elevated execution was used.
- Navigation and inventory are regenerated with this journal. Documentation, workflow, staged diff and post-commit inventory checks are required before push. Fresh exact candidate CI and hosted event behavior remain unproven until observed.

No project execution, ingestion, C-07 native capacity, soak, alpha or release status is closed. Checklist: owner, starting source, exact scheduling scope, failure oracle, dependency overlap, research, trace/resource tradeoff and stop conditions were bound in the PR; local failure and checks are retained here; candidate CI, review and merge remain required.
