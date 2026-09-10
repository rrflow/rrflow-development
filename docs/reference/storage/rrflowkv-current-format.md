# rrflowKV current physical format

**Status:** active implementation reference for the single current typed application-key and hybrid immutable-segment formats; C-06 remains partial until the evidence listed below passes
**Coordinate:** `rrflow://rrflow-instance/data/reference/storage/rrflowkv-current-format`
**Owner:** physical bytes, limits, recovery rules, and pre-release format debt implemented by `rrd-store` and `rrd-lsm`

This record describes the bytes the current checkout can create or read. It is
not the accepted RRFlow 1.0 target. The target architecture is owned by the
[engine data-flow record](../../architecture/engine-data-flow.md); this file
does not own release status, and makes no performance or competitive claim.
Current comparison mechanics and evidence limitations are owned by the
[benchmark-harness reference](rrflowkv-benchmark-harness.md).
The [roadmap](../../roadmap/rrflow-1.0.md) owns the replacement and removal
work. [POAM-002](../../poam/rrflow-1.0-alpha.md) keeps the physical-layout
deficiency open; closed POAM-003 records the accepted single-node reader and
whole-executable pre-release-shape convergence.

## Current and target boundary

The current application-key format is the C-01 `RRKV0001` typed ordered tuple
grammar. All live `RrflowKvStore` key construction and decoding uses it, and a
store whose manifest does not authenticate that application identity is
rejected. The mutable path remains a write-optimized WAL plus MVCC memtable.
Flush now transforms that sorted mutable state into segment v4: an ordered
key/version spine and six Arrow-layout buffers per row group. Point, range,
snapshot, CAS, recovery, and compaction reads use those buffers directly and do
not invoke DataFusion.

This is the first C-06 implementation slice, not completion of C-06 and not a
claim that query output is zero-copy. The segment owns Arrow-compatible buffer
layout and safe mapped-buffer ownership. A later C-06 slice must expose a
selective projection stream to `rrd-store`; Gate F must consume that stream
through a stamped `TableProvider`/`ExecutionPlan` and prove actual
`RecordBatch` borrowing, pushdown, cancellation, and resource bounds. Current
semantic values can still be JSON bytes inside the value-data page, and current
query adapters can still decode values and allocate result arrays.

The active physical readers accept exactly mutation batch v2, manifest v3, and
segment v4. Batch v1, manifest v1, and segment v1/v2/v3 bytes are negative
inputs that return an explicit unsupported-format error before an alternate
decoder can run. There is no row-segment compatibility reader, Fjall store,
backend selector, or migration executor. The disabled optional OpenRaft
adapter remains tracked by POAM-023 and excluded from the single-node alpha.
This record does not turn any rejected format or post-alpha implementation into
a product requirement.

## Implemented object set

All integers in binary objects are unsigned big-endian unless stated
otherwise.

| Object | New-write format | Current role | Implementation |
|---|---:|---|---|
| Application keys | `RRKV0001` | Typed ordered family/address/tuple spine for every `RrflowKvStore` key | [`key_codec.rs`](../../../crates/persistence/rrd-store/src/key_codec.rs) and [`keyspaces.rs`](../../../crates/persistence/rrd-store/src/keyspaces.rs) |
| WAL | 1 | Ordered atomic-batch frames and durability boundary | [`wal.rs`](../../../crates/persistence/rrd-lsm/src/wal.rs) |
| Mutation batch | 2 | Canonical put/delete payload inside one WAL frame | [`batch.rs`](../../../crates/persistence/rrd-lsm/src/batch.rs) |
| Manifest | 3 | Immutable reachable-state inventory, sequence boundary, and authenticated segment schema/codec/page-format identities | [`manifest.rs`](../../../crates/persistence/rrd-lsm/src/manifest.rs) |
| `CURRENT` and checkpoint controls | 1 | Authenticated manifest and retention pointers | [`manifest.rs`](../../../crates/persistence/rrd-lsm/src/manifest.rs) |
| Immutable segment | 4 | Ordered key/version spine plus aligned Arrow-layout column pages and row-group index | [`segment/mod.rs`](../../../crates/persistence/rrd-lsm/src/segment/mod.rs) and [`segment/format.rs`](../../../crates/persistence/rrd-lsm/src/segment/format.rs) |
| Physical snapshot bundle | 1 | Authenticated flush-bounded manifest closure | [`snapshot_bundle.rs`](../../../crates/persistence/rrd-lsm/src/snapshot_bundle.rs) |

