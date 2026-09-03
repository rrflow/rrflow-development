# RRFlow

RRFlow is one reasoning-data engine. RRD is the daemon and embedded runtime for
that engine. Its job is to preserve temporal knowledge and assemble the bounded,
relevant context an AI system needs without making callers understand storage
fields, indexes, graph layout, embedding providers, or projection internals.

This file is the authority for product identity, architecture, current status,
and roadmap. Other documents are supporting contracts, design notes, evidence,
or history. They do not define a second architecture.

The current release-train version is `1.0.0`.

## Canonical RRFlow 1.0 terminology

These names describe one system. They are not aliases for parallel engines or
independent stores.

| Term | Canonical meaning |
|---|---|
| **RRFlow** | The product and the complete reasoning-data engine. Cite the current release as **RRFlow 1.0**. |
| **RRFlow database** | One persistent RRFlow data estate governed by the engine. It contains temporal knowledge and derived acceleration state; it is not an application database replacement. |
| **RRD** | The daemon and embedded runtime boundary for RRFlow. RRD is not a second product or engine. |
| **`RrdEngine`** | The sole in-process composition root and semantic authority for database, graph, memory, query, and context operations. |
| **canonical runtime log** | The ordered source of truth for committed RRFlow changes. Temporal snapshots are resolved from this log. |
| **native RRD LSM** | RRFlow's persistent physical storage implementation. One semantic engine commit becomes one atomic LSM write batch. |
| **KV layer** | The internal ordered key/value representation used by the native RRD LSM. KV is not RRFlow's public data model or a second authority. |
| **RRFlow temporal graph** | Typed records and relations resolved at a runtime read stamp and valid-time coordinate from the same canonical log. It is not a separate graph database. |
| **RRFlow memory** | Durable temporal knowledge—claims, records, relations, schemas, vectors, and evidence—owned by the same engine. It is not a separate memory store. |
| **RRFlowQL** | RRFlow's query language. Use **RRFlowQL**, not the ambiguous shorthand “QL,” in product documentation. |
| **Arrow snapshot** | A typed, immutable, rebuildable query representation bound to an RRFlow read stamp. Arrow is not the source of truth. |
| **DataFusion execution** | The physical evaluation stage over stamped Arrow data after RRFlow selects the authoritative access path. DataFusion is not the database authority. |
| **index projection** | Derived acceleration state bound to its source cursor and relevant schema or catalogue revision. An index can be rebuilt and cannot outrank the canonical log. |
| **context assembly** | Bounded retrieval and deterministic fusion performed by `RrdEngine::assemble_context`; the result is a `ContextPacket` with evidence and its read stamp. |
| **Connectome** | RRFlow's separate client and operator workbench. It observes and invokes RRFlow through public RRD capabilities and never recreates engine logic. |

For citations, use **RRFlow database**, **native RRD LSM/KV layer**, **RRFlow
temporal graph**, **RRFlow memory**, **RRFlowQL**, **Arrow/DataFusion query
execution**, and **Connectome**. Do not describe RRD, the graph, memory,
indexes, or Connectome as additional engines or sources of truth.

## Non-negotiable architecture

`rrd-engine::RrdEngine` is the sole composition root and the only owner of
context assembly. There is one canonical runtime change log and one temporal
snapshot boundary. Claims, records, relations, vectors, schemas, and their
valid-time history are not separate memory systems.

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
reason for every decision. The plan and packet share the same read stamp. A
packet therefore identifies what was read, which avenues were considered, and
why each avenue was or was not selected; it is not an ungrounded text blob.

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
using this scheme are warp points into RRFlowDB, not copies of database state.
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
| `rrd-core` | temporal values, canonical mutations, snapshot resolution |
| `rrd-lsm`, `rrd-store` | durable physical storage and canonical runtime log |
| `rrd-query` | query execution and lexical scoring primitives |
| `rrd-vector`, `rrd-inference` | vector contracts, exact scoring, installed local embedding backends |
| `rrd-security` | identities, policy, authorization, and audit evidence |
| `rrd-engine` | the single composition root and context planner |
| `rrd-contract` | provider- and transport-neutral public protocol |
| `rrd-server`, `rrd-client` | daemon transport and typed Rust client |
| `rrflow-cli`, `rrflow-mcp`, `connectome-ui` | outward adapters; never data authorities |
| remaining estate, cluster, maintenance, SDK, and evaluation crates | operational or client capabilities composed around the same authority |

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

- native durable runtime changes with exact read stamps and valid-time reads;
- active claims and typed records resolved into the same context snapshot;
- dynamic lexical retrieval over discoverable textual properties;
- deterministic automatic semantic retrieval over compatible installed local
  embedding backends and vector collections;
- bounded graph expansion from explicit and retrieved roots;
- deterministic multi-source fusion, deduplication, evidence, digests, and
  resource enforcement;
- a read-stamped context plan exposing every selected or skipped retrieval
  stage, its current physical access path, and its decision reason;
- durable provider-neutral seat identity, provider representation edges, and
  stable `rrflow://` record warp resolution through the context planner;
- authenticated server/client, embedded/daemon MCP, CLI, and the in-workspace
  client boundary test converging on the same context operation;
- persistence/reopen and real transport behavior tests.

Not implemented or not yet production-grade:

- lexical indexing is rebuilt from the bounded snapshot per request; it is not
  yet an incrementally maintained persistent index;
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
- mesh endpoint discovery, the canonical attunement job, and engine-owned
  trigger/routine/hook-adapter/skill operations are not yet implemented.
- standalone Connectome is not yet RRFlow 1.0-conformant: its inherited UI and
  runtime paths have not been consolidated onto the public RRD client, so it
  must not claim the 1.0 release version yet.

These are material gaps. A successful compile is not evidence that context
flows correctly, and none of the gaps above is represented as complete.

## Roadmap

Work proceeds in dependency order without introducing another authority:

1. Define one ingestion and resumable attunement contract that normalizes
   documents, conversations, claims, entities, relations, and embedding work
   into canonical mutations with durable phase checkpoints.
2. Add persistent incremental lexical indexing tied to runtime cursor and
   schema revision, with exact fallback and corruption recovery.
3. Add a costed semantic planner that can choose exact scan or bounded ANN
   candidates, always followed by authoritative exact reranking.
4. Make graph expansion direction- and relation-aware, with explicit traversal
   policy and per-path evidence.
5. Add snapshot- and query-digest caches with strict byte limits and cursor
   invalidation so repeated turns do not rebuild unchanged work.
6. Add relevance feedback, offline quality corpora, latency/memory budgets, and
   regression gates before changing fusion behavior.
7. Integrate automatic turn-boundary context invocation in each host that
   exposes a supported interception point; keep unsupported hosts explicit.
8. Add engine-owned trigger, routine, typed hook-adapter, and skill operations
   only after the event, authorization, idempotency, replay, and audit
   contracts are behavior-tested at the `RrdEngine` boundary.

## Verification

The smallest semantic test is:

```bash
cargo test -p rrd-engine --lib engine::tests::context -- --nocapture
```

Real outward-boundary tests are:

```bash
cargo test -p rrflow-mcp --test stdio --test stdio_daemon
cargo test -p connectome-ui --test client_boundary
cargo test -p rrd-client --test real_server \
  rust_client_negotiates_authenticates_queries_and_reads_audit
```

The full Rust compile gate is:

```bash
cargo check --workspace --all-targets
```

Passing the compile gate alone is insufficient. Context changes must pass the
semantic engine test first and the affected real adapter tests afterward.
