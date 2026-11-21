# Include scoped configuration in the installed test inventory

**Status:** active candidate repair evidence; local selector and harness checks passed, repaired candidate pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/scoped-configuration-ci-inventory-20261121`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), PR456 installed acceptance

The [foundation journal](scoped-configuration-foundation-20261121.md) records
the implementation and local acceptance. Its candidate
`2493fa8b41d39863859d59fecedd7e31320cd9ea` failed
[repository invariants](https://github.com/rrflow/rrflow-development/actions/runs/44878697706/job/136769102422)
because the new installed test was not added to `LIFECYCLE_CASES` in
`scripts/ci/selected_tests.py`. The static resolver correctly reported one
unexpected test and no missing tests. The same target passed all 16 tests
locally, but that did not prove the independent inventory check.

The repair adds the exact new test name to its existing owner. All previous
selectors and the exact-set assertion remain. The offline lifecycle harness
imports this owner, so the driver also becomes required there. No runtime
implementation, workflow, test body or acceptance criterion changes.

Read coverage: `selected_tests.py` selector ownership and `EXACT_TEST_SETS`,
`check_selected_tests.py` in full, `test_selected_tests.py` resolver and exact-set
tests, and the failed job's exact assertion. The PR brief was extended before
the repair. No research, trace, resource or algorithm change is needed. Stop
if this requires weakening the exact inventory or altering runtime behavior.
The failure is retained in [issue 457](https://github.com/rrflow/rrflow-development/issues/457),
labelled `merge-blocked` and assigned to the owner.

Acceptance commands: `python scripts/ci/test_selected_tests.py`,
`python scripts/ci/check_selected_tests.py`,
`python scripts/ci/test_offline_lifecycle.py`, navigation and documentation
checks. Seven selector tests passed; the static check resolved 25 owned
selectors, 16 workflow filters and five test-path literals. The offline harness
suite passed 29 tests with seven platform/environment skips on Windows.
Navigation and documentation checks passed before commit. The
implementation journal's runtime evidence still applies; CI for the exact
repaired revision remains required. No roadmap or POA&M item closes.

The first candidate's supported-SDK CI job also passed all six language drivers
against the live shared corpus, supplying the evidence unavailable locally
without Maven. It does not clear the inventory failure or qualify a later
candidate by itself. The changed selector file was reread in full and its
one-line diff checked; every existing selection is preserved.
