# CI cache experiment and standalone candidate integration

**Status:** active draft integration; dependency and hosted experiment pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-cache-warm-standalone-integration`
**Owner:** C-07 / J-02 candidate pipeline / [PR189](https://github.com/rrflow/rrflow-development/pull/189)

Starting PR189 head `a2cd15b224ed4b461cc4251d17afafae01703492`; ordinarily
merge [PR194](https://github.com/rrflow/rrflow-development/pull/194) exact
head `a385b484529c8eefd6c957342e90d0a6f7c2441b`, containing
[PR177](https://github.com/rrflow/rrflow-development/pull/177) head
`1ab64897bcee1440159d19846e1bf0a6eb3d02b2`. The PR189 brief recorded
this dependency, scope, traceability and stop conditions before the merge.
PR177 and PR194 must land with their own exact candidate and protected
merge-group checks before this draft can be made ready.

The sole predicted and actual conflict was the verify policy commands in
`.github/workflows/ci-reusable.yml`. The resolution retains both the cache
warmer and standalone policy tests, along with all SDK privacy and installed
standalone steps. The existing manual-main-only warmer and its independent
negative tests remain at their original canonical paths; the SDK/standalone
implementation and its independent journals remain at theirs. Reviewed the
workflow, operations record and workflow-policy merged diff against both
parents, and the preceding cache integration journal. No engine behavior,
required CI job, test threshold, upstream source or prior evidence is removed
or adapted. No new trace, cap or performance claim was introduced.

On Windows, `python scripts/ci/test_cache_warm_policy.py` passed 3 tests;
`python scripts/ci/test_rrflowdb_standalone.py` passed 7;
`python scripts/ci/test_candidate_admission.py` passed 11;
`python scripts/ci/test_sdk_private_manifest.py` passed 4 with one POSIX-only
case skipped; `python scripts/ci/test_execution_inventory.py` passed 10;
`python scripts/ci/check_workflow.py` and `cargo fmt --all -- --check` passed.
The original conflict required combining both policy commands, not changing
either policy. Hosted CI for this head, the real standalone process on this
merge, and a main-scoped cache save, subsequent matching candidate restore,
and controlled cold/warm timings were not run. Prior standalone process
evidence remains in [PR194's journal](../gate-h/h-04-standalone-sdk-integration.md).
No speedup, C-07 qualification or alpha completion is claimed; no roadmap or
POAM status changed. Checklist: pre-edit brief and explicit dependency,
conflict and source preservation, independent focused tests, linked journal,
generated navigation and inventory, and exact candidate/merge checks later.

`python scripts/ci/check_documentation.py` passed (741 document statuses and
739 classified coordinates); `python scripts/knowledge/render_navigation.py
--check` passed for 18 indexes; `python scripts/ci/build_execution_inventory.py`
and `--check` passed for 1901 records; `git diff --cached --check` passed.
The inventory is an ignored local projection, not a status authority.
