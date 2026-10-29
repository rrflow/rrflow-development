# PR180: integrate the landed review dependency

**Status:** active local integration evidence; refreshed candidate CI and merge pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-review-main-refresh`
**Owner:** G-06a offline review preparation
**Pull request:** [180](https://github.com/rrflow/rrflow-development/pull/180)

## Scope and traceability

PR179 merged at `11a0fe1b16d70d91334e95efa3975e0b4a439b32` on
2026-10-29 at 00:29:14 UTC. GitHub then reported PR180 as conflicting and the
bounded merge follower stopped. The PR180 change brief already specifies an
ordinary main merge and regenerated projections for a necessary base refresh.

The isolated worktree began clean at
`ba1203165de297c0e16977650658f1d32fe77a88`. A fetched development/main at the
PR179 merge above was integrated with `git merge --no-commit --no-ff`.
Git's local recursive merge completed automatically, including the generated
file inventory, and `git ls-files --unmerged` was empty. No manual content
resolution, history rewrite or policy bypass was needed.

The owning README/AGENTS, CI operation contract, original package journal and
PR brief were inspected. The combined path inventory and source diffs confirm
the packet builder/tests match landed main and the decision validator/tests
match the previous PR180 candidate. Incoming host-hook documentation remains
owned by its landed upstream package. This refresh introduces no runtime,
adapter, schema, model, training or authority change. Existing source behavior
and characterization tests remain in their original canonical modules.

## Validation and limits

- `python scripts/inference/test_review_decisions.py`: seven tests passed.
- `python scripts/inference/test_review_outcomes.py`: 15 tests passed.
- `python scripts/ci/check_workflow.py`: passed.
- `git diff --check` and `git ls-files --unmerged`: clean.

Navigation and file inventory are regenerated with their owning scripts before
publication. Full candidate and merge-group CI must run for the refreshed head;
the previous candidate's 23 passing checks are not substituted for that evidence.
The original journal remains historical and unchanged. This corrective receipt
does not claim PR180 merged, runtime integration or model qualification. Stop
conditions remain failed CI, new conflicts, unexpected source changes or any
need to weaken protected landing. Roadmap and POA&M acceptance are unchanged.