Current default and hard bounds are implementation policy, not values for
callers to duplicate:

| Bound | Current value | Source |
|---|---:|---|
| Mutable encoded WAL payload before maintenance | 64 MiB | `DEFAULT_WAL_PAYLOAD_MAX_BYTES` |
| Mutable MVCC versions before maintenance | 524,288 | `DEFAULT_MEMTABLE_MAX_VERSIONS` |
| Individual WAL frame payload | 16 MiB | `WAL_MAX_PAYLOAD_BYTES` |
| Mutation operations per batch | 1,000,000 | `MAX_OPERATIONS` |
| Key/value bytes | 1 MiB / 8 MiB | batch and segment validators |
| Segment bytes / segment-index bytes | 1 GiB / 64 MiB | segment validator |
| Row-group target / maximum rows / immutable page cache | 64 KiB / 2,048 / 4 MiB | segment writer and database default |
| Snapshot bundle bytes / segments | 1 GiB / 1,000,000 | snapshot-bundle validator |

Before a new batch crosses either mutable threshold, the single writer runs
the ordinary crash-ordered flush path. It never splits one accepted atomic
batch. An individually oversized batch remains one frame and is reported in
maintenance evidence.

## Typed application keys (`RRKV0001`)

Every application key is one self-describing tuple:

```text
format-tag / RRKV0001 / optional tenant / optional scope / family-tag / family / typed fields
```

The format and family tags are `0xf0` and `0xf1`. An absent address component
is `0x00`; present tenant, scope, and other text values use the text tag
`0x21`. Variable-width byte and text fields are split into eight-byte groups
with a canonical padding marker, so embedded NUL, slash, `0xff`, and every
other byte are data rather than delimiters. Fixed numeric fields are
big-endian; signed values flip the sign bit; descending version fields invert
the encoded `u64`, making newer versions sort first. The decoder rejects
unknown types, invalid UTF-8 text, non-canonical Boolean values or padding,
truncation, and trailing partial fields.

The frozen top-level family bytes are:

| Byte | Family | Byte | Family |
|---:|---|---:|---|
| `0x10` | current | `0x19` | vector |
| `0x11` | temporal | `0x1a` | projection delta |
| `0x12` | outgoing edge | `0x1b` | catalogue |
| `0x13` | incoming edge | `0x1c` | runtime commit |
| `0x14` | scalar | `0x1d` | outbox |
| `0x15` | unique | `0x1e` | audit |
| `0x16` | term dictionary | `0x1f` | engine event |
| `0x17` | term statistic | `0x20` | system |
| `0x18` | term posting |  |  |

The C-03 semantic writer currently assigns these implemented subspaces. The
tuple fields shown are logical typed components encoded by `KeyCodec`; they are
not delimiter-concatenated strings.

| Family | Subspace | Current key purpose |
|---|---:|---|
| catalogue | `0x20` / `0x21` | current runtime schema / retained runtime snapshot |
| catalogue | `0x30` / `0x31` / `0x32` | current schema-bound index commit binding / immutable binding revision / scope-level binding-set integrity head |
| scalar | `0x01` | current non-unique scalar entry keyed by scope, index, typed values, record, and descending valid-from |
| unique | `0x01` | current unique-window entry with the same typed value and record coordinates |
| vector | `0x01` / `0x02` | current vector head with commit coordinate / immutable temporal vector change |
| projection delta | `0x02` | generic projection work identity committed with its source mutation |
| projection delta | `0x03` | schema/catalogue/digest-bound scalar or BM25 old/new record fields |
| projection delta | `0x04` | vector-source old/new immutable-version pointers; no HNSW bytes |

An rrflowQL index-catalogue replacement and its commit bindings are written by
one control transaction. Installing or changing a scalar binding performs a
bounded current-record backfill; a unique binding rejects overlapping
non-null value windows before either the catalogue or entries publish. Later
record replacement or retirement removes the exact old scalar/unique key,
writes the exact new key when present, and records one materializer delta in
the semantic transaction. The binding-set head pins the complete projected
definition digest to the exact canonical catalogue bytes. A missing, partial,
or generically replaced projection therefore fails the next semantic commit
closed instead of silently bypassing an index constraint. The transaction also
rewrites the catalogue watermark value it observed, so a concurrent catalogue
transition conflicts at physical commit rather than publishing data under an
unobserved definition.

