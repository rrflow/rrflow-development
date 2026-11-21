# Archived read-path integration

**Status:** active integration evidence; qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-read-path-integration-20261121`
**Owner:** repository gate, Gate F storage and Gate H vectors

## Before-edit brief

Join original read-path source history while preserving the current canonical implementation.

Driver: complete recovery of original source branches and stashes. Baseline e163ad7db0330f7a88dec23841ab4b9ff5dc798c. Owner: repository integration gate, Gate F authenticated storage and Gate H vector retrieval. Source d0e6c7f26e4d3a26232adedce783126044e844a2 includes 08e14533b585213e4811c84be412c2497b8822c2 and 8747cb76bdf10d025d6548d27a983d3c058b88dc; also join original reproduction merge ccb516c8b61effe51eb4cd0934f20d6c3b3617a5. Target: join original parentage, retain batched authenticated reads, SIMD kernels and bounded memory defaults at current split owners; remove no current behavior or assertions.

Reviewed: original three commit patches; current versioned_read/ranges.rs and reads.rs batch/proof paths, full versioned_read_batch_tests.rs; current hnsw/score.rs dispatch and kernels against original functions, hnsw tests first 150 lines, current page-cache/vector-residency defaults. Eight storage tests already passed on 071ff669, as did the complete rrd-vector correctness suite and 71 engine vector-index tests. Those are baseline evidence, not results on this candidate.

Exact scope: crates/persistence/rrd-store/src/access/versioned_read and its batch tests; crates/compute/rrd-vector/src/hnsw and documentation; current memory defaults in rrd-engine, rrd-lsm and rrd-contract; original merge resolutions; this journal and generated index. Preserve current proof-node cache, SIMD underflow floor, 64 MiB page cache and 1 GiB vector pin budget. Original old monoliths map to these split modules. Stop if any reusable assertion is missing or resolution changes these boundaries; repair before readiness. No new benchmark or comparison workload, external research or runtime instrumentation is required to recover existing code. No ARM performance claim.

Acceptance: cargo test -p rrd-store --lib versioned_read_batch_tests --locked; cargo test -p rrd-vector --locked; cargo test -p rrd-engine --test workspace_architecture --locked; docs/navigation/whitespace. Cargo target D:/.usr/Temp/rrflow-root-target, four jobs, no simultaneous source edits. Record actual deltas and failures. Candidate and merge-group ci-gate must pass, then branch deletion. Broader archive recovery continues afterward.

## Integration evidence

Original d0e6c7f26e4d3a26232adedce783126044e844a2 joined at 0835fdae,
including its batching and memory-default ancestors. The original reproduction
merge ccb516c8b61effe51eb4cd0934f20d6c3b3617a5 joined cleanly afterward.
Both joins leave the implementation/test tree unchanged.

The incoming monoliths map to current hnsw/{config,graph,index,score,validate},
versioned_read/{ranges,reads}, deployment/storage_limits and segment.rs.
Retained the current split owners. Batch tests differ from the original only
by equivalent row.div_ceil(2) syntax. Original scalar/AVX2/NEON behavior remains;
the later SIMD_SCORE_FLOOR fallback is retained. The original manual timing
probe is not restored as a correctness assertion. Current proof-node caching
remains bound to the immutable reader. Original 64 MiB page-cache and 1 GiB
vector-pin defaults remain. Physical pinned-read budget is a distinct 4 GiB
limit, not a lost vector default.

The broad combined conflict output was truncated and is not claimed as full
review; the original delta, extracted kernel functions, split-owner paths and
full 386-line batch test were reviewed before resolution. No original assertion
was removed. No source-adaptation or new runtime behavior is claimed.

## Verification

Candidate acceptance is recorded below when complete. ARM NEON execution,
performance measurement, external comparisons and product qualification are
not part of this source integration.

All acceptance passed on a531cf3ff66314b1e03aea3595e7ce5d656c90e6 plus
this journal: authenticated batch reads 8 tests; complete rrd-vector suite
(86 unit tests passed, one explicitly ignored long mixture test, all integration
suites passed); workspace architecture 35 tests; documentation checker,
navigation generation and whitespace. Commands are those in the brief; log
read-path-recovery-tests.log is retained in the session scratch directory.
No runtime/test source differs from baseline e163ad7db. Source recovery does
not close the broader vector, storage or alpha qualification gates.
