# C-06b selective projection optimization for immutable segment range scans

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-06b-selective-projection-optimization-for-immutable-segment-range-scans`
**Owner:** the unchanged historical package receipt identified by its legacy heading and payload digest
**Legacy source:** `docs/roadmap/rrflow-1.0-execution-map.md@b7b061900b67d535ffec2710e6e4c30815d0f216#L2788`
**Legacy payload SHA-256:** `d8697fd7af29cf10f4f19b86da2f5ce30de1c42e643eb48df1dfc61cdf729d07`

This record refracts one completed package receipt out of the former
monolithic execution map. Its fenced payload is preserved byte for byte.
It reports historical evidence and cannot change roadmap completion or
POA&M lifecycle status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
for acceptance and the [change-journal index](../) for discovery.

## Journal payload

```text
gate/package: C-06b / selective range scan projection in `rrd-lsm` immutable segments
starting revision/tree/branch/remote/worktree: commit f033249; tree 0180ed6; branch agent/connectome-temporal-runtime-visualizer; development remote https://github.com/rrflow/rrflow-development.git; origin remote https://github.com/rrflow/rrflow.git; worktree dirty only from this package at capture time
baseline files/digests: crates/persistence/rrd-lsm/src/segment/mod.rs=04355bbeee627340cbf701214f7f4ae2217a3da71fc81e83dbde367cce5eeebb
files read in full before editing: README.md; AGENTS.md; complete C-06a journal in this map; `crates/persistence/rrd-lsm/src/lib.rs`; `crates/persistence/rrd-lsm/src/segment/{mod,format}.rs`; and `crates/persistence/rrd-lsm/tests/{segment,compaction,failure_matrix,manifest,maintenance,mvcc,snapshot_bundle,snapshot_memory,tiered_io,wal}.rs`. Upstream segment code was not copied
source adaptation and research disposition: retained prior C-06v4 spine-column architecture; applied local storage contract to defer value-column reads until after key/sequence filters succeed. No new upstream compatibility layer, provider lifecycle, or storage API was imported
files changed/created/deleted/moved: `crates/persistence/rrd-lsm/src/segment/mod.rs`
  - split row-group reads into `load_key_spine` and `load_value_columns`
  - added shared `value_from_columns` accessor for validity-aware, value-page-only reads
  - rewrote `visible_from` and `visible_ranges` to lazy-load value pages only when a row is within scan range and visibility predicate
  - added unit tests proving unmatched key windows do not load value pages and matched windows do
physical contract changed: visible key/range scans use key spines as default read path and now avoid opening value validity/offset/value pages for rows filtered out by key bounds or sequence visibility; value pages are loaded once per row group for accepted windows, not preemptively
focused and owning proof: `cargo test -p rrd-lsm visible_scan` (2/2 passed), `cargo test -p rrd-lsm` (all owning package suites passed), `cargo fmt --all`, and `cargo fmt --all -- --check` passed
not run and reason: C-06 mixed-family interference, property/fuzz differential, configurable byte/row-group budget, mapped-generation lifetime, fixed-hardware compression/value-placement/filter/cache comparisons, C-07 crash/lifetime completion, and Gates D through J were not run because this is one bounded rrflowKV segment-scan optimization package
remaining known errors: C-06 still needs mixed semantic-family, property/fuzz differential, configurable-budget, mapped-generation lifetime, and fixed-hardware comparisons for compression/value-placement/filter/cache behavior. F-01 remains owner for stamped Arrow/DataFusion read-path execution; rrflowQL still materializes semantic `QueryRow` values before Arrow conversion. C-07 and all later gates remain open
roadmap checkbox changed: no; C-06 remains unchecked and active, Gate C remains 5/7, POAM-002 remains Open, and no alpha outcome or later gate is promoted by this evidence
```