Current vector values are stored as heads containing the exact source commit
coordinate. Every vector insert, replacement, or retirement also writes a
temporal version plus source-specific pointer delta in the same semantic
batch. BM25 term dictionaries/postings, exact-vector access, TurboQuant, and
HNSW generations are not constructed here; Gates E-03 and E-04 consume the
committed deltas and must prove their independent publication and exact
fallback behavior. These semantic values remain encoded value bytes within
the v4 Arrow-layout buffers. A columnar physical envelope does not by itself
qualify their native access paths, candidate generation, recall, or exact
fallback behavior.

Catalogue subfamilies separately address function artifacts, definitions,
transaction bindings, membership revisions, the compare-and-swap head, and
invocation receipts. Artifact bytes remain values addressed by digest; they
are never copied into a key or repeated in a head record. Current per-project
semantic constructors use the global optional address and encode any existing
scope identity as a typed tuple field. The grammar nevertheless freezes and
tests distinct tenant and scope address components; changing their use is a
reviewed direct format convergence, not string concatenation or a compatibility
reader.

`prefix_end` computes the exact exclusive upper bound by carrying through
trailing `0xff` bytes. Range construction therefore uses a complete typed
component prefix and never a conceptual `*`, `~`, `+`, slash, or NUL delimiter.
The conceptual family symbols remain documentation shorthand only.

## WAL and acknowledgment

One accepted mutation batch is one WAL frame. `Authoritative` acknowledgment
follows `sync_data`; `Buffered` acknowledges a written frame without claiming
durability. A failed write or synchronization poisons that writer instance, so
the caller must reopen and recover rather than append after an unknown partial
write.

The deterministic write-fault surface names four ordered boundaries:
`write.prepared` after validation/admission and before append;
`write.wal_appended` after one complete frame is written but before the
authoritative sync; `write.wal_synced` after synchronization but before the
memtable becomes visible; and `write.visible` after the complete batch is
visible but before its receipt returns. Failures after append fence the writer
until reopen. Recovery may retain or discard an unacknowledged complete frame
at the unsynced boundary, but it may never expose only some operations from
that frame; synchronized and visible frames recover completely.

WAL file header, 16 bytes:

| Offset | Bytes | Meaning |
|---:|---:|---|
| 0 | 8 | ASCII `RRDWAL01` |
| 8 | 2 | format version `1` |
| 10 | 2 | header length `16` |
| 12 | 4 | CRC32C over bytes `0..12` |

Frame header, 32 bytes:

| Offset | Bytes | Meaning |
|---:|---:|---|
| 0 | 4 | ASCII `RRD1` |
| 4 | 2 | format version `1` |
| 6 | 1 | record kind `1` for an atomic batch |
| 7 | 1 | zero flags; unknown flags fail closed |
| 8 | 4 | payload length, at most 16 MiB |
| 12 | 8 | first MVCC sequence |
| 20 | 8 | last MVCC sequence |
| 28 | 4 | CRC32C over header bytes `4..28` and payload |

Sequence ranges must be contiguous and non-zero. Recovery replays the longest
valid prefix. An incomplete file header is corruption; an incomplete final
frame header or payload is a reported torn tail. Only
`repair_torn_tail` truncates that tail and synchronizes the file. Complete bad
magic, version, kind, flags, length, sequence, or checksum is corruption and is
never silently repaired.

## Atomic mutation batch v2

The 16-byte header is `RRDBAT02`, a `u16` version `2`, zero `u16` flags, and a
`u32` operation count. Each operation has a `u32` key length and a `u32` tagged
value length followed by key/value bytes. Bit 31 marks delete and requires the
remaining length bits to be zero; a clear bit is a put and permits an empty
value.

Empty batches or keys, mismatched magic/version pairs, unknown flags,
delete-with-length, trailing bytes, and out-of-contract lengths fail closed.
One MVCC sequence is allocated per operation while the complete payload stays
inside one atomic WAL frame.

