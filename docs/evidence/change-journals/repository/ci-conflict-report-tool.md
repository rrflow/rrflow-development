# Read-only merge-conflict report tool

**Status:** active immutable change-journal evidence; POAM-027 remains open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-conflict-report-tool`
**Owner:** OBJ-02 / J-02 CI prerequisite and POAM-027 repository workflow; split from PR #220 at the repository owner's direction on 2026-11-01.

This record lands only the operator-invoked merge-conflict report from PR #220.
At the owner's direction, POA&M IDs stay consecutive: #220's sparse-ID rule in
`scripts/ci/check_documentation.py`, its POA&M governance text and its POA&M
identity tests are deliberately excluded. Nothing here changes POA&M numbering,
ledger validation or any status.

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: OBJ-02 / J-02 CI prerequisite; POAM-027 change-package and repository enforcement. No gate, roadmap or POA&M status change.
starting revision/tree and pull request: public main 876193742ef4298e38b0b2b337fc092efa297847 on branch agent/ci-conflict-report-tool; source of the split is PR #220 head 5697b208b5269f76817d437cb219f79b2434c500 (merge base 6fe8b7590d913beda9b009551e5d2d2644e406d9). The change brief is in the PR #238 description.
change brief: current - open PRs repeatedly conflict with main in the same generated projections and generated indexes, and there is no repository command that previews a merge's conflicting paths against exact commits without touching a checkout. Target - an operator-invoked, read-only report that resolves two revisions to commit IDs, runs Git merge-tree without changing the index, working files or refs, lists conflicting paths, and labels only paths in the existing GENERATED_PROJECTIONS registry with their generator; every other path requires manual review. Unchanged - POA&M numbering stays consecutive and check_documentation.py is untouched; branch protection, candidate CI, the merge queue, journals, version and product authority are unchanged. Stop if the tool would choose a side, claim merge eligibility, or report a result not bound to exact commits.
files and symbols read in full: PR #220 scripts/ci/inspect_merge_conflicts.py (commit_id, parse_merge_tree, inspect, main); PR #220 scripts/ci/test_merge_conflict_policy.py (PoamIdentityTests, MergeInspectionTests); PR #220 journal ci-conflict-reconciliation.md; PR #220 diffs of docs/operations/ci.md, docs/poam/rrflow-1.0-alpha/governance.md, .github/workflows/ci-reusable.yml and the repository journal index; main's GENERATED_PROJECTIONS in scripts/ci/build_execution_inventory.py; the verify job's repository-policy step in .github/workflows/ci-reusable.yml; the inventory section of docs/operations/ci.md; the linked journal record template in docs/roadmap/rrflow-1.0-execution/change-authoring.md.
implementation/evidence paths created, changed, moved, or deleted: created scripts/ci/inspect_merge_conflicts.py (byte-identical to PR #220); created scripts/ci/test_inspect_merge_conflicts.py from PR #220's test_merge_conflict_policy.py, keeping the five merge-inspection tests and removing PoamIdentityTests plus the first-parent-history and shallow-history POA&M tests, which exercise only the excluded check_documentation.py change; the verify job's repository-policy step runs the new test; docs/operations/ci.md gains #220's conflict-preview paragraph plus one sentence naming the test; this journal and its regenerated index. Not taken from #220: scripts/ci/check_documentation.py, docs/poam/rrflow-1.0-alpha/governance.md, POA&M identity tests.
research decision and primary-source/adaptation record: reused PR #220's research. Git merge-tree documentation (https://git-scm.com/docs/git-merge-tree) states that --write-tree does not update the index or working tree and that a conflicted merge can report no individual conflicted path, so the tool takes conflict status from Git's exit code. No upstream code was copied.
trace/resource/debug decision: not applicable beyond the report itself, which emits only commit and tree IDs, conflict paths and registered generator paths, never file contents or credentials. Git may write unreachable merge-tree objects into the object store.
first-failure or characterization oracle and result: the temporary-repository fixtures prove a real conflict with exact IDs and an untouched worktree, index and refs; a registered generated path labelled with its generator and an unregistered path labelled for manual review; clean and unknown revisions; refusal of a configured external merge driver; malformed merge-tree output; and a pathless conflict that stays conflicted.
acceptance commands and exact results: py -3 scripts/ci/test_inspect_merge_conflicts.py -v -> 5 tests OK. Live reports on this host against main 87619374: PR #214 head a56eb62a -> conflicted, one manual-review path crates/transport/rrd-server/src/http/response.rs; PR #169 head 906ac27c -> conflicted, docs/evidence/change-journals/gate-c/README.md and docs/roadmap/rrflow-1.0-file-plan.jsonl, both unregistered; PR #212 head 9471226b -> clean. Each path list matched direct git merge-tree --write-tree --name-only output; git status of the worktree was unchanged afterwards. Repository checks are recorded in the pull request.
surfaced failures and corrections: none in the tool. Observation - generated navigation indexes (owned by scripts/knowledge/render_navigation.py) and the retired file-plan descriptor are not in GENERATED_PROJECTIONS, so the report labels them for manual review. That is the intended fail-safe behavior; extending the registry is outside this package.
checks not run and reason: exact-head candidate CI and protected merge-group CI run on the pull request; not run locally. Linux and macOS runs of the focused test happen in candidate CI.
remaining known errors and owner: a reported generated path is not permission to discard semantic changes; a conflict may involve unlisted paths, and a nonzero Git status stays conflicted even with an empty list. The tool is for trusted checkouts and is not a security sandbox. POAM-027 owns any later workflow change.
full-file reread and diff review: the tool, the reduced test file, the CI step and the operations paragraph were reread in full and the staged diff reviewed against PR #220.
change checklist: change brief in the pull request before landing; focused test first; existing generated-projection owner reused; no new authority, hook, ledger or runtime component; journal linked and index regenerated; candidate CI still required.
roadmap/POA&M status change: none.
pull request, CI run, and merge evidence: https://github.com/rrflow/rrflow-development/pull/238, split from https://github.com/rrflow/rrflow-development/pull/220; candidate and protected merge pending.
```

## Links

- [CI operations](../../../operations/ci.md)
- [POAM-027](../../../poam/rrflow-1.0-alpha/poam-027.md)
