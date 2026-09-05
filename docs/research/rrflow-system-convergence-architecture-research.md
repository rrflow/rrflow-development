# RRFlow 1.0 system-convergence architecture research

**Status:** active supporting research; informs but does not override accepted architecture or roadmap status
**Coordinate:** `rrflow://rrflow-instance/data/research/rrflow-system-convergence`
**Owner:** primary-source evidence for the RRFlow 1.0 execution map
**Audience:** RRFlow owner and engineers executing the 1.0 pre-release gates
**Date:** 2026-09-05
**Scope:** the current RRFlow repository, the path from rrflowMX through rrflowKV and Arrow/DataFusion, native graph/lexical/vector access, installation/attunement, provider-neutral agent context, and external project data adapters
**Assumptions:** one RRFlow instance per project/environment; `RrdEngine` is the sole semantic, authorization, and mutation authority; version remains `1.0.0`; current code is inventory until the roadmap's behavioral evidence passes

## Direct answer

The repository is not a start-over. It contains substantial WAL, MVCC,
manifest, snapshot, query, DataFusion, BM25, HNSW, quantization, transport,
security, and runtime code. It is also not a true alpha yet. The central gaps
are integration and physical semantics: the native store still contains Fjall
compatibility and legacy readers; semantic transactions do not yet update
both graph directions and every synchronous index family atomically; normal
reads reconstruct some state from the runtime log; query execution eagerly
materializes `Vec<QueryRow>` before Arrow; DataFusion's provider wraps that
materialization instead of streaming rrflowKV pages; live query evaluation
reruns two snapshots; and installation/attunement has contracts but no
persisted executor.

The correct convergence target is a hybrid storage engine, not a generic
row-only LSM and not “Arrow everywhere”:

1. `RrdEngine` authenticates, authorizes, binds one `ReadStamp`, validates
   semantic invariants, owns compare-and-swap, and is the only component that
   may request a commit.
2. rrflowMX and rrflowKV implement the same narrow transactional storage port.
   rrflowMX is volatile. rrflowKV owns WAL, MVCC, ordered point/range access,
   manifests, immutable segments, compaction, recovery, and durability.
3. rrflowKV keeps an ordered binary key/version spine. At flush and compaction,
   it also builds Arrow-compatible immutable column pages with explicit schema,
   encoding, compression, checksum, and lifetime metadata. Hot mutation state
   remains key-optimized; immutable scan state becomes columnar.
4. rrflowQL parses and binds semantic queries. Native graph, BM25, scalar,
   exact-vector, HNSW, and RRF operators choose bounded access paths at one
   stamp. DataFusion consumes streaming Arrow batches for analytical work; it
   never owns authorization, canonical state, transactions, or direct commits.
5. HNSW and quantization are replaceable projections over exact canonical
   vectors. Approximate candidates are filtered and exactly reranked; exact
   search remains the oracle and fallback.
6. LFG or any other model adapter receives a bounded, authorized route packet
   and may only select a recipe, propose a branch, or request narrower context.
   It cannot choose raw storage keys, physical indexes, authorization, or
   mutations.
7. PostgreSQL, Turso, Dragonfly, SQL stores, and project databases are
   discovered as governed external sources/adapters. They are never selected
   implicitly as rrflowDB persistence and never become a parallel RRFlow
   authority.

## Evidence reconciliation

### Transaction and storage boundary

