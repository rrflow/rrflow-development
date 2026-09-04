# RRFlow engine data flow

**Status:** active accepted target architecture; implementation gaps remain open
**Coordinate:** `rrflow://rrflow-instance/data/architecture/engine-data-flow`
**Owner:** detailed transactional, storage, query, and context flow

The repository root [README](../../README.md) owns RRFlow's identity and
non-negotiable invariants. The [alpha objective](../objectives/rrflow-1.0-alpha.md)
defines the result, the [roadmap](../roadmap/rrflow-1.0.md) orders delivery, and
the [POA&M](../poam/rrflow-1.0-alpha.md) tracks observed gaps. This record owns
the detailed end-to-end flow only.

## System boundary

RRFlow is one hybrid transactional and analytical reasoning-data engine. The
names describe cooperating layers, not independent products or stores:

| Layer | Responsibility |
|---|---|
| RRFlow database | One persistent AI estate containing governed knowledge, temporal graph state, reasoning state, evidence, and indexes. |
| `RrdEngine` | Sole semantic transaction coordinator: authenticates, authorizes, binds read stamps, validates mutations and model proposals, and invokes the selected storage profile. |
| rrflowKV | Durable physical profile: WAL, sequence allocation, MVCC snapshots, mutable memtable, immutable segments, manifests, flush, compaction, and recovery. |
| rrflowMX | Non-durable physical profile implementing the same semantic storage port for volatile and conformance execution. |
| RRFlowQL | Query language and planning boundary selecting native fast operators or the analytical path. |
| Arrow substrate | Shared columnar buffer model used by eligible immutable segment pages and streamed analytical batches. |
| DataFusion | Bounded vectorized execution over stamped Arrow batches; it does not authorize or commit RRFlow state. |
| Native indexes | Scalar/unique, graph adjacency, BM25, exact-vector, HNSW, and related access paths bound to canonical source coordinates. |
| LFG or provider model | Replaceable constrained adapter that proposes recipes, branches, or bounded query intent. |
| HTTP, WebSocket, SDK, MCP, CLI, Connectome | Transport and client surfaces invoking public `RrdEngine` capabilities without recreating engine logic. |

## Write and commit flow

```text
HTTP / WebSocket / SDK / MCP / embedded caller
                         |
                  typed public request
                         |
                         v
                     RrdEngine
       parse -> authenticate -> authorize -> bind schema/policy
                         |
              optional analytical transform
                         |
            RRFlowQL/DataFusion returns batches
              or model returns a proposal
                         |
                         v
                     RrdEngine
       validate -> lower semantic mutation -> authorize effects
                         |
                         v
                      rrflowKV
       WAL frame -> sequence/MVCC memtable -> commit receipt
                         |
              flush and compaction builders
                         |
                         v
       immutable segment + index roots + manifest publication
                         |
                         v
             changefeed / live delta / trace
```

`RrdEngine` owns the semantic transaction and authorization decision. rrflowKV
owns WAL concurrency, sequence allocation, physical snapshot isolation, flush,
compaction, and durable publication. `RrdEngine` must not hold a global WAL
lock or expose physical keys to callers.

RRFlowQL parses and binds expressions. DataFusion may scan, filter, join,
aggregate, rank, or produce transformed Arrow batches. A DataFusion `DataSink`
or another compute adapter returns a proposed result to `RrdEngine`; it does
not write rrflowKV files, publish manifests, or bypass semantic validation.

One accepted semantic transaction must publish its canonical record changes,
temporal versions, both graph adjacency directions, synchronous index changes,
runtime-log entry, and durable asynchronous index deltas as one rrflowKV write
batch. Index builders may run later, but their source cursor and freshness
state are part of the committed transaction.

## rrflowKV physical target

rrflowKV combines write-optimized state with scan-optimized immutable storage
inside one durability authority:

```text
recent writes                         published immutable state
-------------                         -------------------------
checksummed WAL                       manifest generation
mutable MVCC memtable       flush     ordered key/version spine
point/range/CAS access       ----->    Arrow-compatible column pages
                                      adjacency/posting/index roots
                                      encoding and checksum metadata
```

The ordered key/version spine preserves exact point, prefix, range, temporal,
and conflict semantics. Arrow-compatible pages allow selected immutable
columns to enter `RecordBatch` streams without reconstructing generic Rust row
objects. The same manifest pins both forms so they cannot represent different
commits.

Column pages may contain canonical record values. Scalar, lexical, graph, and
approximate-vector indexes remain derived acceleration state and cannot
outrank their source keys, schema, catalogue, or read cursor. Exact vector
values remain canonical; HNSW and quantized structures remain candidate
indexes followed by exact reranking when the query contract requires it.

The mutable memtable does not need to mimic a columnar file. It is optimized
for high-frequency writes, current state, CAS, and bounded range reads. A
stamped analytical scan merges the visible memtable delta with immutable
segment pages before returning a batch.

The checkout has not reached that layout. Its
[current physical-format reference](../reference/storage/rrflowkv-current-format.md)
records the implemented v3 LZ4 row-block segments, frozen bytes, recovery
rules, and removable legacy readers without promoting them into target
architecture.

## Conditional zero-copy

Memory mapping alone is not a universal zero-copy guarantee. A segment page
may be borrowed directly by Arrow only when its physical type, alignment,
endianness, compression state, validity representation, deletion state, and
requested projection permit it. The snapshot must retain the segment mapping
for the complete lifetime of every emitted batch.

