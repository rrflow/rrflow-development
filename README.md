# RRFlow

RRFlow is one reasoning-data engine. RRD is the daemon and embedded runtime for
that engine. Its job is to preserve temporal knowledge and assemble the bounded,
relevant context an AI system needs without making callers understand storage
fields, indexes, graph layout, embedding providers, or projection internals.

This README is the bootstrap product entry point and knowledge map. It owns
RRFlow's identity, non-negotiable architecture invariants, and current status;
it delegates each detailed subject to exactly one linked memory record. The
RRFlow 1.0 release checklist is owned by
[`docs/roadmap/rrflow-1.0.md`](docs/roadmap/rrflow-1.0.md). Supporting records
may supply design, research, contracts, or evidence, but cannot silently
override an owning record.

The target release-train version is `1.0.0`. It is frozen while the first alpha
baseline is established. The current maturity is **pre-alpha**; the
version identifies the contract line being built and is not a claim that the
RRFlow 1.0 system is complete, optimized, stable, or release-ready. Progress is
recorded as objective and gate evidence rather than version increments.

## Canonical RRFlow 1.0 terminology

These names describe one system. They are not aliases for parallel engines or
independent stores.

| Term | Canonical meaning |
|---|---|
| **RRFlow** | The product and the complete reasoning-data engine. Cite the current release as **RRFlow 1.0**. |
| **RRFlow database** | One persistent RRFlow data estate governed by the engine. It contains temporal knowledge and derived acceleration state; it is not an application database replacement. |
| **RRD** | The daemon and embedded runtime boundary for RRFlow. RRD is not a second product or engine. |
| **`RrdEngine`** | The sole in-process composition root and semantic authority for database, graph, memory, query, and context operations. |
| **RRFlow kernel** | The canonical temporal values, identities, read stamps, mutations, and invariants implemented in `rrd-core`. The kernel defines meaning but does not own transport or physical storage. |
| **canonical runtime log** | The ordered source of truth for committed RRFlow changes. Temporal snapshots are resolved from this log. |
| **rrflowKV** | RRFlow's native hybrid MVCC/LSM physical layer, implemented by `rrd-lsm` and adapted through `rrd-store`. Its target layout combines a WAL and mutable MVCC memtable with immutable segments containing an ordered key/version spine and Arrow-compatible column pages. One semantic engine commit becomes one atomic rrflowKV write batch. rrflowKV is not a second engine or public data model. |
| **rrflowMX** | RRFlow's process-local, non-durable implementation of the same semantic storage port. It supports volatile and conformance execution through `RrdEngine`; it is not a persistent RRFlow database, a cache, an Arrow working set, or another semantic authority. |
| **RRFlow temporal graph** | Typed records and relations resolved at a runtime read stamp and valid-time coordinate from the same canonical log. It is not a separate graph database. |
| **RRFlow memory** | Durable temporal knowledge—claims, records, relations, schemas, vectors, and evidence—owned by the same engine. It is not a separate memory store. |
| **RRFlowQL** | RRFlow's query language. Use **RRFlowQL**, not the ambiguous shorthand “QL,” in product documentation. |
| **Arrow substrate** | The shared columnar buffer model for eligible immutable rrflowKV segment columns and stamped in-memory `RecordBatch` streams. Published canonical segment values may be authoritative; caches and index projections remain rebuildable. Arrow buffers do not own transactions, mutable state, or authorization. |
| **DataFusion execution** | Vectorized physical evaluation over stamped Arrow batches after RRFlow selects authoritative KV, graph, lexical, or vector access paths. DataFusion is not the database authority. |
| **fast path** | Low-latency state, pointer, and bounded graph navigation performed by `RrdEngine` directly against rrflowKV without invoking DataFusion. |
| **analytical path** | RRFlowQL planning plus native graph, lexical, vector, and RRF operators composed with Arrow/DataFusion for broad retrieval, ingestion, joins, and analytics. |
| **index projection** | Derived acceleration state bound to its source cursor and relevant schema or catalogue revision. An index can be rebuilt and cannot outrank the canonical log. |
| **context assembly** | Bounded retrieval and deterministic fusion performed by `RrdEngine::assemble_context`; the result is a `ContextPacket` with evidence and its read stamp. |
| **LFG** | A replaceable local routing-model adapter. LFG may propose a recipe, branch, or bounded query intent; it never owns state, permissions, physical plans, or mutations. |
| **Connectome** | RRFlow's separate client and operator workbench. It observes and invokes RRFlow through public RRD capabilities and never recreates engine logic. |

