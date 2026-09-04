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
| **RRFlow kernel** | The canonical temporal values, identities, read stamps, mutations, and invariants implemented in `rrd-core`. The kernel defines meaning but does not own transport or physical storage. |
| **canonical runtime log** | The ordered source of truth for committed RRFlow changes. Temporal snapshots are resolved from this log. |
| **rrflowKV** | RRFlow's native ordered MVCC/LSM physical layer, implemented by `rrd-lsm` and adapted through `rrd-store`. One semantic engine commit becomes one atomic rrflowKV write batch. rrflowKV is not a second engine or public data model. |
| **RRFlow temporal graph** | Typed records and relations resolved at a runtime read stamp and valid-time coordinate from the same canonical log. It is not a separate graph database. |
| **RRFlow memory** | Durable temporal knowledge—claims, records, relations, schemas, vectors, and evidence—owned by the same engine. It is not a separate memory store. |
| **RRFlowQL** | RRFlow's query language. Use **RRFlowQL**, not the ambiguous shorthand “QL,” in product documentation. |
| **Arrow working set** | Typed, immutable, rebuildable columnar batches bound to an RRFlow read stamp. Arrow is the hot analytical representation, not the mutable LSM or source of truth. |
| **DataFusion execution** | Vectorized physical evaluation over stamped Arrow batches after RRFlow selects authoritative KV, graph, lexical, or vector access paths. DataFusion is not the database authority. |
| **fast path** | Low-latency state, pointer, and bounded graph navigation performed by `RrdEngine` directly against rrflowKV without invoking DataFusion. |
| **analytical path** | RRFlowQL planning plus native graph, lexical, vector, and RRF operators composed with Arrow/DataFusion for broad retrieval, ingestion, joins, and analytics. |
| **index projection** | Derived acceleration state bound to its source cursor and relevant schema or catalogue revision. An index can be rebuilt and cannot outrank the canonical log. |
| **context assembly** | Bounded retrieval and deterministic fusion performed by `RrdEngine::assemble_context`; the result is a `ContextPacket` with evidence and its read stamp. |
| **LFG** | A replaceable local routing-model adapter. LFG may propose a recipe, branch, or bounded query intent; it never owns state, permissions, physical plans, or mutations. |
| **Connectome** | RRFlow's separate client and operator workbench. It observes and invokes RRFlow through public RRD capabilities and never recreates engine logic. |

For citations, use **RRFlow database**, **RRFlow kernel**, **rrflowKV**,
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
coordination, rrflowKV, and the canonical runtime log:

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
 rrflowKV point/range ops                  |
        |                         DataFusion + native RRF
        +--------------------+------------+
                             |
                  one authorized transaction
                             |
             rrflowKV WAL -> memtable -> segments
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
| `rrd-store` | semantic-to-rrflowKV mapping, canonical log/materialized-key maintenance, and storage conformance port |
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
│   └── rrd-store               # semantic/KV mapping and conformance port
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

- a versioned, provider-neutral reasoning-tree contract with typed nodes and
  edges, content-addressed recipes/evidence, read-stamped cursors, and
  fail-closed cursor-advance verification;
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

- reasoning-tree persistence and execution through `RrdEngine` are not yet
  implemented; A-02 freezes their semantics but does not claim a running
  router;
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
- mesh endpoint discovery, the canonical attunement job, and engine-owned
  trigger/routine/hook-adapter/skill operations are not yet implemented.
- standalone Connectome is not yet RRFlow 1.0-conformant: its inherited UI and
  runtime paths have not been consolidated onto the public RRD client, so it
  must not claim the 1.0 release version yet.

These are material gaps. A successful compile is not evidence that context
flows correctly, and none of the gaps above is represented as complete.

## RRFlow 1.0 execution checklist

This checklist is the release order, not an inventory of aspirations. Work may
not skip a gate because a later subsystem already has partial code. A checkbox
changes to `[x]` only in the same reviewed change that supplies its required
behavioral evidence.

Checklist rules:

