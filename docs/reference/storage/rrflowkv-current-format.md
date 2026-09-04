# rrflowKV current physical format

**Status:** active implementation reference for the pre-alpha row-segment format; not the accepted RRFlow 1.0 target
**Coordinate:** `rrflow://rrflow-instance/data/reference/storage/rrflowkv-current-format`
**Owner:** physical bytes, limits, recovery rules, and compatibility debt implemented by `rrd-lsm`

This record describes the bytes the current checkout can create or read. It is
not the [accepted rrflowKV architecture](../../architecture/engine-data-flow.md),
does not own release status, and makes no performance or competitive claim.
The [roadmap](../../roadmap/rrflow-1.0.md) owns the replacement and removal
work; [POAM-002 and POAM-003](../../poam/rrflow-1.0-alpha.md) keep the physical
layout and legacy-path deficiencies open.

## Current and target boundary

The current immutable format is a checksummed, block-indexed, LZ4-compressed
row-record segment. It is not an Arrow-native segment and cannot be described
as zero-copy input to DataFusion. Current query adapters may materialize rows
and allocate Arrow arrays.

The accepted target is one hybrid rrflowKV persistence authority with a
write-optimized WAL and MVCC memtable plus immutable ordered key/version state
and Arrow-compatible column pages pinned by the same manifest. That target is
specified only by the engine data-flow record and is delivered by roadmap
Gate C-06. This reference must be replaced or moved to history when C-06
changes the physical format.

The checkout also contains executable readers for manifest v1, mutation batch
v1, segment v1/v2, and an existing-directory Fjall compatibility path. Those
are observed migration debt, not supported alpha architecture. Gate C-05 must
remove them directly before the alpha baseline; this document does not
normalize them as permanent compatibility requirements.

## Implemented object set

All integers in binary objects are unsigned big-endian unless stated
otherwise.

| Object | New-write format | Current role | Implementation |
|---|---:|---|---|
| WAL | 1 | Ordered atomic-batch frames and durability boundary | [`wal.rs`](../../../crates/persistence/rrd-lsm/src/wal.rs) |
| Mutation batch | 2 | Canonical put/delete payload inside one WAL frame | [`batch.rs`](../../../crates/persistence/rrd-lsm/src/batch.rs) |
| Manifest | 2 | Immutable reachable-state inventory and sequence boundary | [`manifest.rs`](../../../crates/persistence/rrd-lsm/src/manifest.rs) |
| `CURRENT` and checkpoint controls | 1 | Authenticated manifest and retention pointers | [`manifest.rs`](../../../crates/persistence/rrd-lsm/src/manifest.rs) |
| Immutable segment | 3 | LZ4-compressed row-record blocks plus block index | [`segment.rs`](../../../crates/persistence/rrd-lsm/src/segment.rs) |
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
| Row-block target / decoded block cache | 4 KiB / 4 MiB | segment writer and database default |
| Snapshot bundle bytes / segments | 1 GiB / 1,000,000 | snapshot-bundle validator |

Before a new batch crosses either mutable threshold, the single writer runs
the ordinary crash-ordered flush path. It never splits one accepted atomic
batch. An individually oversized batch remains one frame and is reported in
maintenance evidence.

## WAL and acknowledgment

One accepted mutation batch is one WAL frame. `Authoritative` acknowledgment
follows `sync_data`; `Buffered` acknowledges a written frame without claiming
durability. A failed write or synchronization poisons that writer instance, so
the caller must reopen and recover rather than append after an unknown partial
write.

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

The v1 reader is isolated legacy debt. It recognizes `RRDBAT01`, a one-byte
operation kind, three zero operation-flag bytes, and separate `u32` key/value
lengths. Writers emit only v2.

## Manifest, publication, and checkpoints

A manifest has a monotonically increasing generation, its parent digest after
generation 1, durable and WAL sequence boundaries, and every reachable segment
descriptor. Segment order is canonicalized before the manifest SHA-256 digest
is calculated. Each descriptor authenticates content identity, key and
sequence ranges, entry count, and byte count. L0 may overlap; higher levels
must contain ordered, non-overlapping key ranges.

Manifest v2 may carry one non-zero opaque `application_format`. `rrd-store`
currently binds `RRDSK002` and uses one stable non-zero tag for each logical
keyspace. Unknown application identities fail closed, and physical snapshot
installation requires an identical source and target identity. Cross-format
movement must be an explicit logical migration.

