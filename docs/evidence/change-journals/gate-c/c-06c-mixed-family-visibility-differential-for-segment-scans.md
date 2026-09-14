# C-06c mixed-family visibility differential for segment scans

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-06c-mixed-family-visibility-differential-for-segment-scans`
**Owner:** the unchanged historical package receipt identified by its legacy heading and payload digest
**Legacy source:** `docs/roadmap/rrflow-1.0-execution-map.md@b7b061900b67d535ffec2710e6e4c30815d0f216#L2808`
**Legacy payload SHA-256:** `334206a1f79f9d5b4bdc680731ae7aaa20f871bc4fbc9b9f0911de505ab71a16`

This record refracts one completed package receipt out of the former
monolithic execution map. Its fenced payload is preserved byte for byte.
It reports historical evidence and cannot change roadmap completion or
POA&M lifecycle status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
for acceptance and the [change-journal index](../) for discovery.

## Journal payload

```text
gate/package: C-06c / mixed-family visibility correctness differential for `rrd-lsm` immutable segments
starting revision/tree/branch/remote/worktree: commit b108cf2af0dfe7b3933b73615e43faeff48eed1f; tree b974919; branch agent/connectome-temporal-runtime-visualizer; development remote https://github.com/rrflow/rrflow-development.git; origin remote https://github.com/rrflow/rrflow.git; worktree dirty at capture time
baseline files/digests: crates/persistence/rrd-lsm/src/segment/mod.rs=04355bbeee627340cbf701214f7f4ae2217a3da71fc81e83dbde367cce5eeebb
files read in full before editing: README.md; AGENTS.md; complete C-06a and C-06b journals in this map; `crates/persistence/rrd-lsm/src/lib.rs`; `crates/persistence/rrd-lsm/src/segment/{mod,format}.rs`; and `crates/persistence/rrd-lsm/tests/{segment,compaction,failure_matrix,manifest,maintenance,mvcc,snapshot_bundle,snapshot_memory,tiered_io,wal}.rs`. Upstream segment code was not copied
source adaptation and research disposition: retained the v4 spine/Arrow-page contract and added deterministic memtable-oracle differentials over mixed key-family windows with tombstones and multi-version history; no upstream code path was imported
files changed/created/deleted/moved: `crates/persistence/rrd-lsm/src/segment/mod.rs`
  - added `mixed_family_memtable` and `visible_ranges_oracle` test helpers
  - added `visible_scan_matches_memtable_visibility_across_ranges_and_sequences` to compare `visible_from` against `Memtable::visible_from` for mixed families and snapshot cut points
  - added `visible_ranges_match_memtable_oracle_for_mixed_families` to compare `visible_ranges` against a deterministic memtable oracle across disjoint mixed-family ranges
physical contract changed: no physical layout contract change; the package adds behavioral differential evidence that current `visible_from` and `visible_ranges` preserve memtable-semantics for family-interleaved keys and tombstone states under multiple read sequences
focused and owning proof: `cargo test -p rrd-lsm visible_scan_matches_memtable_visibility_across_ranges_and_sequences` (1/1 passed), `cargo test -p rrd-lsm visible_ranges_match_memtable_oracle_for_mixed_families` (1/1 passed), `cargo test -p rrd-lsm` (all owning package suites passed), `cargo fmt --all`, and `cargo fmt --all -- --check` passed
not run and reason: C-06 configurable-budget, mapped-generation lifetime, fixed-hardware compression/value-placement/filter/cache comparisons, mixed-family adversarial generator/property-fuzz sweeps, C-07 crash/lifetime completion, and Gates D through J were not run because this is one bounded rrflowKV segment differential package
remaining known errors: C-06 still lacks full property/fuzz differential evidence, configurable byte/row-group budgets, mapped-generation lifetime proof, and fixed-hardware comparisons for compression/value-placement/filter/cache behavior. F-01 remains owner for stamped Arrow/DataFusion read-path execution; rrflowQL still materializes semantic `QueryRow` values before Arrow conversion. C-07 and all later gates remain open
roadmap checkbox changed: no; C-06 remains unchecked and active, Gate C remains 5/7, POAM-002 remains Open, and no alpha outcome or later gate is promoted by this evidence
```
