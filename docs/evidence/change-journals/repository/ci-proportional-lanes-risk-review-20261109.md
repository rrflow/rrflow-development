# Candidate CI durable-effect routing review

**Status:** active supplementary change-journal evidence; CI and merge pending.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-proportional-lanes-risk-review-20261109`
**Owner:** [CI and runner operations](../../../operations/ci.md#proportional-lanes); supplement to the [proportional candidate CI package](ci-proportional-lanes-20261109.md).

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: OBJ-02 / J-02 CI prerequisite. Review correction to PR #303 lane-risk mapping; no alpha, roadmap, release-gate, or POA&M completion claimed.
starting revision/tree and pull request: 14674232c9f7194bf93f7b71b9168f20bd575790 on agent/ci-proportional-lanes-20261109; PR #303; its candidate CI had already started on the preceding tree.
change brief: review found engine-owned persistent effect modules not included by the first lane map. Add the currently identified runtime/control publishers and engine recovery test modules to the full lane, plus regressions and the CI owner table. Preserve fast lanes for lower-risk engine crates. Do not weaken existing acceptance jobs. Stop if a mutation publisher remains unclassified or the selection/gate regression fails.
files and symbols read in full: `scripts/ci/select_ci_lanes.py` FULL_PATTERNS, matches and select; `scripts/ci/test_select_ci_lanes.py`; `docs/operations/ci.md` proportional-lane table. Read-only source search for `.commit_batch(`, `.commit_transaction(`, `runtime().commit(`, `control().commit(` and `commit_engine_control(` under `crates/authority/rrd-engine/src/engine` identified current effect modules.
implementation/evidence paths created, changed, moved, or deleted: FULL_PATTERNS and its routing regressions; full-lane scope in `docs/operations/ci.md`; this supplement and generated repository journal index. No source paths moved or deleted.
research decision and primary-source/adaptation record: no external code adapted. This is a codebase risk correction based on current engine commit/publish call sites.
trace/resource/debug decision: no runtime instrumentation change. GitHub candidate jobs provide the required execution and duration evidence.
first-failure or characterization oracle and result: the initial list-only policy routed several currently identified effect modules to engine-fast. Added route assertions for endpoint configuration, estate control, attunement, function execution, observation persistence, reasoning-budget persistence and recovery tests; all select full.
acceptance commands and exact results: `python scripts/ci/test_select_ci_lanes.py` -> 14 tests OK; `python scripts/ci/test_ci_lanes_policy.py` -> 9 tests OK; `python scripts/ci/check_workflow.py` -> OK (7 workflows, 15 substantive jobs); `python scripts/ci/check_documentation.py` -> OK (1,034 statuses, 1,032 coordinates); `python scripts/knowledge/render_navigation.py --check` -> OK (18 indexes, 1,033 nodes, 3,610 edges, no drift); `git diff --check` -> clean.
surfaced failures and corrections: the initial classifier's `transaction*` patterns did not include every current engine-side durable publisher; the full-risk path set now covers the modules found by the source search and engine tests. New future effect modules must be added to the risk mapping when authored.
checks not run and reason: current candidate run is bound to the prior head and cannot qualify this correction. GitHub candidate and merge-group checks must run on the new exact PR head. Native soak timing and actual post-landing lane durations remain unmeasured.
remaining known errors and owner: this is a maintained path-risk registry; code review must update it when a new durable effect boundary is introduced. The full workflow qualification is still pending.
full-file reread and diff review: classifier, classifier test, CI policy test, modified CI owner section, and this supplement reviewed; broader PR diff and final candidate digest must be reconciled before merge.
change checklist: focused classifier/policy and repository checks passed; no test partition or merge gate removed; supplemental evidence linked and index regenerated; exact-head candidate CI required.
roadmap/POA&M status change: none.
pull request, CI run, and merge evidence: PR #303, https://github.com/rrflow/rrflow-development/pull/303; the in-progress run on the preceding head is not acceptance evidence for this supplement. New-head CI pending.
```

## Links

- [CI operations](../../../operations/ci.md)
- [Original package journal](ci-proportional-lanes-20261109.md)