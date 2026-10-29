# C-07: refresh Windows CI build workers after fixture merge

**Status:** active integration evidence; fresh candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-ci-workers-fixture-main`
**Owner:** OBJ-02 / C-07 / POAM-035 / [PR157](https://github.com/rrflow/rrflow-development/pull/157)

The PR brief was updated before ordinary-merging development main `393e6b86ee58bb4ea4b77d14a871af339404ebb4` into candidate `2ee56754ce92196ce5e2da05e92977c93dadfb42`. PR150's fixture repair passed its merge-group pipeline and landed on main. The sole conflict was generated file inventory, resolved by regenerating from the combined tree; engine and fixture source, the Windows two-worker experiment, full job and policy guard are preserved. No tests, timeouts, gates or platform lanes were removed.

Reviewed the original worker journal, current PR brief, merge conflicts and workflow/policy diff. No engine implementation, dependency, source adaptation or production diagnostic changed. The earlier candidate run is bound to its own head; the refreshed candidate requires complete fresh CI and its actual Windows outcome and duration. Navigation and inventory regeneration and checks, documentation, workflow and whitespace checks are required before push. C-07 capacity and soak, and any improvement in CI timing, remain unproven; roadmap and POA&M completion statuses are unchanged.