- execute gates in dependency order
  `A -> B -> C -> D -> E -> F -> G -> H -> I -> J`;
- keep one checklist item per coherent commit unless two items cannot be tested
  independently;
- update **Current status** when an item changes observable product behavior;
- do not retain compatibility adapters, aliases, deprecated entrypoints, or
  dual-write paths in the RRFlow 1.0 result;
- do not count compilation, mocked UI state, generated schemas, or an artifact
  file existing as behavioral proof;
- require exact/reference comparison before enabling an approximate index;
- require close/reopen evidence for persisted state and crash/failure evidence
  for acknowledged writes;
- require every public surface to reach the same `RrdEngine` operation; and
- stop at the first failed gate, repair it, and rerun the smallest owning test
  before continuing.

The next executable item is **B-01**. Gate A is complete: authority and names
are frozen, the generic reasoning-tree contract is established, the obsolete
fixed-stage lifecycle is absent, packages use the canonical grouped layout,
and supporting documents are subordinate to this README.

### Gate A — freeze authority, names, and boundaries

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [x] | A-01 | Define RRFlow, RRD, RRFlow kernel, rrflowKV, RRFlowQL, Arrow/DataFusion analytical path, LFG, and Connectome exactly once. | `README.md` | Terminology table, execution topology, and ownership table use one meaning for every term. |
| [x] | A-02 | Define generic `reasoning_tree`, `reasoning_node`, typed `reasoning_edge`, recipe, active cursor, decision evidence, and verification-result semantics. | `rrd-contract`, `rrd-core` | Versioned schema and golden round trips reject unknown fields, invalid edges, and unverifiable cursor advances. |
| [x] | A-03 | Resolve the pending reasoning-ledger removal against A-02 without restoring a hard-coded universal reasoning lifecycle or deleting reusable semantics. | `rrd-core`, `rrd-engine`, CLI | Golden/API diff proves reusable data moved to the generic contract, contains no forced Goal→Plan→Attempt sequence, and focused core, engine, and CLI tests pass. |
| [x] | A-04 | Move crates into the canonical grouped source tree, remove the empty `rrd-graph` boundary, and remove `connectome-ui` after its public-client behavior is present in the separate Connectome repository. | workspace | `cargo metadata`, dependency-direction check, and repository search show the declared layout and no second graph, memory, routing, lifecycle, UI, or provider authority. |
| [x] | A-05 | Remove stale documentation claims or mark supporting documents historical where they describe another architecture. | documentation | Repository link/terminology check finds no supporting document presented as current authority. |

A-02 evidence (2026-09-04):

- `crates/kernel/rrd-core/tests/fixtures/reasoning-tree-v1.json` is the shared frozen
  wire vector used by the kernel and public contract.
- The seven focused reasoning-tree tests reject unknown fields and versions,
  malformed edge topology, missing condition evidence, missing verification,
  mismatched edge selection, and changed read stamps.
- `cargo test -p rrd-core -p rrd-contract` passed all 105 tests in the isolated
  A-02 candidate tree, and `cargo clippy -p rrd-core -p rrd-contract
  --all-targets -- -D warnings` passed.
- `public_contract_and_client_stay_implementation_free` proves `rrd-contract`
  retains zero production workspace dependencies; `rrd-core` is used only by
  the cross-boundary conformance test.

A-03 evidence (2026-09-04):

- The fixed-stage ledger module, engine projection/API, automatic global trace
  lookup, and CLI record/show surface are absent; the legacy golden entry was
  removed rather than treated as a supported wire format.
- Reusable source/digest/summary evidence and verification status now live in
  the A-02 generic types, while `TraceLink::ReasoningCursor` preserves exact
  tree, revision, node, step, and read-manifest correlation.
- `retired_fixed_reasoning_ledger_api_is_absent` and the compiled CLI rejection
  test prevent the removed symbols and commands from returning.
- `cargo test -p rrd-core -p rrd-engine -p rrflow-cli` passed all 200 tests, and
  `cargo clippy -p rrd-core -p rrd-engine -p rrflow-cli --all-targets -- -D
  warnings` passed.