Dictionary materialization, decompression, deletion filtering, type coercion,
memtable merging, computed expressions, and incompatible alignment may require
pool-owned buffers. Every physical plan therefore reports:

- mapped and physically read bytes;
- borrowed Arrow buffer bytes;
- decompressed and decoded bytes;
- copied and newly allocated bytes;
- rows/pages eliminated by key, projection, predicate, and limit pushdown; and
- peak memory, spill bytes, elapsed time, and output bytes.

The release claim is measured conditional zero-copy for eligible pages, never
"zero parsing, zero allocation" for every query. Apache Arrow likewise notes
that IPC reads can allocate when compression or other conditions require it.

## Read and query flow

```text
authorized intent + anchors + explicit budgets
                         |
                  capture one ReadStamp
 runtime cursor + manifest + valid time + schema/catalogue/policy
                         |
                    RRFlowQL planner
              +----------+-----------+
              |                      |
              v                      v
       native fast path       analytical path
   point/range/CAS/tree       native candidate operators
   adjacency prefix scans     graph + BM25 + HNSW/exact
              |                      |
              |          rrflowKV stamped page/memtable scan
              |                      |
              |             Arrow RecordBatch stream
              |                      |
              |           DataFusion bounded execution
              +-----------+----------+
                          |
                deterministic exact rerank/RRF
                          |
             ContextPacket + evidence + same stamp
```

Fast state-machine navigation and bounded graph pointer work do not invoke
DataFusion. Analytical scans use a custom provider whose planning methods
declare supported pushdown without performing storage I/O; its execution stream
performs bounded reads when polled. Pipeline-breaking operators must reserve
memory or spill within the request budget.

Graph traversal uses versioned outgoing/incoming adjacency prefix ranges.
BM25 uses incrementally maintained dictionary, document statistics, postings,
positions, and tombstones. Vector search uses payload/scalar filters to narrow
eligible candidates, HNSW or exact candidate generation according to the cost
plan, and exact reranking. Reciprocal-rank fusion is a pure query-time operator;
verified outcomes may update a versioned policy only for later read stamps.

The local routing model does not read raw Arrow or storage buffers as model
input. `RrdEngine` constructs a bounded, typed route packet or an explicit
Arrow-to-tensor feature bridge. The model returns a grammar-constrained
proposal; deterministic predicates, authorization, CAS, and persistence remain
engine operations.

## Current implementation boundary

| Concern | Present checkout | Required target |
|---|---|---|
| rrflowKV writes | Checksummed WAL frames and a mutable MVCC memtable. | Retain and prove under the final hybrid format. |
| rrflowKV immutable storage | LZ4-compressed row-record blocks with block indexes, manifests, compaction, recovery, and legacy readers. | Ordered key/version spine plus Arrow-compatible column pages; remove compatibility readers before alpha exit. |
| Arrow conversion | Materialized `QueryRow` values are converted into newly allocated typed Arrow arrays. | Stream eligible segment buffers and bounded decoded/memtable overlays through a stamped provider. |
| DataFusion | Real bounded execution over the materialized Arrow snapshot. | Push projection/predicate/limit into rrflowKV and compose native graph/BM25/vector operators at one stamp. |
| Graph/BM25/vector | Useful semantic and projection foundations exist at different completion levels. | Transactionally maintained native access paths with exact fallback, reopen, corruption, and quality proof. |
| Models | Reasoning-tree and provider-neutral router contracts exist. | Persisted tree execution, model-manifest handshake, constrained adapter dispatch, and conformance. |

These differences are tracked by POAM-002 through POAM-005 and roadmap Gates
C, E, and F. No present type, file format, or passing compile closes them.

## External systems

PostgreSQL, Turso, SQLite, Dragonfly, object stores, mesh networks, model
providers, and application databases are optional governed integrations. An
operator explicitly configures an adapter; attunement may discover that an
integration is possible but cannot activate it. External systems do not become
RRFlow's canonical persistence, transaction coordinator, query planner, or
context authority.

## Source constraints

The architecture uses these upstream behaviors without inheriting another
system's authority:

- [Apache Arrow columnar format](https://arrow.apache.org/docs/format/Columnar.html)
  defines relocatable columnar buffers and the mutation tradeoff.
- [Apache Arrow IPC](https://arrow.apache.org/docs/cpp/api/ipc.html) documents
  both zero-copy-capable reads and cases requiring allocation.
- [DataFusion custom providers](https://datafusion.apache.org/library-user-guide/custom-table-providers.html)
  separate planning-time pushdown declarations from execution-time streams.
- [Lance read and write behavior](https://lancedb.github.io/lance/introduction/read_and_write.html)
  demonstrates immutable columnar fragments, deletion metadata, and compaction
  tradeoffs; rrflowKV is not a Lance clone.
- [Qdrant storage](https://qdrant.tech/documentation/manage-data/storage/) and
  [indexing](https://qdrant.tech/documentation/manage-data/indexing/) inform
  WAL/segment recovery, filter-aware candidates, and persisted index behavior.

## Qualification

The architecture is implemented only when the owning roadmap gates prove:

1. exact transaction, snapshot, key ordering, and rrflowMX/rrflowKV
   conformance;
2. crash recovery and compaction while read stamps and mapped buffers remain
   live;
3. atomic record, graph, scalar, BM25, vector, runtime-log, and index-delta
   maintenance;
4. streaming and pushdown against data larger than the allowed query memory;
5. exact-oracle and recall-quality results after update, delete, rebuild, and
   reopen; and
6. fixed-hardware latency, memory, copy/decode, disk, and failure evidence.
