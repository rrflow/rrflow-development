# C-06i rrflowKV persisted row-group filter integration

**Status:** active implementation and verification plan
**Coordinate:** `rrflow://rrflow-instance/data/work-package/c-06i-rrflowkv-persisted-filter-integration`
**Owner:** segment-v5 persisted-filter bytes, integration proofs, and C-06i handoff
**Machine plan:** [`rrflow-1.0-active-change.json`](rrflow-1.0-active-change.json)
**Baseline:** `10c4ac2459d1862982bd94bdbc0ca6644d187928`
**Planning commit:** `a623c9838e1b75357cb96a3939aa2a4f02569e29`

## Outcome

This package integrates exactly one policy retained by the completed C-06i
candidate screen: an authenticated, persisted row-group membership filter.
After a normal manifest reopen, rrflowKV must reject definite in-range point
misses before loading any semantic Arrow-layout page. The filter is never a
semantic index and never establishes presence: positives still execute the
exact sorted key/version lookup.

C-06 remains open. Compression, cache admission, value placement, DataFusion,
native recall operators, the installed product, and alpha release proof are not
part of this package.

## Baseline failure

The fixed candidate corpus recorded the real segment-v4 path performing:

- 16,234 row-group filter checks;
- zero filter negatives;
- 426 semantic page loads.

That is not a useful persisted filter. `Database::open` installs `allow_all`
because v4 has no filter bytes. `Segment::open` can construct a process-local
filter only by reading every key page, which defeats metadata-only reopen.

The retained candidate used 10 bits per unique key, at least 64 bits, and seven
double-hash probes. It occupied 13,304 bytes on the fixed corpus, had zero
member false negatives, and measured a 0.634 percent absent-key false-positive
rate. Those numbers select this implementation; they are not a general
performance claim.

## Single physical format

This repository is pre-release. Segment v5 directly replaces v4:

- segment magic: `RRDSEG05`;
- row-group index magic: `RRDIX005`;
- segment format version: `5`;
- product version remains `1.0.0`;
- manifest format remains `3`;
- snapshot bundle format remains `1`;
- WAL, batch, and application-key formats do not change;
- segment versions 1 through 4 are rejection inputs only;
- there is no v4 reader, migration, fallback, alias, or positive fixture.

`SEGMENT_PAGE_FORMAT_DIGEST` changes because the authenticated row-group index
now contains membership data. The semantic schema and key-codec digests remain
unchanged.

## V5 index encoding

All integer fields are little-endian. The complete index remains covered by the
segment SHA-256 footer.

### Index header: 32 bytes

| Offset | Width | Field | Required value |
|---:|---:|---|---|
| 0 | 8 | magic | `RRDIX005` |
| 8 | 4 | row-group count | header count |
| 12 | 2 | page count | `6` |
| 14 | 2 | row-group header bytes | `32` |
| 16 | 2 | page descriptor bytes | `96` |
| 18 | 1 | filter format | `1` (RRFlow Bloom v1) |
| 19 | 1 | bits per unique key | `10` |
| 20 | 1 | hash functions | `7` |
| 21 | 11 | reserved | all zero |

### Row-group header: 32 bytes

| Offset | Width | Field | Rule |
|---:|---:|---|---|
| 0 | 8 | row start | contiguous across groups |
| 8 | 4 | row count | nonzero |
| 12 | 4 | first-key bytes | `1..=MAX_KEY_BYTES` |
| 16 | 4 | last-key bytes | `1..=MAX_KEY_BYTES` |
| 20 | 2 | page count | `6` |
| 22 | 2 | reserved | zero |
| 24 | 4 | unique-key count | `1..=row_count` |
| 28 | 4 | filter word count | exact derived count |

The first key, last key, and six existing 96-byte page descriptors follow the
header. The filter words then follow as contiguous little-endian `u64` values.
No second length prefix or policy-specific allocation is accepted.

The canonical word count is:

```text
bit_count  = max(unique_key_count * 10, 64)
word_count = ceil(bit_count / 64)
```

Every multiplication, conversion, cursor advance, and index-size calculation
is checked before allocation or slicing. `MAX_INDEX_BYTES` and the segment
limit remain hard ceilings.

## Canonical filter algorithm