A-04 evidence (2026-09-04):

- The 20 current packages live only under `kernel`, `persistence`, `compute`,
  `authority`, `transport`, `adapters`, `operations`, and `evaluation`;
  `workspace_packages_use_the_canonical_grouped_layout` compares every package
  to its exact manifest path and rejects any additional top-level crate group.
- The empty graph boundary and the in-repository Connectome package are absent.
  The separate Connectome repository commit `38f68ce7` supplies its native RRD
  capability handshake, authenticated session, bounded diagnostic/context
  calls, renewal, and close; its Rust tests, strict Clippy, TypeScript check,
  Biome check, and aggregate `pnpm check` passed.
- `rrflow dev` now supervises only RRD and reports a client endpoint, principal,
  and private credential path. A real-process smoke started the daemon, probed
  readiness and capabilities, observed ready status, and stopped cleanly.
- `cargo metadata --locked`, all 16 workspace-architecture tests, `cargo check
  --workspace --all-targets --locked`, `cargo test --workspace --all-targets
  --locked`, and `cargo clippy --workspace --all-targets --locked -- -D
  warnings` passed.
- Version, recursive CI package/feature routing, and generated-surface policy
  checks passed with 20 default-feature packages, five optional-feature
  packages, and 33 contract-derived HTTP operations.

A-05 evidence (2026-09-04):

- All 59 supporting semantic, design, implementation, evidence, research, and
  history documents declare status in their first 12 lines; the root README is
  the only product-architecture, current-status, and roadmap authority.
- Superseded runtime, in-repository Connectome, provider-flight, Clyffy alpha,
  and architecture-triage documents are explicitly historical. Active status
  lines cannot use the retired F/G/M/Q milestone scheme.
- Stale alternate-authority claims, the removed UI/graph layout, obsolete CI
  package counts/topology, a hard-coded branch, and five broken local document
  links were corrected.
- `scripts/ci/check_documentation.py` validates README authority, supporting
  status, prohibited alternate authority/layout claims, and repository-local
  links in CI. The documentation policy, CI policy, Ruff, `git diff --check`,
  and all 16 workspace-architecture tests passed.

Gate A exits only when the worktree contains one architecture, the generic tree
contract is frozen, and the obsolete lifecycle implementation is either
removed with passing evidence or explicitly retained by the canonical contract.

### Gate B — freeze public and model-neutral contracts

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | B-01 | Define install plan, installation result, attunement plan, job, phase checkpoint, status, resume, cancel, and verification envelopes. | `rrd-contract` | Golden JSON and generated schema tests cover every state transition and reject skipped phases or mismatched digests. |
| [ ] | B-02 | Define `RouterBackendDescriptor`, `RouteStepRequest`, and the `select_recipe`, `advance_branch`, and `request_context` decision variants. | `rrd-contract` | Golden vectors prove model/provider neutrality, strict fields, bounded inputs, and stable digests. |
| [ ] | B-03 | Define the LFG model-manifest handshake: model/tokenizer digests, routing schema digest, capabilities, limits, runtime, and quantization. | `rrd-contract`, `rrd-inference` | Mismatched contract, model, tokenizer, or resource declarations fail before inference. |
| [ ] | B-04 | Define one multiplexed WebSocket frame protocol for authenticated request/response, cancellation, subscription, ACK, and backpressure. | `rrd-contract` | Codec golden tests prove correlation, ordering, limits, unknown-frame rejection, and reconnect resume coordinates. |
| [ ] | B-05 | Define GraphQL as a schema-derived ingress adapter that lowers into the same bound RRFlow query representation. | `rrd-contract`, `rrd-query` | Equivalence fixtures show GraphQL and RRFlowQL produce the same authorized logical request without a second executor. |

Gate B exits only when other languages and LFG can implement the contracts from
golden vectors without importing Rust internals.

