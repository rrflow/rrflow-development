# RRFlow database semantics and implementation boundaries

**Status:** active source-grounded reference; multi-model and protocol gaps remain open
**Coordinate:** `rrflow://rrflow-instance/data/reference/data/database-semantics`
**Owner:** concrete DB/QL/transaction/storage semantics; [system overview](../../architecture/system-overview.md) owns component names and [roadmap](../../roadmap/rrflow-1.0.md) owns completion

RRFlow is intended to be one bitemporal, multi-model reasoning database. A
document, relationship, claim, vector and their indexes must agree about the
transaction and historical coordinate being read. Separate implementations of
those algorithms do not mean separate authoritative databases. This reference
distinguishes that design from what the checkout can actually execute. The
[installed proof journal](../../evidence/change-journals/gate-h/bitemporal-public-transactions-20261122.md)
binds the examined baseline and commands; this is not full SurrealDB parity.

## What the component names mean

| Component | Responsibility and current implementation | What it must not imply |
|---|---|---|
| rrflowDB | Logical documents/records, relations, claims, events, vectors, schema and provenance, composed by `RrdEngine`. `rrd-core` defines values; `rrd-store` maps them into one semantic commit. | A folder of JSON documents, a second graph database, or an independent vector transaction authority. |
| rrflowQL | Parse/bind temporal queries, select bounded physical access, execute and explain selected/rejected paths. Native record/relationship reads and BM25 exist; eligible analytics use stamped Arrow/DataFusion batches. | Complete SurrealQL/PostgreSQL syntax, every SQL feature, or all vector operators in the textual planner. |
| rrflowTX | The requested name for temporal transaction coordination. Today the implementation is `RrdEngine` session transactions and `SemanticCommitPlan`, not a separate crate or service. QL mutation programs delegate to that authority. | A second commit log, an independently committing query planner, or automatic commits from a controller. |
| rrflowKV | Consistent, bounded physical access through the storage port. Its durable implementation uses native rrflowLSM, snapshots, manifests and recovery. | SQL planning, model-provider decisions or automatic historical deletion. |
| rrflowLSM | Native WAL, mutable versions, immutable segments, checkpoint/compaction and MVCC snapshots in `rrd-lsm`. It retains historical versions as well as current state. | Only rows with infinite valid/system end times, demonstrated Fjall parity, or unrestricted concurrent writers to one directory. |
| rrflowCLD | Intended columnar historical/analytical projection beneath KV. Current product publication is Parquet; authenticated bounded cold-generation readback exists. A separate `ColumnarFormat` port has optional Parquet/Vortex implementations, but production publication/readback does not use it. | A finished native CLD format, Z-ordering, automatic object-store placement, history eviction, or a unified QL hot/cold scan. |
| rrflowMX | Existing volatile implementation of the semantic storage port. It is distinct from the durable LSM's memtable. The owning architecture records its planned convergence. | Uncommitted durable writes, an automatic RAM-to-WAL flush, or an implemented rename to rrflowMEM/rrflowRAM. |
| TurboQuant | A derived vector candidate index, governed by collection, generation, source stamp, eligibility and search mode; canonical vectors remain available for exact scoring. | Authoritative data, universal exact recall, or a required stage for every query. |

Source boundaries: [kernel values](../../../crates/kernel/rrd-core/src/runtime/model.rs),
[semantic commit plan](../../../crates/persistence/rrd-store/src/access/semantic_commit/plan.rs),
[QL planner](../../../crates/compute/rrd-query/src/plan/planner.rs),
[physical transactions](../../../crates/persistence/rrd-lsm/src/database/transactions.rs),
[cold publication](../../../crates/authority/rrd-engine/src/engine/cold_history.rs),
[columnar port](../../../crates/persistence/rrd-store/src/columnar.rs).

## Two temporal coordinates, not merely a timestamp column