The checked-in v1 bytes are a negative fixture. `RRDBAT01`/version 1 returns
`UnsupportedVersion` before any operation header is decoded; no v1 reader is
present. Writers and readers use only v2.

## Manifest, publication, and checkpoints

A manifest has a monotonically increasing generation, its parent digest after
generation 1, durable and WAL sequence boundaries, and every reachable segment
descriptor. Segment order is canonicalized before the manifest SHA-256 digest
is calculated. Each descriptor authenticates content identity, key and
sequence ranges, entry count, and byte count. L0 may overlap; higher levels
must contain ordered, non-overlapping key ranges.

Manifest v3 may carry one non-zero opaque `application_format`. `rrd-store`
binds the eight ASCII bytes `RRKV0001` as a big-endian `u64`. Every reachable
segment descriptor additionally authenticates segment format v4 plus the
current schema, key-codec, and page-format SHA-256 identities. Unknown, absent,
or different identities fail closed, and physical snapshot installation
requires an identical source and target identity. Cross-format movement is not
an alpha requirement; the final 1.0 tests create the accepted format directly.

Publication holds the operating-system writer lock, validates the expected
`CURRENT`, generation, and parent, writes and synchronizes immutable manifest
bytes, atomically renames the separately checksummed `CURRENT` pointer, and
synchronizes the directory. Named checkpoints are authenticated immutable
pointers that pin a manifest generation; rebinding a name fails, release is
explicit, and garbage collection uses the checkpoint inventory rather than
filename inference.

The manifest-v1 JSON is a negative fixture and returns `UnsupportedVersion`;
no v1/v2 manifest state reader is present. No superseded
application-key reader remains after C-01. These removals still do not qualify
the remaining C-06 evidence.

## Hybrid immutable segment v4

A v4 segment contains a fixed 256-byte `RRDSEG04` header, 64-byte-aligned page
payloads, a bounded `RRDIX004` row-group index, and a 64-byte lowercase ASCII
SHA-256 footer over every preceding physical byte. Segment and page integers
are little-endian because the page values are Arrow-compatible native buffers;
the `RRKV0001` ordered key grammar above remains byte-order-preserving and is
not re-encoded by this layer.

Flush emits rows in `(key ascending, sequence ascending)` order and never
splits the complete version chain for one key across row groups. The default
target is 64 KiB and 2,048 rows, but one key's chain may exceed either target
within the 1 MiB key and 8 MiB value bounds. Each row group owns exactly six
plain, currently uncompressed buffers:

| Page | Arrow-compatible representation | Point-read role |
|---|---|---|
| key offsets | `(rows + 1)` little-endian signed 64-bit offsets | key-spine binary search |
| key data | concatenated non-empty key bytes | key-spine comparison |
| sequences | `rows` little-endian unsigned 64-bit values | MVCC visibility |
| value validity | Arrow least-significant-bit-first validity bitmap | tombstone detection |
| value offsets | `(rows + 1)` little-endian signed 64-bit offsets | selected value range |
| value data | concatenated non-null value bytes | selected value payload |

Every 96-byte page descriptor pins page/column/buffer/logical/physical type,
plain encoding, no-compression metadata, row interval, null count, aligned
offset, physical and logical byte lengths, minimum/maximum statistics, and a
SHA-256 digest. The row-group descriptor pins its strict key range. Standalone
and newly written segment admission, plus snapshot-byte validation, validate
the outer digest, framing, schema/codec/page identities, every page digest,
Arrow offset/validity shape, key/sequence ordering, statistics, and segment
counts. Normal manifest-based reopen authenticates the complete file digest
and metadata against the manifest but deliberately installs conservative
filters instead of reading every data page during segment open. Runtime page
loads authenticate and validate each selected page, so tampering after open
fails closed. Higher-layer checkpoint reconciliation can still issue semantic
reads during `RrflowKvStore` startup; C-06 must measure and separate that work
from segment admission and query-time I/O.

Point lookup first selects a row group by authenticated key bounds and a
derived conservative Bloom filter, then reads only key offsets/data/sequences
and, for a live match, value validity/offsets/data. Range and compaction scans
read the required complete row groups. The Bloom filter is process-local
acceleration state rather than persistence truth: a negative can skip page I/O,
while a positive still executes exact key and MVCC comparison.