### Gate C — make rrflowKV the only local persistent substrate

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | C-01 | Freeze one ordered binary key codec for current records, temporal versions, outgoing/incoming edges, scalar values, term postings, vectors, projection deltas, catalogue state, and runtime commits. | `rrd-core`, `rrd-store` | Ordering/golden tests prove prefix boundaries, round trips, tenant separation, and malformed-key rejection. |
| [ ] | C-02 | Expose the minimal snapshot transaction primitives required by the semantic store: point read, bounded range scan, put, delete, commit, rollback, and conflict. | `rrd-lsm`, `rrd-store` | Native and memory conformance suites agree on read-your-writes, repeatable reads, range ordering, and write conflicts. |
| [ ] | C-03 | Commit canonical record, relation, both adjacency directions, synchronous index changes, runtime log entry, and durable projection deltas as one write batch. | `rrd-store`, `rrd-engine` | Failure injection at every WAL/batch boundary proves all-or-nothing behavior after reopen. |
| [ ] | C-04 | Serve current and temporal reads from direct versioned keys at one `ReadStamp`; remove normal-path whole-log reconstruction. | `rrd-store` | Physical counters and plan evidence show bounded point/range reads while exact snapshot comparisons remain equal. |
| [ ] | C-05 | Remove Fjall selection, compatibility readers, migration-only runtime paths, legacy format branching, and associated dependencies from the 1.0 executable. | `rrd-store`, workspace | Fresh native database tests pass; repository search and dependency metadata contain no Fjall/compatibility execution path. |
| [ ] | C-06 | Prove WAL recovery, manifest recovery, pinned-snapshot compaction, checksums, storage-full behavior, and acknowledged-write durability. | `rrd-lsm` | Crash matrix and reopen suite pass repeatedly with no lost acknowledged write or exposed partial batch. |

Gate C exits only when rrflowKV is the sole local persistent implementation and
its correctness is demonstrated below the semantic engine.

### Gate D — install, configure, and attune one real estate

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | D-01 | Implement `rrflow install` with preview/apply behavior and a minimal `.rrflow/config.toml` locator containing no canonical mutable state or plaintext secret. | `rrflow-cli`, `rrd-engine` | Fresh-project test initializes, authenticates, closes, and reopens the same instance; preview performs no writes. |
| [ ] | D-02 | Persist attunement jobs and checkpoints through `RrdEngine`; implement status, resume, cancel, leases, idempotency, and phase input/output digests. | `rrd-engine` | Kill/restart tests at each transition resume committed work once and never infer completion from emitted events. |
| [ ] | D-03 | Implement only the inventory phase first: ignore rules, secret/generated/cache exclusions, content digests, source classification, and bounded work estimates. | `rrd-attunement` through `rrd-engine` | This repository inventories without `target`, `node_modules`, `.git`, RRFlow database files, or secret payloads; unchanged rerun performs no content work. |
| [ ] | D-04 | Add incremental Tree-sitter parsing with parser/language revision and source-digest provenance. | `rrd-attunement` through `rrd-engine` | Edit-one-file test reparses the changed source, preserves unaffected identities, and resumes after process restart. |
| [ ] | D-05 | Implement normalize, entity-link, lexical-index, embed, vector-index, graph, ground, and verify one at a time. | `rrd-engine` plus owning subsystem | Every phase has an exact fixture, durable checkpoint, failure/retry case, output digest, and independent acceptance test before the next phase begins. |
| [ ] | D-06 | Classify SQL, PostgreSQL, Turso, and other application/operator databases as external sources; never select them as RRFlow persistence implicitly. | `rrd-attunement`, operator adapters | Fixture project proves discovery creates governed source metadata without copying credentials, changing the application database, or creating another RRFlow authority. |
| [ ] | D-07 | Bind installation to the DevForge CoW placement contract: immutable tools/models may live in the shared lower layer; workspace changes and all rrflowKV WAL, manifest, segment, catalogue, and graph state live in the writable upper layer. | `rrflow-cli`, DevForge adapter | Two clones share the same lower digest while independent writes, crash recovery, and deletion in one upper layer cannot affect the other. |
| [ ] | D-08 | Recognize content-addressed dependency mounts as shared immutable inputs rather than copying or attuning dependency caches into each estate. | `rrd-attunement`, DevForge adapter | Rust, Go, Node, and model-cache fixture proves stable mount digests, zero duplicate ingestion, and correct invalidation when a mounted digest changes. |
| [ ] | D-09 | Measure logical size, allocated blocks, compression, WAL growth, and snapshot size separately; never infer zero-byte or sparse-allocation savings from logical file size. | `rrd-lsm`, release harness | Fresh clone and sustained-write reports account for lower, upper, cache, WAL, segment, and snapshot bytes with reproducible filesystem commands. |
| [ ] | D-10 | Add a hibernation preparation/restore contract that quiesces writes, captures a verified rrflowKV snapshot boundary, exports to the configured cold tier, and resumes without changing estate identity. | `rrd-engine`, DevForge adapter | Interrupted export, corrupt object, restore, rollback, and hot-to-cold-to-hot tests prove no acknowledged-write loss and no split authority. |