For citations, use **RRFlow database**, **RRFlow kernel**, **rrflowKV**, **rrflowMX**,
**RRFlow temporal graph**, **RRFlow memory**, **RRFlowQL**, **Arrow/DataFusion
analytical path**, **LFG**, and **Connectome**. Do not describe RRD, rrflowKV,
the graph, memory, indexes, LFG, or Connectome as additional engines or sources
of truth.

## Non-negotiable architecture

`rrd-engine::RrdEngine` is the sole composition root and the only owner of
context assembly. There is one canonical runtime change log and one temporal
snapshot boundary. Claims, records, relations, vectors, schemas, and their
valid-time history are not separate memory systems.

RRFlow is one hybrid transactional/analytical system. Its fast and analytical
paths share authentication, authorization, schema, read stamps, transaction
coordination, one selected storage profile, and the canonical runtime log. A
persistent RRFlow database selects rrflowKV; a deliberately non-durable
process-local composition selects rrflowMX. Both remain subordinate to the same
`RrdEngine` authority:

```text
HTTP / WebSocket / native SDK / embedded caller
                         |
                provider-neutral contract
                         |
                   authenticate session
                         |
                         v
                     RrdEngine
        +----------------+----------------+
        |                                 |
        v                                 v
 fast path                          analytical path
 route/tree state                   RRFlowQL or GraphQL adapter
        |                                 |
 optional RouterBackend                   AST -> bind -> authorize -> plan
 (LFG adapter)                            |
        |                     +-----------+-----------+
 validated decision           |           |           |
        |                   KV scan   graph/BM25   HNSW candidates
 deterministic predicates     |           |           |
        |                     stamped Arrow RecordBatch stream
 storage point/range ops                   |
        |                         DataFusion + native RRF
        +--------------------+------------+
                             |
                  one authorized transaction
                             |
                  +----------+----------+
                  |                     |
       rrflowMX volatile state    rrflowKV write batch
                                        |
                              WAL -> MVCC memtable
                                   -> immutable segments
                                      key/version spine
                                      + Arrow-compatible pages
                  +----------+----------+
                             |
                   changefeed / live deltas
```

LFG never receives raw storage keys and never advances a branch by writing the
database directly. A routing model returns one versioned, grammar-constrained
decision. `RrdEngine` validates that decision, evaluates deterministic branch
conditions against stamped evidence, authorizes the resulting operation, and
performs any compare-and-swap mutation. Model inference latency and rrflowKV
operation latency are measured separately; the size of a model is not a latency
guarantee.

```text
intent + optional anchors + explicit budgets
                      |
                      v
        RrdEngine::assemble_context
                      |
          authenticate and authorize
                      |
       capture one runtime read stamp
                      |
     resolve one valid-time data snapshot
       /              |               \
dynamic BM25   matching local vector   bounded graph BFS
records+claims   spaces, exact score    from discovered roots
       \              |               /
        deterministic reciprocal-rank fusion
                      |
      item/byte/scan/depth enforcement
                      |
  ContextPacket(items, evidence, stamp, digest)
```

Every returned item carries the retrieval source, source rank, source score,
fusion contribution, plan digest, and evidence digest. The packet carries its
runtime cursor, schema revision, catalogue revision, query digest, encoded byte
count, truncation state, and content digest. It also carries one canonical plan
snapshot that records whether seed, lexical, semantic, graph, and fusion stages
were selected or skipped, the physical path each selected stage used, and the
reason for every decision. The plan also carries the compiled security-policy
revision and authorization digest. The plan and packet share the same read
stamp. A packet therefore identifies what was authorized, what was read, which
avenues were considered, and why each avenue was or was not selected; it is not
an ungrounded text blob.