There is one implementation in `segment/format.rs`; the reader, writer, tests,
and physical-policy laboratory use it.

1. Hash the complete key twice with the retained seeded FNV-1a-style stream and
   Murmur final avalanche (`0xcbf29ce484222325` and
   `0x9e3779b97f4a7c15`).
2. Force the second 64-bit hash odd.
3. For probe `i` in `0..7`, set or test
   `(h1 + i * h2) mod bit_count`, using wrapping `u64` arithmetic, then convert
   the already bounded bit position to `usize`. Persisted words must not depend
   on pointer width.

Only unique keys enter the filter even when the row group contains multiple
MVCC versions. The writer already prevents a key's version chain from crossing
a row-group boundary.

## Write, open, and read flow

```text
Memtable ordered versions
  -> row-group builder records each unique key once
  -> six Arrow-layout pages + canonical Bloom words
  -> authenticated v5 index/footer
  -> full page decode reconstructs and compares exact filter
  -> fsync segment
  -> manifest publication

Database::open
  -> authenticate manifest descriptor + complete segment checksum
  -> parse bounded v5 header/index/filter metadata
  -> zero semantic page reads
  -> install immutable Segment generation

point get
  -> row-group key bounds
  -> persisted filter check
     -> definite negative: return no version, load no page
     -> maybe present: exact key spine and MVCC lookup
```

Range and projected reads continue to use exact bounds and MVCC merge. They do
not use a probabilistic filter to skip row groups.

## Corruption and canonicality

The outer checksum is necessary but not sufficient. Tests rewrite the outer
checksum after mutating filter metadata or words. V5 must still reject:

- unknown filter format, bits-per-key, or hash count;
- nonzero reserved bytes;
- zero or greater-than-row-count unique-key count;
- a filter word count that differs from the derived count;
- truncated or trailing index bytes;
- words that do not equal a filter reconstructed from decoded unique keys;
- any inserted member that tests negative.

Writer publication and snapshot validation use the same deep validation, so a
noncanonical but checksummed segment cannot become authoritative.

## Evidence and tracing

`SegmentOpenEvidence` advances to schema version 2 and adds:

- `persisted_filter_count`;
- `persisted_filter_bytes`.

The existing `rrd_lsm.open` and `rrflow_kv.open` events expose those two
low-cardinality integer fields. Existing per-operation counters remain the
authority for filter checks/negatives and page requests/loads. No key, value,
prompt, secret, or project-derived label is emitted.

The new fixed-machine artifact is
`docs/evidence/c06i-rrflowkv-persisted-filter-linux-x86_64.json`. It is produced
from the clean implementation commit, preserves raw child trials, and compares
the integrated normal-reopen observation with the historical candidate
baseline. The executable fails rather than publishing a successful trial when
filter count/bytes drift, open reads semantic pages, a member is missed, the
declared false-positive ceiling fails, or miss-path page loads are not below
filter checks. Other physical policies remain explicitly unintegrated.

## File and proof map

| Boundary | Change | Required proof |
|---|---|---|
| `segment/format.rs` | v5 bytes, canonical filter, bounded parser, deep reconstruction | golden and checksum-rewritten corruption tests |
| `segment/mod.rs` | persisted descriptor filter and evidence; no allow-all path | reopened miss has negatives and no semantic page load |
| physical-policy lab/example | reuse production filter; separate evidence artifact | zero member false negatives and stable corpus identity |
| `rrd-store` | propagate open evidence into existing trace | store-level open/query phase separation |
| segment/hybrid tests | exact MVCC and malformed-byte coverage on v5 | full `rrd-lsm` suite |
| fuzz target/corpus | direct v4-to-v5 rename | bounded sanitizer run with tracked seeds |
| frozen fixtures | v5 segment and affected manifest/snapshot closure | deterministic golden checks and import/reopen tests |
| current documentation | truthful v5 status and remaining gaps | documentation and inventory policies |

## Stop conditions

Stop rather than widen the batch if implementation requires a compatibility
reader, new dependency, separate index file, unbounded input-derived
allocation, semantic-filter authority, range exclusion, format changes outside
the declared segment identity, or an undeclared production repair. Stop on any
false negative, malformed-filter acceptance, MVCC/recovery/snapshot/compaction
regression, dirty evidence, or unsupported alpha/performance claim.
