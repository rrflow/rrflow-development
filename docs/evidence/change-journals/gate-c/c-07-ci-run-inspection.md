# C-07 candidate CI run inspection

**Status:** active local implementation and live informational inspection; candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-ci-run-inspection`
**Owner:** OBJ-02 / C-07 CI prerequisite; [PR216](https://github.com/rrflow/rrflow-development/pull/216)

Starting commit `6fe8b7590d913beda9b009551e5d2d2644e406d9`, tree
`2f232cb39dbac82e83371a5689f65ca501b72fb8`. The PR brief preceded edits.
Operators previously had to manually reconcile long independent job outputs.
This package adds an informational GitHub CLI inspection of explicit run IDs:
it checks bounded job pagination and exact attempt/source, retrieves the tested
commit tree, surfaces incomplete and failed steps, and compares uniquely named
failed steps and durations. `--pr` compares event heads to the current PR head;
a merge group normally has a different head. The existing Rust CI receipt and
required gates remain authoritative. No native test, product runtime or gate
is removed or made optional. Stop on inconsistent provenance, incomplete
inventory or scope requiring a new acceptance authority.

Reviewed README, alpha objective, roadmap/POA&M C-07 references,
`docs/roadmap/rrflow-1.0-execution/change-authoring.md`,
`docs/operations/ci.md`, `.github/workflows/ci-reusable.yml`,
`scripts/ci/collect_storage_acceptance.py`, its tests and the existing
`rrflow-eval` CI receipt boundary, and GitHub job/step metadata. Changed the CI
contract and verify test invocation; created `scripts/ci/inspect_candidate_runs.py`
and `scripts/ci/test_inspect_candidate_runs.py`, this journal and generated
journal navigation. No implementation was deleted or moved.

Research: GitHub's [Actions metrics](https://docs.github.com/en/actions/concepts/metrics)
and [workflow run API](https://docs.github.com/en/rest/actions/workflow-runs)
support inspection of durations and run identity. No upstream code was adapted
and no third-party tool was installed. Existing `gh` authentication supplies
read-only API access; raw logs, credentials and native process contents are not
captured. Trace/resource decision: retain job/step status and elapsed times,
not a runtime trace or resource benchmark; these numbers cannot establish cause.

First oracle: `python scripts/ci/test_inspect_candidate_runs.py` passed six
tests for failure/active step reporting, pass-to-fail duration, ambiguous names,
pagination, empty inventory, duplicate and foreign identity, changed attempt,
missing pages, missing conclusions and invalid timestamps. `python scripts/ci/check_workflow.py`
passed; `python scripts/ci/test_candidate_admission.py` passed 11 tests.
Live command `python scripts/ci/inspect_candidate_runs.py 40991825021
40996295111 --pr 208` read both actual runs. Candidate attempt 1 succeeded;
the merge-group attempt was still running with two failed jobs. The Windows
storage recovery step was success/131 seconds versus failure/300 seconds;
the macOS APFS step was success/337 seconds versus failure/252 seconds. The
tested commits differ but both report tree
`dc0ca190cb278325cc99ebb092a7f3b2b344676f`; PR head matched the candidate event
head, not the merge-group head. GitHub's conclusion alone does not prove why
the steps failed. The macOS diagnostic [40999521873](https://github.com/rrflow/rrflow-development/actions/runs/40999521873)
passed on its distinct diagnostic source and did not reproduce the protected
failure; it is not PR208 merge-group or C-07 acceptance evidence.
The same live command later reported the merge group completed with failure:
the two native jobs and their reducing `ci-gate` failed.

Surfaced failure: an initial patch for the workflow did not apply because the
checked-out main contained additional policy commands; the patch was corrected
against the actual file. No behavior was changed by the failed patch. The
documentation check rejected the initial journal status because it lacked the
required active/historical prefix; the status was corrected. Review also found
that a completed job with a missing conclusion could be omitted from attention;
the reporting condition and focused test were corrected to surface it.
An inventory check reported drift after the journal/navigation edit; rerunning
`python scripts/ci/build_execution_inventory.py` regenerated the inventory.
Remaining: Windows timeout cause, macOS pipe failure cause, and C-07 native
capacity and soak evidence remain open in their owning packages. This tool does
not reduce the duration of the required CI itself and establishes no product
readiness. Full hosted candidate and protected merge-group CI remain pending.
No roadmap, POA&M or alpha status changed.

Checklist: brief before implementation; reviewed owner and boundaries; fixed
scope and stop conditions; tested focused failure cases; inspected live source
identity, job links and timing; preserved existing gate; recorded failures,
limits, research, and pending checks. Regenerated navigation and execution
inventory; reviewed the changed scripts, documentation, workflow and diff.
