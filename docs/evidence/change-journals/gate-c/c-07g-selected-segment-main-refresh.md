# C-07 selected-segment candidate refreshed onto current main

**Status:** active integrated candidate; candidate CI and native qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-selected-segment-main-refresh`
**Owner:** OBJ-02 / C-07g,h / POAM-035 / [PR168](https://github.com/rrflow/rrflow-development/pull/168)

## Scope and trace

Driver: this session. Source parent `c02f33c8254d5bbac95344a8dd6de5bf71c8cc96`; incoming public main `dac051d96924ddcc818c3dfd84d7e035e9f835e1`, including merged PR157. Before repository edits, PR168 recorded the source parents, scope, test oracles and stop conditions. Its prior linked journal describes the selected-segment implementation and the retained failed native attempt. The current native characterization continues against the separate clean `c02f33c8` worktree and cannot qualify this new revision.

Ordinary integration preserved both POAM-035 records, the selected-segment repair and main's CI worker changes. The only merge conflicts were POAM-035 and the generated execution inventory; navigation and inventory were regenerated. `git diff --quiet c02f33c8254d5bbac95344a8dd6de5bf71c8cc96 -- crates` confirmed all crate files matched the source parent after resolution. No implementation, test, resource limit or gate was removed. The merged ancestry retains PR152, PR153, PR158, PR160, PR162, PR165 and PR166 behavior and their journals in the canonical rrd-lsm and installed-harness boundaries. Their separate PRs are not declared superseded until the integrated result lands and replacement evidence is reviewed.

Read in full: `docs/roadmap/rrflow-1.0-execution/change-authoring.md`, PR168 change brief and the user-provided `AGENTS.md`. Inspected the full conflict region, source ancestry, parent journal, affected generated output and staged diff. No new source algorithm or upstream adaptation. Existing fault/reclamation, native progress, numeric admission and exact-source receipts are retained. A preliminary attempt to create a worktree using its not-yet-existing directory as working directory failed before any filesystem mutation; creating it from the repository succeeded. The first inventory generation failed because its conflicted index entry had not been staged; staging the resolved index entry and regenerating corrected that error. These checks and later local tests used the same host as the active characterization, so its timings are not isolated.

## Verification and limits

On Windows with offline locked inputs, `cargo test -p rrd-lsm --test projected_lifetime` passed 13, `cargo test -p rrd-lsm --test projected_read_adversarial` passed six, and `cargo test -p rrd-lsm --test compaction` passed 14. `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::installed_capacity_workload_smoke -- --exact --ignored` passed the real installed mixed-family search/query, kill/reopen and replay smoke. `python scripts/ci/check_documentation.py`, `python scripts/ci/check_workflow.py`, `python scripts/knowledge/render_navigation.py --check`, staged `git diff --check`, and `python scripts/ci/build_execution_inventory.py --check` passed. The earlier parent journal records its full rrd-lsm package suite, strict lint, formatting and failure-before oracle. The integrated smoke is bounded to eight records, not native capacity qualification.

The full 100000-record result, exact new candidate CI, merge queue, native platform matrix and 24-hour soak remain pending. No C-07, objective or POAM closure is claimed. Candidate changes cannot inherit the separate native run as exact-source qualification. No product status changes.

Checklist: source parents and PR brief bound before integration; both conflict sides retained; generated projections regenerated; implementation and replacement oracles traced; previous failures retained; host interference disclosed; staged diff reviewed; merge and qualification pending.
