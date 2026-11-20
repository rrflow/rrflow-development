# rrflowLSM against Fjall, capability by capability

**Status:** active parity work list; 35 present, 22 partial, 27 absent as read on 2026-11-20. It claims no parity
**Coordinate:** `rrflow://rrflow-instance/data/reference/storage/rrflowlsm-fjall-parity`
**Owner:** which Fjall capabilities rrflowLSM has, and the order the missing ones are built in; the [roadmap](../../roadmap/rrflow-1.0.md) and [POA&M](../../poam/rrflow-1.0-alpha.md) keep status authority

Owner, 2026-11-19: "we need to be able to continue to optimize and ensure it has all of the capabilities of fjall first". The [engine baselines direction](../../roadmap/engine-baselines-owner-direction-20261119.md) names Fjall as the baseline rrflowLSM must capture and then beat.

**How this was read.** Fjall `3.1.8` and its tree crate `lsm-tree 3.1.8` were read from their published source, against `crates/persistence/rrd-lsm` at `main` `b63f7d447`. A row is **has** when rrflowLSM offers the capability, **partial** when it offers part or offers it one layer up in `rrd-store`, and **absent** when nothing was found. Tests were matched by name; their bodies were not each read. A row changes state only in the pull request that adds the capability and its test.

Paths under "Where" are relative to `crates/persistence/rrd-lsm/src/` unless they start with `rrd-store`.

## Keyspaces

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 1 | Several keyspaces, each its own LSM tree | partial | One tree; spaces are a key-prefix tag (`rrd-store/src/keyspaces/space.rs`) |
| 2 | Atomic writes across keyspaces | has | One batch is one log frame (`database/transactions.rs`) |
| 3 | Create, list, test and count keyspaces at runtime | absent | Spaces are a compile-time enum |
| 4 | Delete a keyspace | absent | |
| 5 | Options per keyspace | absent | One `DatabaseOptions` for the tree |

## Write path

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 6 | Single insert and remove | has | `batch.rs` |
| 7 | Atomic write batch | has | `batch.rs`, `database/transactions.rs` |
| 8 | Checksummed journal | has | CRC32C per frame (`wal.rs`) |
| 9 | Persist modes: buffer, sync data, sync all | partial | Buffered and `sync_data`; no sync with metadata |
| 10 | Durability per batch and per transaction | has | `database/transactions.rs` |
| 11 | Manual persist | has | `Database::sync`; no test found by name |
| 12 | Journal compression of large values | absent | |
| 13 | Journal rotation and size cap | has | Rotates at each flush (`database/flush.rs`) |
| 14 | Journal preallocation | partial | Linux only, 1 MiB |
| 15 | Memtable size limit | has | `mutable.rs` |
| 16 | Global write-buffer cap | has | `mutable_memory.rs` |
| 17 | Backpressure that slows writers before halting them | partial | The writer flushes in its own call; at the hard cap the write is refused |
| 18 | 2^32-byte values | partial | 1 MiB keys, 8 MiB values, 16 MiB batches |

## Transactions

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 19 | Single-writer transactions | has | `database/transactions.rs` |
| 20 | Optimistic multi-writer, serializable | partial | Only written keys are conflict-checked; write skew is allowed (`transaction.rs`) |
| 21 | Read your own writes | has | `transaction.rs`, `transaction/scan.rs` |
| 22 | Take, fetch-and-update, update-and-fetch | absent | |
| 23 | Rollback | has | `transaction.rs` |
| 24 | Read-only snapshot transaction | has | `database/transactions.rs` |

## Read path

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 25 | Point get | has | `database/reads.rs` |
| 26 | Contains key | absent | |
| 27 | Size of a value without reading it | absent | |
| 28 | Range scan | has | Half-open ranges only |
| 29 | Prefix scan | partial | The helper is in `rrd-store/src/rrflow_kv/codec.rs` |
| 30 | Full iteration | has | `database/reads.rs` |
| 31 | Reverse iteration | absent | Cursors are forward-only (`segment/reader/cursors.rs`) |
| 32 | Lazy iterator that reads key, size or value on demand | partial | Projected batches exist; no row-level iterator |
| 33 | First and last key | absent | |
| 34 | Length and is-empty | absent | |
| 35 | Approximate length | absent | |

## Snapshots and old versions

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 36 | Sequence-number snapshots | has | `database.rs` |
| 37 | Snapshots tracked automatically for cleanup | partial | Transactions and read views are tracked; a plain snapshot is not |
| 38 | Old versions removed in compaction | has | `database/compaction.rs` |
| 39 | Sequence accessors | has | |

## Deletes

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 40 | Tombstones, dropped when nothing is shadowed | has | `database/compaction.rs` |
| 41 | Weak (single) delete | absent | |
| 42 | Range delete | absent | In Fjall's tree crate only |
| 43 | Clear a keyspace | absent | |