Publication holds the operating-system writer lock, validates the expected
`CURRENT`, generation, and parent, writes and synchronizes immutable manifest
bytes, atomically renames the separately checksummed `CURRENT` pointer, and
synchronizes the directory. Named checkpoints are authenticated immutable
pointers that pin a manifest generation; rebinding a name fails, release is
explicit, and garbage collection uses the checkpoint inventory rather than
filename inference.

The manifest-v1 and textual-key readers are current removal debt. They do not
define the 1.0 format.

## Immutable row segment v3

A v3 segment contains a fixed 64-byte `RRDSEG03` header, independently
compressed LZ4 row-record blocks, a bounded `RRDIX003` index, and a 64-byte
lowercase ASCII SHA-256 footer over all preceding physical bytes. Records are
not split between blocks; one record may exceed the 4 KiB target within the
1 MiB key and 8 MiB value bounds.

Each index entry records physical offset and length, decoded length, entry
count, last key, and the SHA-256 digest of its compressed block. Offsets must
cover the data region exactly and last keys must be non-decreasing. Open
validates the outer digest and blocks; runtime reads recheck the selected
compressed-block digest before decoding.

The implementation derives a Bloom filter from authenticated block contents.
It is acceleration state, not persistence truth: a negative may skip a block,
while a positive still executes exact MVCC comparison. All segments in one
database share a bounded decoded-block LRU. Blocks larger than its configured
capacity may be decoded for a caller but are not retained.

The v1 uncompressed and v2 single-compressed-block readers are explicit legacy
branches scheduled for removal by C-05. They are not a reason to preserve row
segments in the C-06 target.

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
4. `v3_rejects_authenticated_length_flags_and_block_corruption` corrupts v3
   header/index/block fields and proves the reader fails closed.
5. `physical_snapshot_bundle_round_trips_installs_atomically_and_continues_writes`
   exports, installs, reopens, verifies the same state, and continues the
   sequence after the imported boundary.
6. `mmap_and_bounded_reads_are_identical_and_measured_separately_from_cache`
   compares current row-segment I/O paths. It does not prove Arrow buffer
   borrowing or zero-copy DataFusion execution.

## Frozen vectors

- [`wal-v1.hex`](../../../crates/persistence/rrd-lsm/fixtures/wal-v1.hex)
- [`batch-v1.hex`](../../../crates/persistence/rrd-lsm/fixtures/batch-v1.hex)
- [`batch-v2.hex`](../../../crates/persistence/rrd-lsm/fixtures/batch-v2.hex)
- [`manifest-v1.json`](../../../crates/persistence/rrd-lsm/fixtures/manifest-v1.json)
- [`manifest-v2.json`](../../../crates/persistence/rrd-lsm/fixtures/manifest-v2.json)
- [`snapshot-bundle-v1.hex`](../../../crates/persistence/rrd-lsm/fixtures/snapshot-bundle-v1.hex)

Changing current pre-alpha bytes requires changing the explicit version and
updating the relevant vector through the test-controlled
`RRFLOW_UPDATE_GOLDENS` path. Readers never infer a version.

## Executable proof

Run focused contract evidence before the package suite:

```bash
cargo test -p rrd-lsm --test mvcc batch_codec_is_canonical_strict_and_frozen -- --exact
cargo test -p rrd-lsm --test wal torn_tail_is_reported_and_only_explicit_repair_truncates_it -- --exact
cargo test -p rrd-lsm --test manifest current_publication_is_ordered_content_addressed_and_compare_and_swap -- --exact
cargo test -p rrd-lsm --test segment v3_rejects_authenticated_length_flags_and_block_corruption -- --exact
cargo test -p rrd-lsm --test snapshot_bundle physical_snapshot_bundle_round_trips_installs_atomically_and_continues_writes -- --exact
cargo test -p rrd-lsm
```

Those tests prove the present physical contract only. C-05 requires removal
evidence for legacy paths. C-06 requires new vectors, property and crash tests,
and fixed-hardware comparison for the hybrid Arrow-compatible target. Gate F
requires streamed projection/predicate/budget counters through DataFusion.
Passing this suite cannot close any of those gates by itself.
