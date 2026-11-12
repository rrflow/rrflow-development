# CI finds a path's package from the workspace member list, at any depth

**Status:** active immutable change-journal evidence; no gate advanced
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-package-lookup-20261112`
**Owner:** the candidate CI pipeline in [`docs/operations/ci.md`](../../../operations/ci.md)

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: candidate CI; the prerequisite for the owner's direction of 2026-11-12 that features live in folders with their own sub-folders, which nests crates below crates/<group>/
starting revision/tree and pull request: development/main 181b0e67; branch agent/ci-package-lookup-20261112; the pull request carries the change brief
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): select_ci_lanes.crate_of read crates/<group>/<crate>/Cargo.toml from the first three path parts, so a file in a nested crate was an unknown path and forced the full matrix; build_execution_inventory.package_name returned the third path part, so a nested crate would take its folder's name and the default gates -> scripts/ci/workspace_packages.py reads the members the root Cargo.toml lists and each member's package name, and maps a path to the member whose directory is its longest prefix; both scripts use it. Unchanged: every lane pattern, the fail-closed rule for unknown paths, and the package of every tracked file today. Stop if any tracked path changed package
files and symbols read in full: scripts/ci/select_ci_lanes.py, scripts/ci/test_select_ci_lanes.py; sections read: build_execution_inventory.py (package_name, planned_gates), check_workflow.py (workspace members), the workflow steps that run both scripts
implementation/evidence paths created, changed, moved, or deleted: created scripts/ci/workspace_packages.py and this journal; changed select_ci_lanes.py (crate_of), build_execution_inventory.py (package_name) and test_select_ci_lanes.py (a workspace manifest in the fixture, a nested member, a nested non-member)
research decision and primary-source/adaptation record: not applicable
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable
first-failure or characterization oracle and result: test_a_nested_member_is_its_own_package_not_an_unknown_path fails against main's select_ci_lanes.py and passes here; over all 934 tracked files under crates/, the old and new lookups name the same package
acceptance commands and exact results: python scripts/ci/test_select_ci_lanes.py: 16 passed; python scripts/ci/test_execution_inventory.py: 11 passed; python scripts/ci/build_execution_inventory.py then --check: OK, 3031 records
surfaced failures and corrections: the first nested non-member fixture sat under crates/persistence and then crates/adapters/rrflow-cli, both of which always run the full matrix by policy; it now sits under crates/evaluation/rrflow-eval
checks not run and reason: none
remaining known errors and owner: none
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
