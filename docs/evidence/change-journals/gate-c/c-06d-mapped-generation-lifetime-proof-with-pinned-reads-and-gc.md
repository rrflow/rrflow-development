# C-06d mapped-generation lifetime proof with pinned reads and GC

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-06d-mapped-generation-lifetime-proof-with-pinned-reads-and-gc`
**Owner:** the unchanged historical package receipt identified by its legacy heading and payload digest
**Legacy source:** `docs/roadmap/rrflow-1.0-execution-map.md@b7b061900b67d535ffec2710e6e4c30815d0f216#L2827`
**Legacy payload SHA-256:** `58c954b89010f2d635c7850a6b8d9cef9b09672bddac65c85f7a395481a6606b`

This record refracts one completed package receipt out of the former
monolithic execution map. Its fenced payload is preserved byte for byte.
It reports historical evidence and cannot change roadmap completion or
POA&M lifecycle status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
for acceptance and the [change-journal index](../) for discovery.

## Journal payload

```text
gate/package: C-06d / mapped-generation lifetime safety against compaction and GC deletion in rrflowKV
starting revision/tree/branch/remote/worktree: commit 2428819eaed89cd4e6a318acb3d05a6bfeae1a10; tree 0f8ec4b61f570444bdf2c8c36b18399d2b324abb; branch agent/connectome-temporal-runtime-visualizer; development remote https://github.com/rrflow/rrflow-development.git; origin remote https://github.com/rrflow/rrflow.git; worktree dirty only from this package at capture time
baseline files/digests: crates/persistence/rrd-lsm/tests/compaction.rs=ec7deca47017b642a282a74ad8a8e5f1bbd7237fc602fd3e18cffa5d86b1113a (blob at start revision)
files read in full before editing: README.md; AGENTS.md; complete C-06a, C-06b, and C-06c journals in this map; `crates/persistence/rrd-lsm/src/lib.rs`; `crates/persistence/rrd-lsm/src/{database,manifest,segment,wal}.rs`; and `crates/persistence/rrd-lsm/tests/{compaction,segment,failure_matrix,manifest,maintenance,mvcc,snapshot_bundle,snapshot_memory,tiered_io,wal}.rs`. Upstream segment/LSM code was not copied
source adaptation and research disposition: retained existing C-06v4 spine-and-page semantics and transaction snapshot rules; added a differential/lifetime test proving `Transaction` snapshots outlive compaction and segment GC based on manifest deltas without introducing any provider-specific storage behavior
files changed/created/deleted/moved: `crates/persistence/rrd-lsm/tests/compaction.rs`
  - added `pinned_transaction_reads_survive_compaction_garbage_collect_cycles`
  - configured compacting threshold via `CompactionPolicy::l0_compaction_trigger`
  - captured pre-compaction and post-compaction segment IDs from manifest snapshots
  - asserted pinned historical reads remain visible before and after `compact` + `garbage_collect`
  - asserted GC only removes segments not present in the current manifest
physical contract changed: no storage layout change; added a runtime-lifetime safety proof for MVCC snapshots/transactions during segment replacement and garbage collection, extending the durability/lifetime matrix expectation before C-07
focused and owning proof: `cargo test -p rrd-lsm pinned_transaction_reads_survive_compaction_garbage_collect_cycles -- --nocapture` (1/1 passed); `cargo test -p rrd-lsm` (all owning package suites passed); `cargo fmt --all`; and `cargo fmt --all -- --check` passed
checks not run and reason: C-06 configurable byte/row-group budgets, fixed-hardware compression/value-placement/filter/cache comparisons, full property/fuzz differential/adversarial coverage, and C-07’s broader crash/recovery matrix were not run because this is one bounded mapped-lifetime package; Gates D through J were not run
remaining known errors: C-06 still lacks configurable budgets and full property/fuzz evidence, fixed-hardware comparisons for compression/value-placement/filter/cache behavior, and the broader mapped-buffer dangling test matrix in C-07; F-01 remains owner for stamped Arrow/DataFusion read-path execution; rrflowQL still materializes semantic `QueryRow` values before Arrow conversion. C-07 and all later gates remain open
roadmap checkbox changed: no; C-06 remains unchecked and active, Gate C remains 5/7, POAM-002 remains Open, and no alpha outcome or later gate is promoted by this evidence
```
