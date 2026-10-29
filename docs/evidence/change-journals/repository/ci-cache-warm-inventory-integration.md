# CI cache experiment and inventory candidate integration

**Status:** active draft integration; upstream landing and hosted experiment pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-cache-warm-inventory-integration`
**Owner:** C-07 / J-02 candidate pipeline / [PR189](https://github.com/rrflow/rrflow-development/pull/189)

Starting PR189 head `04a6f75383949cda1d7bd866d47e754e56e81b8c`;
dependency candidate [PR193](https://github.com/rrflow/rrflow-development/pull/193)
head `bdb954e48d2eb751b4839ed6efa761cc1e294071`, containing public main
`e73ba9287dafba5d8fb8fb6a990fd145ffd01ab3`. The PR189 description records
the integration scope, traceability, tests and stop conditions before the
merge. PR193 has not yet landed, so this remains a dependency-blocked draft.

The normal merge had exactly the forecasted two conflicts: the old generated
`docs/roadmap/rrflow-1.0-file-plan.jsonl` and `scripts/ci/check_workflow.py`.
The former retains PR193's retired descriptor; its exhaustive current
inventory is generated under ignored `target`. The latter retains both the
existing revision-bound inventory generation/retention validator and the
manual, main-only macOS cache-warmer validator, including their independent
calls from the policy entry point. The existing verify workflow retains both
negative-test suites. No runtime behavior, required candidate test, build
profile, threshold or prior evidence was removed. The merge imports all
PR193 and landed main source; the direct integration edits are the two
conflict resolutions, this journal and its generated discovery index. The
original cache experiment journal remains unchanged.

Reviewed the PR193 integration brief and prior journal, the complete cache
warmer and its negative tests, the overlapping conflict and complete
integration diff for policy and workflow files, and the native reopen job's
runner, toolchain, shared key, environment and build command. No upstream
source was adapted, and no runtime trace applies. The previously recorded
GitHub and rust-cache primary documentation defines cache visibility and key
behavior; a cache is replaceable build input, not test or product evidence.

`python scripts/ci/test_cache_warm_policy.py` passed three tests including
rejected triggers, guards, checkout, key and build drift;
`python scripts/ci/test_execution_inventory.py` passed ten tests including
tampered inventory and CI-binding mutations;
`python scripts/ci/test_candidate_admission.py` passed 11 tests;
`python scripts/ci/check_workflow.py` passed with all nine substantive jobs;
and `git diff --cached --check` passed. After regenerating navigation and the
inventory, `python scripts/ci/check_documentation.py`,
`python scripts/knowledge/render_navigation.py --check`, and
`python scripts/ci/build_execution_inventory.py --check` passed. Independent
`git ls-files --stage -z` enumeration compared 1,669 tracked file SHA-256
values and one exact gitlink with 1,888 generated rows; all matched. The
subsequent journal edit requires inventory regeneration before commit. Neither a
candidate nor a protected merge-group result exists for this integration.
The manual warmer cannot run from main before its eventual protected merge;
no default-branch cache save, cross-run restore or timing improvement is
claimed. Keep the draft pending PR193's landing and exact integration checks.
No roadmap, objective, POA&M or C-07 qualification status changed.
