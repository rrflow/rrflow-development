# Git inventory retirement: integrated public main

**Status:** active integration evidence; exact candidate and merge-group pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/git-inventory-main-integration-193`
**Owner:** POAM-014 / CI operations / [PR193](https://github.com/rrflow/rrflow-development/pull/193)

PR193 head `81025f79fb3ca3b3cdbfa4f0fe7fe341e1872518` had a fully green
[candidate](https://github.com/rrflow/rrflow-development/actions/runs/40877713691).
Public main then advanced to `e73ba9287dafba5d8fb8fb6a990fd145ffd01ab3`
when [PR187](https://github.com/rrflow/rrflow-development/pull/187) passed its
merge group and landed. The existing queue monitor stopped because PR193 had
a merge conflict. Its PR brief was extended before integrating main, and the
PR returned to draft while the new candidate is prepared.

The normal merge produced one conflict, `docs/roadmap/rrflow-1.0-file-plan.jsonl`.
The integrated PR keeps the retirement descriptor from PR193; PR187's former
generated rows are represented by regenerating the current Git-derived
inventory instead. All landed PR187 source, contracts, fixtures, tests and
linked journals applied automatically. No source or test behavior from either
side was intentionally deleted, and no candidate CI job or threshold was
removed. The previous inventory record remains available in Git history. The
generator, documentation checker, existing test suite and current CI contract
were reviewed along with the prior PR193 journal, integration brief, merge
status and complete conflict. The files deliberately changed for this
integration are the conflict resolution, this new journal and its generated
repository index; other staged paths are the landed main merge. No external
implementation is adapted and no runtime or resource trace is applicable to
this mechanical integration.

`python scripts/ci/test_execution_inventory.py` passed 10 tests,
`python scripts/ci/test_candidate_admission.py` passed 11 tests,
`python scripts/ci/check_workflow.py` passed, and `git diff --cached --check`
passed. `python scripts/ci/check_documentation.py`,
`python scripts/knowledge/render_navigation.py --check`, and
`python scripts/ci/build_execution_inventory.py --check` passed after
regenerating navigation and inventory. Independent Python enumeration of
`git ls-files --stage -z` compared SHA-256 values against 1,665 tracked file
bytes and the exact parent gitlink revision against one submodule; all matched
1,884 total current and planned rows. The later journal edit requires the
generated inventory to be refreshed before commit. The former green candidate belongs only to its old
checkout; full candidate CI and protected merge-group CI for this integrated
revision have not yet run. No C-07, objective, roadmap or POA&M completion
status is changed.