Gate D exits only when a fresh and an existing project can be installed,
attuned, interrupted, resumed, verified, and reopened through public engine
operations.

### Gate E — make graph and indexes native incremental access paths

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | E-01 | Replace graph reconstruction and linear relation scans with temporal outgoing/incoming adjacency prefix scans. | `rrd-store`, `rrd-query` | Directed/typed/depth-bounded traversal matches the exact graph oracle and physical evidence scales with visited edges, not estate size. |
| [ ] | E-02 | Persist scalar and unique indexes transactionally with record mutations. | `rrd-store`, `rrd-query` | Insert/update/retire/conflict/reopen differential proves index and authoritative record cannot drift. |
| [ ] | E-03 | Persist incremental BM25 dictionary, document statistics, postings, positions, and tombstones at a declared source cursor. | `rrd-query`, `rrd-store` | Incremental results equal a full exact rebuild across update/delete/reopen/corruption fixtures. |
| [ ] | E-04 | Commit canonical vectors with an atomic index delta; search immutable HNSW generation plus exact delta overlay and exact-rerank final candidates. | `rrd-vector`, `rrd-store`, `rrd-engine` | Exact oracle, recall@k, filtered search, update/delete, stale generation, reopen, and interrupted-build tests pass. |
| [ ] | E-05 | Add cost/selectivity estimates choosing point, range, scalar, BM25, exact-vector, or HNSW access without caller-selected internals. | `rrd-query` | Stable explain plans and adversarial fixtures prove correctness fallback when statistics or projections are absent/stale. |

Gate E exits only when graph, lexical, scalar, and vector routes are real
bounded storage access paths with exact fallbacks.

### Gate F — connect rrflowKV to Arrow/DataFusion correctly

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | F-01 | Replace pre-materialized `Vec<QueryRow>` snapshots with a stamped `RrflowKvTableProvider` streaming bounded Arrow `RecordBatch` values from rrflowKV snapshots. | `rrd-query` | Provider tests prove batch streaming and fixed memory bounds on a data set larger than the allowed query memory. |
| [ ] | F-02 | Push supported projection, predicate, limit, and ordering requirements into rrflowKV scans; report unsupported predicates honestly. | `rrd-query`, `rrd-store` | Explain/physical-counter tests show fewer decoded values and bytes for selective queries while results equal the unoptimized oracle. |
| [ ] | F-03 | Implement graph expansion, BM25 candidate generation, HNSW candidate generation, and `math::rrf()` as native physical operators that exchange stamped Arrow batches with DataFusion. | `rrd-query`, `rrd-vector` | Mixed query tests prove one stamp, deterministic ordering, exact reranking, and no external database round trip. |
| [ ] | F-04 | Enforce query memory, spill, elapsed-time, scanned-key, graph-step, candidate, and result-byte budgets across native and DataFusion operators. | `rrd-query`, `rrd-engine` | Each limit has a deterministic truncation or denial fixture with measured resource evidence. |
| [ ] | F-05 | Add read-stamp/query/projection caches with byte accounting and cursor/schema invalidation. | `rrd-engine`, `rrd-query` | Repeated-query benchmark shows bounded reuse; mutation and schema tests prove stale batches are never returned. |