The caller supplies only:

- a scope and query;
- an explicit valid-time coordinate;
- optional record anchors;
- graph-depth, item, output-byte, and scanned-change budgets.

The caller does not choose fields, collections, vector names, embedding
backends, or graph edges. The engine discovers eligible sources from the same
captured snapshot and catalogue. Only installed deterministic text embedding
backends that require no network access are eligible for automatic semantic
retrieval.

There are no editor- or provider-owned automatic hook configurations, parallel
hook lifecycle engines, provider registries, runtime-tool catalogues, or
separate graph-routing databases in the product architecture. Explicit
automation capabilities may be added only as versioned `rrd-contract`
operations composed through `RrdEngine`: a trigger is a condition over a
canonical engine event, a routine is a resumable sequence of authorized engine
operations, a hook adapter only submits typed host events, and a skill is a
versioned instruction/resource reference resolved into governed context.
Storage/index maintenance state remains an internal physical concern and is
not a second source of truth.

## Canonical context contract

The provider-neutral operation is `context-assemble` at
`POST /v1/context/assemble`. Its request and response types live in
`rrd-contract`; its implementation lives in `rrd-engine`. The Rust client,
daemon handler, CLI, MCP server, and Connectome use that operation or call the
same engine method in embedded mode.

Claude, OpenAI, local-model, and future host integrations are plug-in adapters
at the edge, not engine variants. Each adapter translates the lifecycle or
request boundary exposed by its host into this same operation. Provider URLs,
credentials, payload dialects, and interception mechanics remain inside the
replaceable adapter; adding a provider does not add an RRFlow context endpoint,
provider registry, memory path, or lifecycle authority.

Current hard request ceilings are:

| Resource | Maximum |
|---|---:|
| query bytes | 64 KiB |
| seed records | 256 |
| returned items | 512 |
| encoded item bytes | 768 KiB |
| scanned runtime changes | 1,000,000 |
| graph depth | 32 |

Within a request, lexical results are capped to the item budget, matching
vector sources and hits are capped to the item budget, and graph work is capped
to `max_items * 16` edge steps. Any hidden result or work limit sets
`truncated=true`; truncation is never presented as a complete answer.

The same request against the same persisted read coordinate is deterministic.
Behavior tests cover temporal exclusion, lexical retrieval, automatic matching
of a locally installed vector model and collection, graph expansion, claim
resolution, fusion evidence, resource truncation, packet validation, exact
cursor binding, and equality after closing and reopening the engine.

## Persistent seat identity and warp points

An RRFlow seat is a durable identity in the canonical runtime graph. Provider
accounts are temporal `rrflow-represents` edges into that seat; they do not own
the identity and changing a provider does not change the seat. A seat resolves
as self only while at least one provider identity represents it. Persisted
provider identity records contain an opaque provider name and a subject digest,
never provider credentials or provider runtime sessions.

Every record has a stable, path-safe coordinate:

```text
rrflow://<instance>/data/<record-kind>/<record-id>
```

