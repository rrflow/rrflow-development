# RRFlow system overview

**Status:** active accepted master platform architecture; implementation gaps remain open
**Coordinate:** `rrflow://rrflow-instance/data/architecture/system-overview`
**Owner:** canonical platform components, authority boundaries, and high-level relationships
**Decision:** [`../decisions/0001-single-engine-authority.md`](../decisions/0001-single-engine-authority.md)

The repository root [README](../../README.md) is the bootstrap product portal.
This record is the first architecture destination behind that portal. It tells
an operator or AI system what every RRFlow component means and where detailed
architecture lives. The [engine data-flow record](engine-data-flow.md) owns the
complete read, write, Arrow/DataFusion, and context flows; the
[roadmap](../roadmap/rrflow-1.0.md) owns implementation order and completion.

## Platform map

```text
RRFlow — AI governance, reasoning, recall, and context platform
│
├── RRD — embedded and daemon runtime
│   └── RrdEngine — sole semantic, security, and mutation authority
│
├── rrflowDB — persistent per-project AI estate
│   └── rrflowKV — WAL/MVCC/LSM physical persistence
│       ├── hot mutable memtable
│       └── immutable key/version and Arrow-compatible segment target
│
├── rrflowMX — volatile process-local execution profile
│
├── rrflowQL — query language, binding, planning, and execution
│   ├── native KV, temporal graph, lexical, and vector access
│   └── stamped Arrow batches → bounded DataFusion execution
│
├── RRFlow vector subsystem
│   └── canonical vectors → filtered candidates → HNSW/quantized indexes
│       → exact reranking
│
├── RRFlow inference — replaceable embedding and LFG adapters
├── RRFlow Security — identity, sessions, policy, authorization, and audit
└── HTTP / WebSocket / SDK / MCP / CLI / Connectome — outward clients
```

These are cooperating layers of one system. They are not separately
authoritative databases or engines.

## Canonical component terminology

| Term | Canonical meaning | Implementation boundary |
|---|---|---|
| **RRFlow** | Reason Ready Flow, the complete product and platform. | Complete workspace and separate Connectome client. |
| **RRD** | Reason Ready Daemon, the embedded/server runtime hosting RRFlow. | `rrd-server` plus embedded `RrdEngine` composition. |
| **`RrdEngine`** | The only semantic coordinator and mutation authority. | `crates/authority/rrd-engine` |
| **rrflowDB** | One persistent project AI estate containing governed temporal knowledge, reasoning state, evidence, and index definitions. | Semantic state composed by `RrdEngine`; physical durability supplied by rrflowKV. |
| **rrflowKV** | The durable physical engine beneath rrflowDB: WAL, MVCC, LSM, manifests, snapshots, compaction, and recovery. | `rrd-lsm` through `rrd-store` |
| **rrflowMX** | RRFlow Memory Execution, the non-durable process-local implementation of the same semantic storage port. | `RrflowMxEngine` through `rrd-store` |
| **rrflowQL** | RRFlow Query Language: syntax, AST, binding, logical and physical planning, and execution. | `rrd-query` |
| **Arrow substrate** | The columnar buffer model for eligible immutable rrflowKV pages and stamped analytical batches. | Apache Arrow types composed by storage and query boundaries. |
| **DataFusion execution** | Bounded vectorized computation over stamped Arrow batches. It cannot authorize or commit state. | DataFusion integration inside `rrd-query` |
| **RRFlow vector subsystem** | Native vector-database capability inside rrflowDB: exact values, payload filters, candidate indexes, and exact reranking. | `rrd-vector`, `rrd-query`, and `RrdEngine` |
| **RRFlow inference** | Provider-neutral execution of embedding and routing models. | `rrd-inference`; LFG is a constrained routing adapter. |
| **Connectome** | Separate operator and developer workbench using public RRD capabilities. | Separate repository; no embedded engine authority. |

Product documentation uses the exact spellings **RRFlow**, **RRD**,
**`RrdEngine`**, **rrflowDB**, **rrflowKV**, **rrflowMX**, and **rrflowQL**.
Rust package names remain `rrd-*` for internal daemon/runtime boundaries and
`rrflow-*` for outward product adapters.

## Persistence and memory boundary

rrflowDB is the persistent logical estate. rrflowKV is its physical storage
engine. The rrflowKV memtable is the hot mutable tier inside a persistent
rrflowDB instance.

rrflowMX is different: it is an intentionally non-durable composition for
volatile and conformance execution. It does not silently flush or promote into
rrflowDB. Moving state from rrflowMX to rrflowDB requires an explicit,
authorized `RrdEngine` transaction whose result is subject to the ordinary
schema, policy, audit, and durability rules.

## Native multi-model boundary

Temporal records, claims, relations, reasoning trees, graph adjacency,
lexical postings, vectors, and evidence are rrflowDB data families. They share
one transaction coordinator, read-stamp model, canonical runtime log, and
persistence authority.

The temporal graph is not another graph database. RRFlow vector search is not
a separately authoritative vector database. BM25 is not another text store.
Exact vector values may be canonical rrflowDB values; HNSW and quantized
structures are derived candidate indexes and cannot outrank their source data.

## Query and analytical boundary

rrflowQL selects a native fast path or the analytical path. Fast point, range,
CAS, and bounded graph navigation does not require DataFusion. Analytical work
reads one authorized stamp, obtains native KV/graph/lexical/vector candidates,
streams eligible rrflowKV state as Arrow batches, and evaluates bounded
DataFusion operators before deterministic exact reranking or reciprocal-rank
fusion.

DataFusion computes. It never owns the database, WAL, authorization decision,
read stamp, or commit. Any transformed result returns to `RrdEngine` as a
proposal and passes semantic validation and authorization before persistence.

The accepted detailed flow and current implementation differences are owned
by [engine-data-flow.md](engine-data-flow.md). In particular, the present
checkout still materializes rows into newly allocated Arrow arrays and has not
implemented the target Arrow-compatible rrflowKV segment pages.

## Security boundary

RRFlow Security is cross-cutting and enforced by `RrdEngine`:

| Boundary | Required authority | Prohibited bypass |
|---|---|---|
| Ingress | Typed public operation and bounded request | Transport-specific semantic execution |
| Identity and session | Authenticated principal, lease, and instance scope | Provider identity becoming an RRFlow authority |
| Read | Policy, valid time, and one `ReadStamp` | Index, model, or client selecting unstamped state |
| Compute | Authorized plan and resource budget | DataFusion or LFG granting permissions |
| Mutation | Schema validation, effect authorization, and transaction coordination | Direct model, adapter, hook, or DataFusion storage writes |
| Evidence | Correlated audit, plan, source, and commit coordinates | Trace events substituting for canonical state |

## External integrations

PostgreSQL, Turso, SQLite, Dragonfly, application databases, object stores,
mesh networks, and model providers are operator-configured external
integrations. Attunement may discover them but cannot activate them. They never
become rrflowDB's implicit persistence, transaction, query, security, or
context authority.

## Documentation and future memory

During bootstrap, Markdown records remain the reviewed authoring authority.
The planned deterministic JSON/JSONL knowledge package will be a generated,
content-addressed import artifact rather than a second editable truth. Detailed
records move behind rrflowDB warp points only after authorized import,
readback, restart, idempotency, drift-denial, and recovery evidence passes.

The root and boundary READMEs then remain portals. Architecture explanations
live here, decisions live under `docs/decisions/`, exact contracts live under
`docs/reference/`, delivery order lives in the roadmap, and observed gaps live
in the POA&M.
