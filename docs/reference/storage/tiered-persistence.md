# RRFlow tiered persistence

**Status:** active implementation reference; local I/O policies and portable immutable bytes exist, automated hot-to-cold tiering does not
**Coordinate:** `rrflow://rrflow-instance/data/reference/storage/tiered-persistence`
**Owner:** persistence-tier taxonomy, implemented movement primitives, and hibernation boundary

Tiering may change where immutable bytes reside and how they are read. It may
not create another logical database, transaction coordinator, or recall
authority. Canonical RRFlow mutations remain owned by `RrdEngine` and committed
through rrflowKV; rrflowQL/DataFusion consumes stamped data and cannot write
around that boundary.

This record distinguishes implemented local storage mechanics from the
unfinished DevForge placement and hibernation system.

## Canonical tier taxonomy

| Layer | Canonical role | Current implementation |
|---|---|---|
| rrflowKV mutable state | Low-latency WAL-backed MVCC writes and the active memtable | Implemented locally; every acknowledged authoritative batch is synchronized before the mutable state is exposed |
| rrflowKV immutable state | Manifest-addressed sorted segments, compaction input, and retained snapshot state | Implemented locally as segment v5 ordered key/version spines, aligned Arrow-layout pages, and authenticated persisted row-group filters; DataFusion provider integration and the remaining C-06 physical-policy evidence remain open |
| rrflowKV immutable page cache | Process-local reuse of authenticated immutable pages | Implemented as a byte-bounded shared LRU with separate hit, miss, load, eviction, residency, read, decode, borrow, allocation, copy, and decompression counters |
| Immutable application objects | Content-addressed source artifacts and multimodal payload bytes referenced by canonical records | Memory and local adapters implemented; provider-neutral S3 port implemented without a production transport |
| Vector artifact residency | Process-local pinned, cached, or cold opening of immutable vector artifacts | Implemented separately under the [vector residency contract](../vector/memory-tiers.md); never canonical state |
| Physical snapshot transfer | Authenticated manifest closure used to move or restore one rrflowKV image | Local file/bundle export and installation implemented; no hibernation coordinator or configured cold destination |
| DevForge filesystem placement | Shared immutable tool/model lower layer plus per-estate writable workspace and rrflowKV upper layer | Roadmap target only; it is not implemented by rrflowKV segment I/O |

The process-local cache is intentionally custom and specific to authenticated
segment pages. Moka is not a dependency of RRFlow, Arrow, or DataFusion and is
not needed to describe this storage contract. Any later cache substitution
requires measured eviction, admission, memory, and stale-generation evidence;
it cannot change logical results.

## Implemented local immutable-segment I/O

`DatabaseOptions::segment_io` selects `auto`, `mmap`, `io_uring`, or `bounded`
for local immutable segment pages:

- `auto` probes Linux io_uring setup and `IORING_OP_READ`, then uses bounded
  positional reads when the runtime cannot admit the ring;
- explicit io_uring can fail database open when fallback is disabled;
- a runtime unsupported or permission error disables the ring and records the
  bounded fallback when policy permits it;
- explicit mmap maps the local segment and falls back to bounded reads only
  when configured; and
- every non-mmap request is capped by `max_request_bytes`.

All modes authenticate the same content-addressed segment and per-page digests,
apply the same MVCC visibility rules, and load through the same shared page
cache. `SegmentIoStats` reports the requested/selected route,
fallback reason/count, per-route read operations, bytes, and peak request.
`PageCacheStats` separately reports capacity, resident bytes, loads, read,
decoded, borrowed, allocated, copied, and decompressed bytes, hits, misses,
evictions, and filter outcomes.

These are local access modes, not persistence tiers. With explicit mmap, an
aligned uncompressed v5 page can back an `arrow_buffer::Buffer` while an owned
mapping lease preserves its lifetime. Bounded and io_uring paths allocate an
aligned buffer; snapshot-envelope validation copies. Point values and current
query results can still allocate. The [current-format
record](rrflowkv-current-format.md) owns the exact eligibility boundary and
remaining C-06 work; no end-to-end zero-copy DataFusion claim exists yet.

## Implemented portable immutable bytes

The immutable-object boundary and S3 port are defined by the
[multi-model and object contract](../data/multi-model-object-contract.md).
They provide bounded content-addressed streaming, verification, resumable
multipart semantics, ranged reads, and explicit orphan inventory. They do not
move rrflowKV WALs, manifests, or segments automatically, and the repository
contains no production S3 HTTP client.

rrflowKV can flush and export an authenticated snapshot closure to a new local
file, then validate and install that file into a compatible database. Export
and install have explicit crash boundaries; a new manifest publication makes
the imported state visible. This is a transfer primitive, not hibernation: it
does not quiesce engine requests, bind estate identity and configuration,
upload to a configured cold tier, reclaim hot state, or resume the same estate.

## Missing hot-to-cold system

RRFlow currently has no component that:

- classifies rrflowKV pages or segments by measured temperature;
- evicts a manifest-reachable segment from local storage and resolves it from
  a remote content-addressed tier on demand;
- prefetches or promotes a cold page under a bounded policy;
- coordinates quiesce, snapshot, upload, verification, local reclamation,
  restore, and resume through `RrdEngine`;
- proves retention pins keep every required manifest, segment, Arrow buffer,
  and referenced object alive; or
- reports hot, cold, cached, transferred, compressed, logical, and allocated
  bytes as independent quantities across a full estate lifecycle.

DataFusion must not fill that gap by opening arbitrary files or object URLs.
Gate C-06 first establishes manifest-pinned Arrow-compatible pages and safe
buffer lifetimes. Gate C-07 proves compaction and recovery cannot invalidate a
reader. Only then may Gate D-10 move an authenticated snapshot closure through
a configured cold-tier adapter while `RrdEngine` owns quiesce and resume.

The DevForge CoW lower/upper filesystem contract is separately owned by Gates
D-07 through D-09. Shared toolchains, dependency mounts, and model inputs may
live below an estate, but every mutable rrflowKV WAL, manifest, segment,
catalogue, graph, and reasoning-state byte belongs to that estate's writable
upper layer.

## Current executable evidence

`crates/persistence/rrd-lsm/tests/tiered_io.rs` proves exact read equality
between mmap and bounded modes, distinct borrowed-versus-allocated page
evidence, byte/request/cache accounting, and actual io_uring or an explicit
measured fallback on the executing kernel. Segment,
snapshot, failure-matrix, and snapshot-memory suites prove current local
authentication, bounded transfer, crash ordering, and install behavior. The
S3-port tests use an in-memory conformance client.

This evidence qualifies the implemented primitives only. It does not close
C-06, C-07, D-07, D-08, D-09, or D-10, and it is not a claim that RRFlow
currently supports transparent remote segments or hot-to-cold hibernation.