Gate F exits only when DataFusion consumes streamed authoritative access paths
instead of hiding an eager whole-estate materialization.

### Gate G — connect LFG without creating another engine

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | G-01 | Add a provider-neutral `RouterBackend` capability separate from `EmbeddingBackend`; implement LFG as one adapter. | `rrd-inference`, `rrd-engine` | Fake/reference adapter and LFG adapter pass the same descriptor, bounds, timeout, invalid-output, and digest checks. |
| [ ] | G-02 | Build a bounded route packet from one stamped tree, eligible recipes, verified observations, and allowed query fields. | `rrd-engine` | Golden packet excludes raw KV keys, secrets, hidden reasoning, unauthorized fields, and unbounded workspace content. |
| [ ] | G-03 | Grammar-constrain LFG to the three routing decisions and validate again after decoding. | LFG adapter | Corpus includes valid, malformed, unknown-recipe, unauthorized-query, stale-cursor, and prompt-injection cases; invalid decisions produce no mutation. |
| [ ] | G-04 | Execute recipe selection and branch navigation on the fast path without RRFlowQL/DataFusion; keep deterministic predicates and CAS mutation in `RrdEngine`. | `rrd-engine`, `rrd-store` | Trace and physical-plan evidence show bounded rrflowKV operations, no DataFusion plan, conflict denial, and correct reopen state. |
| [ ] | G-05 | Lower `request_context` into semantic RRFlowQL/context intent while leaving physical access selection to the engine. | `rrd-engine`, `rrd-query` | LFG cannot select an index/backend; resulting plan is authorized, stamped, budgeted, and equivalent to a typed SDK request. |
| [ ] | G-06 | Publish model and storage latency separately with task-success, routing-accuracy, invalid-decision, and escalation metrics. | evaluation harness | Reproducible hardware/model manifest and raw samples support every reported latency or quality claim. |

Gate G exits only when the trained LFG artifact passes the conformance corpus
and can steer a persisted tree without direct storage or planner authority.

### Gate H — prove context flow, feedback, live delivery, and Connectome

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | H-01 | Route each context request dynamically across eligible seed, BM25, vector, graph, and cached paths using the captured catalogue and budgets. | `rrd-engine` | Plan evidence states selected/skipped reason, source cursor, work, and contribution for every avenue. |
| [ ] | H-02 | Keep query-time RRF pure; persist explicit verified outcomes and learn versioned weight policies only for later read stamps. | `rrd-engine`, `rrd-query` | Replay at an old stamp is unchanged; feedback update, rollback, cold-start, and quality-regression tests pass. |
| [ ] | H-03 | Replace two-snapshot live-query diffing with commit-impact evaluation and predicate-specific deltas. | `rrd-query`, `rrd-engine` | Ordered update/delete/reconnect/backpressure tests emit each matching committed delta once without full-query rescans. |
| [ ] | H-04 | Serve HTTP, multiplexed WebSocket, Rust SDK, generated SDKs, CLI, MCP, and GraphQL adapter through the same operations and authorization semantics. | transport/adapters | Cross-surface conformance sends the same request and compares status, stamp, digest, denial, and result. |
| [ ] | H-05 | Emit bounded correlated traces for ingress, authorization, planning, KV scans, graph, BM25, HNSW, DataFusion, LFG, commit, attunement, and delivery. | `rrd-engine`, adapters | Trace completeness/crash/export/redaction tests pass; traces observe authoritative job/state records rather than becoming lifecycle state. |
| [ ] | H-06 | Implement Connectome completely on public RRD capabilities and render only persisted status, plans, traces, trees, and deltas. | separate Connectome repository | Real-process browser test starts from health/ready/capabilities, observes attunement and routing, and contains no duplicate retrieval or lifecycle implementation. |
| [ ] | H-07 | Resolve loopback or Zuul Zero/shippin.ai mesh endpoints through a transport adapter, then establish authenticated RRD identity independently of network reachability. | `rrd-client`, mesh adapter | Laptop/phone/devspace fixture proves TLS identity, capability negotiation, endpoint rotation, offline denial, and no mesh-owned database state. |