**Valid time** answers when a fact applies in the modeled world. Records and
relations carry `valid_from` and optional `valid_to`. **Knowledge/system time**
answers which committed information the engine was allowed to know. The current
query contract represents that second axis with a committed cursor, not a SQL
wall-clock `sys_from`/`sys_to` pair. A read stamp additionally binds scope,
schema/catalogue revisions, head digest and accumulator identity.

Suppose `document:alpha` says "Original", effective at valid time 100, committed
at cursor C1. A later commit C2 corrects the same identity to "Corrected", also
effective at 100. The answers must be:

| Valid time | Known cursor | Answer |
|---|---|---|
| 99 | C2 | No applicable document |
| 100 | C1 | Original |
| 100 | C2 | Corrected |
| 100 | C1, after restart | Original again |

Actual native syntax, substituting the returned numeric cursors for C1/C2:

```text
FROM record:document AT VALID 100 KNOWN 4
WHERE id = "alpha" PROJECT title EXPLAIN CONTRACT
```

`KNOWN HEAD` selects current knowledge. It is not a reproducible historical
coordinate until the returned cursor/stamp is retained. The parser also accepts
SQL-style `SELECT` projections for relational sources; it does not thereby
accept arbitrary SQL or the proposed `AS OF` spelling. `SELECT` over a traversal
source is explicitly rejected. See [parser](../../../crates/compute/rrd-query/src/syntax/parser.rs)
and [read stamps](../../../crates/kernel/rrd-core/src/runtime/stamps.rs).

Version keys contain scope, kind, identity, descending effective time and
descending commit cursor. Current materializations accelerate present reads;
immutable versions support historical resolution. Retiring data appends an
effective retirement rather than rewriting the old answer. Physical placement
is separate: recalling an old note does not require moving its authoritative
record back into RAM or creating another canonical copy.

## Documents, links, references and graph edges are different contracts

