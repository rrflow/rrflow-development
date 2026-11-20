# A page-cache hit in rrflowLSM costs no heap work

**Status:** active change-journal evidence; reads of compacted data level with Fjall up to 70,000 claims, still behind at 1,000,000

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/rrflowlsm-page-cache-hit-20261120`

**Owner:** [rrflowLSM against Fjall](../../../reference/storage/rrflowlsm-fjall-parity.md), step 1 of its order; continues [the range-scan seek](rrflowlsm-range-scan-seek-20261120.md)

**Starting revision:** main `155b8ce6f`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage; [kernel contract](../../../roadmap/kernel-contract.md) K10.
- **Current behaviour.** Every page-cache hit pushed a new entry onto a recency heap, and once stale entries passed twice the page count the three heaps were rebuilt from the map. The cache key was hashed with the default keyed hasher.
- **Target behaviour.** A hit updates the page's stamp in place. The page keeps the one heap entry it has; when eviction pops an entry whose stamp is older than its page's, the entry goes back in at the real stamp. The key is hashed with a multiplicative mix.
- **Scope.** `crates/persistence/rrd-lsm/src/segment/page_cache.rs`, one import in `segment.rs`, one unit test in `segment/tests.rs`.
- **Failure oracle.** `repeated_hits_keep_one_order_entry_per_page_and_evict_the_least_recent`, committed failing first: 13 order entries for 3 pages after 1,000 hits.
- **Trace decision.** A `perf` profile of the comparison's read workload was taken first (300,000 reads, 70,000 claims). Page-cache work on hits was about 18 percent of all samples: `load_page_with_access` 5.7, `lookup` 3.8, key hashing 3.5, `refresh` 2.4, `rebuild_orders` 2.3.
- **Stop conditions.** Stop if the eviction victim changes; the new test and the existing replacement tests check it.

## Why the victim is the same

Every heap entry carries a stamp at or below its page's real stamp, because a stamp only grows and an entry is written at the stamp of its time. Eviction pops the smallest entry. If its stamp is the page's real one, no other page can be older. If it is older than real, it is put back at the real stamp and the next smallest is tried. An entry for a page that has left the region is dropped, as before.

The page key is `(cache id, page ordinal)`, both assigned inside this process, so a keyed hash protects against nothing.

## Result

Same machine and method as the earlier runs (WSL2 Linux, ext4, 24 CPUs, Fjall `3.1.8`, seven trials, three for `large`). Ratios are rrflowKV over Fjall.

| Profile | Read rate after maintenance: baseline, after scan seek, after this | Read p95 after maintenance | rrflowKV reads a second: baseline to now |
|---|---|---|---|
| small-batch | 0.71, 0.79, **1.01** | 1.40, 1.25, 1.04 | 28,582 to 41,970 |
| standard | 0.75, 0.79, **1.00** | 1.31, 1.25, 0.99 | 16,569 to 22,373 |
| read-heavy | 0.76, 0.79, **0.98** | 1.36, 1.32, 1.03 | 8,588 to 11,109 |
| sustained | 0.80, 0.82, **1.03** | 1.23, 1.16, 0.95 | 8,212 to 10,709 |
| extended | 0.81, 0.85, **1.03** | 1.26, 1.21, 0.98 | 7,414 to 9,602 |
| large | 0.64, 0.70, **0.77** | 1.59, 1.44, 1.24 | 3,661 to 4,354 |

- Up to 70,000 claims, reads of compacted data are level with Fjall.
- **At 1,000,000 claims rrflowKV is 23 percent behind**, and its peak memory is 1.43 times Fjall's. Neither is addressed here.
- Write rate, recovery time and bytes on disk are within noise of the earlier runs.

## Evidence

| Command | Result |
|---|---|
| `cargo test -p rrd-lsm --locked` (Windows) | 258 passed |
| `cargo test -p rrd-store --all-features --locked` (Windows) | 287 passed |
| `cargo clippy -p rrd-lsm -p rrd-store --all-targets --all-features --locked -- -D warnings` | clean |
| `storage_comparison` on six profiles, `TMPDIR` on ext4 (Linux) | both engines verified in every trial |

Raw output: `eval/results/2026-11-20-rrflowkv-vs-fjall-cache-hit-<profile>.json`.

## Not proven

- One noisy machine. A ratio within a few percent of 1 is level, not a win.
- The cache is still behind one lock, and a point get still takes it seven times. A workload with concurrent readers was not measured.
- The 1,000,000-claim read loss and memory loss have not been profiled.
- Linux and macOS test runs are for the candidate gate.

## Roadmap and POA&M

No status change.
