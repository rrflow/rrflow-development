# Repository conflict-reconciliation workflow

**Status:** active immutable change-journal evidence; POAM-027 remains open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-conflict-reconciliation`
**Owner:** PR #220 repository workflow evidence.

## Evidence record

```text
gate/package and alpha outcome: OBJ-02 / J-02 CI prerequisite and POAM-027; no product or gate status change.
starting revision/tree and pull request: public main 6fe8b7590d913beda9b009551e5d2d2644e406d9 / 3bd842a6132a2815fab0133988898ec2483b6293; PR #220 change brief before source edits.
change brief: seven open PRs each conflicted in a shared contract and generated SDK projections after 51-92 intervening main commits; a contiguous POAM-number rule forced separate drafts to claim ID 047. Provide exact-commit Git conflict inspection and permit sparse POAM IDs while preserving historical records and every existing ledger check. No merge, status or planning authority is created.
files and symbols reviewed: README identity; CI, change-authoring and POAM governance owners; scripts/ci/check_documentation.py poam_integrity_failures, POAM_ROW and existing history checks; scripts/ci/build_execution_inventory.py GENERATED_PROJECTIONS; CI verify job and policy; existing #190/#191/#217 conflict previews and #219 documentation-policy failure.
implementation paths: scripts/ci/inspect_merge_conflicts.py reports commit/tree identities and Git conflict paths, marking only existing GENERATED_PROJECTIONS registry entries with their generator; refuses configured external merge drivers. scripts/ci/check_documentation.py permits sparse valid sorted unique IDs only when every current record has exactly one row and every first-parent historical ID remains. scripts/ci/test_merge_conflict_policy.py covers both boundaries and CI verify runs it. POAM governance and CI operations describe limits; this linked journal/index record the change.
research: Git merge-tree documentation (https://git-scm.com/docs/git-merge-tree) states it does not update the worktree/index and that a conflicted merge can have no individual conflict paths. The tool uses exit status rather than inferring clean status from a path list; it does not infer conflict types. GitHub merge-queue documentation (https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue) retains candidate and merge-group checks. No upstream code was copied.
trace/resource decision: exact commit/tree IDs and paths, no contents or credentials. Git may create unreachable tree objects. The tool refuses configured external merge drivers but is not a sandbox for hostile checkouts. It does not fetch, resolve, modify refs, queue or merge.
first failure: the original documentation check rejected an unused POAM-049 while records 047-048 were absent; the first focused tests could not find the proposed helper before implementation. The new policy retains historical IDs through a non-shallow first-parent Git walk, rather than allowing a deleted row and file to disappear silently.
local results: python scripts/ci/test_merge_conflict_policy.py: 10 tests passed, including sparse/duplicate/missing/unsorted/zero IDs, first-parent deletion, shallow-history refusal, clean and conflicted temporary Git repositories, configured-driver refusal, pathless-conflict handling and malformed output. Live inspection of main 6fe8b759 and #217 head a05f678c reported 20 conflicts, 12 owned by registered generators and 8 requiring manual review; original Git merge-tree reported the same paths. CI workflow policy and documentation policy passed before this journal; regenerate and recheck below.
surfaced limitations: sparse IDs permit coordinated branches to choose distinct IDs but are not a distributed reservation service. The history guard cannot detect a different finding overwriting an existing ID; manual reconciliation remains required. A reported generated path is not permission to discard semantic contract changes. A real conflict may involve unlisted paths; a nonzero Git status remains conflicted even with an empty list. A shallow checkout is refused; the existing verify job uses fetch-depth 0.
checks not run: exact-head candidate and protected merge-group CI; no PRs were resolved, closed or merged by the tool. CI failure on #190 and Windows publication durability in #219 remain open.
review: changed policy, Git parsing, full test fixture and generated-map import reviewed; no first-party runtime changed. Change checklist: brief before edits, observed failure and focused tests, owner projection reused, journal linked, candidate CI still required.
roadmap/POAM status change: none.
pull request and merge evidence: https://github.com/rrflow/rrflow-development/pull/220; candidate and protected merge pending.
```

## Links

- [CI operations](../../../operations/ci.md)
- [POA&M governance](../../../poam/rrflow-1.0-alpha/governance.md)