SurrealDB's published design combines models over one query/storage boundary.
That is the useful architectural comparison, not permission to claim that any
Rust KV store supplies its full behavior. Its documented 4.x architecture
describes snapshot isolation and commit-time write conflict detection, with
stronger isolation available for specified backends. [SurrealDB architecture](https://surrealdb.com/docs/learn/data-models/architecture).

| Construct | Exact behavior being requested | Current RRFlow boundary |
|---|---|---|
| Document | Addressable identity plus nested typed properties, schema modes, validation and indexed retrieval. | `RuntimeRecord` has a `RuntimeRef`, valid interval and properties. `RuntimeValue` includes maps/lists and scalar types. Strict and schemaless table definitions exist. |
| Record link | A field contains a typed record identity; traversing `author.name` dereferences that identity. | `RuntimeRef` exists for record/relation identities, but `RuntimeValue` has no record-reference variant. A string containing `author:tobie` is not a typed link and does not implement automatic dereferencing. |
| Tracked record reference | A schema-declared link maintains incoming references and executes the declared delete policy atomically. | No equivalent property-reference type, reverse-reference catalogue or CASCADE/UNSET/REJECT field policy was found in the reviewed kernel/store/query boundaries. Relation adjacency is not this feature. |
| Graph edge | A first-class relation with its own ID, typed endpoints, metadata and temporal interval; outgoing/incoming traversal. | `RuntimeRelation` stores `from`, `to`, properties and valid interval. Commit effects write its current record, immutable version and directional adjacency keys in the same write set. Relation schemas constrain endpoint kinds, pair uniqueness and degree. |

SurrealDB record links target record IDs directly; tracked references add
reverse traversal and deletion behavior. Its policies include IGNORE, UNSET,
CASCADE, REJECT and custom THEN behavior. CASCADE can delete the referencing
record; it is not merely deletion of an edge. RRFlow must define and test the
temporal consequences before implementing those policies. [Record links](https://surrealdb.com/docs/reference/query-language/language-primitives/record-links),
[record references](https://surrealdb.com/docs/reference/query-language/language-primitives/record-references).

An exact ID permits a targeted lookup instead of scanning an entire table.
Calling this universally **O(1)** is unjustified: LSM levels, indexes, I/O and
visible-version selection still contribute work. RRFlow's adjacency keys are
ordered by scope, endpoint, relation kind, opposite endpoint and relation ID,
supporting bounded prefix scans. Historical traversal must resolve relation
versions at the requested stamp rather than trusting only current adjacency.
See [key encoding](../../../crates/persistence/rrd-store/src/keyspaces/runtime.rs),
[write effects](../../../crates/persistence/rrd-store/src/access/semantic_commit/effects.rs)
and [constraints](../../../crates/persistence/rrd-store/src/access/semantic_commit/constraints.rs).
SurrealDB's `RELATE` similarly creates edge records; this statement does not
assert identical internal key bytes. [RELATE](https://surrealdb.com/docs/reference/query-language/statements/relate).

## Transaction path and concurrency

```text
authenticated operation + idempotency key + deadline
  -> RrdEngine: scope, permissions, transaction lease/read identity
  -> rrflowQL: parse program; bind exact typed mutations in declared order
  -> existing engine commit: validate schema, constraints, expected cursor
  -> SemanticCommitPlan: records + versions + relationships + indexes + receipt
  -> one native storage transaction / WAL publication
  -> durable receipt or explicit failure
```

The public query transaction operation is `query-transaction-execute`, POST
`/v1/query/transactions`. It uses the existing `ExecuteQueryTransaction` payload:
`scope`, `program`, `mutation_bindings`, and `timeout_ms`. For example:

```text
BEGIN; MUTATE $document; MUTATE $relationship; COMMIT;
```

Bindings contain the existing tagged `TransactionMutation` values; they are
not executable strings. Missing/extra bindings are rejected. `CANCEL` uses the
existing abort path. The engine checks begin and commit/abort permissions;
query-read permission alone is insufficient. Retry the identical intent with
the identical idempotency key after an uncertain response. Reusing that key for
different intent is a conflict. This is a bounded native transaction grammar,
not SQL INSERT/UPDATE/DELETE or arbitrary multi-statement SQL support.

The current implementation must be evaluated separately for each ACID property:

- **Atomicity:** semantic family changes and the commit receipt share one write
  plan. A partially valid batch must not publish its earlier valid mutation.
- **Consistency:** schema, references used by relations, uniqueness, budgets and
  authorization are checked at the engine boundary. Missing record-reference
  deletion rules cannot be claimed as enforced consistency rules.
- **Isolation:** physical transactions use snapshots and detect conflicts on
  written keys. The engine additionally binds semantic commits to a global
  expected cursor, so even disjoint work may require a retry. This is not proof
  of serializable read predicates or parallel independent commit execution.
- **Durability:** the selected native durability mode, WAL sync and recovery
  determine acknowledgment guarantees. Reopen tests are necessary but do not
  substitute for power-loss/fault qualification on supported filesystems.

Multiple agents can submit work concurrently and hold distinct transaction
leases. That does not authorize multiple processes independently writing the
same mutable storage root. Each project instance owns its root and exposes
requests through its service. Scratchpads need explicit lifetime, owner, quota
and admission; an in-memory profile is not an automatic durable flush policy.
No hardware-lifetime improvement follows from an API label: measure WAL bytes,
syncs, write amplification, compaction and recovery against the actual workload.

## Query, vector and storage eligibility

The QL planner records selected/rejected candidates and exactness. Index
eligibility checks readiness and source coordinates. BM25 has a bounded
MATCH path and source-bound lexical artifacts. HNSW/TurboQuant search is
implemented in the vector subsystem and exposed through vector operations;
it is not a general textual nearest-neighbor operator in this QL parser.
The engine also contains a typed retrieval-query composer; its existence must
be distinguished from public transport/client integration.

TurboQuant's eligible index generations nominate candidates; current changes
and canonical vectors are checked at the read coordinate. Exact reranking
corrects candidate scores, **not candidates that the approximate search missed**.
Report candidate count, source generation, selected mode, rerank width, work
and measured recall against an exact baseline. Neither recall parity nor speed
against Qdrant is established by the implementation's presence. See
[candidate/delta path](../../../crates/authority/rrd-engine/src/engine/vector/search/operations.rs)
and [prepared search](../../../crates/compute/rrd-vector/src/runtime/search.rs).

Cold history currently publishes bounded Parquet generations and can verify
their digest/source stamp during explicit authenticated readback. This is
different from making rrflowQL merge hot and cold sources under one stamp.
Native CLD format selection, production port integration, pruning/pushdown and
long-term representation benchmarks remain [POAM-090](../../poam/rrflow-1.0-alpha/poam-090.md).
Keep authoritative versions until coverage, recovery and retention are proven.

Two existing transaction/relationship gaps must remain visible: retiring an
endpoint can leave its relation live ([POAM-072](../../poam/rrflow-1.0-alpha/poam-072.md)),
and a refused commit can leave its transaction lease open until abort/expiry
([POAM-073](../../poam/rrflow-1.0-alpha/poam-073.md)). Exposing a QL transaction
program does not repair either behavior. They prevent an unqualified claim of
SurrealDB-style referential integrity or complete transaction lifecycle parity.

## Serving and Connectome project identity

Adding a folder first canonicalizes its absolute path. On Windows the owner's
`d:/localdev/devlabs/rrflow` and `D:\localdev\devlabs\rrflow` identify the same
directory; the retained preview evidence confirms equal installation identities
and digests. That is a directory-identity check, not permission to index files.

The desktop currently uses `.rrflow/config.toml` as the installed-project
locator. An absent locator leads to installation preview; an existing locator
leads to startup/attachment and engine validation. File existence alone is not
health: an invalid locator or mismatched executable must fail visibly. New
installation creates configuration, credentials, recovery/ownership records and
a native storage root. The preview's managed-path count is not a record count,
database schema or engine capability. Durable logical records live in the
native database. Connectome starts `serve --bind 127.0.0.1:0`, records the actual
port, and checks authenticated readiness before presenting a connected project.

The [development loop](../client/connectome.md#development-loop) builds this
checkout's engine and binds the desktop explicitly to that binary. Engine and
UI revisions, executable digest, instance identity, endpoint, source stamp and
actual errors must remain inspectable. Hosted synchronization, automatic
discovery of every local instance and idle lifecycle are separate unfinished
capabilities; an added local project does not demonstrate them.

| Interface | Current distinction |
|---|---|
| HTTP | Native authenticated operation catalogue and real server dispatch. This package connects query transaction programs to it. |
| WebSocket | Native protocol/subscription implementation; not SurrealDB RPC compatibility. |
| GraphQL | Internal schema derivation/lowering and equivalence tests exist. They do not prove a complete authenticated GraphQL listener/client surface. |
| PostgreSQL wire | Product requirement under H-08, not an implemented listener established by this audit. SurrealDB documents its own opt-in PG listener; RRFlow cannot inherit that capability from similar naming. |
| Browser persistence/OPFS | No rrflowLSM OPFS backend established. Connectome's inherited browser dependencies are not native RRFlow persistence. |

SurrealDB's current documentation labels its browser SurrealKV/OPFS and
PostgreSQL-dialect behavior as **4.x**. Cite that version distinction rather
than combining older IndexedDB/SurrealQL protocol behavior into one claim.
[Embedding](https://surrealdb.com/docs/integrate/embedding),
[Postgres protocol](https://surrealdb.com/docs/reference/protocols/postgres-protocol).

## Delivery evidence still required

The owning roadmap remains responsible for native record-link/reference
semantics, temporal referential actions, unified query/vector/cold planning,
full multi-model isolation/fault qualification, protocol delivery and complete
Connectome controls. Each must have a failing real-engine oracle, canonical
implementation and observed UI/public-client result. The public transaction
proof advances one missing integration boundary; it does not close those gaps.
