# Developer build and boundary reasoning direction

**Status:** active change-journal evidence; candidate CI and merge pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/boundary-reasoning-direction`
**Owner:** documentation alignment for OBJ-07/10/14/15 and the Gate D/H/J work

This record documents the owner's 2026-11-02 direction: get a working
RRFlow/Connectome developer build first; benchmark and tune a usable build;
then pursue configurable project-boundary reasoning shards and custom KV-cache
strategies as evidence supports. It does not claim those capabilities exist,
change the fifteen-objective alpha exit, or close any roadmap/POA&M item.

## Package evidence

- Starting revision `e6f3632b32f1c9be33c1951886a9d9a2be648ddf`, tree
  `85c2e6e27c77e0dc02abc7c31c509e4fa2655bdc`; draft
  [PR #245](https://github.com/rrflow/rrflow-development/pull/245).
- Reviewed in relevant sections: `README.md`,
  `docs/objectives/rrflow-1.0-alpha.md`, `docs/roadmap/rrflow-1.0.md`,
  `docs/roadmap/rrflow-1.0/executable-dependency-spine.md`,
  `docs/architecture/engine-data-flow.md`,
  `docs/architecture/engine-scale-and-contracts.md`,
  `docs/poam/rrflow-1.0-alpha/poam-006.md`,
  `docs/roadmap/rrflow-1.0-execution/change-authoring.md`,
  `crates/authority/rrd-engine/src/engine/{attunement,context,flight}.rs`,
  `crates/transport/rrd-server/src/http/router.rs`, and PRs #241 and #243.
- Changed: the README, objective, roadmap and scale rationale; created this
  journal; regenerated journal navigation. No source, dependency, API, version,
  release or implementation-status change.
- Before/after: the developer-build-to-tuning-to-later-shards direction was
  not explicit in the owning records. The roadmap and scale rationale also
  repeated a superseded runtime hold and missing-handler statement. They now
  state the current owner direction and distinguish existing handler inventory
  from the still-unproven end-to-end workflow.
- Research decision: GitHub's dependency-graph documentation, Tree-sitter's
  incremental parsing documentation and vLLM's prefix-cache design inform the
  rationale. They are not copied implementations or RRFlow benchmarks. Their
  links are in the scale rationale.
- Trace/resource/debug decision: documentation only; future experiments must
  retain source/configuration identity, cache boundary, correctness, resource
  and usage evidence. No runtime trace or benchmark was produced.
- First-failure oracle: stale hold and missing-handler claims found at the
  baseline despite the owner's later instruction and current public handlers.
  Replace those claims without changing any gate, granting external effects,
  or asserting that source indexing or the developer alpha is complete.
- Local validation: `python scripts/ci/check_documentation.py` passed
  (933 statuses, 931 classified coordinates); `python
  scripts/knowledge/render_navigation.py --check` passed (18 indexes, 932
  nodes, 3334 edges). The first `python
  scripts/ci/build_execution_inventory.py --check` reported drift because
  the ignored `target/rrflow-1.0-file-plan.jsonl` had not been generated in
  this new worktree; `python scripts/ci/build_execution_inventory.py` wrote
  2318 records and the subsequent `--check` passed. A later `--check`
  correctly reported drift after the journal changed; the inventory was
  regenerated again after the journal was finalized. `git diff --check`
  passed before staging. The changed source and journal diff was reviewed.
  Documentation, navigation and regenerated inventory checks were rerun after
  the substantive edits; candidate CI remains required.
- Remaining work: implement and prove the project journey, source indexing,
  user-visible diagnostics and qualified benchmarking; evaluate future shard
  and cache designs. C-07 native capacity and soak remain open. Runtime tests,
  candidate CI, merge-group CI and the actual PR merge have not yet passed as
  evidence for this candidate.
- Roadmap/POA&M status change: none.

## Links

- [Owner scale and contract rationale](../../../architecture/engine-scale-and-contracts.md#boundary-based-reasoning-after-the-developer-build)
- [Alpha objective](../../../objectives/rrflow-1.0-alpha.md#observable-engine-objective)
- [Release roadmap](../../../roadmap/rrflow-1.0.md#executable-dependency-spine)