All segments in one database share a byte-bounded immutable page LRU. Its
evidence separates `loads`, `bytes_read`, `bytes_decoded`, `bytes_borrowed`,
`bytes_allocated`, `bytes_copied`, and `bytes_decompressed`, plus cache and
filter counters. In explicit mmap mode, an aligned uncompressed page is exposed
as an `arrow_buffer::Buffer` borrowing the mapping; its allocation owner holds
the `Arc<Mmap>` for the buffer lifetime. Bounded and io_uring paths allocate an
aligned Arrow buffer and read into it. Snapshot-byte validation copies into an
Arrow-owned buffer because the envelope does not promise alignment or a stable
mapped owner. Returned point values are still copied into owned `Vec<u8>`.
Therefore only the page-buffer load is currently borrow-eligible; end-to-end
zero-copy query output is not claimed.

Encoding/compression metadata is explicit but only `plain` plus `none` is
accepted today. Adaptive compression, key/value separation, persisted filters,
family grouping, selective provider projection, and cache admission are C-06
candidates that require comparative workload evidence before adoption. A
WiscKey-style value log is not assumed: it must beat stationary value pages on
RRFlow update, compaction, recovery, garbage-collection, scan, and mixed-family
corpora without weakening snapshot reachability.

Segment v1/v2/v3 inputs return `UnsupportedVersion` at every file-open and
snapshot-validation boundary. No reader or migration path for them remains.

## Flush, compaction, snapshots, and garbage collection

Flush order is: synchronize the active WAL; write, synchronize, and rename the
content-addressed segment; create and synchronize the successor WAL; persist
the next manifest; publish `CURRENT`. Before `CURRENT`, recovery sees the old
state. After it, the new segment and successor WAL must both exist.

Compaction selects bounded source-level input and overlapping target-level
segments, performs a forward k-way merge, and publishes all output partitions
with one manifest compare-and-swap. The current default output target is
8 MiB, split only at key boundaries. Explicit compaction retains versions
visible to protected sequences; cooperative automatic maintenance retains all
versions because the copyable low-level snapshot is not a lifetime-tracked
reclamation lease. Garbage collection deletes only objects unreachable from
`CURRENT` and named checkpoints after validating the complete root inventory.

A snapshot bundle begins with `RRDSNP01`, version `1`, zero flags, manifest
length, and segment count. It carries the authenticated manifest, each
descriptor and exact segment byte string, then a 64-byte lowercase ASCII
SHA-256 digest over the preceding envelope. Export first reaches a
flush-bounded manifest whose successor WAL is empty. Install validates the
complete closure before publishing a new local manifest; it never adopts the
source manifest history.

## Concrete fixture and failure examples

These are checked-in, executable examples rather than illustrative pseudocode:

1. [`batch-v2.hex`](../../../crates/persistence/rrd-lsm/fixtures/batch-v2.hex)
   encodes exactly three operations: put `alpha=one`, put `beta=two`, then
   delete `alpha`. The frozen-codec test decodes the bytes, rejects every
   truncation and a trailing byte, checks malformed delete length and version
   cases, and compares a fresh encoding byte-for-byte with the fixture.
2. `torn_tail_is_reported_and_only_explicit_repair_truncates_it` writes a valid
   prefix and torn tail, proves ordinary recovery does not mutate the file,
   repairs at the reported boundary, then reopens the accepted prefix.
3. `current_publication_is_ordered_content_addressed_and_compare_and_swap`
   proves stale publication cannot move `CURRENT` and accepted publication
   names the authenticated immutable manifest.
4. `v4_rejects_authenticated_length_flags_and_page_corruption` corrupts v4
   header/index/page fields and proves the reader fails closed.
5. `physical_snapshot_bundle_round_trips_installs_atomically_and_continues_writes`
   exports, installs, reopens, verifies the same state, and continues the
   sequence after the imported boundary.
6. `mmap_and_bounded_reads_are_identical_and_measure_page_ownership` proves
   identical results while mmap reports borrowed bytes and bounded/io_uring
   reports allocated bytes. It does not prove zero-copy DataFusion output.
7. `versions_remain_exact_when_one_key_exceeds_the_row_group_target` proves the
   writer does not split a key's MVCC chain at a row-group target.
