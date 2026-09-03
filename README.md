# RRFlow

RRFlow is one reasoning-data engine. RRD is the daemon and embedded runtime for
that engine. Its job is to preserve temporal knowledge and assemble the bounded,
relevant context an AI system needs without making callers understand storage
fields, indexes, graph layout, embedding providers, or projection internals.

This file is the authority for product identity, architecture, current status,
and roadmap. Other documents are supporting contracts, design notes, evidence,
or history. They do not define a second architecture.

The current release-train version is `0.1.0`.

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
count, truncation state, and content digest. A packet therefore identifies what
was read and why it was selected; it is not an ungrounded text blob.

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

There are no editor/provider hook configurations, hook lifecycle engine,
work-plan scheduler, source-attunement gate, provider registry, runtime-tool
catalogue, or separate graph-routing database in the product architecture.
Storage/index maintenance state is an internal physical concern and is not a
reasoning lifecycle or a second source of truth.

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
- authenticated server/client, embedded/daemon MCP, CLI, and Connectome paths
  converging on the same context operation;
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

These are material gaps. A successful compile is not evidence that context
flows correctly, and none of the gaps above is represented as complete.

## Roadmap

Work proceeds in dependency order without introducing another authority:

1. Define one ingestion contract that normalizes documents, conversations,
   claims, entities, relations, and embedding work into canonical mutations.
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