For this project, the primary identity coordinate is the
[Clyffy seat](rrflow://rrflow-instance/data/rrflow-seat/clyffy). README links
using this scheme are warp points into the RRFlow database, not copies of
database state.
The CLI resolves a warp by converting it to a record anchor and invoking the
same `RrdEngine::assemble_context` planner and temporal snapshot used by normal
context requests.

Persist or update a seat and one provider representation explicitly:

```bash
rrflow identity bind \
  --seat clyffy \
  --provider openai \
  --provider-identity codex \
  --provider-subject '<provider-subject>' \
  --representation codex-represents-clyffy
```

Resolve self or follow the README warp point:

```bash
rrflow identity resolve --seat clyffy
rrflow context \
  --warp rrflow://rrflow-instance/data/rrflow-seat/clyffy \
  --max-graph-depth 1
```

`identity bind` plans strict seat/provider/relation schema plus mutations, then
commits the plan through the ordinary authenticated data transaction boundary.
It does not bypass mutation authorization or create a parallel identity store.

## Authority and surfaces

Dependency direction is inward toward the engine, never sideways into a new
authority:

| Component | Ownership |
|---|---|
| `rrd-core` | RRFlow kernel: temporal values, identities, canonical mutations, read stamps, and snapshot invariants |
| `rrd-lsm` | rrflowKV WAL, MVCC memtable, immutable segments, cache, compaction, and physical snapshots |
| `rrd-store` | semantic storage port, rrflowMX volatile implementation, semantic-to-rrflowKV mapping, canonical log/materialized-key maintenance, and cross-profile conformance |
| `rrd-query` | RRFlowQL syntax, bound logical plans, physical planning, native query operators, and Arrow/DataFusion execution |
| `rrd-vector` | exact vector truth, HNSW/quantized candidate indexes, reranking, and vector-index conformance |
| `rrd-inference` | process-local, provider-neutral executable model adapters; embedding and routing are separate capabilities |
| `rrd-security` | identities, policy, authorization, and audit evidence |
| `rrd-engine` | sole composition root: sessions, transactions, fast-path routing, analytical planning, context assembly, attunement, and mutation authority |
| `rrd-contract` | versioned provider-, model-, language-, and transport-neutral public operations and envelopes |
| `rrd-server`, `rrd-client` | HTTP/WebSocket daemon transport and typed Rust client; no semantic execution |
| `rrflow-cli`, `rrflow-mcp` | outward command and host adapters; no storage, retrieval, or lifecycle authority |
| Connectome repository | separate operator client using public RRD capabilities; no embedded engine logic or canonical state |
| remaining estate, cluster, maintenance, SDK, and evaluation crates | operational or client capabilities composed around the same authority |

The engine repository converges on this physical grouping. Package names remain
stable unless their product meaning is wrong; directory moves do not create
aliases or duplicate packages:

```text
crates/
├── kernel/
│   └── rrd-core
├── persistence/
│   ├── rrd-lsm                 # rrflowKV physical implementation
│   └── rrd-store               # rrflowMX plus semantic/rrflowKV mapping
├── compute/
│   ├── rrd-query               # RRFlowQL, plans, Arrow/DataFusion, BM25
│   ├── rrd-vector              # exact vector truth and ANN candidates
│   ├── rrd-inference           # embedding and routing backend capabilities
│   └── rrd-attunement          # pure inventory/parse/normalization work
├── authority/
│   ├── rrd-security
│   ├── rrd-estate
│   └── rrd-engine              # sole composition and mutation authority
├── transport/
│   ├── rrd-contract
│   ├── rrd-client
│   └── rrd-server
├── adapters/
│   ├── rrflow-cli
│   ├── rrflow-mcp
│   └── rrflow-edge
├── operations/
│   ├── rrd-cluster
│   ├── rrd-kubernetes
│   ├── rrd-maintenance
│   └── rrd-operator-knowledge
└── evaluation/
    └── rrflow-eval
```

There is no `rrd-graph` crate: graph values live in the kernel, adjacency keys
live in persistence, graph operators live in query, and `RrdEngine` authorizes
and composes them. There is no `connectome-ui` crate in this repository:
Connectome is developed, tested, and released from its separate repository.
`rrd-attunement` may compute bounded deterministic phase outputs, but it cannot
open storage or commit state; `RrdEngine` owns every job transition and
mutation. Moves to this layout occur one group at a time with manifests and
tests updated in the same commit; no mirrored transitional tree is permitted.

Direct query, transaction, backup, diagnostic, and administration operations
remain valid database capabilities. They do not constitute alternate context
engines. Any outward surface that needs model context must use
`context-assemble` rather than constructing its own recall pipeline.

MCP exposes one context tool backed by this operation in both embedded and
authenticated daemon modes. Automatic prompt insertion is possible only in a
host that provides a turn/interception integration point. RRFlow does not claim
to invisibly alter prompts in a host that provides no such API; where a host
does provide that boundary, its adapter must invoke this same operation and
must not implement retrieval itself.

## Connectome bootstrap and project attunement

[Connectome](https://github.com/EonsofStupid/rrflow-connectome) is a standalone
client repository and release boundary. A checkout may be mounted beside the
engine repository during development, but the client does not own engine state
or lifecycle. Its first runtime invariant is a real HTTP bootstrap against the
public RRD endpoints, in order:

```text
GET /v1/health/live
GET /v1/health/ready
GET /v1/capabilities
```

Connectome accepts the runtime only after validating the `rrd` protocol
version and the returned instance resource. Plain HTTP is loopback-only;
off-device and mesh-resolved endpoints require HTTPS. A Zuul Zero or
shippin.ai mesh adapter may resolve a devspace endpoint and supply opaque
transport attestation, but it does not become a database authority and network
reachability alone does not establish RRD identity.

Installation into an existing project will be an explicit, previewable,
resumable attunement operation owned by `RrdEngine`, with a planning estimate
of 30–45 minutes for a substantial estate rather than a completion guarantee.
The intended checkpointed phases are connect, inventory, parse, normalize,
entity-link, lexical index, embed, vector index, graph, ground, and verify.
Every mutation phase must use the canonical transaction boundary, record its
runtime cursor/schema/catalogue coordinates and source digests, and survive a
process restart without silently repeating committed work. Connectome may
start and observe that job only after the contract and engine implementation
exist; client-side phase labels are not evidence of implementation.

## Current status

Implemented now:

- explicit rrflowMX volatile and native rrflowKV persistent compositions using
  the same `RrdEngine`, semantic storage contract, stamped RRFlowQL pipeline,
  and DataFusion executor; the shared logical corpus also verifies rrflowKV
  reopen, while rrflowMX explicitly rejects durability-only operations;
- a versioned, provider-neutral reasoning-tree contract with typed nodes and
  edges, content-addressed recipes/evidence, read-stamped cursors, and
  fail-closed cursor-advance verification;
- a versioned, provider-neutral installation and attunement contract with
  deterministic plans, ordered digest-chained phase checkpoints, explicit job
  states and leases, revision-bound resume/cancel requests, and verification
  evidence;
- a versioned router contract that binds one stamped reasoning cursor to
  bounded signals and eligible recipe, branch, or context decisions without
  exposing provider, model, storage, index, authorization, or mutation state;
- native durable runtime changes with exact read stamps and valid-time reads;
- active claims and typed records resolved into the same context snapshot;
- dynamic lexical retrieval over discoverable textual properties;
- deterministic automatic semantic retrieval over compatible installed local
  embedding backends and vector collections;
- bounded graph expansion from explicit and retrieved roots;
- deterministic multi-source fusion, deduplication, evidence, digests, and
  resource enforcement;
- a read-stamped context plan exposing every selected or skipped retrieval
  stage, its current physical access path, its decision reason, and the exact
  compiled authorization boundary;
- durable provider-neutral seat identity, provider representation edges, and
  stable `rrflow://` record warp resolution through the context planner;
- authenticated server/client, embedded/daemon MCP, and CLI converging on the
  same context operation;
- a separate Connectome repository with native RRD capability negotiation,
  session establishment, bounded diagnostic/context calls, renewal, and close;
- persistence/reopen and real transport behavior tests.

Not implemented or not yet production-grade:

- rrflowKV immutable segment v3 is currently a compressed row-record block
  format. It does not yet contain the target key/version spine plus
  Arrow-compatible column pages, expose borrowed Arrow buffers, or report
  copied, decoded, and allocated bytes per read;
- RRFlowQL currently materializes authoritative source rows before constructing
  newly allocated stamped Arrow batches; DataFusion execution is real, but the
  bounded streaming rrflowKV table provider, conditional zero-copy path, scan
  pushdown, and native mixed operators required by Gate F are not implemented;
- reasoning-tree persistence and execution, router-backend dispatch, and the
  LFG adapter are not yet implemented; A-02 and B-02 freeze their semantics
  but do not claim a running router;
- lexical indexing is rebuilt from the bounded snapshot per request; it is not
  yet an incrementally maintained persistent index;
- context reads with row- or field-restricted data policies currently fail
  closed; heterogeneous context-policy enforcement is not yet implemented;
- context vector retrieval currently performs exact scoring over matching
  snapshot vectors; the context planner does not yet select ANN candidates and
  exact-rerank them;
- graph expansion currently traverses relations in both directions without
  relation-type policy or learned edge weights;
- ingestion is still explicit typed transaction/claim input rather than an
  automatic content normalization and entity-linking pipeline;
- fusion weights are static and there is no feedback learner, query planner
  cost model, or quality regression corpus tied to release gates;
- automatic turn-boundary invocation still requires a supported host adapter.
- mesh endpoint discovery, engine persistence and execution of the canonical
  attunement job, and engine-owned trigger/routine/hook-adapter/skill
  operations are not yet implemented.
- standalone Connectome is not yet RRFlow 1.0-conformant: its inherited UI and
  runtime paths have not been consolidated onto the public RRD client, so it
  must not claim the 1.0 release version yet.

These are material gaps. A successful compile is not evidence that context
flows correctly, and none of the gaps above is represented as complete.

## Knowledge warp points

The checkout is the bootstrap representation of RRFlow memory until rrflowKV
can persist and resolve these records itself. The root README is an entry map,
not the container for every architectural or planning record. Each subject has
one owning document, one stable `rrflow://` coordinate, and one local fallback:

| Memory | Durable warp | Checkout fallback |
|---|---|---|
| Knowledge structure and ownership | [`rrflow://rrflow-instance/data/documentation-index/rrflow-knowledge-map`](rrflow://rrflow-instance/data/documentation-index/rrflow-knowledge-map) | [`docs/README.md`](docs/README.md) |
| RRFlow 1.0 alpha objectives | [`rrflow://rrflow-instance/data/objective/rrflow-1.0-alpha`](rrflow://rrflow-instance/data/objective/rrflow-1.0-alpha) | [`docs/objectives/rrflow-1.0-alpha.md`](docs/objectives/rrflow-1.0-alpha.md) |
| RRFlow 1.0 release roadmap | [`rrflow://rrflow-instance/data/roadmap/rrflow-1.0`](rrflow://rrflow-instance/data/roadmap/rrflow-1.0) | [`docs/roadmap/rrflow-1.0.md`](docs/roadmap/rrflow-1.0.md) |
| RRFlow 1.0 alpha POA&M | [`rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha`](rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha) | [`docs/poam/rrflow-1.0-alpha.md`](docs/poam/rrflow-1.0-alpha.md) |
| Provider-neutral agent bootstrap | [`rrflow://rrflow-instance/data/reference/agent-bootstrap`](rrflow://rrflow-instance/data/reference/agent-bootstrap) | [`docs/reference/agent-bootstrap.md`](docs/reference/agent-bootstrap.md) |

The objective defines the result, the roadmap orders delivery, and the POA&M
tracks observed deficiencies and their closure evidence. The records link by
stable identifiers and do not repeat each other's authority. The next
executable item is
[A-06](docs/roadmap/rrflow-1.0.md#gate-a--freeze-authority-names-and-boundaries);
B-03 is paused until documentation ownership and source-boundary terminology
are aligned.

## Verification

The smallest semantic test is:

```bash
cargo test -p rrd-engine --lib engine::tests::context -- --nocapture
```

Real outward-boundary tests are:

```bash
cargo test -p rrflow-mcp --test stdio --test stdio_daemon
cargo test -p rrd-client --test real_server \
  rust_client_negotiates_authenticates_queries_and_reads_audit
```

The separate Connectome repository owns its own boundary gates:

```bash
pnpm run check
pnpm run test:smoke
```

The full Rust compile gate is:

```bash
cargo check --workspace --all-targets
```

Passing the compile gate alone is insufficient. Context changes must pass the
semantic engine test first and the affected real adapter tests afterward.