8. `v4_bytes_match_the_checked_in_format_vector` freezes the complete current
   segment bytes independently of the snapshot envelope.

## Frozen vectors

- [`rrflow-kv-key-codec-v1.hex`](../../../crates/persistence/rrd-store/fixtures/rrflow-kv-key-codec-v1.hex)
- [`rrflow-kv-store-layout-v1.hex`](../../../crates/persistence/rrd-store/fixtures/rrflow-kv-store-layout-v1.hex)
- [`wal-v1.hex`](../../../crates/persistence/rrd-lsm/fixtures/wal-v1.hex)
- [`batch-v1.hex`](../../../crates/persistence/rrd-lsm/fixtures/batch-v1.hex) — rejection input only
- [`batch-v2.hex`](../../../crates/persistence/rrd-lsm/fixtures/batch-v2.hex)
- [`manifest-v1.json`](../../../crates/persistence/rrd-lsm/fixtures/manifest-v1.json) — rejection input only
- [`manifest-v3.json`](../../../crates/persistence/rrd-lsm/fixtures/manifest-v3.json)
- [`segment-v4.hex`](../../../crates/persistence/rrd-lsm/fixtures/segment-v4.hex)
- [`snapshot-bundle-v1.hex`](../../../crates/persistence/rrd-lsm/fixtures/snapshot-bundle-v1.hex)

Changing current pre-alpha bytes requires changing the explicit version and
updating the relevant vector through the test-controlled
`RRFLOW_UPDATE_GOLDENS` path. Readers never infer a version.

## Executable proof

Run focused contract evidence before the package suite:

```bash
cargo test -p rrd-store --test key_codec --locked
cargo test -p rrd-store --test native_index_commit --locked
cargo test -p rrd-store --lib rrflow_kv::tests::rrflow_kv_multi_family_transaction_recovers_all_or_none_at_every_wal_boundary --locked -- --exact
cargo test -p rrd-store --locked
cargo test -p rrd-lsm --test failure_matrix --locked
cargo test -p rrd-lsm --test mvcc batch_codec_is_canonical_strict_and_frozen -- --exact
cargo test -p rrd-lsm --test wal torn_tail_is_reported_and_only_explicit_repair_truncates_it -- --exact
cargo test -p rrd-lsm --test manifest current_publication_is_ordered_content_addressed_and_compare_and_swap -- --exact
cargo test -p rrd-lsm --test segment v4_bytes_match_the_checked_in_format_vector -- --exact
cargo test -p rrd-lsm --test segment v4_rejects_authenticated_length_flags_and_page_corruption -- --exact
cargo test -p rrd-lsm --test tiered_io mmap_and_bounded_reads_are_identical_and_measure_page_ownership -- --exact
cargo test -p rrd-lsm --test snapshot_bundle physical_snapshot_bundle_round_trips_installs_atomically_and_continues_writes -- --exact
cargo test -p rrd-lsm
```

The key-codec tests prove C-01's ordered application-key contract, frozen bytes,
strict malformed-key rejection, real-store persistence, and close/reopen
readback. The native-index and complete semantic-fault tests contribute C-03's
insert, replacement, retirement, unique-conflict, catalogue-drift rejection,
source-delta, rrflowMX/rrflowKV differential, exact-key failure-boundary, and
rrflowKV-reopen evidence. C-03 is accepted by the canonical roadmap; that does
not qualify Gate E's bounded read paths/materializers or Gate C-06's physical
format. The remaining lower-level tests prove only the present object format.
C-05's lower source, dependency, opener, physical-reader, and upper-shape
closure evidence is recorded in its execution journals; C-05d records the
audit correction that forced C-05e through C-05h before the gate could close.
The optional post-alpha cluster implementation remains unqualified under
POAM-023. The v4 segment vector, direct MVCC comparisons, and ownership
counters are now concrete C-06 evidence. C-06 remains open for property/fuzz
differential coverage, a selective projected-page interface and counters,
mixed-family interference, configurable row-group budgets, persisted-filter
or open-cost resolution, and fixed-hardware compression/value-placement/cache
comparisons. Gate F separately requires a stamped streamed DataFusion provider
with projection/predicate/budget evidence. Passing this suite cannot close
those remaining gates by itself.
