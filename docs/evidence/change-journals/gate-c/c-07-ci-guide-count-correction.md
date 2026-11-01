# CI guide count correction

**Status:** active draft evidence; C-07 and the alpha remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-ci-guide-count-correction`
**Owner:** PR #216, OBJ-02 / C-07 tooling prerequisite.

Starting local merge `a3b76d8ebb1371c2c2de0b5a0b3ce20f37c724f9` includes
the [CI inspection integration](c-07-ci-inventory-integration.md). The PR brief
recorded this documentation correction before edits. Full CI-guide review found
a second reference to a five-case installed lifecycle corpus, while the
integrated runner requires seven, and a fixed count of 20 workspace packages,
while the integrated workflow policy reports 21. The guide now refers to the
required installed corpus and workspace-package coverage without duplicating
those changeable counts. This changes no executable selection or gate.

Reviewed `docs/operations/ci.md` in full, the integrated CI workflow and policy
output, the offline runner case set and the existing CI inspection journals.
This record, the guide and the regenerated journal index are the changed paths.
No runtime trace, upstream code, resource or performance claim is introduced.
The prior integration record retains the nine inspector tests, 18 offline-policy
tests and workflow check. Navigation regeneration, `check_documentation.py`
(833 document statuses and 831 classified coordinates), `check_workflow.py`,
execution inventory regeneration and `--check` (2,105 records), and staged
whitespace checks passed. Full changed-file review, #224's protected
landing and #216's own candidate and merge-group runs remain outstanding.
No roadmap or POA&M status changed.
