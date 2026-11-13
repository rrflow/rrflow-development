# Scan-resistant page-cache pre-edit traceability (C-06j)

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/scan-resistant-page-cache-pre-edit-traceability-c-06j`
**Owner:** the unchanged historical package receipt identified by its legacy heading and payload digest
**Legacy source:** `docs/roadmap/rrflow-1.0-execution-map.md@b7b061900b67d535ffec2710e6e4c30815d0f216#L3913`
**Legacy payload SHA-256:** `811cf9b7917433ac48eb0cdbd87e830cf97791740fa6598601ee1a26e792e51e`

This record refracts one completed package receipt out of the former
monolithic execution map. Its fenced payload is preserved byte for byte.
It reports historical evidence and cannot change roadmap completion or
POA&M lifecycle status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
for acceptance and the [change-journal index](../) for discovery.

## Journal payload

```text
gate/package: C-06j-rrflowkv-scan-resistant-cache-integration / production exact-byte cache policy and real mixed-family differential
alpha outcome or prerequisite advanced: prepares the final measured C-06 physical-policy decision for OBJ-02 and the future bounded F-01 source path; it does not claim that C-06, DataFusion execution, installation, reasoning/recall, or an alpha outcome is complete
starting revision/tree/branch/remote/worktree: clean baseline 9592b886f2c6f2f716933fcf46e8c0595de3c315 / bc177bbdb4af4adb1e30ae0cfe8f6ae46d2fd949; planning commit d3a36b07d1b219ce3db3be4da9413b7a8a109f16; branch agent/connectome-temporal-runtime-visualizer; development target development/main at https://github.com/rrflow/rrflow-development.git; official origin https://github.com/rrflow/rrflow.git remains promotion-only; baseline Cargo.lock SHA-256 316533e7512dbb9029e494386503ca8430b0c69853b3b78ef02319dcbab93a27; runtime source untouched at this checkpoint
baseline ownership and useful behavior retained: crates/persistence/rrd-lsm/src/segment/mod.rs owns the sole process-local cache below Database/RrdEngine. Retain immutable Arc<LoadedPage> ownership, exact Buffer capacity charging, one database-shared cache, I/O and decode outside the mutex, strict total capacity, lazy-invalidated recency heaps, current load/physical/decode/filter evidence, and cache-independent MVCC/segment identity. Retain the feature-gated laboratory only as a candidate screen
behavior requiring direct replacement: one admit-every-fitting-miss global exact LRU permits a projected scan to displace repeatedly used pages; policy is not configurable; no probationary/protected production state exists; completed concurrent losing loads are not classified; simulated hit counts do not prove the real persisted reader
canonical source mapping: PageCacheStats, PageCache, CacheEntry, new_page_cache, and load_page_with_evidence remain at their current rrflowKV physical owner in segment/mod.rs for this slice. DatabaseOptions/create/open and the existing rrd_lsm::open trace remain the only composition/configuration path. Public exports remain in rrd-lsm/src/lib.rs. The physical laboratory and example remain evidence-only consumers. No file is moved or deleted, and no second cache, persistence, query, lifecycle, or reasoning authority is created
research decision and adaptation: primary RocksDB, InnoDB, TinyLFU, Moka, and DataFusion sources support testing scan resistance, second-use promotion, exact counters, and separate storage/query-cache boundaries. RRFlow selects a configurable family-neutral probationary/protected LRU for real-reader proof, retains exact LRU as oracle/operator policy, defers TinyLFU/Moka/sharding/coalescing, and rejects caller-controlled bypass and semantic-family capacity. The full source record is docs/research/rrflowkv-page-cache-and-mixed-workload-research.md
failure-first and evidence plan: add a public test before runtime types; require invalid ratio denial before path creation, same-durable-state reopen under both policies, an independently digested eight-family corpus larger than capacity, exact-LRU post-scan reloads, strictly fewer scan-resistant post-scan loads, exact region accounting, promotions, oversize rejection, and no semantic/durable difference. The clean artifact is docs/evidence/c06j-rrflowkv-scan-resistant-cache-linux-x86_64.json and remains one-host integration evidence only
trace/resource/debug decision: add low-cardinality cache policy, protected ratio, and capacity to the existing open trace; add exact policy/region/admission/promotion/demotion/rejection/duplicate-load statistics; emit no per-page key, value, family, prompt, provider, or project data. Record duplicate loads without claiming C-07 coalescing or scalability
planning validation: d3a36b0 is the direct single-parent child of the clean baseline and changes only docs/roadmap/rrflow-1.0-active-change.json. With the untracked research drafts safely removed and restored through a recoverable Git stash, python3 scripts/ci/check_change_plan.py passed planning-only with zero post-plan paths. No runtime test is substituted at this checkpoint
first real-reader result and successor plan: the unchanged small mixed-family laboratory produced exact-LRU post-scan loads=45 and naive scan-resistant post-scan loads=45. The projected stream generated 601 hits and 343 loads; unconditional second-hit promotion therefore promoted pages repeatedly touched inside the same scan. No workload, ratio, or assertion was weakened. The complete 1,549-line projected reader was reviewed, and successor planning commit 982b7a2d14210d2444da8679cf9040b3c7261740 authorizes an opaque engine-generated per-stream reuse scope plus a same-scope suppression counter. With runtime edits safely stashed, check_change_plan passed that successor planning commit with zero post-plan paths before the edits were restored
remaining known errors: the production cache is still admit-every-miss exact LRU until the planned runtime commit; the real mixed-family differential and clean artifact do not exist yet; C-06, C-07, all F gates, and D through J remain open
roadmap checkbox changed: no
```