## Compaction and flush

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 44 | Leveled strategy | has | `database/compaction.rs` |
| 45 | Level size ratio and thresholds | partial | No ratio; lower levels compact at two segments |
| 46 | Level count | has | `max_level` |
| 47 | FIFO strategy with size limit and time-to-live | absent | |
| 48 | Pluggable compaction strategy | absent | Selection is fixed |
| 49 | Move a segment down without rewriting it | partial | Single-segment promotion exists; a rewrite-free path was not confirmed |
| 50 | One-call major compaction | partial | `compact()` runs one bounded step |
| 51 | Compaction filters | absent | |
| 52 | Background worker pool | partial | No threads; the owner offers one step at a time under the write lock |
| 53 | Parallel compactions | absent | |
| 54 | Flush on memtable size | has | `database/transactions.rs` |
| 55 | Manual flush | has | `database/flush.rs` |
| 56 | Sealed-memtable queue flushed in the background | absent | Flush runs inside the writing call |

## Filters, indexes, cache and compression

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 57 | Bloom filters with bits per key or false-positive rate per level | partial | Fixed at 10 bits and 7 hashes (`segment/format/layout.rs`) |
| 58 | Filter pinning and partitioning policy | partial | Always in memory; no policy |
| 59 | Skip filters on the last level | absent | |
| 60 | Block index, full or two-level, with pinning | partial | First and last key per row group, in memory |
| 61 | Data block size policy | partial | Row groups of 2,048 rows or 64 KiB |
| 62 | Restart intervals and key prefix truncation | absent | Whole keys are stored |
| 63 | Hash index inside a block | absent | |
| 64 | Block cache with a set size and counters | has | `segment/page_cache.rs` |
| 65 | File-descriptor cache | has | `io/handles.rs` |
| 66 | Compression policy per level | partial | Adaptive LZ4 per page, not per level |
| 67 | Index block compression | absent | |

## Large values and bulk load

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 68 | Key-value separation | absent | A lab observation type only |
| 69 | Blob cleanup | absent | Needs 68 |
| 70 | Pre-sorted bulk ingestion | absent | |

## Recovery, concurrency, observation and platform

| # | Fjall capability | State | Where, or what is missing |
|---|---|---|---|
| 71 | Recovery that truncates a corrupt journal tail | partial | A torn tail needs an explicit repair (`tail_repair.rs`) |
| 72 | Manifest recovery | has | `manifest/` |
| 73 | Block checksums | has | SHA-256 per page and per file |
| 74 | Poisoning after a failed write | has | `wal.rs` |
| 75 | Format version check | has | |
| 76 | Lock file | has | `manifest/open.rs` |
| 77 | Temporary database, deleted on drop | absent | |
| 78 | A shared handle: writers queue inside, readers never wait on maintenance | partial | Writes take `&mut self` behind a lock in `rrd-store`; flush and compaction hold it |
| 79 | One process per directory | has | Plus a shared-lock read-only inspector |
| 80 | Runtime counters | has | `database/options.rs` |
| 81 | Disk space of the database and each keyspace | partial | Segment bytes only |
| 82 | Block, filter and cache metrics | has | `segment/page_cache.rs` |
| 83 | Options persisted per keyspace | partial | Each segment persists its page policy; database options are not persisted |
| 84 | Linux, macOS and Windows | has | `filesystem/native.rs` |

## What rrflowLSM has that Fjall does not

Named checkpoints; snapshot-bundle export and atomic install; multi-get and multi-range scan; typed resource admission for disk, recovery, compaction and transactions; a read-only inspector with plan-then-apply tail repair; SHA-256 integrity per page and file with content-addressed manifests; failure injection at named boundaries; key-only projection; three segment I/O modes; a path-capability filesystem layer.

## Measured against Fjall

The [comparison](rrflowkv-benchmark-harness.md#fjall-comparison-baseline) on 2026-11-20 is recorded in [its journal](../../evidence/change-journals/gate-c/rrflowkv-vs-fjall-comparison-20261120.md). rrflowKV wrote faster, recovered faster and used fewer bytes. It read slower after maintenance on every profile, and used more peak memory at one million claims.

Candidate causes of the read loss, read from the code and not confirmed by a profile:

1. A range read starts at row 0 of the first overlapping row group and walks forward; the seek that point reads use is not used (`segment/reader/cursors.rs`).
2. It does not stop at the end of the range: it examines the rest of the group and every later group descriptor.
3. A row group is 2,048 rows or 64 KiB, against Fjall's 4 KiB blocks.
4. Each returned row takes three page-cache lookups under one global lock, and each point get about seven (`segment/read.rs`, `segment/page_cache.rs`).
5. Several heap allocations and two value copies happen per row (`segment/reader/stream.rs`).

## Order

Each line is one package with its own test, and each is measured with the comparison before and after.

1. The read path: seek within the group, stop at the range end, hold the pages for the scan (no format change).
2. Rows 26, 27, 33, 34, 29, 22, 9, 77 and 81: the small read and write operations.
3. Row 31 reverse iteration, then rows 32 and 35.
4. Rows 41, 42 and 43: deletes.
5. One segment-format revision: rows 57, 59, 61, 62, 63, 66 and 67.
6. Row 48, then rows 45, 47, 49, 50 and 51: compaction.
7. Row 20 serializable optimistic transactions, and row 37.
8. Rows 56, 52, 53, 78 and 17: flush and compaction off the write lock. This ends the single-writer design.
9. Rows 1, 3, 4, 5 and 83: physical keyspaces.
10. Rows 68, 69 and 18: key-value separation.
11. Rows 70, 71, 12 and 14.