SurrealDB's current architecture documentation describes one query layer over
transactional point/range storage, with document, graph, and index changes in
the same transaction and snapshot isolation plus write-conflict detection.
That supports RRFlow's single-engine direction, but it does not prove RRFlow's
implementation. RRFlow must retain its own exact differential, crash, and
reopen tests. [SurrealDB architecture](https://surrealdb.com/docs/learn/data-models/architecture)

FoundationDB's tuple layer demonstrates why an ordered byte encoding must
preserve tuple order, and its transaction documentation makes conflict ranges
and optimistic commit behavior explicit. RRFlow should borrow those contract
properties, not claim FoundationDB's strict serializability. The 1.0 target
remains the declared snapshot isolation plus write-conflict behavior until a
stronger contract is separately accepted and proven. [FoundationDB developer guide](https://apple.github.io/foundationdb/developer-guide.html)

### Hybrid LSM and Arrow pages

Arrow specifies an in-memory columnar layout optimized for locality,
vectorization, and eligible zero-copy sharing, with alignment requirements.
It does not provide RRFlow's WAL, concurrency control, key ordering, manifest,
or mutable transaction semantics. [Arrow columnar format](https://arrow.apache.org/docs/format/Columnar.html)

Lance's file-format documentation provides useful page-level precedents:
independent per-column pages, random-access row ranges, explicit page metadata,
and aligned buffers, while leaving table semantics and search structures to
higher layers. RRFlow should use those principles as a comparison, not adopt
Lance as an authority or promise universal zero-copy. [Lance file format](https://github.com/lance-format/lance/blob/main/docs/src/format/file/index.md)

Published columnar-LSM work shows a sound transition point: use LSM flush and
compaction to transform mutable records into immutable columnar components.
This supports retaining a key/version-oriented WAL and memtable while producing
column pages in immutable rrflowKV segments. It also means schema evolution,
write amplification, point-read amplification, and compaction cost must be
measured rather than assumed away. [Columnar Formats for Schemaless LSM-based Document Stores](https://www.vldb.org/pvldb/vol15/p2085-alkowaileet.pdf)

“Zero-copy” is therefore conditional. An uncompressed, aligned, type-compatible
page with a pinned mapped lifetime may be borrowed. Compressed, encrypted,
misaligned, evolved-schema, dictionary-remapped, or otherwise incompatible
pages require decoding or copying into a budgeted Arrow pool. Every path must
report physical bytes read, decoded bytes, copied bytes, allocated bytes, and
borrowed bytes.

### DataFusion boundary

DataFusion's custom-provider guidance states that data must be fetched during
physical execution, not planning, or optimizer pushdown and resource controls
cannot reduce work. `TableProvider::scan` must stream execution partitions and
declare each filter as exact, inexact, or unsupported. That directly rejects
the current eager `Vec<QueryRow>` snapshot as the 1.0 endpoint. [DataFusion custom table providers](https://datafusion.apache.org/library-user-guide/custom-table-providers.html)

DataFusion exposes bounded memory pools and spill behavior, but those controls
cover DataFusion reservations, not every native graph/vector/storage allocation.
RRFlow must enforce one cross-operator budget in `RrdEngine` and propagate it
to storage scans, native operators, and DataFusion. [DataFusion memory pools](https://docs.rs/datafusion/latest/datafusion/execution/memory_pool/trait.MemoryPool.html)

Moka is not required by Arrow or DataFusion and is not currently a workspace
dependency. Its weighted-capacity concurrent cache is a viable implementation
candidate only for Gate F-05 after measurement. The decision criterion is a
byte-bounded hit-rate/latency benchmark plus exact `ReadStamp`, schema, and
catalogue invalidation. A cache must never hold canonical state or conceal
stale results. [Moka crate documentation](https://docs.rs/moka/latest/moka/)

### Vector, filtering, and fusion

The HNSW paper establishes the approximate multi-layer navigable graph design;
it does not supply transactional freshness, filtered-query correctness, or an
exactness guarantee. [HNSW paper](https://arxiv.org/abs/1603.09320)

Qdrant's current indexing guidance explains why payload indexes feed
cardinality estimates, why full scan can outperform HNSW below a threshold,
why filter-aware HNSW depends on payload indexes, and why strict mode can reject
unsafe unindexed filters. RRFlow should retain exact fallback, require index
freshness/source-cursor evidence, and make the access decision inside the
planner. [Qdrant indexing](https://qdrant.tech/documentation/manage-data/indexing/)

BM25 and reciprocal rank fusion remain distinct operations: BM25 is an exact
lexical scoring model over persisted term/document statistics; RRF combines
ranked result lists and must be a pure query-time function. A verified outcome
may later update a versioned policy for future stamps, but a query must not
rewrite its own weights. [BM25 review](https://www.staff.city.ac.uk/~sbrp622/papers/foundations_bm25_review.pdf), [RRF paper](https://research.google/pubs/reciprocal-rank-fusion-outperforms-condorcet-and-individual-rank-learning-methods/)

### Attunement and provider-neutral context

Tree-sitter is explicitly incremental and robust to incomplete source, so it
fits a digest- and grammar-revision-bound parse phase. Parse errors are data to
record and ground, not a reason to pretend a project was fully attuned.
[Tree-sitter introduction](https://tree-sitter.github.io/)

Provider instructions must remain thin host adapters. Codex discovers scoped
`AGENTS.md` files; Claude documents importing `AGENTS.md` from `CLAUDE.md` to
avoid duplication; Gemini supports hierarchical context and `@file` imports.
These sources support the repository's forwarding stubs and reject provider
files as lifecycle or memory authorities. [OpenAI Codex agent loop](https://openai.com/index/unrolling-the-codex-agent-loop/), [Claude project memory](https://code.claude.com/docs/en/memory), [Gemini context files](https://geminicli.com/docs/cli/gemini-md/)

Dragonfly is a Redis/Memcached-compatible external in-memory datastore; Turso
and libSQL are application database technologies with their own replication
and consistency rules. Their presence in a discovered project should create a
governed source descriptor and optional adapter, never replace rrflowMX or
rrflowKV automatically. [Dragonfly documentation](https://www.dragonflydb.io/docs), [Turso embedded replicas](https://docs.turso.tech/features/embedded-replicas/introduction)

### DevForge placement and release evidence

Linux OverlayFS permits a shared read-only lower layer and a per-instance
writable upper/work directory, but the upper and work directories have
filesystem requirements and crash behavior still depends on explicit durable
writes. RRFlow's WAL, manifest, segments, and mutable catalogue must be wholly
inside one instance's upper layer; lower tools/models are immutable inputs
identified by digest. [Linux OverlayFS documentation](https://docs.kernel.org/filesystems/overlayfs.html)

Release artifacts need verifiable provenance describing where, when, and how
they were produced. That supports Gate J's signed artifact, SBOM, and clean
machine verification rather than a bare successful local build.
[SLSA provenance](https://slsa.dev/spec/v1.2/provenance)

## Current-code gap matrix

| Claim | Current evidence | Missing proof | Roadmap owner |
|---|---|---|---|
| rrflowKV is persistent | `rrd-lsm` has WAL, MVCC versions, manifest/CURRENT, immutable segments, recovery, snapshots, compaction, and failure injection | one final format, no legacy readers, transaction conflicts, hybrid column pages, crash matrix | C-01..C-07 |
| rrflowMX and rrflowKV share semantics | `RrflowMxEngine`, native engine, and common `Engine` trait exist | minimal transaction port and identical conformance corpus including conflict/rollback | C-02 |
| semantic writes are atomic | `NativeRuntimeCommitPlan` batches runtime data, log, outbox, cursor, audit, and outcome | both adjacency directions plus scalar/unique/BM25/vector index deltas in that same batch | C-03 |
| reads are direct and stamped | `ReadStamp`, direct materialized keyspaces, and validation exist | remove normal-path whole-log reconstruction and prove bounded point/range work | C-04 |
| no compatibility backend | native format is default | Fjall selection/dependency, TextV1, batch/segment/manifest/catalog compatibility branches and migration runtime remain | C-05, J-01 |
| DataFusion is integrated | query execution uses DataFusion, `MemorySource`, spill pool, timeout, and output limits | real rrflowKV streaming provider, pushdown, cross-operator resource accounting | F-01, F-02, F-04 |
| native graph/BM25/vector are real | graph traversal, BM25 code, exact vector oracle, HNSW, catalogues, and planner exist | persistent incremental access paths and same-stamp native physical operators | E-01..E-05, F-03 |
| dynamic context works | engine context/retrieval functions and RRF helpers exist | planner-selected eligible avenues with selected/skipped evidence, pure RRF, versioned feedback | H-01, H-02 |
| live delivery works | durable subscriptions and WebSocket delivery exist | commit-impact predicate deltas; current semantic live query reruns two snapshots | H-03 |
| install/attunement works | strict B-01 plan/job/checkpoint contracts exist | installer, pure attunement compute crate, persisted engine executor, phase-by-phase real fixtures | D-01..D-10 |
| LFG is pluggable | embedding inference exists and B-02 router wire contract is frozen | manifest handshake, `RouterBackend`, LFG adapter, constrained decode, route execution | B-03, G-01..G-06 |
| automation is governed | synchronous function/trigger foundation exists | canonical event envelope, persisted trigger conditions, resumable routine graphs, skill packages, explicit removable host adapters | I-01..I-07 |

## Decisions and exclusions

- Keep and converge the real implementations; do not restore every deleted or
  superseded fragment and do not start a parallel engine.
- Do not import SurrealDB, Qdrant, Lance, Turso, Dragonfly, or DataFusion as a
  second source of truth. Use documented properties as acceptance references.
- Do not claim universal zero-copy, sub-millisecond latency, superior
  performance, production readiness, or complete context flow before the named
  tests and fixed-hardware evidence exist.
- Do not add Moka during architecture cleanup. Gate F-05 owns a measured
  build-or-reject decision.
- Do not add GraphQL, SDK, Connectome, LFG, trigger, routine, skill, mesh, or
  provider-hook behavior before its preceding contract/storage/query gates.
- Do not change the `1.0.0` version during convergence.

## Claim-to-source ledger

| Claim family | Source | Publisher / author | Date or version | URL | Access note |
|---|---|---|---|---|---|
| Arrow layout and eligible zero-copy | Arrow Columnar Format | Apache Arrow | v25.0.1, accessed 2026-09-05 | https://arrow.apache.org/docs/format/Columnar.html | Official specification |
| Column pages and alignment precedent | Lance File Format | Lance project | main, accessed 2026-09-05 | https://github.com/lance-format/lance/blob/main/docs/src/format/file/index.md | Official format documentation |
| Columnar transformation at LSM events | Columnar Formats for Schemaless LSM-based Document Stores | Alkowaileet et al., PVLDB | 2022 | https://www.vldb.org/pvldb/vol15/p2085-alkowaileet.pdf | Peer-reviewed paper |
| Streaming provider and pushdown | Custom Table Provider | Apache DataFusion | accessed 2026-09-05 | https://datafusion.apache.org/library-user-guide/custom-table-providers.html | Official documentation |
| Memory-pool and spill semantics | `MemoryPool` | Apache DataFusion docs.rs build | 55.0.0, accessed 2026-09-05 | https://docs.rs/datafusion/latest/datafusion/execution/memory_pool/trait.MemoryPool.html | Official crate API documentation; matches the pinned workspace release |
| Unified transactional data models | Architecture | SurrealDB | accessed 2026-09-05 | https://surrealdb.com/docs/learn/data-models/architecture | Official documentation |
| Ordered tuples and conflict ranges | Developer Guide | FoundationDB | 7.4.7, accessed 2026-09-05 | https://apple.github.io/foundationdb/developer-guide.html | Official documentation |
| HNSW design | Efficient and robust approximate nearest neighbor search using HNSW graphs | Malkov and Yashunin | 2016/2018 | https://arxiv.org/abs/1603.09320 | Original paper |
| Filtered vector planning | Indexing | Qdrant | accessed 2026-09-05 | https://qdrant.tech/documentation/manage-data/indexing/ | Official documentation |
| BM25 semantics | The Probabilistic Relevance Framework: BM25 and Beyond | Robertson and Zaragoza | 2009 | https://www.staff.city.ac.uk/~sbrp622/papers/foundations_bm25_review.pdf | Primary technical review |
| RRF semantics | Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods | Cormack, Clarke, Buettcher | 2009 | https://research.google/pubs/reciprocal-rank-fusion-outperforms-condorcet-and-individual-rank-learning-methods/ | Publisher record for original paper |
| Incremental resilient parsing | Tree-sitter Introduction | Tree-sitter project | accessed 2026-09-05 | https://tree-sitter.github.io/ | Official documentation |
| Codex instruction discovery | Unrolling the Codex agent loop | OpenAI | 2026, accessed 2026-09-05 | https://openai.com/index/unrolling-the-codex-agent-loop/ | First-party engineering article |
| Claude instruction import | How Claude remembers your project | Anthropic | accessed 2026-09-05 | https://code.claude.com/docs/en/memory | Official documentation |
| Gemini instruction hierarchy/import | Provide context with GEMINI.md files | Google | updated 2026-06-18 | https://geminicli.com/docs/cli/gemini-md/ | Official documentation |
| Optional weighted cache | Moka crate | Moka project | 0.12.16, accessed 2026-09-05 | https://docs.rs/moka/latest/moka/ | Official crate API documentation |
| External cache classification | Dragonfly Docs | Dragonfly | updated 2026-08-04 | https://www.dragonflydb.io/docs | Official documentation |
| External application DB classification | Embedded Replicas | Turso | accessed 2026-09-05 | https://docs.turso.tech/features/embedded-replicas/introduction | Official documentation; page marks this feature legacy |
| CoW upper/lower behavior | Overlay Filesystem | Linux kernel | accessed 2026-09-05 | https://docs.kernel.org/filesystems/overlayfs.html | Official kernel documentation |
| Release provenance | SLSA Provenance | Linux Foundation / SLSA | v1.2, accessed 2026-09-05 | https://slsa.dev/spec/v1.2/provenance | Approved specification |

## Research limitations and stop condition

The research determines architecture and testable boundaries; it does not
benchmark RRFlow, prove the present code, decide distributed consensus, or
authorize reuse of third-party source. Zuul Zero/shippin.ai protocol details
were not publicly verifiable in this pass, so Gate H-07 must treat the mesh as
an adapter contract supplied by its operator, not invent protocol behavior.

Discovery stopped after every consequential design slot had a primary or
first-party source, current code had been reconciled against each slot, and
additional searches were returning duplicate implementation patterns rather
than changing a boundary decision.
