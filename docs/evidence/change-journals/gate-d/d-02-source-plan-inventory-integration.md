# D-02: reconcile source-bound plan proposal with current engine and inventory

**Status:** active draft integration; dependency and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-source-plan-inventory-integration`
**Owner:** OBJ-14 / D-02 / D-03 / POAM-038 / [PR148](https://github.com/rrflow/rrflow-development/pull/148)

## Change brief and traceability

Starting proposal head `7c1e63828287cff4165b6de0ca46a6fbf5c645f9`, tree
`fa8615ba4752670b71f76eeb17fb67d5bb8b6f0b`. The PR brief was extended before
merging [PR193](https://github.com/rrflow/rrflow-development/pull/193) exact
head `bdb954e48d2eb751b4839ed6efa761cc1e294071` into an isolated worktree.
The merge has no textual conflicts; documentation validation exposed a missing
architecture-index link, which was restored before the final check. PR193 includes current main
`e73ba9287dafba5d8fb8fb6a990fd145ffd01ab3` and remains subject to its own
protected merge. Preserve its inventory retirement descriptor, the proposal,
and all existing source and immutable journals. Regenerate navigation and the
ignored Git-derived inventory.

The design originally described its `86adada6` baseline, where the installed
engine did not retain the project root. Current code retains an admitted
`installed_project_root` on installed open, as described in the prior
[root-capability journal](d-02-retained-project-root.md). Correct the design's
starting point and remaining-work statement. The handle alone does not
authorize source reads or establish freshness or a snapshot. `Start` and
Connect still do not revalidate the plan's source, the installation inventory
is not an atomic D-03 snapshot, and ordinary-edit replan is not implemented.
No runtime, wire contract, permission, objective, roadmap or POAM status changes.

Reviewed the complete proposal, README product identity, D-02/D-03 owner,
POAM-038/039 and root-capability journal, and inspected the current
`engine/core.rs`, `engine/installation.rs`, `installation/inventory.rs`,
`engine/attunement.rs` and installation test boundaries. No upstream source
is adapted. Native tracing and performance measurement do not apply to this
prose integration; they remain mandatory for future implementation claims.
The oracle is direct source comparison: a retained anchor must not be reported
as a verified current source or complete project snapshot.

## Evidence and remaining work

The prior proposal's acceptance scenarios remain requirements for future
installed-engine implementation, not tests executed by this integration.
Navigation, documentation, workflow and inventory checks and their exact
results are recorded below. No candidate or protected merge-group result is
claimed for this head. PR193 must land first; then this draft needs exact-main
reconciliation, its full candidate CI and protected merge. No source-bound
handoff/replan, C-07, native capacity, soak or alpha completion is claimed.

Checklist: brief before changes; one engine authority and existing owners;
source and behavior examined; immutable prior journal retained; no source
deletion or replacement; new linked journal; generated projections checked;
known gaps, pending gates and driver (current Codex session) stated.

## Local validation

On Windows, `python scripts/ci/check_documentation.py` initially failed because
`docs/architecture/README.md` omitted the proposal. Restored that ordinary
index link; the next check passed with 733 statuses and 731 classified
coordinates. `python scripts/knowledge/render_navigation.py --check` passed
with 18 indexes, 732 nodes and 2839 edges;
`python scripts/ci/build_execution_inventory.py --check` passed with 1887
records; `python scripts/ci/check_workflow.py` and `git diff --cached --check`
passed. The proposal and final scope diff were reviewed. These checks do not
exercise source capture or runtime execution.