Gate H exits only after one prompt can be followed from ingress through LFG or
analytical routing, storage/index work, fused context, mutation, live delivery,
and Connectome using correlated evidence from one engine.

### Gate I — add explicit automation scaffolding without automatic hooks

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | I-01 | Define one canonical engine-event envelope with producer, action, target, scope, stamp, idempotency key, provenance, and authorization coordinates. | `rrd-contract`, `rrd-core` | Golden and replay tests reject ambiguous identity, duplicate mismatches, unbounded payloads, and events outside the authenticated estate. |
| [ ] | I-02 | Implement triggers as persisted conditions over canonical committed events; a trigger may request an authorized operation but cannot commit independently. | `rrd-engine` | Match/non-match, denial, duplicate, ordering, recursion-depth, and restart tests prove deterministic bounded behavior. |
| [ ] | I-03 | Implement routines as versioned resumable graphs of authorized engine operations with explicit inputs, checkpoints, budgets, cancellation, verification, and terminal status. | `rrd-engine` | Kill/restart, retry, compensation, stale-input, denial, and maximum-step tests prove no busy loop or silently repeated mutation. |
| [ ] | I-04 | Implement hook adapters as stateless host translators that submit typed events only when explicitly installed and configured; ship no editor/provider-owned automatic hook. | outward adapters | Claude/OpenAI/reference adapter conformance produces the same envelope; uninstall removes the adapter cleanly and leaves canonical state readable. |
| [ ] | I-05 | Implement skills as versioned instruction/resource packages referenced by identity and digest, resolved through governed context rather than executed as storage or lifecycle code. | `rrd-contract`, `rrd-engine` | Install/resolve/update/retire tests prove provenance, authorization, version pinning, prompt-budget enforcement, and no implicit mutation. |
| [ ] | I-06 | Add previewable install/configure/uninstall scaffolding for triggers, routines, hook adapters, and skills after their individual contracts pass. | `rrflow-cli`, adapters | Fresh/existing project tests show exact planned files/records, explicit consent, idempotent apply, clean uninstall, and no session-start loop. |

Gate I exits only when automation is explicit, bounded, replayable, removable,
and subordinate to `RrdEngine`; installation alone is never evidence that a
trigger, routine, adapter, or skill worked.

### Gate J — RRFlow 1.0 release proof

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | J-01 | Remove every deprecated item, legacy/compatibility path, editor/provider-owned automatic hook, duplicate source of truth, and stale generated artifact. | workspace | Strict warning/dependency/search gates and all-target builds are clean. |
| [ ] | J-02 | Run unit, property, fuzz corpus, differential, crash/reopen, storage-full, security denial, resource-budget, adapter, and real-process suites. | workspace | Release evidence records commands, versions, passed/failed counts, and retained failure artifacts. |
| [ ] | J-03 | Install and attune both an empty fixture and this existing repository from released artifacts, then restart and repeat representative fast/heavy queries. | release harness | Both estates verify with stable digests; unchanged rerun is incremental and no manual database repair is needed. |
| [ ] | J-04 | Publish fixed-hardware rrflowKV, graph, BM25, exact/HNSW, DataFusion, context, LFG, and end-to-end latency/memory/disk benchmarks. | evaluation harness | Raw data, configuration, warmup, concurrency, percentiles, recall/quality metrics, and failed runs accompany every claim. |
| [ ] | J-05 | Produce reproducible signed binaries, SDKs, schema/golden bundle, LFG conformance manifest, SBOM, default configuration, backup/restore rehearsal, and operator runbook. | release tooling | Clean-machine installation and artifact verification pass without repository-local caches or undeclared files. |

RRFlow 1.0 is releasable only when every Gate J item and every prerequisite is
checked. Until then the repository may describe implemented and measured
behavior, but it must not claim the complete target system is production-ready.

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
