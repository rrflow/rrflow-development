# RRFlow 1.0 code execution map

**Status:** active supporting execution map; it cannot mark a release gate complete
**Coordinate:** `rrflow://rrflow-instance/data/execution-map/rrflow-1.0`
**Owner:** file, symbol, dependency, test, and stop-condition mapping for the canonical RRFlow 1.0 roadmap
**File baseline:** generated inventory committed with this map
**Reviewed:** 2026-09-08

The canonical [RRFlow 1.0 roadmap](rrflow-1.0.md) owns dependency order,
checkboxes, and accepted completion evidence. This record turns each unchecked
gate into bounded code work. It does not duplicate product identity,
architecture authority, objective status, or the POA&M.

The companion
[`rrflow-1.0-file-plan.jsonl`](rrflow-1.0-file-plan.jsonl) inventories every
current tracked or non-ignored file with its complete line/byte span, digest,
assigned gates, and disposition. It also lists every currently planned path
before that path is created. The inventory is generated and checked by
[`build_execution_inventory.py`](../../scripts/ci/build_execution_inventory.py).

The primary-source reasoning behind the choices below lives in the
[system-convergence research](../research/rrflow-system-convergence-architecture-research.md).
Accepted component authority remains in the
[system overview](../architecture/system-overview.md), and the target runtime
sequence remains in the
[engine data-flow record](../architecture/engine-data-flow.md).

## How to execute this map

For every work package:

1. Verify every prerequisite in the roadmap's executable dependency spine is
   complete and the worktree contains no unexplained changes.
2. Read every listed file in full. Resolve every listed symbol against the
   current revision. If a symbol moved, update this map in a documentation-only
   commit before changing behavior.
3. Run the package's characterization test before editing. A pre-existing
   failure is recorded as the package baseline; it is not hidden or repaired
   by unrelated work.
4. Change only the listed files. A newly discovered required file pauses the
   package until this map and the JSONL inventory are reviewed and updated.
5. Run the smallest named test first, then the owning package suite, then the
   dependency-direction and documentation checks. Stop on the first failure.
6. Commit one package. Update a roadmap checkbox only when that gate's exact
   acceptance evidence exists in the same reviewed change.

Line numbers are deliberately not used as long-lived anchors because they
become false after the first edit. The JSONL freezes the complete baseline line
span and digest; this map names semantic symbols, modules, and intended target
paths. Together they cover every line without pretending a stale line number
is an architectural contract.

### Pre-release consolidation protocol

This map describes one current RRFlow 1.0 system, not a support matrix for
earlier implementations. A current implementation may be retained, rewritten
in its canonical boundary, or removed after its useful behavior is accounted
for. It may not be parked in a parallel runtime, forwarding API, migration
executor, or unresolved documentation archive.

Before changing a capability family, the implementing package must:

1. read every current source, test, fixture, example, and benchmark assigned to
   the gate;
2. run the smallest available characterization corpus and record any existing
   failure without broad repair;
3. map each useful behavior and invariant to one canonical destination and one
   acceptance test in the traceability matrix below;
4. perform the direct convergence inside the listed files, allowing compiler
   errors to surface during the edit but not using compilation as proof; and
5. remove the conflicting path only after the mapped equal-or-stronger evidence
   passes.

Git ancestry, branch or merge status, a rename, a deletion, a generated file,
and a successful compile are inventory facts only. None proves consolidation.

## Product terms versus implementation packages

These mappings eliminate the present naming ambiguity:

| Product term | Exact meaning | Implementation boundary |
|---|---|---|
| RRFlow | the complete AI governance, reasoning, recall, and project-operation product | this repository and its self-contained signed default distribution |
| RRD | the one Reason Ready Daemon/runtime for a project instance | `rrd-server` process over one `RrdEngine`; embedded callers still use `RrdEngine` |
| `RrdEngine` | sole authentication, authorization, semantic transaction, mutation, and orchestration authority | `crates/authority/rrd-engine` |
| rrflowDB | the durable, per-project AI estate as observed through `RrdEngine` | composition of core types, store repositories, rrflowKV, native indexes, and query execution; never a parallel crate or backend |
| rrflowKV | RRFlow's local durable physical profile | `rrd-lsm` plus `RrflowKvStore` in `rrd-store` |
| rrflowMX | RRFlow Memory Execution, the non-durable profile behind the same transaction contract | the in-memory implementation in `rrd-store`; not Dragonfly, Redis, or a cache |
| rrflowQL | parse, bind, logical plan, physical selection, native operators, and DataFusion analytical execution | `crates/compute/rrd-query` |
| Arrow substrate | the typed columnar batch/buffer interchange for analytical execution | Arrow crates used by rrflowKV page readers, native operators, rrflowQL, and DataFusion |
| DataFusion | compute-only analytical planner/executor | a dependency of `rrd-query`; no direct commit, authorization, or lifecycle authority |
| RRFlow vector subsystem | exact vectors plus payload indexes, HNSW/quantized projections, filtered candidates, and exact reranking | `rrd-vector` plus atomic canonical/index deltas in `rrd-store` |
| LFG | one replaceable local routing-model adapter | `RouterBackend` in `rrd-inference`, adapter in planned `rrflow-lfg`, orchestration in `RrdEngine` |
| Clyffy | this repository's primary durable RRFlow seat specialization; not a product, kernel, repository, daemon, model, store, or lifecycle | seat/provider/representation state in rrflowDB; same-stamp identity resolution, authorization, routing, and commit in `RrdEngine` |
| external project data | PostgreSQL, Turso, Dragonfly, SQL stores, object stores, and other operator/application systems | discovered source descriptors and explicit adapters; never implicit rrflowDB persistence |

The storage port is `rrd_store::StorageEngine`; `RrdEngine` remains the sole
composition and mutation authority. C-02 narrows that storage port to
transactional point/range primitives plus semantic repositories. The code must
not reintroduce `Engine`, `NativeEngine`, `PersistentEngine`, or `EngineBox` as
aliases. The concrete profile names are `RrflowKvStore`, `RrflowMxStore`, and
`StorageProfile`.

## Frozen target source tree

The current eight source groups are retained unless A-07's dependency audit
finds a concrete violation. New packages are added only at the gate that owns
their capability:

```text
crates/
├── kernel/
│   └── rrd-core                 # pure identities, values, stamps, schemas
├── persistence/
│   ├── rrd-lsm                  # byte-key WAL/MVCC/segments/manifest/recovery
│   └── rrd-store                # key codec and semantic repositories
├── compute/
│   ├── rrd-query                # rrflowQL + native/DataFusion physical plans
│   ├── rrd-vector               # exact and approximate vector projections
│   ├── rrd-inference            # provider-neutral inference ports
│   └── rrd-attunement           # pure bounded phase computation (planned)
├── authority/
│   ├── rrd-security             # identity, policy, and audit vocabulary
│   ├── rrd-estate               # desired/observed estate state
│   └── rrd-engine               # sole semantic composition authority
├── transport/
│   ├── rrd-contract             # implementation-free public envelopes
│   ├── rrd-client               # public RRD client
│   └── rrd-server               # HTTP/WebSocket daemon adapter
├── adapters/
│   ├── rrflow-cli
│   ├── rrflow-mcp
│   ├── rrflow-edge
│   ├── rrflow-local-process     # planned host process effect adapter
│   ├── rrflow-kubernetes        # planned Kubernetes effect adapter
│   ├── rrflow-lfg               # planned model adapter
│   ├── rrflow-host-events       # planned explicit host translators
│   ├── rrflow-mesh              # planned endpoint resolver only
│   └── rrflow-devforge          # planned CoW/mount/hibernation adapter
├── operations/
│   ├── rrd-cluster
│   └── rrd-operator-knowledge
└── evaluation/
    └── rrflow-eval
```

No crate named `rrflowDB`, `rrflowKV`, `rrflowMX`, or `rrflowQL` is added.
Those are product capabilities composed through RRD; forcing a one-term/one-
crate shape would fragment the engine again.

## Target dependency and authority direction

```text
outward clients / CLI / MCP / Connectome / host / mesh adapters
                  │
                  ▼
       rrd-contract   rrd-client   rrd-server
                  │        │            │
                  └────────┴─────┬──────┘
                                 ▼
                            RrdEngine
                 ┌───────────────┼───────────────┐
                 ▼               ▼               ▼
          authority policy   compute ports   semantic repositories
                                 │               │
                                 ▼               ▼
                         rrflowQL/vector     StorageEngine
                              │             ┌─────┴─────┐
                              ▼             ▼           ▼
                         DataFusion     rrflowMX     rrflowKV
                    (compute/propose)  (volatile)  (WAL/MVCC/LSM)
```

Forbidden directions:

- contract/client packages importing engine, store, query, vector, or model
  implementations;
- transports, SDKs, Connectome, models, functions, triggers, routines, skills,
  or mesh adapters opening storage;
- DataFusion or an inference adapter committing directly;
- rrflowKV depending on DataFusion to serve point/range/CAS operations;
- an external project database becoming the RRFlow transaction authority;
- first-party build, install, runtime, verification, or recovery code resolving
  from a sibling checkout, submodule, escaping path, undeclared generator, or
  host-specific absolute location; and
- default installation or runtime fetching a required artifact or depending on
  an external database, query engine, vector service, mesh, client, or model
  provider to become ready.

## Target runtime flows

### Authorized semantic write

```text
ingress -> authenticate -> authorize -> parse/bind -> capture transaction stamp
       -> optional bounded compute/proposal -> validate semantic mutation plan
       -> encode one physical batch:
          current record + temporal version
          + outgoing edge + incoming edge
          + scalar/unique/BM25/vector index deltas
          + runtime log + durable projection delta + outbox + audit + cursor
       -> StorageEngine commit/conflict
       -> WAL durable acknowledgement (rrflowKV only)
       -> committed impact -> subscriptions/triggers/traces
```

No observer event means “committed.” Only the storage commit receipt advances
canonical state.

### Stamped fast and analytical read

```text
authenticate -> authorize fields/rows -> capture ReadStamp + catalogue
       -> cost/selectivity planner
          ├─ fast: point/range/adjacency/index scan -> typed result
          └─ analytical: rrflowKV page stream -> Arrow RecordBatch
                         -> native graph/BM25/HNSW/exact/RRF operators
                         -> DataFusion transforms/joins/aggregates
                         -> bounded result/evidence
       -> verify stamp/catalogue -> serialize -> return/live deliver
```

Borrowed Arrow buffers are allowed only for aligned, uncompressed,
type-compatible pages whose mapped lifetime is pinned through batch
consumption. All other paths decode or copy into the query pool and report the
bytes.

### Installation and incremental attunement

```text
acquire signed default bundle -> verify complete manifest -> deny network
       -> preview install -> resolve bundle-resident template/profile/stubs
       -> explicit apply -> create locator/identity/estate through RrdEngine
       -> persist attunement job
       -> authorize bounded metadata/content enumeration
       -> pure deterministic inventory proposal
       -> atomically commit tree snapshot/change set/checkpoint/runtime-log/audit
       -> parse committed snapshot -> normalize -> entity-link -> lexical-index
       -> embed -> vector-index -> graph -> ground -> verify
       -> commit each digest-bound checkpoint through RrdEngine
       -> close/reopen/readback

authenticated filesystem hint -> committed EngineEvent -> inventory diff
committed project change set -> determine affected phases only
       -> policy eligibility -> explicit capability/routine/skill decision
       -> bounded resumable phase work (never a blanket reinstall loop)
```

## Current implementation inventory and exact disposition

| Boundary | Current code that is real | Required convergence |
|---|---|---|
| `rrd-core` | canonical IDs, scopes, runtime values/mutations, read stamps, bitemporal values, reasoning-tree contract, trace links | keep semantic types; move physical key encoding out of `key.rs` during A-07/C-01; add only event primitives proven generic |
| `rrd-lsm` | WAL, version chains, snapshots, manifest/CURRENT, immutable row-block segments, cache, compaction, snapshot bundles, I/O tiers, failure injection | add transaction conflicts; replace current segment format with key spine + column pages; delete every pre-1.0 reader; prove crash/lifetime safety |
| `rrd-store` | `RrflowKvStore`, `RrflowMxStore`, `StorageEngine`, runtime commits, current projections, archives/backups/object tiers, and absolute rrflowKV diagnostic workloads | narrow the broad trait; freeze binary keys; make semantic batch/index maintenance atomic; add direct stamped reads; never restore a backend selector |
| `rrd-query` | parser, binder, plan, index catalogue, BM25 implementation, DataFusion execution, spill pool, live polling | replace eager `Vec<QueryRow>` and `MemorySource`; add streaming provider/pushdown/native operators; replace two-snapshot live diff |
| `rrd-vector` | exact oracle, immutable segments, HNSW, filtered planning, catalogues, compact dense artifacts, quantization, and accelerator code | bind all artifacts to canonical source cursors; persist atomic deltas; exact-rerank; remove alternate TurboQuant catalogue paths; benchmark codecs before retaining them |
| `rrd-inference` | provider-neutral embedding jobs/backend and local FastEmbed adapter | add manifest handshake and separate `RouterBackend`; LFG remains an outward adapter |
| `RrdEngine` | one opening/composition authority, security/session/query/data/vector/context/retrieval/subscription/function operations; current security bootstrap is nevertheless a static rrflowKV path opener used by separate CLI and Kubernetes initialization flows | absorb cold-start security into the digest-bound `initialize_instance` action; add persisted attunement/routing/events/routines/skills; make all paths use the final transaction/index/provider contracts; do not add transport imports |
| transport/adapters/SDKs | catalogue-derived HTTP/OpenAPI, durable subscriptions, a Rust client implementing 28 of 33 HTTP operations plus the dedicated socket, and TypeScript, synchronous Python, context-aware synchronous Go, synchronous Java, and asynchronous .NET generic calls over all 33 HTTP descriptors with partial runtime enforcement and no qualified socket/remote/package surface, plus MCP and CLI | split clients by accepted responsibility; close operation/validation/secret/retry/cancellation/frame/resolver/package/browser/interpreter/toolchain/concurrency gaps; freeze multiplex WS and GraphQL lowering; make missing harness configuration fail or explicitly skip; prove every surface only against the same installed engine semantics after storage/query behavior exists |
| estate/operations | substantial desired/observed, process, backup, recovery, and operator-source foundations; estate state currently uses a public direct-store repository and monolithic JSON control value; `rrd-maintenance` is an unused standalone direct-store state-machine draft | preserve the fenced reconciliation/recovery semantics while making estate values pure and commits engine-owned; reconcile the overlapping operational hierarchy; converge reusable maintenance validation into generic routines; remove both private storage authorities rather than wrapping them |
| `rrd-cluster` | useful placement/stamp/consistency/transfer/reshard contracts, OpenRaft storage and TLS mechanics, bounded snapshot/artifact transfer, telemetry, simulation, and one-host tests; current code also owns parallel IDs/topology/schema, opens storage directly, accepts raw commits, persists private JSON transfer state, and is failing against the current rrflowKV application format | retain the safety semantics through pure contracts and injected distributed execution ports beneath `RrdEngine`; remove the parallel authorities and old-format paths directly; keep `clustered_server` unavailable until a later roadmap amendment schedules and accepts independent-host full-engine qualification |
| `rrd-kubernetes` | namespaced structural CRD, deterministic five-resource rendering, digest-pinned image validation, retained one-replica StatefulSet/PVC, restricted container settings, PDB/NetworkPolicy, server-side apply, status/finalizer controller, checked manifests, and four local tests; current code also invents desired RRFlow state, runs two parallel bootstrap paths, force-takes fields, equates TCP/ready replica with application readiness, watches cluster-wide, and deletes without engine intent/receipt proof | preserve the useful Kubernetes mechanics in planned `rrflow-kubernetes`, an outward adapter consuming sealed engine install/effect plans and returning bounded observations/receipts; use standard conditions, authenticated readiness, explicit field ownership, engine-fenced deletion/retention, and real API-server/clean-install/full-engine evidence; remove the old operations package with no forwarding crate |

## Implementation-requirements traceability

This is the reviewed current-tree accounting required by A-07 and POAM-014. It
is updated in a documentation-only change before a listed implementation file
moves or a newly discovered behavior expands a gate. A row identifies work
that must be carried into the one target system; it does not mark that work
complete.

External implementation patterns are bounded by the source-pinned
[SurrealDB](../research/surrealdb-capability-inventory.md) and
[Qdrant](../research/qdrant-capability-inventory.md) references. Their
adaptation protocols require an exact upstream symbol, rejected assumptions,
an RRFlow-owned destination, and independent semantic/fault/resource/reopen
proof; neither reference can change this map's status.

### A-07.0 reviewed starting revision

The traceability baseline is Git commit
`b5ec39162771ced38a83dd512dda5b0720aef714`, tree
`3c8d11fe697cd88645179f15441a7485d934f240`. It is an implementation
inventory, not engine-conformance evidence. At that revision:

- the generated execution inventory contains 896 records and passes its
  deterministic check;
- all 21 workspace/root Rust manifests, the locked dependency graph, and all
  32 library or binary module roots were read completely;
- the ordered per-file SHA-256 ledger for the 21 Rust manifests has digest
  `0f0a3fea21ad278fed66472a97c89d24490b7625625f6fd3ea85f62f2ee29b86`,
  and the corresponding 32-root ledger has digest
  `bd6b041ce9ac9a4da64883937670b44b3e0645ad8be8d770fa286bcdaa094a88`;
- the 11 non-workspace package/solution/fixture manifests have ordered-ledger
  digest
  `2e2b04c1cffab3516869c1738a3562d4553567b70eed3900ea9d46cc44a098f0`;
  these include the Rust and TypeScript evaluation fixtures and each generated
  SDK package boundary; and
- `Cargo.lock` has SHA-256
  `bafc36ae835b2b7af47fd560140f721f7282a9d1bee0d18c28c802025538cd7b`.

The two tables below are one trace, not competing inventories. The package
table freezes the actual dependency and public-root baseline. The capability
table binds those roots to all known implementation modules,
tests/fixtures/examples/benchmarks, accepted invariants, destination gates,
and equal-or-stronger replacement proof. The
[mandatory direct-convergence table](#mandatory-direct-convergence) then names
the conflicting files, types, fields, formats, commands, and authorities that
must disappear. A capability is not accounted for unless all three views agree.

| Current package and fully read module root(s) | Normal first-party dependencies | Current public/root surface | Required direct disposition |
|---|---|---|---|
| `rrd-core` — `crates/kernel/rrd-core/src/lib.rs` | none | `authenticated_log`, `claim`, `data`, `digest`, `error`, `ident`, `key`, `reasoning_tree`, `reference`, `runtime`, `schema`, `temporal`, and `trace`; re-exported claim, data, reasoning, transaction/read-stamp, schema, temporal, and trace types | Keep storage-independent semantic vocabulary in the kernel. Move physical `key` encoding to the rrflowKV boundary in C-01; preserve only explicit semantic identities and causal evidence. |
| `rrd-lsm` — `crates/persistence/rrd-lsm/src/lib.rs` | `rrd-core` | `Database`, `Snapshot`, `WriteBatch`, WAL/recovery receipts, MVCC memtable values, manifest/CURRENT, row-block `Segment`, cache/I/O statistics, compaction, and snapshot bundles | Become the sole rrflowKV physical implementation below the narrowed port; reject all pre-1.0 readers in C-05 and replace row-only segments with the C-06 ordered key spine plus Arrow-compatible pages. |
| `rrd-store` — `crates/persistence/rrd-store/src/lib.rs`; `src/bin/durability-child.rs` | `rrd-core`, `rrd-lsm` | `StorageEngine`, `StorageProfile`, `RrflowMxStore`, `RrflowKvStore`, runtime access/commit helpers, projection/control/invocation records, archives/backups, object tiers, S3 adapter, physical counters, and the conflicting public `Trigger` | Retain one MX/KV semantic port and the direct profile names; narrow it in C-02 and make C-03 mutations/index/event/audit effects atomic. Remove or rename storage-level vocabulary that claims later engine trigger authority; keep backup/object mechanics below engine-owned plans. |
| `rrd-query` — `crates/compute/rrd-query/src/lib.rs` | `rrd-core`, `rrd-store` | RRFlowQL syntax/binding/plans, `QueryExecution`, eager `QueryRow`/`QueryBatch`, `ArrowSnapshot` and row conversion, `execute_snapshot` DataFusion execution, BM25, index catalogue, live polling, and `StampedQueryPipeline` | Keep one rrflowQL compute subsystem. Replace eager snapshot/materialization and polling paths with C-04/F-01 stamped streaming, native pushdown/operators, bounded DataFusion execution, and one resource ledger; indexes converge through Gate E. |
| `rrd-vector` — `crates/compute/rrd-vector/src/lib.rs` | `rrd-core`, `rrd-store` | exact scoring/oracles, collection/catalogue and payload indexes, immutable/compact/quantized/TurboQuant segments, HNSW, filter/planner/runtime, accelerator admission, quantization lifecycle, and alternate catalogue paths | Keep one RRFlow vector index subsystem under engine transactions. Preserve exact-oracle, filtering, HNSW, quantization, and measured accelerator behavior; remove competing catalogue/artifact paths after E-04/E-05 evidence. |
| `rrd-inference` — `crates/compute/rrd-inference/src/lib.rs` | `rrd-core` | provider-neutral `EmbeddingModelSpec`, backend descriptor/registry/trait, source reader, jobs, resource/trust/network controls, coordinator, prepared embedding, feature-hash reference backend, and optional local FastEmbed backend | Retain deterministic embedding as non-authoritative compute accepted through `RrdEngine`; add the separate model-neutral `RouterBackend` contract/implementation at B-03/G-02 rather than overloading embedding. |
| `rrd-contract` — `crates/transport/rrd-contract/src/lib.rs`; `src/bin/rrd-contract-export.rs` | none | transport-neutral attunement, capability, diagnostic, function, inference, knowledge, context, seat/provider representation, platform, reasoning-tree, router, SDK-conformance, transaction/data/query/index/vector/retrieval/subscription/backup/audit/estate, endpoint/OpenAPI, identity, deployment, session, and error envelopes | Remain implementation-free public vocabulary. A-07 directly resolves `AutomationCatalogue`/`FunctionTrigger*`, `MemoryEstate*`, `DeploymentMode`, the `rrd` wire name, topology/resource grammar, and alternate successful fields; generated surfaces change from this one contract. |
| `rrd-client` — `crates/transport/rrd-client/src/{lib,client,endpoint,error,operation,retry,session,subscription,transport}.rs` | `rrd-contract` | `RrdClient`, `ClientConfig`, `RequestOptions`, credential-private and redacted `Session`, `SubscriptionSocket`, 28 typed HTTP operation methods, HTTP/mTLS carriage, WebSocket ACK/reconnect, and client errors | Remain a pure public Rust SDK. A-07.1b completed the direct responsibility split and public session-secret boundary. B-04/H-04/H-07 bind all 33 operations plus the multiplexed socket and close validation, secret-wrapper, retry/certainty, cancellation, resource, trace, resolver, and package-conformance gaps. |
| `rrd-server` — `crates/transport/rrd-server/src/lib.rs`; `src/main.rs` | `rrd-contract`, `rrd-engine` | `RrdHttpServer`, HTTP errors, JWT and mutual-TLS configuration; the binary additionally owns the conflicting `initialize` command, instance discovery, physical paths, caller clock, readiness files, and shutdown files | Keep only HTTP/WebSocket hosting over an installed `RrdEngine`. Move cold start to D-01, engine time to the composition boundary, and readiness/control to authenticated receipts; remove path/marker/lifecycle authority. |
| `rrd-security` — `crates/authority/rrd-security/src/lib.rs` | `rrd-contract`, `rrd-core`, `rrd-store` | principal/role/grant/data-policy/JWT/identity types, authorization decisions, audit records, `SecurityState`, and the direct-store `SecurityRepository` | Preserve pure validation, deny-by-default decisions, verifier-only credentials, and audit semantics; C-03/D-01 move all state and effects through one stamped `RrdEngine` transaction and remove `SecurityRepository` storage authority. |
| `rrd-estate` — `crates/authority/rrd-estate/src/lib.rs`; `src/commands/rrd-deployment-catalog.rs` | `rrd-contract`, `rrd-core`, `rrd-store` | monolithic `EstateDocument`/`EstateRepository`, desired/observed operations, leases/receipts, authority hierarchy, local authorization, local process catalogue/driver, backup/recovery jobs, and reconcilers | Preserve domain validation, fencing, prepared-effect receipts, backup, and recovery invariants. Make estate types pure, persist native records/relations through `RrdEngine`, merge duplicate topology/security concepts, and move host process effects to one outward adapter with no forwarding crate. |
| `rrd-engine` — `crates/authority/rrd-engine/src/lib.rs` | `rrd-cluster`, `rrd-contract`, `rrd-core`, `rrd-estate`, `rrd-inference`, `rrd-operator-knowledge`, `rrd-query`, `rrd-security`, `rrd-store`, `rrd-vector` | `RrdEngine`, `RrdOperation`, authorized invocation/session/security/query/data/vector/context/retrieval/function/estate/distributed operations, capability catalogue, offline edge types, operator facade, and runtime exports | Remain the only composition, authorization, transaction, routing, and persistence authority. Absorb every direct-store authority through narrow injected ports; do not import transports or let compute/adapters commit independently. |
| `rrflow-cli` — `crates/adapters/rrflow-cli/src/main.rs` plus `src/bin/{rrd-backup-controller,rrd-estate-admin,rrd-estate-controller,rrd-recovery-controller,rrd-security-bootstrap}.rs` | `rrd-client`, `rrd-contract`, `rrd-engine` | binary-only `rrflow` command tree, development supervisor, direct embedded opener, operator invocation recording, and five standalone controller/bootstrap binaries | Retain one installed operator adapter over public/engine operations. D-01 adds preview/apply install; remove standalone bootstrap/controllers, default `.rrflow/rrd`, caller-authored identity/time, and supervisor lifecycle state after canonical evidence exists. |
| `rrflow-mcp` — `crates/adapters/rrflow-mcp/src/main.rs` | `rrd-client`, `rrd-contract`, `rrd-engine` | binary-only MCP discovery/initialize/ping/tools surface and `rrflow_context`; private embedded-or-daemon `RuntimeAuthority` | Remain a provider-neutral outward adapter. It may translate MCP to the same installed operations but cannot own storage, reasoning lifecycle, attunement, identity, or completion truth; H-04/H-07 prove remote and embedded parity. |
| `rrflow-edge` — `crates/adapters/rrflow-edge/src/main.rs` | `rrd-engine` | binary-only offline `build` and `query` commands over `OfflineEdgeIndex` | Retain only a deterministic derived/offline artifact profile with provenance and resource evidence. It never becomes another authoritative database or deployment mode. |
| `rrd-cluster` — `crates/operations/rrd-cluster/src/lib.rs`; `src/bin/rrd-cluster-node.rs` | `rrd-core`, `rrd-lsm`, `rrd-store` | placement/consistency/transfer/authority/telemetry/simulation contracts plus optional OpenRaft storage/transport and a process node; currently also direct storage, topology, mutation, transfer-state, and trace authority | Preserve safety contracts behind an injected engine proposal/apply port; remove direct opens, raw commits, private state/formats, and duplicate identities. Distributed availability is outside the first alpha until a later gate qualifies the full engine on independent hosts. |
| `rrd-kubernetes` — `crates/operations/rrd-kubernetes/src/lib.rs`; `src/main.rs`; `src/bin/rrd-kubernetes-crd.rs` | `rrd-contract`, `rrd-core` | `RrdInstance` CRD/spec/status, resource renderer, `controller` module, finalizer/status/apply mechanics, operator and CRD binaries | Preserve Kubernetes mechanics in the outward `rrflow-kubernetes` adapter. Consume sealed install/effect plans and return observations/receipts; remove bootstrap, caller-time/path, desired-state, TCP-readiness, force-apply, and eager-deletion authority. |
| `rrd-maintenance` — `crates/operations/rrd-maintenance/src/lib.rs` | `rrd-contract`, `rrd-core`, `rrd-store` | `MaintenanceRun`, inventory/decision/protection/validation/observation/projection/event types, fixed stages/classes/reduction policy, defaults, and direct-store `MaintenanceRepository` | Generalize only the useful safety semantics into canonical event-triggered I-03 routines with focused evidence. Remove the fixed lifecycle, policies, private scope/events/JSON repository, cursor-zero replay, direct-store path, and then the package itself with no wrapper. |
| `rrd-operator-knowledge` — `crates/operations/rrd-operator-knowledge/src/lib.rs` | `rrd-core`, `rrd-vector` | external binding/source-revision/search/sync contracts, bounded controls/evidence, adapter/writer traits, reference adapters, and optional pgvector implementation | Retain as an optional external operator-data adapter. External Postgres/Turso/Dragonfly/etc. remain non-authoritative sources selected at installation; `RrdEngine` owns acceptance, stamps, policy, context use, and all canonical RRFlow persistence. |
| `rrflow-eval` — `crates/evaluation/rrflow-eval/src/main.rs` | none | binary-only paired `run`/`summarize`/`verify` runner that invokes configured Codex, Claude, or Gemini CLIs and records provider output | Keep provider execution and comparative trials in evaluation only. It supplies evidence and never becomes an engine router, host hook, install authority, or required runtime dependency. |

The five generated SDK roots are independently reviewed because Cargo metadata
cannot account for them:

| Package boundary and public source root | Current public surface | Characterization and canonical destination |
|---|---|---|
| TypeScript — `sdks/typescript/{package.json,pnpm-workspace.yaml,tsconfig.json}`; `src/{index,client,endpoint,error,operation,retry,session,transport}.ts` | `RrdClient`, `ClientConfig`, `Session`, `RequestOptions`, resource types, errors, generated `OperationId`, and one generic all-33-operation HTTP call; no socket module exists | A-07.1c completed the direct source split and conformance filename change. Unit/live conformance, generator, lockfile, absent WebSocket, browser/package behavior, and all B-04/H-04/J requirements are accounted in the TypeScript capability row below. |
| Python — `sdks/python/pyproject.toml`; `src/rrd_client/{__init__,client,models}.py` | synchronous `RrdClient`, `Session`, `RequestOptions`, resource/envelope models, errors, generated operation literals, and one generic call | Unit/live conformance, generator, lockfile, wheel/sdist behavior, and required sync/async/socket/package matrices are accounted in the Python row. |
| Go — `sdks/go/go.mod`; `client.go`, `models.go`, `endpoints_gen.go` | context-aware `Client`, `Config`, `Session`, `RequestOptions`, credentials/error/resource types, generated `OperationID`, typed discovery/session helpers, and one generic call | Unit/race/live conformance, generators, module packaging, explicit transport ownership, and required platform/socket matrices are accounted in the Go row. |
| Java — `sdks/java/pom.xml`; all seven `src/main/java/io/rrflow/rrd/*.java` roots | `RrdClient`, `OperationId`, `RequestOptions`, `Session`, `ResourceSegment`, and client/API exceptions | Unit/live conformance, generation, Maven/JAR/offline behavior, async/blocking/socket carriage, and platform matrices are accounted in the Java row. |
| .NET — solution plus both project manifests; four `src/Rrflow.Rrd.Client/*.cs` roots | asynchronous `RrdClient`, `RrdClientOptions`, `RequestOptions`, sessions/credentials/resources, generated `OperationId`/endpoint catalogue, and client/API exceptions | Unit/live conformance, generation, NuGet/offline behavior, explicit HTTP/WebSocket carriage, trimming/AOT, and platform matrices are accounted in the .NET row. |
| Evaluation fixtures — `eval/fixtures/{rust-service/Cargo.toml,ts-service/package.json}` and shared `fixtures/*.json` | no runtime public API; controlled consumer and golden/conformance inputs | Remain characterization inputs only. A fixture never establishes authority or completion; its owning capability row names the implementation and stronger real-process evidence it must exercise. |

| Capability family | Current implementation modules and public surface that must be read in full | Characterization inventory that must be preserved or strengthened | Canonical convergence and equal-or-stronger replacement evidence | Gates |
|---|---|---|---|---|
| rrflowKV WAL, MVCC, manifests, recovery, compaction, hot reads, block filtering, and decoded-block caching | `rrd-lsm/src/{wal,memtable,database,manifest,segment}.rs`; `rrd-store/examples/ai_hotset_benchmark.rs` | `rrd-lsm/tests/{wal,mvcc,manifest,failure_matrix,compaction,segment,snapshot_memory}.rs`; `rrd-store/tests/{durability,snapshot,benchmark_evidence}.rs` | Keep the useful durability, snapshot, bounded-cache, filter, and physical-counter behavior while replacing the port and row-only segment format; prove the final key spine, Arrow pages, buffer lifetime, and measured cache decision. | C-02, C-04, C-06, C-07, F-05, J-04 |
| rrflowMX/rrflowKV semantic equivalence | `rrd-store/src/{engine,rrflow_kv,ds}.rs` | `rrd-store/tests/{engine,snapshot,runtime,unified_data,rrflow_kv_operator,rrflow_kv_model_soak}.rs`; vector engine differential tests | Narrow the one `StorageEngine` port, then run the same transaction, point/range, snapshot, graph, index, and query corpus against `RrflowMxStore` and `RrflowKvStore`; durability assertions apply only to rrflowKV. | A-07, C-02, C-03, C-04 |
| Stamped transactions, identity, authorization, and audit | `rrd-core/src/runtime.rs`; `rrd-store/src/{engine,rrflow_kv,control}.rs`; `rrd-engine/src/engine/{transaction,query_transaction,session,security,invocation,control}.rs`; `rrd-security/src/lib.rs` | `rrd-store/tests/{control_journal,durability,snapshot}.rs`; `rrd-engine/src/engine/tests/{query_transaction,security,transaction_stamp,recovery}.rs`; `rrd-security/tests/security_authority.rs`; `rrd-server/tests/http_process.rs` | Preserve authenticated `ReadStamp`, `DataTransaction`, conflict, idempotency, audit-chain, and policy semantics while making one transaction port and one cross-surface authorization path. | C-02, C-03, H-04, H-05, J-02 |
| Initial security and credential installation | `rrd-engine/src/engine/security_bootstrap.rs`; `rrflow-cli/src/bin/rrd-security-bootstrap.rs`; `rrflow-cli/src/dev/supervisor.rs`; `rrd-kubernetes/src/lib.rs`; `rrd-security/src/lib.rs::SecurityRepository::{initialize,load}` | `rrflow-cli/tests/security_bootstrap.rs`; bootstrap/reopen path in `rrd-engine/tests/engine_authority.rs`; initialization, replay, drift, redaction, rotation, and MX/KV audit cases in `rrd-security/tests/security_authority.rs`; supervisor and Kubernetes rendering assertions | Preserve strict decoding, bounded non-empty regular inputs, unique identities, full policy validation, verifier-only persistence, atomic initial policy/audit, exact replay, drift denial, and no network-listener bootstrap. Replace the database/absolute-path/caller-time helper and adapter-authored policies with D-01's exact-plan `initialize_instance`: exclusive fresh-target proof, engine clock, capability-scoped/versioned secret sources and sinks, typed verifiers, atomic installed binding/policy/checkpoint/audit, prepared external-effect receipts, restart-safe credential delivery, and rejection of partial or already-installed cold start. Delete the standalone binary, static engine opener, private bootstrap JSON, all callers, and successful old shape after equal-or-stronger conformance passes. | A-07, C-02, C-03, D-01, D-02, H-05, J-01 through J-03, J-05 |
| Deterministic embedding, vector search, compact artifacts, HNSW, quantization, and accelerator admission | `rrd-inference/src/{lib,fastembed_local}.rs`; `rrd-vector/src/{contract,catalog,exact,filter,plan,segment,compact,hnsw,quantization,accelerator,runtime}.rs`; `rrd-engine/src/engine/{inference,vector}.rs` | `rrd-inference/tests/pipeline.rs`; `rrd-vector/tests/{golden,exact_model,engine_differential,model_binding,compact_dense,online_hnsw,quantization_matrix,accelerator,recall_gate}.rs`; `rrd-engine/src/engine/tests/{vector_index,native_inference}.rs` | Keep deterministic model/provenance binding and exact oracles; commit canonical vectors and index deltas atomically, bind projections to one source cursor, filter candidates, and exact-rerank before results become authoritative. Use the source-pinned Qdrant capability reference only as a behavior, algorithm, and failure oracle: journal the exact upstream symbol and rejected assumptions, translate into RRFlow identities and transaction semantics, and prove the result independently without importing its product model, API, or storage format. | D-05, E-04, E-05, F-03, H-01, J-04 |
| Edge packaging and public delivery | `rrd-engine/src/edge.rs`; `rrflow-edge/src/main.rs` | `rrflow-edge/tests/{offline,evidence}.rs`; `rrd-client/tests/real_server.rs`; `rrd-server/tests/http_process.rs`; `rrflow-mcp/tests/{stdio,stdio_daemon}.rs` | Retain deterministic offline artifact and provenance checks as outward packaging evidence; all reads and mutations continue through public RRD capabilities with no edge-owned engine state. | H-04, H-07, J-03, J-05 |
| Rust SDK operation, transport, credential, retry, cancellation, subscription, and conformance behavior | `rrd-client/src/{lib,client,endpoint,error,operation,retry,session,subscription,transport}.rs`; `rrd-client/Cargo.toml`; `rrd-contract/src/{lib,sdk_conformance}.rs`; executable endpoint catalogue and WebSocket descriptor | `rrd-client/src/session.rs::tests`; `rrd-client/tests/{real_server,sdk_conformance}.rs`; `rrd-client/examples/sdk_conformance_server.rs`; `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py` | Retain the A-07.1b direct source split, credential-private/redacted session handle, implementation-free normal dependency, typed calls, loopback restriction, explicit TLS, identity checks, four-MiB HTTP limit, real server/mTLS/WSS fixtures, and durable ACK/reconnect behavior. Bind all 33 operations exactly once; validate complete payload/envelope/status/media/identity contracts; add a limited secret wrapper and zeroization policy; replace boolean immediate retry and local-drop cancellation with semantic certainty and correlated cancellation; bound/validate multiplexed frames; add W3C propagation and authenticated endpoint rotation; make conformance configuration and scenario coverage fail closed; seed only through D-01/public operations and compare full rrflowMX/rrflowKV engine evidence across surfaces. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| TypeScript SDK generation, HTTP transport, credential, retry, cancellation, packaging, browser, and conformance behavior | `sdks/typescript/{package.json,pnpm-lock.yaml,pnpm-workspace.yaml,biome.json,tsconfig.json}`; `sdks/typescript/scripts/generate.ts`; `sdks/typescript/src/{index,client,endpoint,error,operation,retry,session,transport,generated/endpoints,generated/rrd-openapi}.ts` | `sdks/typescript/tests/{client.test,sdk-conformance}.ts`; `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py`; shared live harness | Retain the A-07.1c direct source split and filename, deterministic repository-local generation, all-33 HTTP descriptor typing, loopback restriction, canonical ID/resource construction, idempotency requirement, bounded body reading, external abort, fail-closed manifest input, locked tools, and current mock/live characterization. Generate full runtime validators; enforce exact payload/envelope/status/media/identity/stamp/receipt contracts; make credentials opaque; replace broad immediate retry/local abort with semantic certainty/server cancellation; add the real bounded multiplexed WebSocket module and authenticated Node/browser transports; build deterministic ESM JavaScript/declarations; and prove package/offline/browser plus installed rrflowMX/rrflowKV cross-surface behavior. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| Python SDK generation, sync/async HTTP transport, credential, retry, cancellation, packaging, and conformance behavior | `sdks/python/{pyproject.toml,uv.lock}`; `sdks/python/scripts/generate.py`; `sdks/python/src/rrd_client/{__init__,client,models,generated/__init__,generated/endpoints}.py` | `sdks/python/tests/{test_client,sdk_conformance}.py`; `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py`; shared live harness | Preserve deterministic repository-local generation, all-33 HTTP operation literals/descriptors, loopback and redirect denial, canonical ID/resource construction, mutation-idempotency requirement, bounded body reading, fail-closed manifest input, locked tools, buildable wheel/sdist, and current mock/live characterization. Split the runtime into the planned seams; generate full runtime models; enforce exact payload/envelope/status/media/identity/stamp/receipt contracts; make credentials opaque; replace broad immediate replay/local timeout with semantic certainty/server cancellation; add native async plus bounded multiplexed WebSocket and authenticated remote transports; ship `py.typed` and reproducible signed offline artifacts; and prove dependency, interpreter, platform, consumer, and installed rrflowMX/rrflowKV cross-surface behavior. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| Go SDK generation, context-aware HTTP transport, credential, retry, cancellation, concurrency, module distribution, and conformance behavior | `sdks/go/go.mod`; `sdks/go/{client,models,endpoints_gen,client_test}.go`; `sdks/go/cmd/{generate,conformance}/main.go` | `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py`; shared live harness | Preserve deterministic repository-local generation, all-33 operation constants/descriptors, first-argument contexts, loopback and redirect denial, canonical ID/resource construction, mutation-idempotency requirement, identical attempt bytes, bounded body reading, fail-closed manifest input, dependency-free HTTP baseline, and current unit/race/live characterization. Split the package into the planned idiomatic Go files; generate concrete request/result bindings; enforce exact payload/envelope/status/media/identity/stamp/receipt contracts; make credentials opaque; replace broad immediate replay/local-only context cancellation with semantic certainty and correlated server cancellation; own explicit bounded concurrent HTTP/WebSocket transports and endpoint identity; and prove module archive, dependency, toolchain, OS/architecture/race, external-consumer, and installed rrflowMX/rrflowKV cross-surface behavior. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| Java SDK generation, synchronous/async HTTP transport, credential, retry, interruption/cancellation, concurrency, Maven/JAR distribution, and conformance behavior | `sdks/java/pom.xml`; `sdks/java/scripts/generate.py`; `sdks/java/src/main/java/io/rrflow/rrd/{OperationId,RequestOptions,ResourceSegment,RrdApiException,RrdClient,RrdClientException,Session}.java` | `sdks/java/src/test/java/io/rrflow/rrd/{RrdClientTest,SdkConformanceTest}.java`; `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py`; shared live harness | Preserve deterministic repository-local generation, all-33 operation enum/descriptors, reusable JDK client, loopback and redirect denial, canonical ID/resource construction, mutation-idempotency requirement, identical attempt bytes, bounded body reading, interrupt restoration, explicit manifest-absent skip, and current unit/live characterization. Split the runtime into the planned Java classes; generate concrete request/result bindings; enforce exact payload/envelope/status/media/identity/stamp/receipt contracts; make credentials opaque; replace broad immediate replay and local timeout/interruption with semantic certainty plus correlated caller/server cancellation; own an explicit bounded closeable HTTP/WebSocket carriage and endpoint identity; and prove byte-reproducible main/source/Javadoc artifacts, exact offline Maven closure, stable module identity, JDK/Maven and OS/architecture/concurrency matrices, external consumers, and installed rrflowMX/rrflowKV cross-surface behavior. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| .NET SDK generation, asynchronous HTTP/WebSocket transport, credential, retry, cancellation, concurrency, NuGet distribution, and conformance behavior | `sdks/dotnet/Rrflow.Rrd.slnx`; `sdks/dotnet/scripts/generate.py`; `sdks/dotnet/src/Rrflow.Rrd.Client/{Errors,Models,OperationId.g,RrdClient}.cs`; client project and lock files | `sdks/dotnet/tests/Rrflow.Rrd.Client.Tests/{RrdClientTests,SdkConformanceTests}.cs`; test project and lock files; `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py`; shared live harness | Preserve deterministic repository-local operation generation, all-33 enum/descriptors, reusable asynchronous client, loopback and redirect denial, canonical ID/resource construction, mutation-idempotency requirement, identical attempt bytes, absolute-deadline remainder, bounded body reading, disposal, and current mock/live characterization. Pin and centralize the workspace policy; split the runtime into the planned .NET files; generate concrete request/result bindings and source-generated JSON metadata; enforce exact payload/envelope/status/media/identity/stamp/receipt contracts; make credentials opaque; replace broad immediate replay and conflated cancellation with semantic certainty plus correlated caller/server cancellation; own explicitly bounded `SocketsHttpHandler` and `ClientWebSocket` carriage plus endpoint identity; and prove byte-reproducible signed NuGet/source/symbol artifacts, exact offline feed/toolchain closure, trimming/Native-AOT and external consumers, SDK/runtime and OS/architecture/concurrency matrices, and installed rrflowMX/rrflowKV cross-surface behavior. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| Temporal graph, BM25, hybrid retrieval, and context evidence | `rrd-query/src/{bm25,index,execute,plan}.rs`; `rrd-engine/src/engine/{context,retrieval,retrieval_query}.rs` | `rrd-query/tests/{query,index_catalogue,golden}.rs`; `rrd-engine/src/engine/tests/{context,index_foundation}.rs`; `rrd-engine/tests/{runtime_query_trace,runtime_data_plane_trace}.rs` | Replace broad snapshot reconstruction with transactional adjacency/BM25/vector access paths, cost-selected at one stamp and fused with bounded deterministic evidence. | E-01, E-02, E-03, E-05, F-03, H-01, H-02, H-05 |
| Arrow/DataFusion analytical execution | `rrd-query/src/{arrow,fusion,execute,pipeline,plan}.rs` | `rrd-query/tests/{golden,query,index_catalogue}.rs` and the DataFusion-focused unit tests inside the listed source modules | Replace complete `Vec<QueryRow>` materialization with a pinned stamped provider; push supported work into rrflowKV, compose native operators, enforce one resource budget, and report every read/decode/copy/allocation. | F-01 through F-05 |
| Generic reasoning trees, routing, and governed mutation | `rrd-core/src/reasoning_tree.rs`; `rrd-contract/src/{reasoning_tree,router}.rs`; `rrd-engine/src/engine/{context,transaction}.rs` | `rrd-core/tests/{reasoning_tree_contract,reasoning_trace_link}.rs`; `rrd-contract/tests/{reasoning_tree_contract,router_contract}.rs`; `rrd-engine/src/engine/tests/{context,transaction_stamp}.rs` | Preserve the accepted generic tree and three bounded routing decisions; add persisted CAS execution, the model-manifest handshake, constrained LFG dispatch, and engine-selected physical work without a fixed lifecycle. | B-03, G-01 through G-05, H-01, H-05 |
| Durable seat identity, provider representation, and routing attribution | `rrd-contract/src/memory_estate.rs`; `rrd-engine/src/engine/memory_estate.rs`; `rrd-engine/src/operator.rs`; `rrd-core/src/claim.rs`; `rrflow-cli/src/command.rs` | `rrd-engine/src/engine/tests/memory_estate.rs`; `rrflow-cli/tests/operator_surface.rs::identity_bind_resolve_and_readme_warp_share_the_persistent_engine`; `rrd-contract/tests/router_contract.rs`; claim/store golden and grounding tests | Preserve subject-digest redaction, temporal provider-to-seat representation, unrepresented denial, replacement, persistent reopen, and warp-to-context behavior. Rename the ambiguous `MemoryEstate*` surface directly; make D-01 install the specialization-selected seat without a generic Clyffy default; bind the authenticated provider identity, visible representation edge, seat, policy, route packet, proposal, mutation, audit, and trace at one coordinate; reject arbitrary producer actor strings as identity; replace broad snapshot resolution with canonical native access. | A-07, C-03, D-01, G-01 through G-05, H-01, H-04, H-05, J-01 |
| Context projection maintenance draft | `rrd-maintenance/src/lib.rs` | no focused package test and no caller outside the package; the exhaustive preservation/disposition matrix is owned by `docs/reference/context/context-maintenance.md` | Preserve source-cut inventory/accounting, proposal and evidence completeness, review attribution, optimistic conflicts, digest lineage, atomic publication, generation ownership, observation, and compensating rollback through new generic-routine acceptance tests. Replace the direct `StorageEngine`, private scope/event/repository, cursor-zero replay, JSON-wrapper records, fixed seven-stage lifecycle, hardcoded classes/reduction/token policy, and package-local API with generic I-03 routine state and authorized `RrdEngine` operations; remove the standalone crate only after every preserved/generalized matrix row is covered, with no wrapper. | A-07, C-03, C-04, H-02, H-05, I-01, I-03, J-01 |
| Governed functions and proposed-transaction bindings | `rrd-contract/src/{function,lib}.rs`; `rrd-engine/src/engine/function/{mod,catalogue,execution,javascript,webassembly,transaction_binding}.rs`; `rrd-engine/src/engine/{transaction,security,mod}.rs`; `rrd-engine/src/capabilities.rs`; `rrd-engine/Cargo.toml`; `rrd-store/src/control.rs`; locked `rquickjs`/`wasmi` versions in `Cargo.lock` | `rrd-contract::function::tests`; `rrd-contract/tests/function_contract.rs` plus its golden fixture; `rrd-engine::engine::tests::function`; `rrd-engine/tests/function_conformance.rs` plus the shared profile fixture; `rrd-store::control::tests`. The selected JavaScript corpus covers MX/KV result equality and KV reopen; no effect-complete audit crash test, full JavaScript/Wasm runtime-build/target corpus, physical-size boundary, installation test, cross-language fixture, or outward conformance exists. | Retain the A-07.1a canonical names, closed golden contract, old-field rejection, isolated module boundary, and initial MX/KV/reopen corpus. Bind closed schemas, content-addressed artifacts, explicit runtime profiles/builds, prepared receipts, and physically satisfiable limits; replace private control JSON/direct-store access with typed stamped state; commit allowed receipt/audit/event/outbox/domain/index effects atomically; keep JSON sandboxes separate from vectorized rrflowQL/DataFusion functions; add offline install, full MX/KV, crash/reopen/upgrade, resource/security, and real-surface proof. | A-07, C-01 through C-04, D-01, F-03, H-04, H-05, I-01, I-02, I-06, J-01 through J-05 |
| Installation, attunement, and explicit automation | `rrd-contract/src/attunement.rs`; the distinct governed-function boundary under `rrd-engine/src/engine/function/`; `rrflow-cli/src/{command,dev}.rs` | `rrd-contract/tests/attunement_contract.rs`; `rrd-engine/src/engine/tests/{function,deployment_conformance,lifecycle,recovery}.rs`; `rrflow-cli/tests/operator_surface.rs` | Build bundle-resident preview/apply, installed credential references, persisted phase jobs, incremental project specialization, canonical events, resumable routines, digest-bound skills, and optional host translators under the one `RrdEngine` authority. Governed functions remain a subordinate capability and do not supply an automation runtime. Estate provisioning and local authorization remain traced in their dedicated estate/security rows rather than duplicated here. | A-07, D, H-04, I, J-01, J-03, J-05 |
| Project command discovery, installed capability bindings, and external activities | `rrd-contract/src/function.rs`; `rrd-engine/src/engine/function/`; `rrd-core/src/{runtime,trace}.rs`; `rrd-engine/src/runtime/trace.rs`; `rrd-estate/src/local_process.rs`; `rrflow-cli/src/dev/supervisor.rs`; `rrflow-eval/src/main.rs` | function contract/golden/unit/profile-conformance tests; engine lifecycle/trace tests; local-process characterization; CLI supervisor tests; no command-capability, discovery, activity, adapter, or command cross-profile conformance test exists | Retain the bounded function sandbox under function-only names, durable causal trace evidence, and the useful no-shell/path/identity/timeout/effect-gap process safety. Keep provider CLI invocation confined to evaluation. Add pure discovered-fact/candidate/binding/activity/observation/receipt contracts; deterministic discovery; engine-owned prepared dispatch, accepted receipt, reconciliation, and re-inventory; and one outward local activity adapter. Distinguish authenticated direct-process argv from package-script closures that may invoke a shell; deny incomplete closures, ambient authority, adapter-authored completion, and direct storage/query/index access. | A-07, C-03, C-04, D-03, D-06, H-05, I-01, I-03, I-06, I-07, J-01 through J-05 |
| Installed project, estate, instance, environment, and physical topology | `rrd-engine/src/runtime/{instance,mod}.rs`; `rrd-contract/src/{platform,lib}.rs`; `rrd-estate/src/authority.rs`; `rrd-server/src/{main,http/server}.rs`; `rrd-cluster/src/{lib,contract}.rs` | `rrd-engine/tests/runtime_instance.rs`; `rrd-contract/tests/platform_terminology.rs`; `rrd-estate/tests/authority_catalogue.rs`; `rrd-cluster/tests/contracts.rs`; initialization callers inventoried across server, CLI, MCP, Rust client fixtures, and engine tests | Preserve canonical IDs, strict format/input rejection, exact-root containment, foreign-store denial, digest validation, desired/observed and cluster placement/snapshot/transfer safety. Replace the three competing hierarchies with one project ↔ estate ↔ instance relationship graph, operation-specific resource paths, a minimal D-01 locator, and one engine-persisted installed-estate binding shared by rrflowMX and rrflowKV. Remove startup-created manifests, historical word-order authority, private JSON binding, and duplicate operational topology with no compatibility reader. | A-07, B-04, C-02, C-03, D-01 through D-03, D-06, H-04, H-07, J-01, J-03, J-05 |
| Distributed placement, consensus, consistency, replica recovery, transport, and qualification | `rrd-cluster/src/{contract,authority,artifact_transfer,artifact_trace,openraft_adapter,transport,node_runtime,sim,telemetry}.rs`; `rrd-cluster/src/bin/rrd-cluster-node.rs`; `rrd-engine/src/engine/distributed.rs`; `rrd-engine/src/runtime/cluster_transfer.rs` | every `rrd-cluster/tests/{contracts,distributed_authority,artifact_transfer,model_check,simulation,openraft_storage,openraft_snapshot_file,openraft_cluster,openraft_transport,openraft_process}.rs`; `rrd-engine/tests/{distributed_data_plane,runtime_cluster_transfer_trace}.rs`; current full-package failure and hang retained in the distributed reference | Preserve epoch/quorum/failure-domain validation, read modes/stamps/vectors, route evidence, idempotency, Raft log/vote/snapshot safety, authenticated bounded transport, resumable digest-checked artifact closure, admission/telemetry, and replayable faults. Replace loose cluster/tenant/table/scope identity, monolithic JSON catalogue/schema, direct storage/object opening, raw probe/runtime-commit ingress, private transfer-session authority, cursor-zero replay, old/defaulted formats, synthetic trace names, and one-host evidence presented as engine conformance. Replicate only an engine-compiled effect-complete proposal and apply it through an injected engine-owned rrflowKV port; run the complete graph/BM25/vector/RRF/reasoning/Arrow/DataFusion corpus on independent hosts. The first alpha remains single-node; a later roadmap amendment must schedule clustered availability. | A-07, C-01 through C-07, D-01, D-06, E, F, H-04, H-05, H-07, J; later distributed gate required |
| Deployment profiles and cross-profile conformance | `rrd-contract/src/lib.rs::{DeploymentMode,ServiceCapabilities}`; `rrd-engine/src/engine/core.rs::deployment_mode`; `rrd-server/src/http/capabilities.rs`; `fixtures/rrd-deployment-conformance-v1.json`; every generated SDK projection of `deployment_mode` | `rrd-contract/tests/public_contract.rs`; `rrd-engine/src/engine/tests/deployment_conformance.rs`; `rrd-client/tests/real_server.rs`; `rrd-server/tests/http_process.rs`; `rrflow-edge/tests/offline.rs` | Preserve strict fixture validation, MX durability-operation denial, KV writer exclusion/reopen, current DataFusion invocation, real socket/child-process behavior, TLS identity checks, and deterministic edge artifact reads as narrow characterization. Split deployment form, storage profile, endpoint presentation, and security facts; remove root/TLS/client-location inference and speculative active values; replace the two-document overclaim with layered storage-semantic, durability, form, endpoint, cluster, and derived-artifact corpora proving the full graph/index/vector/Arrow/DataFusion/reasoning flow. | A-07, B-04, C-02 through C-04, D-01, E, F, G-04, G-05, H-01 through H-05, H-07, J |
| Local RRD process launch, readiness, observation, and shutdown | `rrd-estate/src/local_process.rs`; `rrd-estate/src/commands/rrd-deployment-catalog.rs`; `rrd-engine/src/engine/estate_control.rs`; `rrflow-cli/src/{command,dev/supervisor}.rs`; `rrflow-cli/src/bin/rrd-estate-controller.rs`; `rrd-server/src/main.rs` | local-process unit test; `rrd-estate/tests/deployment_catalog.rs`; `rrflow-cli` supervisor/controller unit tests; `rrd-server/tests/local_estate_driver.rs`; all catalogue, process-record, marker, initializer, and debug-hold callers/fixtures | Preserve typed no-shell invocation, strict relative-path validation, PID/start/image reauthentication, bounded graceful/forced stop, child reaping, prepared/effect lost-ack convergence, create-new durability patterns, and retained data. Move host code into one injected outward adapter; resolve only the installed artifact/effect plan; bind verified bytes to the executed image; authenticate challenge-bound readiness/control; commit typed plans/receipts through `RrdEngine`; bound/redact diagnostics and resources; remove both supervisors, catalogue/controller binaries, direct-store/static entry points, process JSON, marker authority, startup initialization, plaintext environment state, and all successful prior shapes. | A-07, C-02, C-03, D-01, D-02, H-04, H-05, J-01 through J-03, J-05 |
| Kubernetes deployment projection, reconciliation, readiness, deletion, and qualification | `crates/operations/rrd-kubernetes/src/{lib,controller,main}.rs`; `crates/operations/rrd-kubernetes/src/bin/rrd-kubernetes-crd.rs`; `deploy/kubernetes/{rrdinstances.rrflow.io-crd,operator-rbac,example-rrdinstance}.json`; workspace/CI/package callers | `crates/operations/rrd-kubernetes/tests/{contract,crd}.rs`; no controller, API-server, installation, effect-gap, readiness, deletion, security, or complete-engine conformance test exists | Preserve closed admission, deterministic rendering, pinned image, one retained RWO instance, restricted container foundations, owner/PDB/NetworkPolicy mechanics, and Secret-read denial. Move to `rrflow-kubernetes`; accept only a sealed install or prepared engine effect plan; return observations/receipts for engine acceptance; remove direct desired-state/bootstrap/time/path authority, force apply, phase status, TCP readiness, cluster-wide default, eager finalizer deletion, and placeholder release claims. Prove a clean installed rrflowKV RRD that reopens documents, temporal graph, scalar/BM25/vector indexes, RRF, reasoning/context/evidence, and stamped streamed Arrow/DataFusion execution through the public endpoint. | A-07, B-04, C-02, C-03, D-01, D-02, E, F, G-04, G-05, H-01 through H-05, H-07, J-01 through J-05 |
| Estate desired/observed control and external-effect reconciliation | `rrd-estate/src/{lib,authority,reconcile,backup_job,backup_reconcile,recovery}.rs`; `rrd-store/src/control.rs`; `rrd-engine/src/engine/{estate,estate_control}.rs`; estate controller adapters and server estate-read handler | `rrd-estate/tests/{authority_catalogue,estate_authority,reconciler_recovery,backup_jobs,backup_reconciliation,recovery}.rs`; `rrd-store/tests/control_journal.rs`; `rrd-engine/tests/engine_authority.rs`; `rrd-server/tests/http_process.rs`; `rrflow-cli/tests/backup_controller.rs` | Preserve monotonic desired/observed state, fencing, prepared-before-effect, receipts, lost-ack convergence, activity, backup, and recovery invariants. Make `rrd-estate` pure; split the JSON aggregate into native typed records/relations/indexes; reconcile the duplicate operational-authority hierarchy; and execute every boundary through one installed, authenticated, stamped `RrdEngine` transaction. | A-07, C-01 through C-04, C-06, C-07, D-01, D-02, E-01, E-02, F-01, F-05, H-04, H-05, J-01 through J-05 |
| Durable trace, context-path evidence, and lifecycle-named residue | `rrd-core/src/trace.rs`; `rrd-engine/src/runtime/{mod,trace}.rs`; `rrd-engine/tests/runtime_trace.rs` | trace schema/golden tests; concurrent writer and native-reopen tests in `runtime_trace.rs` | Preserve correlated start/finish/annotation evidence, incomplete-span visibility, deterministic identities, data classes, typed causal links, redaction, freshness, and observable verification. Route durable emission through authorized engine operations; add selected/skipped source, cache/model-context-compaction, work/byte/token/latency, contribution, and outcome evidence sufficient to detect repeated, stale, duplicate, conflicting, missed, or lost context under a fixed comparison rubric. Reserve traces for evidence and converge lifecycle/workflow-named code directly without restoring hooks or inventing hidden reasoning. | A-07, H-05, I, J-01 |

The `<package>/...` shorthand resolves through the frozen target source tree;
for example, `rrd-lsm/src/wal.rs` means
`crates/persistence/rrd-lsm/src/wal.rs`. The generated JSONL contains the exact
repository path and complete byte/line span for every file. This matrix binds
the cross-file behavior that a mechanical inventory cannot infer.

### Mandatory direct convergence

RRFlow has no legacy implementation class. A current symbol or test containing
`legacy` identifies superseded pre-release residue to account for and remove,
not a supported generation to rename or preserve. The table below retains the
exact current symbol spellings only so deletion can be proved. Useful behavior
must first gain equal-or-stronger evidence at its canonical destination; no
successful old-shape fixture, decoder, default, alias, or migration path
survives the owning gate.

| Requirement | Current state and remaining disposition | Gate |
|---|---|---|
| Fjall, `Store`, `PersistentBackend`, `PersistentEngine`, migration, and upgrade runtime surfaces | absent; keep absent and enforce with dependency metadata and symbol searches | A-07, C-05 |
| `rrd-store/src/keyspaces.rs` current RRDSK002 codec | replace with one frozen ordered typed tuple codec and one rejection path for every other storage format | C-01, C-05 |
| `rrd-lsm/src/segment.rs::decode_legacy`, `SegmentStorage::Legacy`, and their seek/validation branches | delete when the hybrid segment format lands; retain exact/read/filter/lifetime behavior through the canonical segment corpus and keep earlier bytes only as fail-closed rejection inputs | C-05, C-06, C-07 |
| pre-1.0 version branching in `rrd-lsm/src/{batch,manifest,segment}.rs` | retain only the final 1.0 version; corrupt/unknown versions fail | C-05, C-07 |
| missing-field and derived-old-shape paths in `rrd-core::{data,runtime,schema,temporal}` and `rrd-contract` | during the full A-07/C-03 contract review, distinguish intentional current optional semantics from defaults that exist only to accept superseded state; make retained optionality explicit in the one schema and remove every old-shape success test/decoder | A-07, B-03, C-03, E-04, J-01 |
| `StorageEngine` | current canonical storage port; narrow it in C-02 without reintroducing aliases | A-07, C-02 |
| `RrflowKvStore`, `RrflowMxStore`, `StorageProfile` | current concrete names; preserve direct naming and verify every caller uses them | A-07 |
| `rrd-contract::DeploymentMode`, `ServiceCapabilities::deployment_mode`, `RrdEngine::deployment_mode`, TLS-derived server mode, generated SDK projections, and `rrd-deployment-conformance-v1.json` | preserve closed validation/serialization, MX durability denial, KV writer exclusion/reopen, current DataFusion invocation, real socket/child-process behavior, TLS identity checks, and deterministic edge-artifact reads only as narrow characterization; split deployment form, storage profile, endpoint presentation, and security facts in planned `rrd-contract/src/deployment.rs`, supply the installed descriptor from the composition root, remove all root/TLS/location inference and speculative artifact/cluster values, regenerate SDKs with no old field, and replace the seed fixture's overclaimed conformance with layered semantic/durability/form/endpoint/cluster/artifact corpora | A-07, B-04, C-02 through C-04, D-01, E, F, G-04, G-05, H-01 through H-05, H-07, J |
| `.rrflow/instance.toml`, `InstanceManifest`, `InstanceMode`, `ProjectAuthorityBinding`, `RrdEngine::{open_project_store,open_bound,bind_project_authority}`, `rrd-server initialize`, and startup/test `ensure_dedicated*` callers | preserve strict version/unknown-field rejection, canonical IDs, create-new publication, exact-root and store containment, digest checking, foreign-store denial, and no silent rebind; replace the manifest/private JSON control value with the sole D-01 `.rrflow/config.toml` locator plus an engine-persisted installed-estate binding shared by rrflowMX and rrflowKV, route fixtures through that contract, and remove every initializer/reader/export with no alias or compatibility path | A-07, C-02, C-03, D-01 through D-03, H-04, J-01, J-03, J-05 |
| `RrdEngine::bootstrap_security_store`, `engine/security_bootstrap.rs`, `SecurityBootstrapOutcome`, `rrd-security-bootstrap`, its absolute-path/caller-time manifest, `security-bootstrap.json`, CLI supervisor invocation, and Kubernetes init-container invocation | preserve only strict input decoding, bounded regular-file reads, unique identity/state validation, verifier-only persistence, atomic policy-plus-audit initialization, exact replay, drift denial, and listener separation. Reimplement those invariants inside D-01 `initialize_instance` over an exclusive fresh-target lease, engine clock, exact reviewed action digest, typed credential verifier, capability-scoped/versioned secret port, prepared delivery receipts, and one installed-binding/policy/checkpoint/audit transaction; prove every crash and file/provider race, then delete every listed symbol, binary, document shape, caller, and success fixture with no alias or reader | A-07, C-02, C-03, D-01, D-02, H-05, J-01 through J-03, J-05 |
| `LocalProcessDriver`, `LocalDeploymentCatalog`, `rrd-deployment-catalog`, `rrd-estate-controller`, `rrflow-cli::dev::supervisor`, `supervisor.json`, process-record JSON, and ready/shutdown marker-file authority | preserve only typed no-shell arguments, strict path bounds, authenticated PID/start/image identity, child reaping, bounded graceful/forced stop, retained data, and effect-gap replay; implement them once in the outward local-process adapter over an immutable installed/fenced plan and typed engine receipt, close the artifact-digest-to-executed-image race, authenticate readiness/control, bound/redact diagnostics and resources, then delete every listed implementation/state/binary/shape with no wrapper | A-07, C-02, C-03, D-01, D-02, H-04, H-05, J-01 through J-03, J-05 |
| `rrd-kubernetes`, `RrdInstanceSpec` as desired RRFlow authority, `RrdInstancePhase`, `rrd-server initialize`, separate `rrd-security-bootstrap` init, caller bootstrap time, unconditional SSA `.force()`, TCP-only readiness, cluster-wide default watch, and eager five-resource finalizer deletion | preserve only the closed CRD/admission, deterministic render, digest-pinned image, one retained RWO StatefulSet, restricted container, owner/PDB/NetworkPolicy, and no-controller-Secret-read foundations; implement them in outward `rrflow-kubernetes` over sealed D-01 bootstrap and engine-prepared effect plans, standard Kubernetes conditions, explicit field ownership/conflict, authenticated application readiness, engine-accepted observations/receipts, fenced retention/deletion, and a complete real-cluster RRFlow corpus; then delete the old operations package and every listed successful shape with no forwarding crate or conversion reader | A-07, B-04, C-02, C-03, D-01, D-02, E, F, G-04, G-05, H-01 through H-05, H-07, J-01 through J-05 |
| `rrd-contract::PLATFORM_TERMS`, generic `ResourcePath` ordering, historical vocabulary order test, `EstateAuthorityResourceKind`, `rrd-estate::AuthorityResourceKind`, and separate cluster identity/scope strings | preserve bounded typed identifiers, duplicate rejection, strict parent checks where canonical, desired/observed and receipt lineage, and cluster placement/snapshot/transfer invariants; freeze one project/estate/instance topology and operation-specific resource grammar, split job/health/secret/security/cluster concepts into their sole owners, unify identity binding, and remove active dependence on the historical table and duplicate catalogue | A-07, B-04, C-03, D-01, D-02, H-04, H-07, J-01 |
| `DistributedAuthorityCatalogue`, `RrdRaftStore`, `rrd-cluster-node`, raw `RrdRaftOperation::{Probe,RuntimeCommit}`, `transfer-sessions-v1`, `cluster.*` traces, and direct cluster `StorageEngine`/`rrd_lsm`/object access | preserve the exact safety behavior enumerated by the distributed contract and its characterization tests; replace the catalogue with typed engine-owned placement state and pure proposals, make consensus an injected `RrdEngine` execution port over the sole current rrflowKV format, accept only engine-compiled authorized commit proposals, persist transfer jobs/receipts through the engine while retaining bounded local staging, consume installed identity/credentials, and rename traces through A-07; remove every listed bypass/shape with no wrapper before any cluster value is advertised | A-07, C-01 through C-07, D-01, D-06, E, F, H-04, H-05, H-07, J-01 through J-05; later distributed gate required |
| `rrd-estate::{LocalOperatorPolicy,LocalOperatorAuthorization,LocalEstatePermission}` and `RrdEngine::*_estate_*_store` path-opening administration | preserve the seven exact least-privilege effects, strict bounds, policy-derived actor, and denial before unauthorized creation; absorb them into canonical principal grants, installed bindings, engine-observed time, and one stamped invocation/transaction path, then delete the file-policy types and arbitrary-path entry points without a wrapper | A-07, C-02, C-03, D-01, H-04, H-05, J-01 |
| `rrd-estate::EstateRepository`, storage-parameterized reconcilers, `EstateDocument`, `server/state/estate/*/document`, and nested `EstateAuthorityState` | preserve the fully enumerated desired/observed, lease/fencing, prepared/effect/observation, idempotency, activity, backup, and recovery requirements in the canonical estate-control record; make the crate pure, reconcile every overlapping topology resource, replace the monolithic JSON/control-journal replacement with native typed records/relations/indexes and one stamped transaction, then delete the direct-store API and old key/shape with no forwarding reader | A-07, C-01 through C-04, C-06, C-07, D-01, D-02, E-01, E-02, F-01, F-05, H-04, H-05, J-01 through J-05 |
| rrflowKV semantic and AI-access benchmarks | current absolute rrflowKV diagnostics; add immutable source/environment provenance and fixed-hardware thresholds before release use | C-06, F-05, J-04 |
| `rrd-query/src/arrow.rs::ArrowSnapshot` | replace with stamped batch/page adapters | F-01 |
| `rrd-query/src/execute.rs::execute` eager loading | split into native access and streaming execution | F-01..F-04 |
| `rrd-query/src/live.rs::poll_live_query` two-snapshot diff | replace with commit-impact evaluation | H-03 |
| `rrd-core/tests/golden.rs` Go/bbolt/LFG parity-engine narrative | remove the provider-specific alternate-engine claim while retaining only the characterized Rust contract vectors needed until C-01 freezes the final codec; Go remains eligible only as an outward capability adapter, never an RRFlow storage or routing authority | A-07, C-01, J-01 |
| `MemoryEstate*`, `memory_estate.rs`, hardcoded Clyffy CLI defaults, caller-committed seat plans, and broad-snapshot seat resolution | preserve provider-neutral seat/provider/relation validation, hashed subject storage, temporal representation replacement, unrepresented denial, persistent reopen, and warp-to-context behavior; rename directly to canonical seat/specialization vocabulary, derive the selected seat from D-01 installation, move planning and commit into one engine operation, and use the final native graph/index path with no alias or generic persona default | A-07, C-03, D-01, E-01, E-02, G-02, H-01, H-04, J-01 |
| `Producer.actor`, CLI `assert --actor`, and examples/tests that treat `agent:clyffy` as sufficient identity | retain a bounded human-readable attribution label only when derived from an authenticated principal and same-stamp provider representation; never authorize, resolve self, or persist routed work from the caller string; replace successful impersonation-capable shapes and prove foreign/revoked/unrepresented denial plus attributed reopen | A-07, C-03, G-02, G-04, H-04, H-05, J-01 |
| Clyffy as a separate kernel, repository, release manifest, management plane, lifecycle, or provider-event family | keep absent. Clyffy is this repository's specialization-selected durable seat; first-party execution remains Rust inside RRFlow, `RrdEngine` orchestrates, model backends only propose, and optional Go/provider/host integrations remain outward capabilities | A-07, D-01, G-01 through G-05, I-01 through I-06, J-01, J-03 |
| `rrd-maintenance::MaintenanceRepository` and its fixed maintenance state/event vocabulary | satisfy every preserve/generalize row in the canonical context-maintenance disposition matrix through focused generic-routine tests, then remove the crate, direct store dependency, private scope, cursor-zero replay, JSON-wrapper records, hardcoded taxonomy/reduction/token policy, and package-local API with no alias or compatibility reader | A-07, C-03, C-04, H-02, H-05, I-01, I-03, J-01 |
| `LEGACY_VECTOR_ARTIFACT_CATALOG_VERSION`, its alternate encoder/decoder, `VectorRuntime::suppress_legacy_turboquant`, and the `ensure_vector_index` compatibility adapter | preserve projection provenance, lifecycle restoration, exact oracle/rerank, mmap, bounded-memory, and recall evidence in one source-stamped vector catalogue and planner; then delete the older catalogue/version/suppression/ensure path and its successful fixtures | E-04, E-05, H-01, J-01 |
| defaulted older estate substate in `rrd-estate::EstateDocument`, optional recovery-policy decoding in `rrd-estate::{backup_job,recovery}`, and their successful missing-field fixtures | preserve current desired/observed, backup, recovery-policy, receipt, lease, and recovery-point semantics in one required canonical estate schema; remove decoding behavior and success tests that exist only for pre-release documents/jobs and add negative old-shape rejection vectors | A-07, C-05, D-02, D-10, J-01, J-02 |
| `claim-transactions` and other public transport paths described as compatibility/fallback surfaces | preserve any distinct bounded operation semantics only through the canonical multi-model transaction, subscription, and WebSocket contracts; delete duplicate capability/handler/client success paths after cross-surface conformance passes | B-04, H-03, H-04, J-01 |
| Five missing Rust catalogue operations, manual route/mutation/retry flags, ordinary-string bearer storage, partial response validation, generic successful-status handling, library-default WebSocket limits far above RRD, dedicated socket behavior, and manifest-absent conformance success; the former implementation monolith and directly printable public bearer field are absent after A-07.1b | retain the A-07.1b direct client/endpoint/error/operation/retry/session/subscription/transport split, credential-private redacted session handle, implementation-free dependency direction, typed surface, loopback restriction, explicit Rustls transport, expected-instance negotiation, four-MiB HTTP cap, durable ACK/reconnect, and real loopback/mTLS/WSS characterization. Derive exact bindings from the catalogue; validate every payload/envelope/status/media/identity; use a limited secret wrapper with explicit copy/zeroization semantics; implement operation-semantic retry/uncertainty and correlated cancellation; configure bounded multiplexed frames; make missing harness input fail or explicitly skip; replace label-only corpus coverage and direct-store fixture installation with structural D-01/public-operation MX/KV cross-surface evidence. No manual route table, ordinary credential string, dedicated socket protocol, or permissive test path survives its owning gate. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| Compile-time-only TypeScript payload types, partial ArkType envelope, any-`2xx` success, ignored media, plain serializable credentials, broad immediate retry, local-only abort, implicit redirect following, loopback-only HTTP, absent WebSocket/W3C, and private raw-source package; the former `src/index.ts` implementation monolith and underscored conformance filename are absent after A-07.1c | retain the A-07.1c direct client/endpoint/error/operation/retry/session/transport modules, narrow export root, direct conformance filename, deterministic local generation, 33-operation generic HTTP type surface, canonical request/resource construction, mutation idempotency requirement, response byte cap, fail-closed conformance input, locked toolchain, and current mock/live characterization. Generate one complete runtime binding per descriptor; reject invalid request/response/status/media/identity; make credentials opaque; implement semantic certainty/cancellation; reject redirects; add the real subscription module plus authenticated bounded Node/browser transport; and ship deterministic ESM JavaScript/declarations through the signed offline distribution. Replace the direct-seeded label-only harness with D-01-installed structural MX/KV/browser/cross-surface evidence. No permissive envelope, plain credential object, source-export package, or second browser lifecycle survives. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| earlier cluster adapter domains | retain only explicit fail-closed format rejection evidence; no opener or migration path may accept the superseded domain | C-05, J-01 |
| Former function catalogue type family and private `AutomationHead` | absent from executable source after A-07.1a; `FunctionCatalogue`, `ReplaceFunctionCatalogue`, `ListFunctionCatalogue`, and `FunctionCatalogueHead` are direct names with no forwarding alias; keep the source guard and closed golden contract passing | A-07, J-01 |
| Former `FunctionTrigger*`, `trigger_id`, and catalogue `triggers` family | absent from executable source and rejected as wire fields after A-07.1a; retain `TransactionFunctionBinding`, `TransactionMutationKind`, `TransactionFunctionEffect`, `binding_id`, and `transaction_bindings`, and reserve `Trigger` exclusively for the later post-commit canonical engine-event predicate | A-07, I-01, I-02, J-01 |
| `server/state/*/function-catalogue/{head-v1,revision/*}` whole-catalogue JSON/control-journal replacement; the former `automation` key prefix is rejected rather than read | replace this remaining private shape with typed function artifacts, definitions, transaction bindings, immutable catalogue membership, prepared invocation receipts, and one CAS head through `RrdEngine`; catalogue references never repeat executable bytes and public maxima must fit the physical transaction | C-01, C-03, C-04, J-01, J-02 |
| `rrflow-eval::run_trial` direct provider-CLI process execution | retain only as an evaluation harness with bounded evidence; it never becomes a runtime capability runner, installed binding, activity adapter, or proof of project-command conformance | A-07, D-06, G-06, I-03, J-02, J-04 |
| `rrd_core::RuntimeEvent` plus planned public `EngineEvent` | converge into one semantic engine-event vocabulary and one lowering path; no forwarding type or parallel event log | I-01, J-01 |
| direct-store durable trace helpers under `rrd-engine/src/runtime/trace.rs` | preserve trace evidence behavior behind authorized `RrdEngine` operations; traces never advance jobs or routine state | H-05, I-03, J-01 |
| provider/session-start hooks anywhere | remain absent; optional host translators submit explicit typed events only | I-04, J-01 |

## Gate A work packages

### A-06.1 — knowledge package contract (KB-02)

Precondition: this execution map and research record are reviewed.

Files:

- repair the pre-existing `platform_terminology.rs` reference to the retained
  historical vocabulary source table without promoting that table back to
  current authority; A-07 still owns the vocabulary audit;
- modify `crates/transport/rrd-contract/src/lib.rs` only to export the new
  module;
- create `crates/transport/rrd-contract/src/knowledge.rs`;
- create `crates/transport/rrd-contract/tests/knowledge_contract.rs`;
- create `crates/transport/rrd-contract/fixtures/knowledge-package-v1.json`;
- do not modify `crates/transport/rrd-contract/src/bin/rrd-contract-export.rs`
  or the OpenAPI fixture in KB-02: the knowledge package is not yet an HTTP
  operation, and a later ingress gate must add any public operation and schema
  through the existing generator.

Required types: `KnowledgeRecordV1`, `KnowledgeClassification`,
`KnowledgeProvenance`, `KnowledgeManifestEntryV1`, `KnowledgeExclusionV1`, and
`KnowledgePackageV1`. Closed decoding must bind coordinate, normalized source
path, classification, owner coordinate, UTF-8 body digest, provenance,
deterministic order, exclusions, and package digest. Package digest input is a
versioned domain separator followed by length-prefixed canonical entry bytes;
never hash ambiguous concatenated strings.

Tests: unknown fields/version, duplicate coordinate/path, unsafe path,
non-normalized line endings, body mismatch, unstable ordering, exclusion
without reason, eligible omission, and package-digest mismatch.

Run:

```text
cargo test -p rrd-contract --test knowledge_contract --locked
cargo test -p rrd-contract --locked
cargo clippy -p rrd-contract --all-targets --locked -- -D warnings
```

Stop if the contract imports a workspace implementation package or embeds a
provider path, secret, runtime storage key, or Markdown parser behavior.

### A-06.2 — deterministic exporter and CI (KB-03, then KB-04)

KB-03 files:

- create `scripts/knowledge/export.py` as the single bootstrap exporter;
- create `scripts/knowledge/test_export.py`; and
- produce only ignored/test-temporary packages; never check a generated
  package in as editable authority.

KB-04 files, which must not change during KB-03:

- modify `scripts/ci/check_documentation.py` to use the frozen eligibility and
  ownership rules;
- modify `.github/workflows/ci-reusable.yml` to run exporter reproducibility;
- extend `scripts/knowledge/test_export.py` only with the CI drift cases owned
  by KB-04.

Execution order: parse headers and links; classify eligible records and
explicit exclusions; normalize UTF-8/LF; order by coordinate then source path;
encode through the KB-02 schema; calculate record and package digests; export
twice to independent temporary directories; compare bytes.

Stop if an eligible Markdown file can disappear without an exclusion record,
if the exporter changes source Markdown, or if path ordering depends on the
host locale/filesystem.

### A-06.3 — flat-record classification (KB-05)

Each row is a separate full-file review and commit. Destination directories
and their `README.md` indexes are created only with their first real record.

| KB-05 baseline record | Planned destination/classification |
|---|---|
| `docs/anytype-ui-research.md` | `docs/reference/client/connectome.md`; preserve only source-pinned projection, graph, timeline, inspector, platform-separation, and measured-renderer requirements; remove historical product topology, retired implementation status, and client-owned lifecycle authority |
| `docs/blueprint-triage.md` | merge every still-open verified deficiency into the POA&M, then remove |
| `docs/clyffy-kernel-alpha.md` | merge accepted provider-neutral orchestration requirements into the system overview or agent-bootstrap owner, then remove |
| `docs/context-path-profiler.md` | merge accepted context measurement requirements into the engine-flow owner and H/J gates, then remove |
| `docs/prompt-flight-experiments.md` | `docs/evidence/test-plans/model-context-effect.md`; preserve only provider-neutral controlled-comparison requirements and claim no execution evidence |
| `docs/runtime-graph.md` | merge accepted temporal-graph semantics into the system and engine-flow owners, then remove |
| `docs/context-maintenance-v1.md` | `docs/reference/context/context-maintenance.md`; rewrite as the target context-projection contract and record the standalone `rrd-maintenance` crate for direct convergence into generic routines |
| `docs/estate-control-v1.md` | `docs/reference/operations/estate-control.md` |
| `docs/instance-topology.md` | `docs/architecture/instance-topology.md`; it defines accepted logical and physical deployment topology |
| `docs/local-estate-authorization-v1.md` | `docs/reference/security/local-estate-authorization.md` |
| `docs/local-process-driver-v1.md` | `docs/reference/deployment/local-process-driver.md` |
| `docs/package-workflows.md` | `docs/reference/automation/project-command-capabilities.md`; rename the ambiguous flat filename because `workflow` is reserved for durable routines; active target contract after full code/contract/test review, with implementation explicitly unclaimed and mapped to D-06/I-03/I-06 |
| `docs/qdrant-capability-inventory.md` | `docs/research/qdrant-capability-inventory.md` |
| `docs/surrealdb-capability-inventory.md` | `docs/research/surrealdb-capability-inventory.md` |
| `docs/rrflow-surrealdb-differential.md` | merge only its reproducibility failures and future comparison requirements into `docs/research/surrealdb-capability-inventory.md`, then remove the document, broken host-bound harness, unqualified result, and stored-ratio assertion; create no evidence archive before J-04 |
| `docs/rrflow-rename-ledger.md` | merge its still-valid identity/cutover rules into the system overview, version policy, roadmap, and A-07 execution requirements, then remove; its V1 compatibility narrative and denial of canonical rrflowMX/rrflowKV names are rejected |
| `docs/versioning.md` | `docs/reference/release/version-policy.md` |
| `docs/rrd-public-contract.md` | `docs/reference/protocol/public-contract.md` |
| `docs/rrd-server-v1.md` | `docs/reference/protocol/server.md` |
| `docs/rrd-live-subscriptions-v1.md` | `docs/reference/protocol/subscriptions.md` |
| `docs/rrflowql-live-query-v1.md` | `docs/reference/query/live-query.md` |
| `docs/rrflowql-multimodel-v1.md` | `docs/reference/query/multi-model.md` |
| `docs/rrflowql-transactions-and-joins-v1.md` | `docs/reference/query/transactions-and-joins.md` |
| `docs/rrflowql-index-catalogue-v1.md` | `docs/reference/query/index-catalogue.md` |
| `docs/rrd-unified-catalogue.md` | `docs/reference/data/schema-catalogue.md` |
| `docs/rrd-data-services-object-contract.md` | `docs/reference/data/multi-model-object-contract.md` |
| `docs/rrd-time-travel-rollback.md` | `docs/reference/data/time-travel-and-rollback.md` |
| `docs/rrd-vector-collections-v1.md` | `docs/reference/vector/collections.md` |
| `docs/rrd-vector-search.md` | `docs/reference/vector/search.md` |
| `docs/rrd-hnsw-online-v2.md` | `docs/reference/vector/hnsw-projection.md`; remove version from title unless it is a wire/format version |
| `docs/rrd-quantization-lifecycle-v1.md` | `docs/reference/vector/quantization-lifecycle.md` |
| `docs/rrd-vector-memory-tiers-v1.md` | `docs/reference/vector/memory-tiers.md` |
| `docs/rrd-inference-edge.md` | split without duplicated text into `docs/reference/inference/embedding-and-model-bound-search.md`, `docs/reference/vector/compact-dense-artifact.md`, and `docs/reference/deployment/edge.md` |
| `docs/rrd-functions-v1.md` | `docs/reference/automation/functions.md`; explicitly subordinate to I gates |
| `docs/rrd-deployment-modes-v1.md` | `docs/reference/deployment/modes.md` |
| `docs/rrd-tiered-persistence.md` | `docs/reference/storage/tiered-persistence.md` |
| `docs/rrd-logical-archive.md` | `docs/reference/storage/logical-archive.md` |
| `docs/rrd-lsm-fjall-ai-audit.md` | merge accepted benchmark requirements into C/J and retain its already-linked raw results as evidence, then remove the duplicate narrative when C-05 removes Fjall |
| `docs/rrd-persistent-scenario-matrix.md` | `docs/evidence/test-plans/persistence-scenario-matrix.md`; do not label unexecuted cells as evidence |
| `docs/rrd-security-v1.md` | `docs/reference/security/authority.md` |
| `docs/rrd-security-bootstrap-v1.md` | merge stable security-bootstrap requirements into the agent-bootstrap and security owners, then remove; D-01 creates an executable installation guide only after behavior exists |
| `docs/rrd-cluster-m7.md` | `docs/reference/distributed/cluster-contract.md`; remove retired milestone from active status/title |
| `docs/rrd-kubernetes-v1alpha1.md` | `docs/reference/deployment/kubernetes-operator.md`; Kubernetes API version may remain where technically real |
| `docs/rrd-rust-client-v1.md` | `docs/reference/sdk/rust.md` |
| `docs/rrd-typescript-client-v1.md` | `docs/reference/sdk/typescript.md` |
| `docs/rrd-python-client-v1.md` | `docs/reference/sdk/python.md` |
| `docs/rrd-go-client-v1.md` | `docs/reference/sdk/go.md` |
| `docs/rrd-java-client-v1.md` | `docs/reference/sdk/java.md` |
| `docs/rrd-dotnet-client-v1.md` | `docs/reference/sdk/dotnet.md` |
| `docs/rrd-unified-retrieval-v1.md` | `docs/reference/context/retrieval.md` |
| `docs/operations/ci.md` | retain at `docs/operations/ci.md`; create the operations index in the same commit |

#### KB-05 execution queue

This is the bounded work order inside KB-05; it is not a second release plan
or completion ledger. Exactly one row is reviewed and committed at a time. A
row leaves this queue only when the same commit adds its resolved-review row,
structured package journal, canonical owner/index changes, generated inventory,
and required verification. A newly discovered dependency or conflicting
authority stops the row and updates this map before work continues.

No rows remain. Every flat supporting record has one resolved-review row and
one package journal; `docs/README.md` is the only top-level `docs/*.md` record.
The complete KB-05/A-06 acceptance corpus passed in the `ci-operations`
package, so the canonical roadmap records A-06 and KB-05 complete.

A-07.0 traceability is complete in its journal below. Execute A-07.1
package/type vocabulary and A-07.2 causal evidence vocabulary as separate
journaled packages. B-03 is the next implementation package only after A-07
is complete.

Resolved full-file reviews:

| Baseline record | Canonical record | Review result |
|---|---|---|
| `docs/rrd-functions-v1.md` | `docs/reference/automation/functions.md`; A-07/C/D/F/H/I/J roadmap and execution packages | Preserved closed content digests, sorted catalogue/binding identity, bounded canonical JSON values, byte/depth/item/numeric/resource limits, fresh JavaScript/Wasm execution, Wasm import denial, one-attempt transaction semantics, pinned catalogue revisions, no recursive derived-effect rematch, validator rejection, and derived-event commit behavior. Corrected the stronger-than-evidence deterministic claim and rejected function/catalogue/lifecycle conflation. Full code-path review exposed wrong `AutomationCatalogue`/`FunctionTrigger*` names; direct `StorageEngine` access; private monolithic JSON catalogue keys; a one-MiB control-value limit that cannot represent advertised maximum Wasm content; journal duplication of executable bytes; separate allowed audits before domain commit; replay against an unbound runtime build; incomplete JavaScript/Wasm profiles and typed error limits; absent schemas/artifact/prepared receipts; and no golden, MX/KV, install, outward, or cross-runtime corpus. The canonical record now fixes exact terminology, module destinations, content-addressed persistence, effect-complete commit/recovery, binary/bundle installation, DataFusion separation, and acceptance evidence without claiming runtime completion. POAM-008 remains open. |
| `docs/package-workflows.md` | `docs/reference/automation/project-command-capabilities.md`; `docs/reference/automation/README.md`; D-06/I-03/I-06 roadmap and execution packages | Renamed the ambiguous flat filename because `workflow` is reserved for durable routines. Preserved semantic capability names, explicit installation, exact bindings, bounded effects, freshness, verification, redaction, evidence, cross-surface authority, and the complete rejection of provider/editor/session hooks. Corrected the unsafe claim that literal `pnpm run` argv makes a command shell-free: direct-process and package-script bindings now have distinct closed invocation semantics, and mutable or unenumerable lifecycle closure fails closed. Added distinct discovered-fact, candidate, installed-binding, prepared-plan, observation, accepted-receipt, and routine-step objects; effect uncertainty/reconciliation; platform-enforced sandbox/resource policy; project re-inventory; rrflowMX/rrflowKV recovery semantics; and the strict graph/index/DataFusion boundary. Full current code/test review proved only the bounded function sandbox, synchronous transaction-function bindings, durable trace foundations, local RRD process safety fragments, duplicate CLI supervisor, and an evaluation-only provider CLI harness. It found no project-command contract, candidate discovery, engine operation, local activity adapter, installation surface, or conformance test. POAM-008 remains open and exact planned files are now assigned; no runtime capability was claimed. |
| `docs/clyffy-kernel-alpha.md` | `docs/reference/seat-identity.md`; `docs/reference/agent-bootstrap.md`; `docs/architecture/{system-overview,engine-data-flow}.md`; D/G roadmap and execution packages | Preserved Clyffy as this repository's durable provider-neutral seat, explicit provider-representation metadata, bounded provider capability truth, credential separation, three-proposal model routing, engine-owned deterministic decisions, and reproducible provider/model evaluation requirements. Rejected Clyffy as another kernel/repository/release/runtime, RRO/Automaton and Fjall/compatibility authority, separate multi-instance MCP or management plane, provider/session hooks and lifecycle events, fixed product tiers, obsolete milestone/gate completion, and claimed competitive results without final subsystem evidence. Full contract/code review proved persisted subject-digest representation, replacement, unrepresented denial, reopen, and warp-to-context behavior plus the three router proposal shapes; it also exposed ambiguous `MemoryEstate*` naming, Clyffy-hardcoded CLI defaults, caller-committed plans, broad snapshot resolution, arbitrary `Producer.actor` attribution, no installed specialization, no authenticated same-stamp seat/authorization/route binding, and no `RouterBackend`. POAM-007/009 and A/C/D/G/H/J own direct convergence. |
| `docs/rrd-security-bootstrap-v1.md` | `docs/reference/security/authority.md`; `docs/reference/agent-bootstrap.md`; D-01 roadmap/execution package | Preserved strict input decoding, bounded non-empty regular-file intent, unique identities, complete policy validation, verifier-only persistence, atomic initial policy/audit publication, exact replay, drift denial, and the rule that a network listener never offers unauthenticated bootstrap. Rejected its executable/current status, standalone command and manifest dialect, arbitrary database/absolute credential paths, caller-selected time, false validation-before-open and authenticated-transition claims, resolve-then-open race, provider-specific symlink/mode policy in the engine, no-op non-Unix privacy check, incomplete idempotency coordinates, unbounded credential validity example, SHA-256 as a generic verifier, and CLI/Kubernetes-authored policy. The accepted target is the sole local D-01 `initialize_instance` action with exact preview/apply, fresh-target proof, engine time, typed verifiers, capability-scoped/versioned secret adapters, prepared delivery receipts, one installed-binding/policy/checkpoint/audit transaction, and crash/reopen/secret-accounting evidence; no task guide was published before behavior exists. |
| `docs/rrflow-rename-ledger.md` | `docs/architecture/system-overview.md`; `docs/reference/release/version-policy.md`; this roadmap/execution map | Preserved the single in-place pre-release identity, exhaustive naming-surface audit, repository-contained source, no-alias/direct-cutover rule, fixture verification, and separation of naming from capability evidence. Rejected its V1 compatibility-domain framing, stale statement that rrflowMX/rrflowKV cannot be canonical names, stale milestone/status authority, and successful earlier-format read requirement. RRFlow now explicitly has no legacy/deprecation line: required semantics are absorbed into one owner before superseded paths and success fixtures are removed. |
| `docs/versioning.md` | `docs/reference/release/version-policy.md` | Preserved the frozen `1.0.0` product-version source, mirrored-package guard, explicit-owner change control, independently versioned technical identities, and separate Connectome repository boundary. Added the release reference index and updated the executable version guard, CODEOWNERS, and root warp map atomically so no old-path check or duplicate policy remains. |
| `docs/prompt-flight-experiments.md` | `docs/evidence/test-plans/model-context-effect.md` | Replaced the retired workbench/provider lifecycle with a content-addressed, provider-neutral experiment contract. Preserved fixed task/project/context comparisons, observable event and token/resource accounting, repeated-trial discipline, retained failures, and no hidden chain-of-thought; removed Codex/Claude command flags, hardcoded effort profiles and arms as runtime authority, lexical acceptance, unproved fresh-session claims, prompts in process arguments, and loopback UI behavior. The new record explicitly claims no harness or evidence. |
| `docs/blueprint-triage.md` | Current deficiencies merged into `docs/poam/rrflow-1.0-alpha.md`; implementation requirements were already owned by C-01, C-06, C-07, F-01, F-04, and F-05 | Removed the superseded Fjall/Vortex/Clyffy architecture, stale ports and task claims, fixed retrieval thresholds, and non-authoritative upstream comparisons. Preserved the valid distinction between mapping bytes and borrowing eligible Arrow buffers, plus bounded stream/backpressure, snapshot-retention, block-pinning, cancellation, compaction, and cache-accounting requirements; no implementation-specific async bridge was mandated before the stamped provider is designed. |
| `docs/rrflowql-multimodel-v1.md` | `docs/reference/query/multi-model.md` | Preserved the tested source-family, temporal, traversal, and typed-predicate semantics; replaced the removed Fjall claim with rrflowMX/rrflowKV evidence; and made the eager `QueryRow`/Arrow allocation plus native-access-path gaps explicit. |
| `docs/rrflowql-index-catalogue-v1.md` | `docs/reference/query/index-catalogue.md` | Preserved the tested persistent catalogue, snapshot artifacts, lifecycle, exact selection, uniqueness, and corruption behavior; removed stale Fjall and incomplete-index claims; and distinguished reconciliation evidence from native incremental maintenance. |
| `docs/rrflowql-transactions-and-joins-v1.md` | `docs/reference/query/transactions-and-joins.md` | Preserved the tested same-stamp join and engine-owned mutation-program behavior; removed the retired milestone and Fjall claims; and separated existing eager execution from the C/F transaction and streaming targets. |
| `docs/rrflowql-live-query-v1.md` | `docs/reference/query/live-query.md` | Preserved the tested resumable semantic-delta contract; removed stale three-engine and milestone-style claims; and made the current two-snapshot materialization distinct from H-03 commit-impact evaluation. |
| `docs/rrd-unified-catalogue.md` | `docs/reference/data/schema-catalogue.md` | Preserved the tested logical model, validation, atomic schema/data, and reopen semantics; identified empty-table derivation as C-05 removal inventory; and separated the coherent replay oracle from C-04 native reads. |
| `docs/rrd-vector-collections-v1.md` | `docs/reference/vector/collections.md` | Preserved the tested collection, named-vector, point, payload-definition, deletion, and reopen behavior; identified the control-record catalogue and whole-log reads; and withheld native index and recall claims until C/E/F evidence exists. |
| `docs/rrd-vector-search.md` | `docs/reference/vector/search.md` | Preserved the tested exact oracle, planner, HNSW, overlay, rerank, reopen, and bounded recall evidence; removed duplicated collection/retrieval/quantization narrative; and exposed whole-log reconstruction plus the older catalogue reader as C-04/C-05 work. |
| `docs/rrd-hnsw-online-v2.md` | `docs/reference/vector/hnsw-projection.md` | Preserved format-v2 identity, deterministic traversal, immutable generation, exact-overlay, filtering, and recall semantics; clarified that incremental advance clones the active graph; and exposed JSON graph storage plus whole-log candidate discovery as E-04/C-04 work. |
| `docs/rrd-quantization-lifecycle-v1.md` | `docs/reference/vector/quantization-lifecycle.md` | Preserved the tested canonical lifecycle, binary mmap codecs, exact rerank, recovery, byte, bias, and recall evidence; exposed whole-log full rebuild input; and marked the TurboQuant ensure adapter for C-05/J-01 removal. |
| `docs/rrd-vector-memory-tiers-v1.md` | `docs/reference/vector/memory-tiers.md` | Preserved the tested hard-bounded pinned, byte-bounded cached LRU, transient cold mmap/owned, pressure, and restart behavior; separated artifact residency from rrflowMX and DataFusion memory; and exposed whole-log candidate reconstruction plus JSON HNSW cold loading as C-04/E-04 work. |
| `docs/rrd-inference-edge.md` | `docs/reference/inference/embedding-and-model-bound-search.md`; `docs/reference/vector/compact-dense-artifact.md`; `docs/reference/deployment/edge.md` | Split the fully reviewed mixed record without duplicated authority: preserved provider-neutral provenance and same-stamp search, the verified aligned mmap artifact and accelerator gate, and the narrow offline CLI evidence; exposed non-durable backend installation, whole-log vector input, non-Arrow pages, absent production GPU adapters, and the edge helper's lack of rrflowDB/install/attunement behavior. |
| `docs/rrd-unified-retrieval-v1.md` | `docs/reference/context/retrieval.md` | Preserved the strict recursive keyword/vector, RRF, rerank, shaping, stage-evidence, permission, and reopen semantics; corrected the false single-scan claim; and exposed repeated whole-log reads, metadata-only payload-index admission, in-memory fusion/shaping, absent graph leaves, engine-only delivery, and lack of reasoning-tree persistence as C/E/F/G/H work. |
| `docs/rrd-data-services-object-contract.md` | `docs/reference/data/multi-model-object-contract.md` | Preserved the tested mixed-model transaction, authenticated stamp, projection work, audit, idempotent outcome, immutable-object visibility, and local/S3-port semantics; removed stale compatibility, Fjall, milestone, and production-provider claims; and exposed JSON keyspaces, whole-log reconstruction, non-transactional object bytes, absent native adjacency/index paths, and non-streaming DataFusion input as C/E/F work. |
| `docs/rrd-time-travel-rollback.md` | `docs/reference/data/time-travel-and-rollback.md` | Preserved the tested independent valid-time/known-cursor reads, historical index rejection, and forward-only structural compensation design; removed stale backend and outward-surface claims; and exposed the eager replay path plus the untested, plan-only record/relation compensation boundary as C/E/F/H and future delivery work. |
| `docs/rrd-tiered-persistence.md` | `docs/reference/storage/tiered-persistence.md` | Preserved the tested local mmap/io_uring/bounded segment I/O, separate cache accounting, immutable-object port, and snapshot movement primitives; removed stale RRD and gate names; and distinguished those primitives from unimplemented Arrow-page zero-copy, remote segment placement, DevForge CoW placement, and engine-coordinated hot-to-cold hibernation. |
| `docs/rrd-logical-archive.md` | `docs/reference/storage/logical-archive.md` | Preserved the tested bounded stable-cut export, framed integrity checks, resumable restore, exact runtime/audit replay, catalogue pruning, and optional object/catalogue closure; distinguished a generic logical source from rrflowKV-only restore; exposed the logical-only CLI, excluded projections/telemetry/leases/physical pages, and pre-release `rrd` format names; and kept this recovery path separate from C/E/F online persistence, indexing, and DataFusion execution. |
| `docs/rrd-lsm-fjall-ai-audit.md` | Requirements merged into C-06, C-07, and J-04; raw results remain under `eval/results/` and are linked by `docs/history/rrd-lsm-promotion-benchmark.md` | Removed the duplicate flat narrative after confirming Fjall is absent from workspace manifests and runtime source. Preserved mixed-family policy admission, bounded maintenance/backpressure, operational counters, compaction/cache interference, long-duration RSS, exact provenance, and physical-byte accounting as current acceptance requirements; retained dated results only as historical evidence that cannot close a 1.0 gate. |
| `docs/rrd-persistent-scenario-matrix.md` | `docs/evidence/test-plans/persistence-scenario-matrix.md` | Replaced the stale 16-item mixed checklist with separate current-characterization and planned-acceptance matrices after reading every cited surviving test file. Corrected renamed rrflowMX/rrflowKV tests, removed nonexistent format-upgrade and Fjall-migration scenarios, retained honest reopen/security/backup/multimodal behavior, exposed eager/log-replay/compatibility-path limits, and added explicit C/E/F/G/H proof for one persistent Arrow/DataFusion reasoning and recall flow. |
| `docs/runtime-graph.md` | Accepted current semantics merged into `docs/architecture/engine-data-flow.md`; overlapping boundaries remain in `docs/architecture/system-overview.md` and canonical query/data references | Preserved the tested bounded global-cursor paging, reopen/hash-chain, valid/known-time snapshot, and directed-traversal behavior plus the implemented cursor-addressed event nodes, deterministic `emitted` edges, and structural differential that E-01 must cover against an exact oracle. Removed obsolete RRO composition, compatibility/Fjall authority, migration promises, duplicated catalogue/query material, stale endpoints, and a milestone-style next-work list; made whole-log graph reconstruction an explicit C-03/C-04/E-01 gap. |
| `docs/context-path-profiler.md` | Accepted engine evidence and optimization requirements merged into `docs/architecture/engine-data-flow.md`, H-05, and implementation traceability | Rejected a standalone profiler, provider lifecycle, hardcoded review cadence, deletion quota, hidden chain-of-thought view, and obsolete Clyffy/Automaton sequencing. Preserved fixed-rubric context-path comparison, selected/skipped route and resource attribution, repeated/missed/stale/duplicate/conflicting/compaction-loss diagnostics, immutable evidence drill-down, and reversible evidence-gated consolidation under `RrdEngine`; distinguished the tested durable trace foundation from unimplemented end-to-end instrumentation. |
| `docs/context-maintenance-v1.md` | `docs/reference/context/context-maintenance.md` | Preserved immutable canonical history, source-stamped derived projections, protection, deterministic proposal, explicit review, evidence-gated activation, observation, and compensating rollback. Replaced hot/warm/cold storage ambiguity with delivery behavior, removed hardcoded classes and reduction percentages, made the seven safety steps a generic routine template rather than a new lifecycle, exposed every current `rrd-maintenance` authority violation and absence of tests/callers, and assigned direct convergence with no wrapper. |
| `docs/estate-control-v1.md` | `docs/reference/operations/estate-control.md` | Preserved strict desired/observed separation, monotonic generations, supersession, idempotency, leases/epochs, prepared-before-effect, receipts, lost-ack convergence, activity evidence, quiesced backup, recovery policy/holds/prune/restore, typed adapters, and real process-kill characterization. Rejected the public direct-store repository, storage-parameterized reconcilers, caller clock/paths, split audit, whole-estate JSON/control-journal replacement, unreachable aggregate-cardinality implication, stale Connectome private/synthetic projection claims, and broad nested operational “authority” as the target. Defined typed estate record/relation/index families, one installed authorized transaction path, explicit rrflowMX/rrflowKV behavior, native fast access, and stamped read-only Arrow/DataFusion analytics; POAM-019 owns persistence convergence, while the accepted instance topology and POAM-020 own resource-kind reconciliation. |
| `docs/instance-topology.md` | `docs/architecture/instance-topology.md` | Rejected the conflicting claim that one estate manages many project instances, the SurrealDB-like organization → estate → project → environment → instance → namespace → database → tenant chain, format-1 compatibility/migration promises, and `.rrflow/instance.toml` as canonical installation authority. Established one project ↔ one estate/rrflowDB ↔ one RRD instance for the first alpha; made environments evidenced project context, workspace derived build topology, and organization/tenant/namespace/database non-mandatory topology; separated embedded/server/cluster deployment from rrflowMX/rrflowKV storage; and defined physical cluster/node/shard/replica/segment relationships. Full code review preserved strict IDs, containment, digest, desired/observed, placement, stamp, transfer, and reshard safety while exposing three competing active topology models, startup-created authority, private absolute-path JSON binding, arbitrary public resource ordering, and split cluster identities as POAM-020 direct-convergence work. |
| `docs/local-process-driver-v1.md` | `docs/reference/deployment/local-process-driver.md` | Preserved typed no-shell invocation, strict relative-path validation, PID/start/executable reauthentication, bounded readiness and graceful/forced shutdown, child reaping, prepared/effect lost-ack convergence, create-new durability patterns, retained data, and the real controller crash matrix. Rejected OS/process/storage dependencies inside `rrd-estate`, the standalone catalogue and controller, the duplicate CLI supervisor, arbitrary roots/time/configuration, startup initialization, private JSON authorities, file-existence readiness/completion, plaintext environment state, unbounded diagnostics, and the digest-to-executed-image race. Defined one planned outward adapter consuming immutable installed/fenced plans, authenticated challenge-bound readiness/control, engine-accepted typed receipts, explicit rrflowMX/rrflowKV semantics, and final cross-platform/resource/failure proof; POAM-021 owns direct convergence. |
| `docs/rrd-deployment-modes-v1.md` | `docs/reference/deployment/modes.md` | Rejected the six-value scalar mode as a target because it mixes rrflowMX storage, embedded/server form, client-relative transport, an edge artifact, and unavailable cluster behavior. Preserved strict fixture validation, MX durability-operation denial, KV writer exclusion/reopen, current DataFusion invocation, real socket/child-process behavior, TLS identity checks, and deterministic mmap artifact reads as narrow characterization. Established independent deployment-form, storage-profile, and endpoint-presentation coordinates, a valid alpha matrix, explicit install/discovery binding, one common `RrdEngine`/rrflowQL/Arrow/DataFusion flow, and layered semantic/durability/form/endpoint/cluster/artifact proof. Full code and generated-surface review exposed root- and TLS-derived mode inference plus a two-document seed corpus falsely named conformance; POAM-022 owns direct convergence. |
| `docs/rrd-security-v1.md` | `docs/reference/security/authority.md` | Preserved bounded principal/role/grant validation, API-key and bounded JWT identity, exact third-party binding, credential-revision invalidation, row/field query policy, independent audit lineage/export, and current mutual-TLS evidence. Rejected `rrd-security` as an independent policy/storage authority, obsolete RRO provisioning, Fjall compatibility claims, and missing-policy access as an accepted mode. Exposed its public direct-store repository, separate policy/data observations, operation-wide mutation authorization, standalone bootstrap, split domain/audit commits, and incomplete production transport/identity proof as POAM-016 work owned by A/C/D/F/H/J. |
| `docs/local-estate-authorization-v1.md` | `docs/reference/security/local-estate-authorization.md` | Preserved strict bounded file decoding, exact estate/action/time/key scope, seven real least-privilege effect distinctions, denial before unauthorized database creation, policy-derived journal actor, typed results, idempotent replay, quiesced backup admission, and fenced identity-based prune/restore safeguards. Rejected the file as policy truth, caller-selected authorization time and physical paths, static store-opening methods as canonical engine operations, and broad allowed audit as authorization. Proved the supposed one-authority test actually succeeds through a second local policy without a canonical estate-admin grant; POAM-017 owns direct absorption. The owning suite also exposed successful older estate/backup shapes, now tracked by POAM-018. |
| `docs/rrd-public-contract.md` | `docs/reference/protocol/public-contract.md` | Preserved the strict `rrd` protocol identity/version, explicit resource and correlation coordinates, request/response/error envelopes, mutation idempotency requirement, typed multi-model/query/vector/delivery vocabulary, deployment and capability discovery, generated endpoint/OpenAPI authority, golden samples, and shared conformance inputs. Corrected the stale 34-operation claim to the executable 33-operation catalogue and `memory` to the actual `rrflow_mx` wire value; separated exported Rust types, catalogued operations, runtime capability discovery, and behavioral proof; and mapped every contract family to the required rrflowMX/rrflowKV, native graph/BM25/vector, rrflowQL/Arrow/DataFusion, reasoning, attunement, and cross-surface gates. Full review exposed a nonexistent backup route in the frozen sample, retired gate labels in runtime capability text, and successful alternate transaction/vector branches; POAM-015 now owns their direct convergence rather than this documentation move silently changing behavior. |
| `docs/rrd-rust-client-v1.md` | `docs/reference/sdk/README.md`; `docs/reference/sdk/rust.md` | Preserved the implementation-free normal dependency, typed async methods, loopback cleartext restriction, explicit Rustls network path, expected protocol/instance checks, four-MiB HTTP cap, explicit request coordinates, durable subscription ACK/reconnect behavior, and real loopback/mTLS/WSS characterization. Corrected the false entire-catalogue claim to 28 of 33 HTTP operations and the false read-retry claim; distinguished local future drop from server cancellation; and exposed partial payload/envelope/result/status/media validation, bearer disclosure through derived `Session` debug, immediate boolean retries, library-default WebSocket limits far above RRD, incomplete frame identity checks, absent endpoint resolver/W3C propagation, string-erased transport causes, and HTTP/1.1-only carriage. Proved that the default Rust conformance test returns success without executing a scenario, while a manually configured live harness runs only an overstated rrflowKV fixture that bypasses installation. The canonical record freezes the exact module, operation, retry/certainty, credential, subscription, resolver, trace, corpus, MX/KV, and cross-surface requirements owned by A-07/B-04/D-01/H/J; POAM-011 remains open. |
| `docs/rrd-typescript-client-v1.md` | `docs/reference/sdk/typescript.md` | Preserved repository-local deterministic OpenAPI generation, locked TypeScript tooling, the generic all-33 HTTP operation type surface, loopback restriction, canonical ID/resource envelope construction, mutation idempotency requirement, bounded streamed body reading, caller abort, fail-closed manifest configuration, and current mock/live characterization. Corrected the stale 34-operation claim to 33 and the implication of runtime payload validation or browser/package qualification. Full source and adversarial review proved that an unpinned query is replayed and then accepts `201 text/plain` with an invalid payload, while the session serializes its bearer; it also exposed broad error/timeout retry, missing exact validators/status/media/digest checks, implicit redirects, absent HTTPS/WebSocket/W3C, no browser credential/CORS proof, and a private raw-TypeScript package with no JavaScript/declarations. The canonical record fixes the Node/browser/transport/package boundary and assigns direct A-07/B-04/D-01/H/J convergence; POAM-011 remains open. |
| `docs/rrd-python-client-v1.md` | `docs/reference/sdk/python.md` | Preserved repository-local deterministic OpenAPI generation, locked Python tooling, the generic all-33 HTTP operation literal/descriptor surface, loopback and redirect denial, reusable HTTPX client, canonical ID/resource construction, mutation idempotency requirement, bounded streamed body reading, fail-closed manifest input, buildable wheel/sdist, and current mock/live characterization. Corrected the stale 34-operation claim to 33 and the implication that Pydantic validates operation payloads or that the package is qualified. Full source and adversarial review proved that an invalid unpinned query is replayed and then accepts `201 text/plain` with an invalid payload while a public mutable session dictionary serializes its bearer; it also exposed missing async/WebSocket/remote transport, exact validators/status/media/GET-correlation/digest checks, broad immediate timeout/transport retry, local-only deadlines, discarded error details, ambient HTTPX trust/resource defaults, absent W3C propagation, missing `py.typed` and release metadata, and no interpreter/platform/offline-consumer matrix. The canonical record fixes the one shared sync/async semantic layer, transport/package boundary, and installed rrflowMX/rrflowKV full-engine conformance without treating any client result as graph/index/Arrow/DataFusion/reasoning proof; POAM-011 remains open. |
| `docs/rrd-go-client-v1.md` | `docs/reference/sdk/go.md` | Preserved repository-local deterministic OpenAPI generation, the standard-library-only HTTP baseline, all-33 operation constants/descriptors, first-argument `context.Context`, loopback and redirect denial, a reusable concurrent HTTP client, canonical ID/resource construction, mutation idempotency requirement, identical client-attempt bytes, bounded body reading, fail-closed manifest input, and current unit/race/live characterization. Corrected the stale 34-operation claim to 33 and the implication of strict operation validation, credential-safe errors, read-only retry, or module qualification. Full source and adversarial review proved that an invalid unpinned query is replayed and then accepts `201 text/plain` with an invalid payload while exported session fields disclose the bearer through JSON and formatting; it also exposed untyped request/results, missing exact status/media/GET-correlation/digest checks, broad immediate retry after transport and per-attempt timeout, local-only cancellation, ambient shared default-transport policy, absent close/HTTPS/WebSocket/W3C paths, and no minimum-toolchain/platform/module-archive/offline-consumer proof. The canonical record keeps Go an outward concurrent client, fixes its idiomatic one-package file and module boundary, and assigns installed rrflowMX/rrflowKV full-engine conformance without treating any client result as graph/index/Arrow/DataFusion/reasoning proof; POAM-011 remains open. |
| `docs/rrd-java-client-v1.md` | `docs/reference/sdk/java.md` | Preserved repository-local deterministic OpenAPI generation, the reusable JDK HTTP baseline, all-33 operation enum/descriptors, loopback and redirect denial, canonical ID/resource construction, mutation idempotency requirement, identical client-attempt bytes, bounded body reading, thread-interrupt restoration, explicit manifest-absent skip, and current unit/live characterization. Corrected the stale 34-operation claim to 33 and the implication of bounded operation validation, safe credentials, or Maven/JAR qualification. Full source and adversarial review proved that an invalid unpinned query is replayed and then accepts `201 text/plain` with an invalid payload while public record components disclose the bearer through formatting and Jackson; it also exposed untyped request/results, missing exact status/media/GET-correlation/digest checks, broad immediate retry after I/O and request timeout, no async/server cancellation, ambient JDK proxy/executor/pool policy, absent connect-timeout/close/HTTPS/WebSocket/W3C paths, non-reproducible JAR bytes, an empty-cache offline Maven failure, derived module identity, and no JDK/platform/consumer/signed-closure proof. The canonical record keeps Java an outward client, fixes its class/artifact boundary and explicit carriage-selection test, and assigns installed rrflowMX/rrflowKV full-engine conformance without treating any Java result as graph/index/Arrow/DataFusion/reasoning proof; POAM-011 remains open. |
| `docs/rrd-dotnet-client-v1.md` | `docs/reference/sdk/dotnet.md` | Preserved repository-local deterministic OpenAPI generation, the dependency-free reusable asynchronous HTTP baseline, all-33 operation enum/descriptors, loopback and redirect denial, canonical ID/resource construction, mutation idempotency requirement, identical client-attempt bytes, absolute-deadline remainder, bounded body reading, disposal, and current mock/live characterization. Corrected the stale 34-operation claim to 33 and the implication of typed operation validation, safe credentials, or qualified NuGet output. Full source and adversarial review proved that an invalid unpinned query is replayed and then accepts `201 text/plain` with an invalid payload while public records disclose the bearer through formatting and `System.Text.Json`; it also exposed arbitrary object/`JsonElement` request/results, missing exact status/media/GET-correlation/digest checks, broad immediate retry after transport and timeout cancellation, conflated cancellation sources, ambient handler proxy/cookie/pool/header policy and independent client timeout, absent remote/WebSocket/W3C paths, a false-pass manifest-absent conformance test, non-reproducible unsigned NuGet bytes, an empty-feed solution-restore failure, placeholder metadata, and no pinned SDK/platform/external/trimmed/Native-AOT consumer proof. The canonical record keeps .NET an outward client, fixes its workspace/class/package boundary, and assigns installed rrflowMX/rrflowKV full-engine conformance without treating any .NET result as graph/index/Arrow/DataFusion/reasoning proof; POAM-011 remains open. |
| `docs/qdrant-capability-inventory.md` | `docs/research/qdrant-capability-inventory.md` | Replaced a flat feature list with a source-pinned Qdrant `v1.19.1` reference at commit `6ab21cac18ebb6f4ae29102c7f8f5cc11affd5de`. Preserved useful collection/segment, WAL visibility, exact/HNSW, filter planning, lexical/sparse, recursive hybrid query, quantization, memory-placement, resource, recovery, edge, security, and operational behavior as bounded implementation and failure inputs. Rejected Qdrant as a dependency, sidecar, catalogue, estate, graph, transaction, API, storage-format, compatibility, or lifecycle authority. Full RRFlow source review exposed cursor-zero history reconstruction across vector paths, JSON HNSW generations, schema-only payload-index admission, Rust-collection fusion, a successful TurboQuant compatibility adapter, and eager DataFusion `MemorySource` materialization. The canonical record maps each gap to C/E/F/H/J, mandates exact upstream provenance plus independent oracle/fault/resource evidence before adaptation, and forbids performance or ease claims until J-04's like-for-like comparison passes; no runtime capability or gate completion is claimed. |
| `docs/surrealdb-capability-inventory.md` | `docs/research/surrealdb-capability-inventory.md` | Replaced a flat RRFlow feature/status scorecard with a source-pinned SurrealDB `v3.2.4` reference at commit `93ab219d69f09d8f999851b0359c80ebe6726102`; isolated `v3.3.0-beta.3` as a preview watchpoint. Preserved useful modular-core, native-model, ordered-key, effect-complete mutation, snapshot/conflict, graph-adjacency, index lifecycle, physical write-fanout, schema/extension, live/changefeed, security, protocol, operations/UI, and readiness behavior as bounded implementation and failure inputs. Rejected SurrealDB as a dependency, storage/backend menu, namespace topology, query/wire format, compatibility, migration, product, cloud, adapter, or lifecycle authority. Full RRFlow reads confirmed substantive WAL/read-stamp/DataFusion/subscription foundations while exposing JSON row/keyspace values, a broad storage port, separately committed catalogues, relation-ID-only persistence, cursor-zero reconstruction, eager `MemorySource` batches, snapshot index artifacts, relation-scan traversal, and two-snapshot live polling. The canonical record explicitly reconciles all thirteen superseded scorecard sections, maps each retained pattern and gap to C through J, freezes an exact adaptation protocol and comparison contract, and claims no runtime capability or gate completion. |
| `docs/rrflow-surrealdb-differential.md` | `docs/research/surrealdb-capability-inventory.md#rejected-local-claim-diagnostic-and-retained-corrections` | Removed the 60-line ratio summary, 591-line Python comparator, 640-line stored result, and Rust test that asserted favorable stored ratios after full review. The baseline test passed two tests because it parsed checked-in numbers; a fresh current `engine_benchmark` build proved the advertised reproducer exits before trial zero because the script supplies an obsolete child argument. The result also omitted RRFlow revision/binary/build closure, used SurrealDB `3.0.5` rather than the pinned `3.2.4` reference, compared unlike embedded/HTTP/readiness timing layers, used unequal session values and field verification, and lacked fixed hardware, resource symmetry, failures, confidence treatment, or complete graph/BM25/vector/Arrow/DataFusion/context workloads. The canonical SurrealDB record now preserves each defect as a required J-04 correction. No artifact was archived, no ratio or superiority claim survives, and the sole planned comparator remains `scripts/release/compare_deployment.py` plus its pinned baseline manifest after prerequisites pass. |
| `docs/anytype-ui-research.md` | `docs/reference/client/README.md`; `docs/reference/client/connectome.md`; H-06/H-07/J-03/J-05 roadmap and execution packages | Replaced the flat historical workbench snapshot with one client contract source-pinned to Anytype `v0.55.4` commit `f4677a073e41be6bfcf34a21b433027a3b3851aa`. Preserved stable object identity, multiple projections over one result, selection-centered graph and typed timeline interaction, frontend/engine separation, composable inspectors, local-only lens preferences, explicit action preview, cursor delivery, exact instance scope, and worker/WebGL only after measurement. Rejected the historical estate/table/sidebar product map, retired in-repository implementation status, full-snapshot polling, fixed provider flights/effort controls, browser middleware as storage authority, UI-authored lifecycle state, hidden control tags, and mocks/builds as engine proof. Full review of separate Connectome commit `38f68ce7adda9d03501f3591165f0e14996899ec` found a real buildable React/Tauri client and four focused native tests, but also SurrealDB package/Wasm authority, handwritten partial RRD contracts, stale scalar deployment modes, duplicated attunement/automation definitions, a retired parallel diagnostics protocol, an unaccepted control protocol, hardcoded cloud topology, fixed reconnect, and two mock-only browser tests. The canonical contract now freezes connection/session, resource/stamp/evidence bindings, live resume/ACK, graph/context boundaries, action/credential rules, bounded rendering, accessibility, direct convergence, and installed rrflowMX/rrflowKV end-to-end acceptance. POAM-011 and H-06 remain open; no client or engine capability was claimed. |
| `docs/operations/ci.md` | `docs/operations/README.md`; `docs/operations/ci.md`; A-06/J-01/J-02/J-03 workflow and evidence gates | Retained the CI record as the sole operations owner after tracing its complete history, candidate caller/reusable workflow, scheduled storage diagnostics, all three ARC values, installer, workflow policy, knowledge exporter/policy, SDK orchestrator, benchmark owner/test, CODEOWNERS, and gate requirements. Added its stable coordinate and parent index; separated candidate checks, scheduled diagnostics, physical runner administration, external GitHub state, and alpha/release evidence. Source review found that the scheduled benchmark escaped the policy and used floating action tags plus persisted checkout credentials, the installer hardcoded an unrelated repository fallback, CODEOWNERS protected only one workflow, the topology job name implied a Connectome process it never starts, and the benchmark owner still described a removed SurrealDB artifact. The package directly pins the workflow, discovers and checks every workflow, disables all checkout credential persistence, broadens workflow ownership, resolves the target repository explicitly, corrects diagnostic provenance, and states what remains unproved. It creates no CI, benchmark, engine, persistence, or lifecycle completion claim beyond the executed assertions. |
| `docs/rrd-server-v1.md` | `docs/reference/protocol/server.md` | Preserved the generated catalogue/router authority, loopback and mutual-TLS boundaries, typed envelopes, exact authorization, durable idempotency and lifecycle state, restart reconciliation, WebSocket subscription semantics, and real-process evidence. Removed the copied endpoint inventory, retired milestone/status claims, stale deployment link, and any implication that callable routes prove target storage or recall behavior. The later subscription review corrected this reference to recognize the existing real-process missing-client-certificate, wrong-server-name, HTTPS, and WSS coverage while leaving untrusted-client-chain, rotation, revocation, external identity, and deployment qualification open. Exposed claim-only transactions, the non-atomic three-transition commit, one-request TLS connections, bounded follow/two-snapshot polling, incomplete trace propagation, and absent multiplexed WebSocket/GraphQL/SDK qualification as direct B/C/H/J convergence work. |
| `docs/rrd-live-subscriptions-v1.md` | `docs/reference/protocol/subscriptions.md` | Preserved the implemented immutable stream definition, one runtime-cursor order, durable ACK and outstanding-delivery state, cumulative ACK, generation fencing, reconnect/restart replay, retention floor, lease renewal, bounded backpressure, dual authorization, changefeed/live-query frames, Rust client, and WSS behavior. Removed the milestone title, compatibility language, and implication that durable transport completes live-query execution. Documented the current JSON control record, heartbeat-driven single-subscription socket, session-bound ownership, logical rather than physical retention floor, and two-snapshot live-query cost as B-04/H-03/J work; corrected the adjacent server reference against its existing mutual-TLS real-process test. |
| `docs/rrd-cluster-m7.md` | `docs/reference/distributed/cluster-contract.md` | Preserved placement epochs, quorum/failure-domain validation, read consistency and real partial-order stamps, route evidence, cross-shard denial, membership/reshard fencing, Raft durability mechanics, authenticated bounded transport, snapshot/artifact integrity and resume, safe admission/GC, telemetry, and replayable fault inputs. Rejected milestone/current-stable/production claims, separate cluster/tenant/table/scope authority, monolithic JSON topology, direct store/object access, raw runtime-commit ingress, private transfer-session truth, old/defaulted formats, synthetic trace lanes, and loopback/process tests as engine qualification. Established an engine-compiled proposal/apply port, full graph/BM25/vector/RRF/reasoning/Arrow/DataFusion distributed corpus, and independent-host qualification. The first alpha remains single-node and `clustered_server` is unavailable until the roadmap owner adds and accepts a distributed gate. POAM-023 owns convergence and the clean-baseline OpenRaft failure/hang. |
| `docs/rrd-kubernetes-v1alpha1.md` | `docs/reference/deployment/kubernetes-operator.md` | Preserved the real namespaced CRD/status-subresource/admission foundation, deterministic rendering, digest-pinned image validation, one retained RWO StatefulSet/PVC, restricted container settings, owner references, PDB/NetworkPolicy mechanics, and denial of controller Secret reads. Rejected Kubernetes spec/status as RRFlow desired/completed truth, direct instance/bootstrap/time/path authority, both parallel initializer paths, hardcoded resource policy, cluster-wide default watch, partial owned-resource observation, unconditional field takeover, TCP/replica readiness, misleading applied digest, eager finalizer release, namespace-wide client trust, placeholder release claims, and four local tests as deployment qualification. Established planned outward `rrflow-kubernetes`, a sealed D-01 cold-start handoff, engine-prepared effects and accepted receipts, standard conditions, authenticated readiness, explicit SSA conflicts, fenced retention/deletion, real API-server evidence, and a clean installed rrflowKV corpus covering graph/BM25/vector/RRF/reasoning/Arrow/DataFusion. POAM-024 owns direct convergence. |

No row authorizes a blind move. The file must first be read in full, compared
to current code and its target owner, and then retained as the owner, merged
without duplicated text into that owner, or removed.

No new history destination is created by this sequence. Accepted current
knowledge moves into its one owner; unresolved work becomes a POA&M row; raw
reproducible results remain evidence; redundant narrative is removed.

#### KB-05 package journal

##### `governed-functions`

```text
gate/package: A-06 / KB-05 / governed-functions
revision: parent c7204f4; result is the commit containing this entry
baseline files/digests: docs/rrd-functions-v1.md=8bc0d2008f549fb6514fd920117c7b58e735c07e595ee359eea24cd811902dda; rrd-contract/function.rs=cca64b3eb22fd039c3ead75f6cd2aef836198cbb8ba0549cf97f5cf68223bf1c; rrd-contract/lib.rs=2a9b9c23126bcc7793057d64c2b7fad8937473b2376aeb72d1394c3f22812a19; rrd-engine/automation.rs=e587ad19d39a0e203f187397b3c9d47c5fbadefa9780fd5c056054ba20754f08; engine automation test=c8d4f3a0247c8689d62adfd115420e23e6ddfa59b14672a652ef283e255a6e8f; rrd-store/control.rs=1b3c405761fe94c844cbe82c9a796ef0f000cd7a400278b744b60c6ef21f1fa2; rrd-engine/transaction.rs=baa8f082080634b507133a0d7c2ed438a619f3b6831da6bfb34a6dd3cfefe12e; rrd-engine/capabilities.rs=a8dd2ac1fb3c0119f11264fb3f03760738b6ad7079b3ece184a90fe86cffe736; rrd-engine/mod.rs=8751c99a0cef91f29f896a097ffa9e86fa3a3841599e0afcf88a1e8697c577a3; rrd-engine/Cargo.toml=c5d8c9722dc434fd2079f9fcde1ee843822dc20c442f52ea160b5d9b1cf6294a; Cargo.lock=bafc36ae835b2b7af47fd560140f721f7282a9d1bee0d18c28c802025538cd7b
files read in full: root README; flat function record; automation reference index and adjacent project-command contract; complete function contract, engine automation implementation and test, store control implementation, engine capabilities, engine module root, engine transaction implementation, and engine manifest. The contract module root, canonical roadmap, POA&M, execution map, engine-data-flow owner, and deterministic inventory generator were read in full in the immediately preceding package and reused only after unchanged hashes or all function-relevant spans were revalidated; the new canonical function record was then reread in full after authoring. The security audit call path, locked QuickJS/Wasmi entries, current Wasmi configuration source, and all named function test paths were traced before classification
external primary references reviewed: SurrealDB custom-function and event documentation for typed invocation permissions and synchronous-versus-asynchronous separation without adopting its authority model; official QuickJS embedding API for memory, stack, and interrupt controls; locked Wasmi configuration/source for fuel, start-function, feature, and structural-limit controls; the WebAssembly deterministic profile for floating-point, relaxed-operation, and resource-dependent behavior; Apache DataFusion UDF and custom TableProvider guidance for typed vectorized Arrow computation and streamed RecordBatch execution
files changed/created/deleted/moved: create docs/reference/automation/functions.md; update root/reference/automation indexes, engine-data-flow, canonical A-07/C-03/I-02/I-06 roadmap requirements, POAM-008, this traceability/direct-convergence/resolved-review/queue/A/C/D/H/I/journal map, deterministic planned-file inventory, and generated file plan; delete docs/rrd-functions-v1.md; no Rust behavior, Cargo manifest, public type, endpoint, SDK, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion implementation changed
contract or behavior changed: target documentation and deterministic file-planning behavior changed; runtime behavior did not. Governed functions are now bounded engine-controlled computation; transaction function bindings are pre-commit validation/proposal rules; triggers are only post-commit event predicates; routines own multi-step progress; activities own nondeterministic external work; rrflowQL native functions remain vectorized Arrow/DataFusion computation. The target directly renames every overloaded catalogue/trigger spelling, separates content-addressed artifacts from typed catalogue state, binds schemas/runtime builds/prepared receipts, requires effect-complete atomic publication and recovery, and installs only manifest-verified offline artifacts through RrdEngine
smallest test command and result: before and after editing, cargo test -p rrd-contract function::tests --locked — 3 passed; cargo test -p rrd-engine --lib engine::tests::automation --locked — 4 passed; cargo test -p rrd-store control::tests --locked — 2 passed
owning package command and result: cargo test -p rrd-contract --all-targets --locked — 58 passed; cargo test -p rrd-engine --all-targets --locked — 108 passed, including 16 workspace-architecture tests; cargo test -p rrd-store --all-targets --locked — 140 passed; cargo clippy -p rrd-contract -p rrd-engine -p rrd-store --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo check --workspace --all-targets --locked — passed; deterministic inventory reported 805 current/generated/planned records; documentation policy reported 88 statuses and 75 classified coordinates; generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; 11 knowledge-export tests, workflow policy, frozen 1.0.0 version, Ruff, Python compilation, Cargo formatting, and diff checks passed
failure/crash/differential evidence: current tests characterize function-value bounds and digests, immutable catalogue revisions, fresh QuickJS/Wasmi execution, selected runtime limits, Wasm import denial, one-attempt bindings, derived-event rejection/commit behavior, revision-pinned replay, and control-journal integrity. They do not prove the accepted schemas and artifacts, physically satisfiable aggregate sizes, portable runtime determinism, prepared receipts, runtime-build-pinned recovery, one atomic allowed-audit/domain/index/event/outbox commit, rrflowMX/rrflowKV function equivalence, crash/reopen/upgrade behavior, offline installation, or outward conformance. Passing tests cannot close A-07, C-03, H-04, I, or J; POAM-008 owns the gap
command corrections: the first deterministic-inventory generation encountered the deliberately deleted flat file because that deletion was not yet represented in the Git index used by the generator; only the exact package paths were staged and the identical generation/check then passed. A later explicit restaging command stopped before generation because both ordinary `git add` and path-limited `git add -u` reject an already absent pathname whose deletion was already staged; existing paths were staged separately and inventory generation then passed. The first post-edit focused-test launcher used a mistyped nonexistent worktree path, so no test process started and no state changed; the exact commands were rerun from the repository root and passed. Full-file review also found and removed one residual description of the present sandbox as deterministic because current evidence does not prove portability
not run and reason: full workspace tests, external SDK conformance, final JavaScript/Wasm supported-target corpus, function-specific MX/KV and crash/ENOSPC/resource matrices, clean offline installation, MCP/Connectome, benchmarks, and release qualification were not run because this package classifies and plans one documentation authority without changing executable behavior; the missing fixtures and implementation gates must exist before those commands can prove the target
remaining known errors: 11 KB-05 records remain; A-06/A-07 are incomplete; POAM-008 remains; old function catalogue/binding names and shapes, private monolithic JSON/control-journal storage, direct store access, split success audits, runtime-unbound replay, incomplete runtime limits, and absent schemas/artifacts/prepared receipts/install/public surfaces remain in code until their mapped A/C/D/H/I/J packages execute
roadmap checkbox changed: no
```

##### `project-command-capabilities`

```text
gate/package: A-06 / KB-05 / project-command-capabilities
revision: parent 347f24d; result is the commit containing this entry
baseline files/digests: docs/package-workflows.md=2205de80c791341bdf41972a688d728abffd34bf9d83049dc665e22e506c579f; rrd-contract/function.rs=cca64b3eb22fd039c3ead75f6cd2aef836198cbb8ba0549cf97f5cf68223bf1c; rrd-engine/automation.rs=e587ad19d39a0e203f187397b3c9d47c5fbadefa9780fd5c056054ba20754f08; engine automation test=c8d4f3a0247c8689d62adfd115420e23e6ddfa59b14672a652ef283e255a6e8f; engine lifecycle test=08c535c3f6316e03575acf1b72dc3a78184349db1516960d29027ba3d169b230; rrd-core/runtime.rs=85cc2f5570e76192a5ac8f2a90086b49a191c787a3d43b48dba981db550501d2; rrd-core/trace.rs=79ed0de628e157f233702610f3f76b3af5a770f62f224d0dbf3da8a10b3837cb; engine runtime/trace.rs=8c689fd10999efc1fce852f898d820e5a7de34dcf84e280bc646763bacd45da0; runtime_trace test=12ce4538ae87c56c1a5fe0ebf912e98abf57e241845dccaa106886059092ef3e; rrd-estate/local_process.rs=1d3e36756d69b933f3828546fd925f7559f63a48a663d2f31f172ea32f75e64c; rrflow-cli/dev/supervisor.rs=911067a5308b9fd29101aa2aa0ee72e59134e53eb9585cb52f0c31572e0156c9; rrflow-eval/main.rs=ff2b8c78833256bf181d03d07a6e7f0f6058ab37115b53786470dd0a7b457d57
files read in full: root README; flat package-command record; reference, agent-bootstrap, system-overview, engine-data-flow, single-engine decision, context-maintenance, local-process, canonical-roadmap, POA&M, and execution-map owners; complete function contract, engine function implementation and tests, lifecycle tests, kernel runtime/event and trace types, engine trace implementation and tests, and evaluation harness; complete local-process and CLI-supervisor reviews from their immediately preceding packages reused only after unchanged hashes and relevant command/effect spans were revalidated; deterministic inventory generator revalidated before its exact planned-file changes
external primary references reviewed: Rust std::process::Command argument/environment/path semantics; OCI process configuration fields and limits without adopting hooks; Temporal activity/idempotency/heartbeat/cancellation semantics without embedding Temporal; SLSA build-provenance parameters/dependencies/builder/output identities without claiming SLSA conformance
files changed/created/deleted/moved: create the automation reference index and docs/reference/automation/project-command-capabilities.md; update root/reference/bootstrap/engine-flow links and authority boundaries, canonical D-06/I-03/I-06 roadmap requirements, POAM-008, this traceability/direct-convergence/resolved-review/queue/D/I/journal map, deterministic planned-file inventory, and generated file plan; delete docs/package-workflows.md; no Rust behavior, Cargo manifest, public type, endpoint, SDK, engine, process, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion implementation changed
contract or behavior changed: target documentation and deterministic file-planning behavior changed; runtime behavior did not. Workflow is reserved for durable routines. The target now separates discovered command facts, inactive candidates, immutable installed bindings, prepared activity plans, adapter observations, accepted engine receipts, and routine steps; distinguishes direct-process from package-script invocation closure; treats exit zero as an observation; makes uncertain effects explicit; requires re-inventory after project mutation; and keeps graph/index/DataFusion selection plus every durable transition under RrdEngine
smallest test command and result: before and after editing, cargo test -p rrd-contract function::tests --locked — 3 passed; cargo test -p rrd-engine --lib engine::tests::automation --locked — 4 passed; cargo test -p rrd-engine --test runtime_trace --locked — 4 passed; cargo test -p rrd-engine --lib engine::tests::lifecycle --locked — 4 passed
owning package command and result: cargo test -p rrd-contract --all-targets --locked — 58 passed; cargo test -p rrd-engine --all-targets --locked — 108 passed, including 16 workspace-architecture tests; cargo clippy -p rrd-core -p rrd-contract -p rrd-engine --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo check --workspace --all-targets --locked — passed; deterministic inventory reported 796 current/generated/planned records; documentation policy reported 88 statuses and 74 classified coordinates; generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; 11 knowledge-export tests, workflow policy, frozen 1.0.0 version, Ruff, Python compilation, Cargo formatting, and diff checks passed
failure/crash/differential evidence: current tests characterize bounded deterministic JavaScript/WebAssembly functions, immutable catalogue/retry pinning, synchronous proposed-transaction effects, current lifecycle state, and durable trace MX/KV parity/reopen. They do not exercise a project-command candidate, installed binding, prepared activity, adapter observation, accepted receipt, effect uncertainty, project re-inventory, capability scaffolding, or external activity across rrflowMX and rrflowKV. Passing them cannot close D-06, I-03, or I-06; POAM-008 owns the gap
command correction: an initial read-only command used a nonexistent working-directory spelling and was rerun against the exact repository root before any conclusion or edit; no data changed
not run and reason: full workspace test suite, external SDK conformance, final process sandbox/platform matrix, capability/activity crash corpus, complete MX/KV semantic differential, clean installation, Connectome, and release qualification do not prove a documentation-only KB-05 classification and remain owned by A-07 and C through J
remaining known errors: 12 KB-05 records remain; A-06/A-07 are incomplete; POAM-008 remains; no capability/activity contracts, command discovery, engine capability/activity operations, local project-activity adapter, install surface, or conformance fixture exists; AutomationCatalogue, FunctionTrigger, RuntimeEvent, direct-store trace helpers, the duplicate CLI supervisor, and evaluation-only direct provider invocation remain scheduled dispositions
roadmap checkbox changed: no
```

##### `clyffy-kernel-alpha`

```text
gate/package: A-06 / KB-05 / clyffy-kernel-alpha
revision: parent a3e2952; result is the commit containing this entry
baseline files/digests: docs/clyffy-kernel-alpha.md=ed778bb447946de1cccd605178ef8d002f9ab81b84049677fc36211904bff4a6; complete reviewed canonical-document set=4cab921c6b72f8ca005a2e89de4de1ce376e3da69233334cb32007ea51ae0398; complete reviewed implementation/test set=493a0234e96cb613d49b89bcb4b392ca0a65c63826bd97e44e3e4c0fb3a6ba79
files read in full: root README; flat Clyffy record; seat-identity, agent-bootstrap, single-engine, system-overview, engine-data-flow, objective, canonical-roadmap, POA&M, package-workflow, and execution-map records; complete router contract and test; complete inference implementation; complete seat/provider contract, engine implementation, engine test, CLI command implementation, and CLI operator-surface test; complete kernel key/claim implementations and golden test; complete engine operator implementation; complete store throughput example plus removal/operator/grounding tests; complete cluster node runtime revalidated against the immediately preceding full distributed-package review; deterministic inventory generator revalidated in full
files changed/created/deleted/moved: update seat-identity, agent-bootstrap, system-overview, engine-data-flow, canonical roadmap, POA&M, this terminology/traceability/direct-convergence/resolved-review/queue/D/G/journal map, deterministic inventory generator, and generated file inventory; clarify that the current Producer actor is a non-authoritative label; correct false Go parity-engine and separate Clyffy management-plane Rust documentation; delete docs/clyffy-kernel-alpha.md; no Rust behavior, Cargo manifest, public contract, fixture, endpoint, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion implementation changed
contract or behavior changed: target documentation and deterministic file-planning behavior changed; runtime behavior did not. Clyffy is now unambiguously this repository's primary durable RRFlow seat specialization, selected explicitly by the planned specialization rather than a generic template. The accepted causal path binds authenticated provider identity, temporal representation, seat attribution, independent authorization, route packet/proposal, engine-selected work, mutation, audit, trace, and reopen at one coordinate. RrdEngine remains the Rust orchestration authority; model backends only propose; Go/provider/host systems remain outward capabilities
smallest test command and result: cargo test -p rrd-contract --test router_contract --locked — 6 passed before editing and 6 passed after editing; cargo test -p rrd-engine --lib engine::tests::memory_estate --locked — 1 passed before editing and 1 passed after editing; cargo test -p rrflow-cli --test operator_surface identity_bind_resolve_and_readme_warp_share_the_persistent_engine --locked — 1 passed before editing and 1 passed after editing; cargo test -p rrd-core --test golden --locked — 1 passed after the documentation correction
owning package command and result: cargo test -p rrd-contract --all-targets --locked — 58 passed; cargo test -p rrd-cluster --test contracts --locked — 10 passed; cargo clippy -p rrd-core -p rrd-contract -p rrd-cluster --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; cargo check --workspace --all-targets --locked — passed; deterministic inventory reported 783 current/generated/planned records; documentation policy reported 87 statuses and 72 classified coordinates; generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow, frozen 1.0.0 version, Ruff, Cargo formatting, Python syntax, and diff checks passed
authoring correction: the first multi-hunk agent-bootstrap patch found a stale context and made no change; it was reapplied against the exact reviewed lines before verification
failure/crash/differential evidence: no product verification command failed. Existing tests prove strict three-proposal router shapes and bounds; unrepresented-seat denial; subject-digest-only provider storage; temporal representation replacement; persistent seat reopen; and warp-to-context behavior. They do not prove an installed specialization, generic-template neutrality, session-to-provider authentication, same-stamp representation/authorization/routing, RouterBackend dispatch, attributed CAS, native seat access, provider conformance, or the final graph/BM25/vector/Arrow/DataFusion reasoning flow; POAM-007 and POAM-009 retain those exact deficiencies
not run and reason: no full workspace test suite, installation/attunement executor, provider/RouterBackend/LFG conformance, identity denial matrix, complete rrflowMX/rrflowKV semantic differential, crash/ENOSPC/resource matrix, graph/BM25/vector/RRF/streamed Arrow/DataFusion context flow, SDK/MCP/Connectome corpus, or release/deployment qualification was run because this package classifies one documentation authority and changes no implementation behavior; those proofs require their dependency-ordered gates
remaining known errors: 13 KB-05 records remain; A-06/A-07 are incomplete; POAM-007 and POAM-009 remain; `MemoryEstate*` naming, hardcoded Clyffy CLI defaults, caller-committed identity plans, broad snapshot seat resolution, arbitrary `Producer.actor` attribution, absent installed specialization, absent same-stamp authenticated seat routing, and absent RouterBackend remain in code until their mapped A/C/D/G/H/J packages execute
roadmap checkbox changed: no
```

##### `rrd-security-bootstrap-v1`

```text
gate/package: A-06 / KB-05 / rrd-security-bootstrap-v1
revision: parent 509306b; result is the commit containing this entry
baseline files/digests: docs/rrd-security-bootstrap-v1.md=1cb25fadfa955433d59cc07a6bcd6b840d3240d8ab3664b38cd1240c426ea1dc; security authority=e63e4ccaddafa29ffeab70db956974b4c70223bf677f5328833c7f07aeeae45a; agent bootstrap=4478d0326273e947403bb497d56446e8adee337996a3d598d7b230ceea7c8aa6; POA&M=91e95999de62284da7bce6fdf9beb2a05aa86d3897e5aecc85f25ee0db202d3f; canonical roadmap=f8d9741ec45eec053112c2a253ae84835478bf2420db0e37db8aa58ceb7c95be; execution map=1ccf103dbf50248c8905dde757d33e6367a8bae40e51c762ca3180fd5ecb76a9; deterministic inventory generator=879fcc1675b66bf9f6ec6fb12969660a6af65d8f6e848b559ecbe65edea5dfd3; reviewed bootstrap/security/supervisor/Kubernetes implementation-and-test set=b25a157ed713e6c614731edee391919f06d6603dfacdaac41b9acb7a32d34d93
files read in full: root README; flat security-bootstrap record; security and reference indexes; canonical security authority and agent-bootstrap owners; objective, canonical roadmap, POA&M, and this execution map; root, rrd-engine, rrflow-cli, and rrd-security manifests; complete engine security-bootstrap, module root, security, session, and invocation implementations; complete CLI bootstrap binary/test and 931-line development supervisor; complete rrd-security implementation and authority test; complete engine security test and engine-authority integration test. The Kubernetes implementation/test, workspace-architecture test, CLI development diagnostics, and deterministic inventory generator were read in the immediately preceding package and revalidated unchanged by digest before their bootstrap-specific caller spans were retraced. Current OWASP secret lifecycle, NIST verifier, cap-std capability-I/O, Linux openat2 resolution, RustCrypto zeroization, and Kubernetes Secret guidance were reviewed as implementation constraints
files changed/created/deleted/moved: update the canonical security authority, agent-bootstrap owner, D-01 roadmap requirement, POAM-016, this current-inventory/traceability/direct-convergence/resolved-review/queue/D-01/journal map, deterministic inventory generator, and generated file inventory; delete docs/rrd-security-bootstrap-v1.md; no Rust, Cargo manifest, public contract, binary, CLI, Kubernetes manifest, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion behavior changed
contract or behavior changed: target documentation and deterministic file-planning behavior changed; runtime behavior did not. Initial security is now one local D-01 initialize_instance action under exact preview/apply, exclusive fresh-target proof, engine-observed time, typed verifiers, capability-scoped/versioned secret sources and sinks, prepared external-effect receipts, and an atomic installed-binding/policy/checkpoint/audit commit. The target forbids a network bootstrap, arbitrary engine paths, adapter-authored grants, a second manifest dialect, and cold-start repair of an already installed estate
smallest test command and result: cargo test -p rrflow-cli --test security_bootstrap --locked — 1 passed before editing and 1 passed after editing
owning package command and result: cargo test -p rrd-security --test security_authority --locked — 6 passed before editing; cargo test -p rrd-security --all-targets --locked — 6 passed after editing; cargo test -p rrd-engine --lib engine::tests::security --locked — 7 passed; strict rrd-security and rrflow-cli all-target Clippy both passed
cross-boundary command and result: cargo test -p rrd-engine --test engine_authority one_authority_coordinates_security_data_catalogues_audit_and_reopen --locked — 1 passed before editing and 1 passed after editing; cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; cargo check --workspace --all-targets --locked — passed; deterministic inventory reported 784 current/generated/planned records; documentation, generated-surface, workflow, frozen-version, Ruff, Python syntax, Cargo formatting, and diff checks passed
failure/crash/differential evidence: no verification command failed. Full code review disproved the flat record's validation-before-open and authenticated-transition claims: bootstrap_security_store opens caller-selected rrflowKV first, and SecurityRepository::initialize performs an unauthenticated direct-store control batch. The current tests positively prove bounded/private Unix file handling, unique/valid policy materialization, verifier-only persistence, atomic policy-plus-audit initialization, exact replay, drift denial, selected redaction/rotation, and rrflowKV reopen. They do not test file replacement between canonicalize/metadata/open, non-Unix privacy, an engine clock, exact installation-plan binding, fresh-target authority, credential-delivery effects, rrflowMX bootstrap parity, interruption boundaries, a closed application listener, or clean installation; POAM-016 and D-01 retain those gaps
not run and reason: no complete D-01 install/crash/secret-provider corpus, rrflowMX/rrflowKV security semantic differential, filesystem/ACL/Kubernetes race matrix, server/client/SDK/MCP/Connectome cross-surface suite, full workspace tests, release installation, or deployment qualification was run because this package classifies and strengthens target requirements without implementing the replacement path
remaining known errors: 14 KB-05 records remain; A-06/A-07 are incomplete; POAM-016 remains; current code still contains SecurityRepository direct storage, missing-policy anonymous access, split policy/data and domain/audit commits, the static database-path bootstrap, standalone binary and old manifest/test shape, CLI private bootstrap/supervisor state, and Kubernetes second-init invocation until the dependency-ordered convergence gates execute
roadmap checkbox changed: no
```

##### `rrd-kubernetes-v1alpha1`

```text
gate/package: A-06 / KB-05 / rrd-kubernetes-v1alpha1
revision: parent cb91bf3; result is the commit containing this entry
baseline files/digests: docs/rrd-kubernetes-v1alpha1.md=bd45606fd8a568742afa9d461778e34747a2ae60d010a0a146406a997f077d12; complete tracked crates/operations/rrd-kubernetes manifest/source/test tree=45f40a54549a0d9f19aa69a3c133a87122c41dd2e5b0265410cd772ddc2bd139; complete checked deploy/kubernetes tree=14dff1b7fcf8245d7d1b8b21912889b33eca5ade0fee1f583aec76f270f69a00; README.md=5abedc5cd536921f9d9f3b473dc809c6f83175076510150beb1e1e7ee96bc3b0; deployment/README.md=4c83551863480998ae66324a2ed6a4a0c15a4f385ec20f976a377c4df110026e; instance-topology.md=ebd3bf39119ba4529744d89b8af787e2953c3b6e77dac43f60447d2125a79396; deployment-modes.md=c27a31a21ddffdf6bd22c415a320448fb14ead6e5159dd02f4043178a9efe809; estate-control.md=80f8c35df7cda9d3f6f6ea9f6995d7a7f14d68ba64cef6805bf8ef59d465dc08; rrflow-cli/dev.rs=4ddba67456a960b0d295ac2c2a95842fba4a677fece09e7fc1d26da545baea67; workspace_architecture.rs=0dc13a64f8db2026dbb67f69e6e11294ec09beca296a68861e9537d4d5103250; ci-reusable.yml=eee1ed22ffdfb50acd0b32db8d040a8954c97dd91bfe13c46b3676d36651ba22
files read in full: root README; flat Kubernetes record; deployment/reference/knowledge indexes; accepted deployment modes, local-process adapter, instance topology, estate control, security authority, agent bootstrap, single-engine ADR, objective, canonical roadmap, POA&M, and this execution map; every manifest, source file, binary, and test in crates/operations/rrd-kubernetes; all three checked deploy/kubernetes JSON documents; complete rrflow-cli development diagnostics, workspace-architecture test, reusable CI workflow, and deterministic execution-inventory generator; current upstream Kubernetes custom-resource, API-convention, server-side-apply, finalizer, StatefulSet, disruption, probe, RBAC, NetworkPolicy, and Lease contracts
files changed/created/deleted/moved: create docs/reference/deployment/kubernetes-operator.md; update root README, deployment index/modes, instance-topology and estate-control cross-links, POA&M, this target-tree/current-inventory/traceability/direct-convergence/resolved-review/queue/journal, deterministic inventory generator, and generated file inventory; delete docs/rrd-kubernetes-v1alpha1.md; no Rust, Cargo manifest, public type, CRD, checked deployment JSON, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion behavior changed
contract or behavior changed: target documentation and deterministic file-planning behavior changed; no RRFlow runtime or public protocol behavior changed. The accepted target now classifies Kubernetes as outward rrflow-kubernetes rather than an operations authority, solves empty-volume bootstrap through the sole D-01 installer boundary, binds reconciliation to sealed engine plans and accepted receipts, requires standard conditions/authenticated readiness/explicit field ownership/fenced deletion, keeps first-alpha deployment to one persistent rrflowKV RRD, and requires the complete graph/BM25/vector/RRF/reasoning/Arrow/DataFusion corpus before Kubernetes support can be claimed
smallest test command and result: cargo test -p rrd-kubernetes --test contract --test crd --locked — 4 passed after editing
owning package command and result: cargo test -p rrd-kubernetes --all-targets --locked — 4 passed before editing and 4 passed after editing; cargo clippy -p rrd-kubernetes --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; cargo check --workspace --all-targets --locked — passed; deterministic inventory reported 784 current/generated/planned records; documentation policy reported 89 statuses and 72 classified coordinates; generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow, frozen 1.0.0 version, Ruff, Cargo formatting, and diff checks passed
command correction: one combined final-audit shell command did not start because a Markdown backtick in its search expression was not shell-safe; the literal-safe status, queue, old-path, and inventory searches were rerun separately and passed
failure/crash/differential evidence: no product verification command failed. The passing corpus proves only deterministic local rendering, four validation cases, generated CRD equality, and string-level checked-manifest assertions. It does not execute controller reconciliation, a Kubernetes API server, admission/defaulting, watch/relist, field conflicts, controller restart, installation, application readiness, effect gaps, finalization, retained-data recovery, or one persistent RRFlow query corpus; POAM-024 records those exact deficiencies rather than treating the passing tests as operator qualification
not run and reason: kind/k3d or another real API-server corpus, a live controller, clean offline cluster installation, cross-platform/multi-architecture image rollout, complete rrflowMX/rrflowKV semantic differential, rrflowKV crash/ENOSPC/pinned-reader matrix, graph/BM25/vector/RRF/reasoning/streamed Arrow/DataFusion endpoint flow, SDK/MCP/Connectome conformance, full workspace tests, and release qualification were not run because this package changes documentation classification only and none of those implementation/evidence gates exists yet
remaining known errors: 15 KB-05 records remain; A-06/A-07 are incomplete; POAM-024 remains; the current operations package, direct CRD authority, hardcoded renderer, two bootstrap paths, phase status, TCP readiness, cluster-wide watch, force apply, eager finalizer deletion, broad namespace-client label, placeholder image, and missing controller/API-server/full-engine tests remain in the checkout until their dependency-ordered gates execute
roadmap checkbox changed: no
```

##### `rrd-cluster-m7`

```text
gate/package: A-06 / KB-05 / rrd-cluster-m7
revision: parent 03ba77c; result is the commit containing this entry
baseline files/digests: docs/rrd-cluster-m7.md=eb31edb914db1422e18af35bc8efe6832225daf6d3701f8035f4e88a1c7332a9; complete tracked crates/operations/rrd-cluster manifest/source/test tree=57de09cf0430f324240884a3dee72f49457729505ff67490e4f45226f409f204; rrd-engine/distributed tree=7d14546ba6c85b3b716dddffaff7eb8dd635763184e4b0f700f3ae9885317e38; rrd-engine/runtime/cluster_transfer tree=e55e3ddf5ae0c0835830ed9cde735eedfba033c5c49a5510e12a2a95afbfc683; distributed_data_plane.rs=04f01435a011d28fdb707876d29636c9482584743f1b123211ff115bb067db77; runtime_cluster_transfer_trace.rs=cb4b7d671592b4be0e6c7f11aa43378a826b0a92447985b1fb381aebdb882059
files read in full: root README; flat cluster record; new distributed contract and index; objective, canonical roadmap, POA&M, and this execution map; system-overview, engine-data-flow, instance-topology, deployment-modes, public-contract, cluster ADR, historical data-services research, and reference indexes; every manifest, source file, binary, and test in crates/operations/rrd-cluster; complete rrd-engine distributed and runtime/cluster-transfer modules plus both cross-boundary integration tests; deterministic inventory generator previously read in full and revalidated unchanged
files changed/created/deleted/moved: create docs/reference/distributed/README.md and docs/reference/distributed/cluster-contract.md; update root README, reference index, deployment modes, instance topology, historical cross-reference, POA&M, this traceability/direct-convergence/resolved-review/queue/journal, and generated file inventory; delete docs/rrd-cluster-m7.md; no Rust, Cargo manifest, public type, wire fixture, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion behavior changed
contract or behavior changed: no executable behavior; one accepted target contract now makes clustering an unavailable post-alpha deployment form, keeps RrdEngine as the sole authorization/transaction/index/audit authority, restricts consensus to engine-compiled proposals and replica apply, preserves real placement/consistency/reshard/recovery/security safety requirements, and requires the complete graph/BM25/vector/RRF/reasoning/Arrow/DataFusion corpus before distributed availability can be scheduled or advertised
smallest test command and result: cargo test -p rrd-cluster --test contracts --locked — 10 passed after documentation editing
owning package command and result: cargo test -p rrd-cluster --all-features --all-targets --locked — 36 tests passed across the unit, artifact-transfer, contract, distributed-authority, and model-check targets before openraft_cluster failed; cargo clippy -p rrd-cluster --all-targets --all-features --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-engine --features cluster-transfer --test distributed_data_plane --test runtime_cluster_transfer_trace --locked — 3 passed after clearing the explicitly authorized shared Cargo build cache that had filled the filesystem; cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; cargo check --workspace --all-targets --locked — passed; deterministic inventory reported 765 current/generated/planned records; documentation policy reported 89 statuses and 71 classified coordinates; generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow, frozen 1.0.0 version, Ruff, Cargo formatting, and diff checks passed
failure/crash/differential evidence: the first cross-boundary build attempt failed with ENOSPC because the shared Cargo target occupied 334 GiB and the filesystem was full; cargo clean removed 358.3 GiB of rebuildable target output and the identical command then passed. The package-wide run failed when real consensus replica apply reported unsupported rrflowKV application format None. The isolated real_consensus_replicates_canonical_runtime_truth_to_every_voter test reproduced that failure at tests/openraft_cluster.rs:424. The isolated real_consensus_elects_fails_over_installs_snapshot_and_changes_membership test did not terminate within 120 seconds and was interrupted. Passing loopback, process, transport, artifact, model, and engine-boundary tests therefore characterize useful mechanics but do not qualify clustered RRFlow
not run and reason: the failing/hanging all-feature package suite was not repeated after a documentation-only change; no complete single-node semantic/graph/BM25/vector/RRF/reasoning/Arrow/DataFusion corpus through consensus, deterministic crash/partition/ENOSPC/resource matrix, every public surface, independent-host deployment, clean installation, Connectome, or release qualification was run because no distributed implementation gate exists yet
remaining known errors: 16 KB-05 records remain; A-06/A-07 are incomplete; POAM-023 remains; the parallel identities/catalogue/direct openers/raw mutation ingress/private transfer truth/old formats/synthetic traces and OpenRaft failure/hang remain in code; clustered_server remains unavailable and must not be emitted until the roadmap owner schedules and accepts a distributed gate
roadmap checkbox changed: no
```

##### `rrd-deployment-modes-v1`

```text
gate/package: A-06 / KB-05 / rrd-deployment-modes-v1
revision: parent bb15a77; result is the commit containing this entry
baseline files/digests: docs/rrd-deployment-modes-v1.md=9f36df0c157408546bf8ec0340462871b6d68b710faa71dc63731c6afe17ced9; README.md=a08566cc5489f5cf7024e7c6aa41bec403ad019261cfe197867466d317b7512e; deployment/README.md=619f2d775df4828a1962af3366b355b84afcb9a6eda56b7cd8d4acc1c9a107ee; instance-topology.md=08fc12331f3e01731479d0f33e3e3e63336d900ea490694b4929ae5a8b390106; public-contract.md=6383994c71cfa263cea9e0062e29e77810f5b26fdfb93fa4c68b69b3e3cc33d5; server.md=7c5aa4415113e1a45d1a9392b92718e3b4f14ac297f3192e832480a607143900; rrd-rust-client-v1.md=b2604cac516a59d1ad631187061e657fa72f612480ee0a1ddaa4cfdb6332e3d5; contract/lib.rs=2a9b9c23126bcc7793057d64c2b7fad8937473b2376aeb72d1394c3f22812a19; public_contract.rs=606a771a31deffc083dd37fe693fbaf8139819fdc35cd47d6d92b2a00a274e08; deployment fixture=b9a80ee3e1e98a0d710df947d10e93962e6959ff8279285ccde22db3e348a825; engine/core.rs=5dd7448a4fcfc64cf7bfcb464906ca481bead762e5cffdaf0a53f9e0330a1762; engine deployment test=5f5fe5ce8ce1e68ab838bd8754c9ff47ff7bb48ec05b0eff1bc53b691fb877a5; server capabilities=582f2a340ee35ee9be7e0c01cc4891b6f082341e5e55e9f42681b68d4c60c7bb; server process test=42b31866baa94d69007fedf608cd6e35ae5ebba1d120932c4f46e0a287262e42; client real-server test=256788be4023550d433b24c285b019a3ff874385bb695603e62b752060e4ed22; edge offline test=98dbd0bde01a6b23a7d0a612a6d79bce66996eadcdb81acd4665a917a0ae2849
files read in full: root README; flat deployment-modes record; deployment index; accepted instance topology; public-contract and server references; the complete still-flat Rust-client record surfaced by link validation; POA&M; this execution map; deterministic inventory generator; deployment fixture; engine composition/deployment test; server capability implementation; edge offline test; prior complete contract-lib/public-contract/client-real-server/server-process reviews reused only after unchanged hashes and every deployment-relevant span and generated SDK projection were revalidated
files changed/created/deleted/moved: create docs/reference/deployment/modes.md; update root README, deployment index, instance topology, public-contract/server references, the Rust-client record's stale link/claim only, POA&M, this traceability/direct-convergence/resolved-review/queue/journal, deterministic inventory generator, and generated file inventory; delete docs/rrd-deployment-modes-v1.md; no Rust, public type, wire fixture, server, client, SDK, engine, storage, query, graph, index, vector, reasoning, or DataFusion behavior changed
contract or behavior changed: none; one accepted reference now separates deployment form, storage profile, endpoint presentation, security, and artifacts; fixes the valid alpha matrix and install/discovery authority; and requires layered storage-semantic, rrflowKV-durability, deployment-form, endpoint, cluster, and artifact proof over one RrdEngine/rrflowQL/Arrow/DataFusion flow
smallest test command and result: exact deployment characterization filters passed before and after editing: rrd-contract 1, rrd-engine 2, rrflow-edge 1, rrd-client loopback 1, rrd-client mutual-TLS 1, and rrd-server standalone process 1
owning package command and result: cargo test -p rrd-contract --all-targets --locked — 58 passed; cargo clippy -p rrd-contract --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; cargo check --workspace --all-targets --locked — passed; deterministic inventory reported 766 current/generated/planned records; documentation policy reported 88 statuses and 69 classified coordinates; generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow, frozen 1.0.0 version, Ruff, Cargo formatting, and diff checks passed
failure/crash/differential evidence: documentation policy first failed on the still-flat Rust-client record's direct link to the removed file; that complete 68-line record was read before only its stale deployment claim/link was corrected, and the policy then passed. Current engine evidence proves the old eager DataFusion executor ran one exact-ID query over two records on rrflowMX and rrflowKV, plus MX durability denial, KV writer exclusion, and KV reopen at cursor three. Current client/server evidence proves selected socket, TLS-identity, child-process, and reopen behavior. The edge row proves only its separate mmap feature-hash artifact. None proves native graph/index/vector access, streamed stamped Arrow/DataFusion, complete MX/KV semantics, reasoning/context/evidence, installed topology, cluster behavior, or resource bounds; POAM-022 owns that convergence
not run and reason: full workspace test suite, external SDK conformance, complete transaction/index/crash/ENOSPC/resource matrices, independent-host cluster tests, clean installation, Connectome, and release qualification do not prove a documentation-only KB-05 classification and remain owned by A-07 and C through J
remaining known errors: 17 KB-05 records remain; A-06/A-07 are incomplete; POAM-022 remains; the scalar DeploymentMode, root/TLS inference, generated SDK field, overclaimed seed fixture/test names, startup initializer, and unqualified edge/distributed values still exist in code
roadmap checkbox changed: no
```

##### `local-process-driver-v1`

```text
gate/package: A-06 / KB-05 / local-process-driver-v1
revision: parent 02b3ebd; result is the commit containing this entry
baseline files/digests: docs/local-process-driver-v1.md=e965da3e63ccfe1079be190328210d05eb8fe5d177d992217bf2b2110b1b22d4; rrd-estate/Cargo.toml=dda167d115215735608328172839533d979e07a086020aae8fef045d79159c6b; rrd-estate/local_process.rs=1d3e36756d69b933f3828546fd925f7559f63a48a663d2f31f172ea32f75e64c; rrd-deployment-catalog.rs=114aca08c11b493786cd869eef725544612f3b56da23375ffdcb1b243cfdcbc5; deployment_catalog.rs=8d6ebc290f04432fb7d5bb2b730bfda38b2c760e365cf443420a37d86c67cef2; rrd-engine/estate_control.rs=3c3eb4684b50144958240ac252c9f9a3bfdc4c58ab819cc1528f1646d4547d61; rrflow-cli/dev/supervisor.rs=911067a5308b9fd29101aa2aa0ee72e59134e53eb9585cb52f0c31572e0156c9; rrd-estate-controller.rs=645f461c4cb99ab0bf6435a5911a4fe146c95b2e0ee3452312a4c37d558cf7e8; rrflow-cli/command.rs=f589ebe63c81bc2215ad8227b4e1b504ab5ea198550bc2489259258c0aac87e2; rrd-server/main.rs=f7cd1604b23c0f13c31517f4208e946f5ca66a91ca5881be17270f74191793b6; local_estate_driver.rs=e34ac61817c210cf513b9e6bda6dd1ea6cd2fee163f48f28ce54b5b495017120
files read in full: root README; flat local-process record; deployment/reference indexes and edge profile; accepted instance-topology, estate-control, local-estate-authorization, security-authority, protocol-server, objective, POA&M, canonical-roadmap, and execution-map owners; rrd-estate manifest, public root, complete local-process implementation, deployment-catalogue command, and catalogue test; complete rrflow-cli development supervisor, estate-controller binary, and relevant command surface; complete rrd-engine estate-control composition with relevant static local-process path revalidated; complete rrd-server main and local-estate-driver real-process test; complete deterministic execution-inventory generator
files changed/created/deleted/moved: create docs/reference/deployment/local-process-driver.md; update root README, deployment index, instance-topology/estate-control/server cross-links and bootstrap guidance, POA&M, this target tree/traceability/direct-convergence/resolved-review/queue/journal, deterministic inventory generator, and generated file inventory; delete docs/local-process-driver-v1.md; no Rust, Cargo manifest, public type, endpoint, fixture, SDK, engine, process, storage, query, graph, index, vector, reasoning, or DataFusion behavior changed
contract or behavior changed: none; one target reference now makes local process control an outward effect adapter under the installed RrdEngine boundary, solves cold-start versus online authority explicitly, binds launch inputs/artifact/process/readiness/shutdown/receipts/resources, distinguishes rrflowMX semantics from rrflowKV crash recovery, and rejects the current parallel catalogue/controller/supervisor/private-file authorities rather than qualifying them
smallest test command and result: cargo test -p rrd-estate --lib local_process --locked — 1 passed before editing and 1 passed after editing; this characterizes hard-link/copy identity only and does not close the authenticated artifact-to-exec race
owning package command and result: cargo test -p rrd-estate --test deployment_catalog --locked — 1 passed before and after; current rrflow-cli supervisor filter — 4 passed before and after; rrd-estate-controller — 2 passed before and after; cargo test -p rrd-estate --all-targets --locked — 22 passed after editing; cargo clippy -p rrd-estate --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-server --test local_estate_driver --locked — 4 passed before editing in 223.39 seconds and 4 passed after editing in 224.03 seconds; cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; cargo check --workspace --all-targets --locked — passed; deterministic inventory reported 765 current/generated/planned records; documentation policy reported 88 statuses and 68 classified coordinates; generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow, frozen 1.0.0 version, Ruff, Cargo formatting, and diff checks passed
command correction: cargo test -p rrflow-cli --lib dev::supervisor --locked failed before execution because rrflow-cli has no library target; the corrected cargo test -p rrflow-cli --bin rrflow dev::supervisor --locked command passed all 4 selected tests before and after editing
failure/crash/differential evidence: the real-process suite preserves same-PID lost-ack replay, start/stop controller kills around prepared/applied/observed/completed boundaries, forged PID/start denial, bounded forced-stop fallback, and data retention. It also positively depends on the private deployment catalogue/process record, initialization manifest, marker files, static store/controller path, and rrflowKV-only setup. It does not prove a sole installed authority, rrflowMX semantics, exact executed-image binding, authenticated readiness/control, bounded diagnostics/resources, public operation, clean installation, or final cross-platform adapter; POAM-021 records those exact deficiencies and closure corpus
not run and reason: full workspace tests, external SDK conformance, every filesystem/process race and fault injection, ENOSPC/resource exhaustion, public HTTP/WebSocket/MCP/Connectome process control, clean-machine package installation, release distribution, and final Linux/Windows/macOS matrices do not prove a documentation-only KB-05 classification and remain owned by A-07, C, D, H, and J
remaining known errors: 18 KB-05 records remain; A-06/A-07 are incomplete; POAM-016 through POAM-021 remain; both current process supervisors, catalogue/controller binaries, direct store opener, process/catalogue/supervisor JSON, startup initializer, marker-file authority, digest-to-exec race, plaintext environment state, and unbounded diagnostics still exist in code
roadmap checkbox changed: no
```

##### `instance-topology`

```text
gate/package: A-06 / KB-05 / instance-topology
revision: parent 088b5e4; result is the commit containing this entry
baseline files/digests: docs/instance-topology.md=c1807592de3ef0171d4e7295cd12795ef863e2b8ce984fc1b68a6d7bd42e570f; rrd-engine/runtime/instance.rs=5080c3a475169a25dd7f9c1a06531801c2f7e2b6e3fd0297e3e7daccb463099d; runtime_instance.rs=116e26edc456421e690e18f36afd25c62c7a3247238403b93ff61a1cc3d49a0f; rrd-contract/platform.rs=7857f478f2a599d8c3650676886ea086cd93a4f823020e314ec6a8adbefa4508; platform_terminology.rs=f5ff4476eb145de911e39a078d739dd2a8ea66e61205ebaf9379661bf6b495c9; history/platform-vocabulary-note.md=f008b1c2a29bba7810e53d2244d13714ce83db3fa21407b1185293c0c1a54145; rrd-estate/authority.rs=50bcd975e6d0772389c9529111d1af9f78a8deff39fd4fa00421bd041694d595; authority_catalogue.rs=60ffc8a3451a1727a65d37ef92643c4805fe70718c11eca053a99a84d49eb2ea; rrd-cluster/contract.rs=55dd79663aaa485bd3b6149ff71b39947854de9cdfd70da3a2a72b2f80ba8944; cluster/contracts.rs=64fd9df17e9bd0ddfd852173d048758225218ad6158799850ba009f010a50d6e; rrd-server/main.rs=f7cd1604b23c0f13c31517f4208e946f5ca66a91ca5881be17270f74191793b6; rrd-server/http/server.rs=424ad733122907fed24ee26983512a40d6ab7a9263b70437e0c7fb2c3378348c; rrflow-cli/dev/supervisor.rs=911067a5308b9fd29101aa2aa0ee72e59134e53eb9585cb52f0c31572e0156c9
files read in full: root README; flat instance-topology record; documentation, architecture, research, estate-control, objective, POA&M, canonical-roadmap, and execution-map owners; system overview and engine-flow owner; historical platform-vocabulary note; rrd-engine instance implementation/export and complete runtime-instance test; rrd-contract platform module and complete terminology test, with previously reviewed unchanged public resource/estate-kind definitions revalidated; rrd-estate authority implementation and complete authority-catalogue test; rrd-cluster manifest, root, complete contract, and complete contract test; rrd-server main and HTTP composition; rrflow-cli development supervisor; initialization caller inventory across engine, server, CLI, MCP, Rust client examples/tests, and server tests
files changed/created/deleted/moved: create docs/architecture/instance-topology.md; update root README, architecture index/system overview, research index, estate-control cross-links, POA&M, this traceability/direct-convergence/resolved-review/queue/journal, documentation policy, and generated file inventory; delete docs/instance-topology.md; no Rust, manifest, public type, wire fixture, server, CLI, MCP, SDK, runtime, storage, query, graph, index, cluster behavior, or DataFusion file changed
contract or behavior changed: none; one accepted architecture owner now locks one project ↔ one estate/rrflowDB ↔ one RRD instance, treats environments as explicit project context, separates deployment form from rrflowMX/rrflowKV profile and logical identity from physical cluster placement, defines install/binding resolution and direct convergence, and rejects the former multi-project-estate hierarchy and compatibility/migration promises
smallest test command and result: cargo test -p rrd-engine --test runtime_instance --locked — 7 passed before editing and 7 passed after editing; these characterize strict current manifest/binding safety and do not accept that format as the target
owning package command and result: cargo test -p rrd-contract --test platform_terminology --locked — 3 passed before and after; cargo test -p rrd-estate --test authority_catalogue --locked — 2 passed before and after; cargo test -p rrd-cluster --test contracts --locked — 10 passed before and after
cross-boundary command and result: cargo test -p rrd-server --bin rrd-server --locked — 5 passed; cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; deterministic inventory wrote 751 records; documentation policy reported 88 statuses and 67 classified coordinates; generated-surface parity reported 33 HTTP operations and OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; CI workflow, frozen 1.0.0 version, Ruff, Cargo formatting, and diff checks passed
failure/crash/differential evidence: the passing platform test positively freezes a historical file's term order, the server test positively retains the independent initialize command, and the runtime test proves only the current rrflowKV JSON/path binding behavior. They do not prove the accepted installation topology, rrflowMX parity, atomic semantic binding, operation-specific resource paths, cross-surface identity, clustered execution, relocation, or clean deployment. POAM-020 records the exact convergence and rejection proof
not run and reason: full workspace, server real-process, CLI/MCP/client/SDK conformance, security denial matrix, crash/ENOSPC, rrflowMX/rrflowKV semantic differential, clustered process/fault, DataFusion, reasoning, Connectome, and release qualification suites do not prove a documentation-only KB-05 classification and remain owned by their named gates
remaining known errors: 19 KB-05 records remain; A-06/A-07 are incomplete; POAM-020 remains; `.rrflow/instance.toml`, `ProjectAuthorityBinding`, startup initializers, historical `PLATFORM_TERMS`, arbitrary `ResourcePath` ordering, duplicate estate authority topology, and unbound cluster identities still exist in code
roadmap checkbox changed: no
```

##### `estate-control-v1`

```text
gate/package: A-06 / KB-05 / estate-control-v1
revision: parent 5b63111; result is the commit containing this entry
baseline files/digests: docs/estate-control-v1.md=ee6ce754ed532c1f9fcc5632b5e22dc8c6a06cc8eb52843314252765badc06d5; rrd-estate/Cargo.toml=dda167d115215735608328172839533d979e07a086020aae8fef045d79159c6b; rrd-estate/lib.rs=37d585386aff6152f6021d6e978a2288aa733d476e69719f1de85225736cfeab; authority.rs=50bcd975e6d0772389c9529111d1af9f78a8deff39fd4fa00421bd041694d595; reconcile.rs=aab8f34a072d6e3dda185a87d18582cf95865d8b757a0f0b83a0f58d199c36ac; backup_job.rs=35659494cfeb35328c0bc419e400fdbea06528f0f5b048e21c1197caa56a9816; backup_reconcile.rs=650ecb11f0c8fbea11d2589d28588ecab4e669132ee31de6d0db04cd57bf3a62; recovery.rs=6d22f4dedbf9a15965357aab533968007a1b1392988d12ba12157b224604a822; engine/estate_control.rs=3c3eb4684b50144958240ac252c9f9a3bfdc4c58ab819cc1528f1646d4547d61; store/control.rs=1b3c405761fe94c844cbe82c9a796ef0f000cd7a400278b744b60c6ef21f1fa2; server/local_estate_driver.rs=e34ac61817c210cf513b9e6bda6dd1ea6cd2fee163f48f28ce54b5b495017120
files read in full: root README; flat estate-control record; documentation/reference/research indexes; system overview, single-engine ADR, alpha objective, canonical roadmap, POA&M, and relevant complete engine-flow/execution-map owners; rrd-estate manifest, public aggregate/repository, operational-authority catalogue, desired/observed reconciler, backup job/reconciler, recovery, and all focused estate tests; rrd-store control implementation/test; rrd-engine core, estate read, estate control, and complete engine-authority test; rrflow-cli estate/backup controller implementations and complete backup-controller test; server estate handler/capabilities/router and complete local-estate-driver test; contract estate snapshot/endpoint and public-contract test; Rust SDK conformance test and its real-server fixture
files changed/created/deleted/moved: create docs/reference/operations/README.md and docs/reference/operations/estate-control.md; update root README, reference/research indexes, local-estate-authorization cross-link, POA&M, this implementation traceability/direct-convergence/resolved-review/queue/journal, and generated file inventory; delete docs/estate-control-v1.md; no Rust, manifest, public type, endpoint, fixture, SDK, runtime, storage, query, graph, index, or DataFusion file changed
contract or behavior changed: none; the canonical record separates estate deployment control from reasoning/routine lifecycle, preserves every useful current safety semantic, defines typed record/relation/index families and one installed authorized engine flow, states exact rrflowMX versus rrflowKV behavior, and makes rrflowQL/Arrow/DataFusion a same-stamp analytical reader rather than a writer or control authority
smallest test command and result: cargo test -p rrd-store --test control_journal --locked — 2 passed before editing and 2 passed after editing
owning package command and result: cargo test -p rrd-estate --locked — 22 passed before editing; cargo test -p rrd-estate --all-targets --locked — 22 passed after editing; cargo clippy -p rrd-estate --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-engine --test engine_authority --locked — 3 passed before editing and 3 passed after editing; exact public-contract estate read — 1 passed before and after; exact authenticated HTTP estate read — 1 passed before and after; backup controller effect-gap test — 1 passed before and after; cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; deterministic inventory reported 752 records; documentation policy reported 88 statuses and 66 classified coordinates; generated-surface parity reported 33 HTTP operations and OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow, frozen 1.0.0 version, Ruff, Cargo formatting, and diff checks passed
failure/crash/differential evidence: cargo test -p rrd-server --test local_estate_driver --locked passed 4 real-process tests before editing in 225.98 seconds and after editing in 225.27 seconds, including start/stop controller kills on both sides of effects; backup effect-gap recovery passed; selected MX/KV CAS parity and KV reopen/lost-ack/takeover tests passed. The suite also positively requires two older successful shapes, and current estate control has no complete MX/KV semantic differential, write-amplification, native-index, same-stamp DataFusion, atomic audit/outbox, cross-surface mutation, or clean-install evidence
not run and reason: full engine/server/client/CLI/workspace test suites, external SDK conformance manifest, every process/filesystem/object failure, ENOSPC, distributed topology, native graph/index/DataFusion resource corpus, Connectome repository checks, and release qualification do not prove a documentation-only KB-05 classification and remain owned by their named gates
remaining known errors: 20 KB-05 records remain; A-06/A-07 are incomplete; POAM-016 through POAM-019 remain; estate persistence is one copied JSON aggregate behind a public direct-store repository, local controllers bypass canonical invocation, older shapes succeed, the operational authority hierarchy overlaps other owners, and no accepted native graph/index/DataFusion estate flow exists yet
roadmap checkbox changed: no
```

##### `local-estate-authorization-v1`

```text
gate/package: A-06 / KB-05 / local-estate-authorization-v1
revision: parent a8dad0c; result is the commit containing this entry
baseline files/digests: docs/local-estate-authorization-v1.md=cc0e13b0701d790eacff05b3e5de57cceb02238a9133905b0bb712b85e1c57ff; rrd-estate/local_authorization.rs=02054dacb1b4a6aa06e68509859f0a2a75df4453f797df9ffe62233915700f42; rrd-estate/lib.rs=37d585386aff6152f6021d6e978a2288aa733d476e69719f1de85225736cfeab; rrd-estate/backup_job.rs=35659494cfeb35328c0bc419e400fdbea06528f0f5b048e21c1197caa56a9816; rrd-estate/recovery.rs=6d22f4dedbf9a15965357aab533968007a1b1392988d12ba12157b224604a822; engine/estate_control.rs=3c3eb4684b50144958240ac252c9f9a3bfdc4c58ab819cc1528f1646d4547d61; rrd-estate-admin.rs=10971201428c8f2b9d36cc4f7f2bb18a14661c00d14a31dacb62c2da499a88ba; rrd-recovery-controller.rs=f19ddee8f5c9c67bb156038ac66647208e0e2c463d463252ff1f9330ccd6b2e1; estate_admin.rs=876fecbddf01a8e4c2fa0eaf44714d7d9caeb1307ecefc5c97cd9ec59cb5a93a; engine_authority.rs=01b5f3c109db3079bccc89cce37ac6a21486f6fd3444bce3826f0d5476eb9e65
files read in full: root README; flat local-estate authorization record; current security index and authority; research index; rrd-estate manifest, public root/aggregate repository, local authorization, backup-job, and recovery implementations plus complete local-authorization, backup-job, and recovery tests; rrd-engine manifest/public root/composition root/core/estate-read/estate-control implementations and complete engine-authority test; rrflow-cli manifest, complete estate-admin and recovery-controller adapters, and complete estate-admin process test; prior complete system-overview, ADR, objective, roadmap, POA&M, canonical invocation, public contract/test, and workspace-architecture reviews reused only after unchanged hashes and relevant spans were revalidated
files changed/created/deleted/moved: create docs/reference/security/local-estate-authorization.md; update the security and research indexes, security authority, POA&M, this resolved-review/queue/journal, and generated file inventory; delete docs/local-estate-authorization-v1.md; no Rust, public contract, fixture, engine, estate, CLI, storage, SDK, or runtime file changed
contract or behavior changed: none; the canonical reference makes locality an adapter property, retains the seven real effect distinctions, and requires their direct absorption into one installed, stamped, canonical security/invocation/transaction path
smallest test command and result: cargo test -p rrd-estate --test local_authorization --locked — 1 passed before editing and 1 passed after editing
owning package command and result: cargo test -p rrd-estate --all-targets --locked — 22 passed before editing and 22 passed after editing; cargo clippy -p rrd-estate --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrflow-cli --bin rrd-estate-admin --locked — 1 passed before editing and 1 passed after editing; cargo test -p rrflow-cli --test estate_admin --locked — 1 passed before editing and 1 passed after editing; cargo test -p rrd-engine --test engine_authority --locked — 3 passed before editing and 3 passed after editing; cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; deterministic inventory reported 753 records; documentation policy reported 87 statuses and 64 classified coordinates; generated-surface parity reported 33 HTTP operations and OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow policy, frozen 1.0.0 version policy, Ruff, Cargo formatting, and diff checks passed
failure/crash/differential evidence: the passing engine test proves canonical security can omit `EstateAdmin` while the separate local policy still authorizes estate creation; POAM-017 records that dual authority. The owning suite also requires successful older estate documents and backup jobs with missing fields; POAM-018 records that pre-release compatibility path. Existing tests cover selected reopen/replay and fenced restore behavior, but this package creates no new crash, cross-surface, MX/KV authorization differential, or resource evidence
not run and reason: full rrflow-cli/engine/workspace suites, server/client/SDK conformance, every permission denial, credential/file race and platform ACL matrices, install resolution, DataFusion, reasoning, and release qualification do not prove a documentation-only KB-05 classification and remain owned by their named gates
remaining known errors: 21 KB-05 records remain; A-06/A-07 are incomplete; POAM-016 through POAM-018 remain; local estate mutation is not one canonical authorized operation and current estate decoding still accepts older successful shapes
roadmap checkbox changed: no
```

##### `rrd-security-v1`

```text
gate/package: A-06 / KB-05 / rrd-security-v1
revision: parent ffc6f98; result is the commit containing this entry
baseline files/digests: docs/rrd-security-v1.md=0b1bde890a3d8883e7184b45f3f2178493fee91192f57b9d30e5c25d18ff39be; rrd-security/src/lib.rs=965ebebdd0a43f65e280aa86a7ae30613bd5eb832debbb2a398a76b6b1e7d499; security_authority.rs=cdef0351005a93bff951d42bd994cb791371f64dae7c3f22b1d0c2f374cc3fad; engine/security.rs=44596adfd3116b5bef241e36766ff9cc7da4ef28b1f7d7e372300466eb32dc25; engine/invocation.rs=887c2b6aaf29b2e02f9c437f962a8ee9570de4d2cac0393ddc43fe6a516394f1; engine/session.rs=dc57d917ada01315e372dc79f109ca43d385de73a4123d85b67841339c518d9a; server/http/auth.rs=1c5789505264a3e4299cbd60c3fcc0a1bbc75dc5b9d19243a1a6a0c031ac0268; server/http/server.rs=424ad733122907fed24ee26983512a40d6ab7a9263b70437e0c7fb2c3378348c; server/http_process.rs=42b31866baa94d69007fedf608cd6e35ae5ebba1d120932c4f46e0a287262e42; client/real_server.rs=256788be4023550d433b24c285b019a3ff874385bb695603e62b752060e4ed22
files read in full: root README; flat security record; reference, architecture, decision, objective, POA&M, canonical-roadmap, and execution-map owners; rrd-security manifest, implementation, and complete authority test; rrd-engine manifest, public root, composition modules, security, bootstrap, session, invocation, diagnostic, and complete security test; rrd-contract action/audit definitions; rrd-store control implementation and focused catalogue/control tests; rrd-server authentication, envelope, audit/session handlers, HTTP composition, public root, and complete real-process HTTP test; complete rrd-client real-server test; CLI security-bootstrap implementation/test; MCP daemon test; SDK conformance server example
files changed/created/deleted/moved: create docs/reference/security/README.md and docs/reference/security/authority.md; update the reference index, POA&M, this resolved-review/queue/journal, and generated file inventory; delete docs/rrd-security-v1.md; no Rust, wire fixture, server, client, engine, storage, SDK, or runtime file changed
contract or behavior changed: none; one canonical reference now identifies the semantics to preserve, assigns security enforcement to RrdEngine, and maps every observed implementation deviation to executable closure gates
smallest test command and result: cargo test -p rrd-engine --lib engine::tests::security --locked — 7 passed before editing and 7 passed after editing
owning package command and result: cargo test -p rrd-security --all-targets --locked — 6 passed before editing and 6 passed after editing; cargo clippy -p rrd-security --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-server --test http_process initialized_security_authority_binds_sessions_and_denies_ungranted_routes --locked — 1 passed before editing and 1 passed after editing; cargo test -p rrd-client --test real_server remote_transport_requires_mutual_tls_and_exact_server_identity --locked — 1 passed before editing and 1 passed after editing; cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; deterministic inventory reported 754 records; documentation policy reported 87 statuses and 63 classified coordinates; generated-surface parity reported 33 HTTP operations and OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow policy, frozen 1.0.0 version policy, Ruff, Cargo formatting, and diff checks passed
failure/crash/differential evidence: passing characterization does not reject `SecurityRepository` direct storage access, absence-driven anonymous loopback application access, separate policy/data observations, operation-wide mutation authorization, or split domain/audit commits; POAM-016 records the required direct convergence; no new crash, semantic differential, or resource evidence was created
not run and reason: full server/client/workspace suites, external SDK conformance, complete credential/certificate rotation matrices, failure injection, rrflowMX/rrflowKV security differentials, DataFusion authorization, clean installation, and release qualification do not prove a documentation-only KB-05 classification and remain owned by their named gates
remaining known errors: 22 KB-05 records remain; A-06/A-07 are incomplete; POAM-016 remains; security does not yet share one atomic policy/data/index/audit transaction or qualify install, provider identity, every adapter, and production transport behavior
roadmap checkbox changed: no
```

##### `rrd-public-contract`

```text
gate/package: A-06 / KB-05 / rrd-public-contract
revision: parent 9c3d50b; result is the commit containing this entry
baseline files/digests: docs/rrd-public-contract.md=9b344bb4926c608fb21f60b6e65fa1f34f0a2b8a7ccbc78654c9d20d6373a1d0; rrd-contract/src/lib.rs=2a9b9c23126bcc7793057d64c2b7fad8937473b2376aeb72d1394c3f22812a19; public_contract.rs=606a771a31deffc083dd37fe693fbaf8139819fdc35cd47d6d92b2a00a274e08; public-contract-v1.json=08095dcd45f54845c3043e3797c60e1d88c7efcaf1fce70575554f3400eccbfa
files read in full: root README; flat public-contract record; knowledge/reference/protocol indexes; protocol server and subscription owners; objective, POA&M, canonical roadmap, and execution map; rrd-contract manifest, lib.rs, capability-surface and SDK-conformance modules, public golden, deployment/SDK corpora, and public-contract tests; engine and server capability builders; Rust client manifest, implementation, and SDK-conformance test
files changed/created/deleted/moved: create docs/reference/protocol/public-contract.md; update protocol index, POA&M, this resolved-review/queue/journal, and generated file inventory; delete docs/rrd-public-contract.md; no Rust, wire fixture, generated SDK, server, client, engine, or runtime file changed
contract or behavior changed: none; the canonical reference now distinguishes exported representation, catalogued exposure, runtime discovery, and actual single-engine acceptance, and maps each contract family to its required rrflowMX/rrflowKV/native-index/rrflowQL/Arrow/DataFusion/reasoning proof
smallest test command and result: cargo test -p rrd-contract --test public_contract --locked — 31 passed before editing and 31 passed after editing
owning package command and result: cargo test -p rrd-contract --all-targets --locked — 58 passed; cargo clippy -p rrd-contract --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-engine --test workspace_architecture public_contract_and_client_stay_implementation_free --locked — 1 passed; deterministic inventory check reported 755 records; documentation policy, generated-surface parity at 33 HTTP operations/OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715, workflow policy, frozen 1.0.0 version policy, Cargo formatting, and diff checks passed
failure/crash/differential evidence: the passing baseline did not reject the frozen sample's nonexistent POST /v1/backups/create binding; POAM-015 records that defect plus retired runtime capability labels and alternate successful pre-release branches; no new crash, reopen, engine differential, or resource evidence was created
not run and reason: engine/server/client real-process suites, external SDK conformance, full workspace tests, crash matrices, DataFusion streaming, native-index, reasoning, and deployment qualification do not prove a documentation-only KB-05 classification and remain owned by their named gates
remaining known errors: 23 KB-05 records remain; A-06/A-07 are incomplete; public capability drift in POAM-015 remains; the contract types do not establish persistent rrflowKV, equivalent rrflowMX semantics, streamed Arrow/DataFusion, native graph/BM25/vector execution, persisted reasoning, installation, or cross-surface qualification
roadmap checkbox changed: no
```

##### `rrd-live-subscriptions-v1`

```text
gate/package: A-06 / KB-05 / rrd-live-subscriptions-v1
revision: parent 369f8fe; result is the commit containing this entry
baseline files/digests: docs/rrd-live-subscriptions-v1.md=f54ea77b13fa4c615f2486d268b313783020ee35b3f9eb64d2bcc7f2b97260d4; unchanged implementation anchors verified by SHA-256 before editing
files read in full: flat subscription record; protocol index and server owner; query live reference; engine subscription implementation/tests; query live implementation/tests; server subscription handler/WebSocket adapter; prior complete contract/client reviews reused only after unchanged hashes and relevant contract, client, loopback, and mutual-TLS spans were revalidated
files changed/created/deleted/moved: create docs/reference/protocol/subscriptions.md; update protocol index, query cross-links, server evidence row, execution map, and generated file inventory; delete docs/rrd-live-subscriptions-v1.md
contract or behavior changed: none; documentation now separates implemented durable delivery from B-04 multiplexing and H-03 commit-impact work
smallest test command and result: cargo test -p rrd-contract durable_subscription_contract_bounds_retention_backpressure_and_stream_shape --locked — 1 passed
owning package command and result: cargo test -p rrd-engine subscription --locked — 5 passed; cargo test -p rrd-query --test live_query --locked — 2 passed
cross-boundary command and result: cargo test -p rrd-client --test real_server --locked — 3 passed; documentation, generated-surface, inventory, workflow, version, and formatting checks run after editing
failure/crash/differential evidence: existing engine corpus passed durable close/reopen and replay; existing query corpus passed rrflowMX/rrflowKV semantic-delta comparison; the first inventory regeneration failed because its tracked-path input reads the Git index and the intended deletion was not staged, so only this package was staged before a clean rerun; no new runtime evidence created
not run and reason: full workspace tests, SDK conformance, crash matrix, and release qualification are not substitutes for a documentation-only KB-05 classification
remaining known errors: dedicated heartbeat-polled subscription socket is not B-04 multiplexing; live query still materializes two snapshots instead of H-03 commit-impact evaluation; complete trace and generated-SDK qualification remain open
roadmap checkbox changed: no
```

##### `kb-05-execution-queue`

```text
gate/package: A-06 / KB-05 / execution-queue
revision: parent 4bc9dec; result is the commit containing this entry
baseline files/digests: generated inventory reported 756 current, generated, and planned paths; 24 unresolved KB-05 records were derived by comparing the planned and resolved-review tables against the tracked worktree
files read in full: README.md; AGENTS.md; scripts/ci/build_execution_inventory.py; relevant canonical roadmap, KB-05 execution-map, resolved-review, journal-template, and actual flat-path inventory sections
files changed/created/deleted/moved: update AGENTS.md, this execution map, scripts/ci/build_execution_inventory.py, and the generated file inventory; no product record or runtime source moved
contract or behavior changed: none; the supporting map now has one finite per-record order and repository instructions require a structured journal for every bounded package
smallest test command and result: python3 scripts/ci/build_execution_inventory.py --check — 756 records, passed
owning package command and result: ruff check scripts/ci/build_execution_inventory.py — passed
cross-boundary command and result: documentation policy, generated-surface parity, CI workflow policy, version policy, Cargo formatting, and diff checks passed
failure/crash/differential evidence: review found a dependency cycle that deferred the security-bootstrap KB-05 record until D-01 even though D-01 depends on A-06; the flat record will now merge stable requirements during KB-05 while creation of an executable guide is assigned to D-01
not run and reason: Rust package/workspace tests, SDK conformance, crash matrices, and release qualification were not run because no Rust, wire contract, runtime behavior, SDK, or release artifact changed
remaining known errors: 24 KB-05 records remain; A-06 and A-07 are incomplete; B-03 and all later implementation work remain gated
roadmap checkbox changed: no
```

##### `rrd-rust-client`

```text
gate/package: A-06 / KB-05 / rrd-rust-client
revision: parent f29f1f8; result is the commit containing this entry
baseline files/digests: docs/rrd-rust-client-v1.md=138c7ed6d5de5695d61e3f243d9f6cd060e139c3cc5aace1ca2cb4e78e3fe89f; rrd-client/Cargo.toml=3ee74758aee12e2dedf21cb5519b7d037467b2f9865d754d1a1a3b6548ecd125; client/lib.rs=8261bb3bd65554a9b1437a85709b90acc943ba2c4e6ba8d987cd5b7d43fd43ff; real_server.rs=256788be4023550d433b24c285b019a3ff874385bb695603e62b752060e4ed22; sdk_conformance.rs=8dccb4ccb301d4a0cef4e2554c96a6e1112529318e355603ab33f1ecac50a73b; sdk_conformance_server.rs=8fab285eb4a3a7081031cbcf858e1d604571346b9bb2ec1701c6807a888cf158; run_sdk_conformance.py=5870f73b9bad92c14befd94b9bbab4f396f8ddd664c56cec0ce7e3a64c0b1a39; check_generated_surfaces.py=fcd235248e027a4587444438480f1061f1314ba15e95a82c1068afe84fbad783; SDK corpus=b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; contract/sdk_conformance.rs=69e24d5a6299b257a819dce1eba1df6df656401fc6ce51719a7982ab16fb663b; contract/lib.rs=2a9b9c23126bcc7793057d64c2b7fad8937473b2376aeb72d1394c3f22812a19; public_contract.rs=606a771a31deffc083dd37fe693fbaf8139819fdc35cd47d6d92b2a00a274e08
files read in full: AGENTS.md; root README; flat Rust-client record; reference and protocol indexes; public-contract, server, subscription, and deployment-mode references; canonical roadmap, POA&M, and relevant complete execution-map owners; rrd-client manifest, 1,235-line implementation, real-server test, SDK-conformance test, and live harness example; shared SDK corpus; SDK corpus contract module; conformance runner, generated-surface checker, and deterministic execution-inventory generator; public-contract test; complete endpoint catalogue plus request/response/session/subscription contract spans; the complete contract root was reused only after its unchanged digest was verified and every client-relevant span was revalidated; locked Tokio/Rustls/Tungstenite versions and Tungstenite WebSocket defaults were inspected
files changed/created/deleted/moved: create docs/reference/sdk/README.md and docs/reference/sdk/rust.md; update root/reference/public-contract links, canonical H-04/H-07 roadmap requirements, POAM-011, this implementation inventory/traceability/direct-convergence/resolved-review/queue/A-07/B-04/H/journal map, deterministic planned-file inventory, and generated file plan; delete docs/rrd-rust-client-v1.md; no Rust, Cargo manifest, wire fixture, generated SDK, endpoint, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion behavior changed
contract or behavior changed: none; the canonical reference now fixes the SDK as an outward client of the one RrdEngine, corrects current exposure to 28 of 33 HTTP operations plus the dedicated subscription socket, and freezes exact operation-binding, validation, credential, retry/certainty, cancellation, frame, endpoint, trace, conformance, MX/KV, and cross-surface requirements without claiming the engine or SDK is alpha-qualified
smallest test command and result: cargo test -p rrd-client --test real_server --locked — 3 passed before and after editing; the suite exercised the real loopback server, current narrow deployment corpus, and HTTPS/WSS mutual-TLS fixture
owning package command and result: cargo test -p rrd-client --all-targets --locked — 4 reported tests passed, but one was the manifest-absent SDK test that returned before any scenario and is explicitly not counted as conformance; cargo clippy -p rrd-client --all-targets --locked -- -D warnings — passed
cross-boundary command and result: the example harness plus absolute shared-corpus path executed the Rust SDK test — 1 passed and reported corpus SHA-256 b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; public_contract_and_client_stay_implementation_free — 1 passed; deterministic inventory — 815 records; documentation policy — 89 statuses/77 coordinates with indexes and links passed; generated-surface parity — 33 HTTP operations/OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow policy, frozen 1.0.0 version policy, Ruff, Cargo formatting, and diff checks passed
failure/crash/differential evidence: with RRD_SDK_CONFORMANCE_MANIFEST removed, cargo reported the SDK test passed in 0.00 seconds even though its first branch returned without executing a scenario; this is retained as a POAM-011 failure, not evidence. The configured live run proved only the present direct-seeded rrflowKV corpus and its capability connection-drop row. The first post-edit live-harness shell attempt grouped an && chain under background execution, leaving the parent without cleanup variables; it was interrupted, process/temp searches proved no orphan remained, and the corrected structured invocation passed. Existing real-server negative rows reject a missing client certificate and wrong server name; no new crash/reopen, rrflowMX/rrflowKV differential, server-cancellation, after-commit/lost-response, WebSocket-resource, installation, or complete engine evidence was created
not run and reason: TypeScript/Python/Go/Java/.NET live conformance, full workspace tests, every catalogue operation, complete request/response/error/fault matrix, D-01 installation, storage-profile differential, persistent graph/BM25/vector/RRF/reasoning/Arrow/DataFusion flow, Connectome, clean distribution, and release qualification do not prove this documentation-only KB-05 classification and remain owned by A-07/B-04/D/H/J
remaining known errors: 10 KB-05 records remain; A-06/A-07 are incomplete; POAM-011 remains; the five missing Rust operations, monolith, partial validation, secret-bearing Debug, boolean immediate retry, local-only cancellation, library-default WebSocket bounds, incomplete frame identity, absent resolver/W3C propagation, direct-seeded harness, label-only scenario accounting, and manifest-absent false pass remain in code until their mapped gates execute
roadmap checkbox changed: no
```

##### `rrd-typescript-client`

```text
gate/package: A-06 / KB-05 / rrd-typescript-client
revision: parent 1bdd664; result is the commit containing this entry
baseline files/digests: docs/rrd-typescript-client-v1.md=b65850ae2b093d922cd7a41b6d47e2c62c51b0fa4ddc7c3394f1d15aa4374879; package.json=999ecc6e2ce632a6e926a252d2449628a3193305c26a972cfb4cca0da2bc8c10; biome.json=2c180ada5847bb9c6a85831da06d7c2c2072ee5c1e91b484c02aced6d1c00eb8; pnpm-workspace.yaml=ac02d96368617c760f093cfe61fdec64b6244007ab3553e0d6621f706f54a353; tsconfig.json=830949da640bacc2f2371dc7eb46e8bdad1d3546ebbd2a2364a8b1e6096ad40c; pnpm-lock.yaml=041e0de682eaf9b9852e1623dc6b9328b4c8645bc85a6a8f270d8258975a92ce; generate.ts=6f3ac002eea0594568bede31720ee160f2cc8f94272455aeefcfcefdff54c2f7; src/index.ts=6ecc0ede843c3d1347d815d45d5acd201392ef0bb4eecc964ff890633fc8e3fd; generated/endpoints.ts=e98dcfcf521e552e6c1480f5de1d35cc6e47bde2af36e354a0575a2c76980741; generated/rrd-openapi.ts=cb93949580d7f4d4295b1844e72ec31b3a16ce194dd297f5b5295d30042af81c; client.test.ts=b4a37a801d8f1ee8534b2499bb2b10366de4826eccef5968b5fccde849380927; sdk_conformance.ts=b5c5a54401a9041814c6f70c26bf7787372ca297f4a6bd486a30bbb4d21d72c0; runner=5870f73b9bad92c14befd94b9bbab4f396f8ddd664c56cec0ce7e3a64c0b1a39; checker=fcd235248e027a4587444438480f1061f1314ba15e95a82c1068afe84fbad783; SDK corpus=b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; live harness=8fab285eb4a3a7081031cbcf858e1d604571346b9bb2ec1701c6807a888cf158; corpus contract=69e24d5a6299b257a819dce1eba1df6df656401fc6ce51719a7982ab16fb663b
files read in full: AGENTS.md; root README; flat TypeScript-client record; SDK index and canonical Rust SDK reference; relevant public-contract, canonical-roadmap, POA&M, and complete execution-map owner sections; TypeScript package/Biome/workspace/TypeScript configuration, complete pnpm lock, generator, 394-line runtime, generated endpoint map, mock test, and conformance entry; conformance orchestrator and generated-surface checker. The 24,523-line generated schema was not manually treated as authored prose: its complete bytes were regenerated/compared, its export topology and all 33 operation/status/media bindings were inspected, and its digest was recorded. The shared corpus, live harness, and corpus contract were reused only after their unchanged digests matched the immediately preceding complete Rust-SDK review; current WHATWG Fetch/DOM, Node package-entry, TypeScript declaration-publishing, npm private-package, and RFC 9110 constraints were inspected from their primary publishers
files changed/created/deleted/moved: create docs/reference/sdk/typescript.md; update the SDK index, public-contract surface table, POAM-011, this implementation inventory/traceability/direct-convergence/resolved-review/queue/A-07/journal map, deterministic planned-file inventory, and generated file plan; delete docs/rrd-typescript-client-v1.md; no TypeScript/Rust runtime, package manifest, lock, generated schema, wire fixture, endpoint, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion behavior changed
contract or behavior changed: none; the canonical record fixes TypeScript as a built outward Node/browser client of the one RrdEngine, corrects the stale 34-operation claim to a generic 33-HTTP-operation type surface with no WebSocket, and freezes exact runtime-validation, credential, retry/certainty, cancellation, transport, packaging, browser, MX/KV, and cross-surface requirements without claiming SDK or engine qualification
smallest test command and result: pnpm --dir sdks/typescript test — 4 mock-focused tests passed after editing; the same 4 passed inside the baseline and post-edit package checks
owning package command and result: pnpm --dir sdks/typescript check — generation check, Biome over 7 files, no-emit strict typecheck, and 4 tests passed before and after editing under Node 24.16.0/pnpm 11.22.0
cross-boundary command and result: the real example harness plus absolute shared-corpus path executed the TypeScript conformance entry — passed and reported corpus SHA-256 b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; missing RRD_SDK_CONFORMANCE_MANIFEST failed closed with exit 1; npm pack --dry-run reported a 70,658-byte/1,529,059-byte-unpacked raw-source archive with no built JavaScript/declarations; workspace architecture — 16 passed; deterministic inventory — 829 records; documentation policy — 89 statuses/78 coordinates; generated-surface parity — 33 HTTP operations/OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow policy, frozen 1.0.0 version policy, Ruff, Cargo formatting, and diff checks passed
failure/crash/differential evidence: a corrected injected-fetch probe replayed an unpinned query twice, accepted 201 text/plain with an invalid success payload, and JSON-serialized the synthetic bearer. Its first invocation failed only in tsx evaluation because top-level await was emitted as CommonJS and never invoked the client; wrapping it in an async function produced the recorded client result. The first live-harness command was rejected before execution because recursive cleanup was disallowed; explicit single-file cleanup then passed and left no harness process or temporary directory. The live corpus remains a direct-seeded rrflowKV fixture and proves none of D-01, rrflowMX parity, WebSocket, remote TLS, browser, package installation, complete operation validation, crash/reopen, or label completeness
not run and reason: the all-language conformance orchestrator, minimum-supported Node runtime, real browser/bundler/CORS/WebSocket matrix, HTTPS/mTLS/mesh rotation, installed rrflowMX/rrflowKV differential, complete catalogue/fault/resource corpus, full workspace tests, Connectome, clean offline distribution, and release qualification do not prove this documentation-only KB-05 classification and remain owned by A-07/B-04/D/H/J
remaining known errors: 9 KB-05 records remain; A-06/A-07 are incomplete; POAM-011 remains; TypeScript still has a monolithic runtime, erased payload validation, permissive status/media/result/error handling, serializable credentials, broad immediate retry, local-only cancellation, implicit redirects, no remote endpoint/WebSocket/W3C path, direct-seeded label-only conformance, and no built/offline/browser-qualified package
roadmap checkbox changed: no
```

##### `rrd-python-client`

```text
gate/package: A-06 / KB-05 / rrd-python-client
revision: parent c4e6e4b; result is the commit containing this entry
baseline files/digests: docs/rrd-python-client-v1.md=00f3cb1ff0c03c3f8ec06e442eced58e1d048ba8b57030d5fd206e3cd5b178d0; pyproject.toml=a54c713877573e2897f7a647099c67b4f517991dd109f41b36565e18f3ff5b7a; uv.lock=031e6b9e670b23a197353f83c9162cb71639d24e8f0659353cfe0c2ae42d8a11; generate.py=3f9deed96b26a467dcc01f395d1bd50474941d012215c6c284f9f11d64125974; package __init__.py=334c7d5f0aef21a96d035eed1f0fb8d9b16b86b3f0bdb94d36184ced7ef6da55; client.py=f4205382d04090a7ed87adc6e01143bec0dcf1e2b1cbfdf89003379587f582d4; models.py=a2cffd71dfdd66d677e9afe9cd2baa6bc3b17482ec965aa318138dec3d6e6987; generated __init__.py=90f31e40c1c1fc0ace6365c6b731d8296016d88727835f98149160ec3d975281; generated endpoints.py=7dd3bceab280ce07bb431075d5b347b28d1a01702d0ed36aebece330f25a6e9e; test_client.py=22c465b520ed06573a21d69d023f09912aa25a57de321e6333be8fd78da409f8; sdk_conformance.py=adef43216f75e3aa4013b16c1d19924f6c52bdf36a80ddeaebc321f78747b84c; runner=5870f73b9bad92c14befd94b9bbab4f396f8ddd664c56cec0ce7e3a64c0b1a39; checker=fcd235248e027a4587444438480f1061f1314ba15e95a82c1068afe84fbad783; SDK corpus=b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; live harness=8fab285eb4a3a7081031cbcf858e1d604571346b9bb2ec1701c6807a888cf158; corpus contract=69e24d5a6299b257a819dce1eba1df6df656401fc6ce51719a7982ab16fb663b
files read in full: AGENTS.md; root README; flat Python-client record; SDK index and canonical Rust/TypeScript SDK references; relevant public-contract, canonical-roadmap, POA&M, and complete execution-map owner sections; complete Python project configuration, 595-line uv lock, generator, package roots, 312-line runtime, models, all 33 generated endpoint descriptors, mock tests, and conformance entry; complete conformance orchestrator. The shared corpus, live harness, corpus contract, and generated-surface checker were reused only after unchanged digests matched the immediately preceding complete SDK reviews. The complete deterministic inventory generator and execution map were reused from that immediately preceding package after baseline digests were verified and every Python-relevant current/traceability/convergence/queue/journal span was revalidated. Current PyPA wheel/packaging, PEP 561, HTTPX client/async/timeout/resource, RFC 9110/6455, W3C Trace Context, and OWASP logging constraints were inspected from their primary publishers
files changed/created/deleted/moved: create docs/reference/sdk/python.md; update the SDK index, public-contract surface table, POAM-011, this implementation inventory/traceability/direct-convergence/resolved-review/queue/A-07/journal map, deterministic planned-file inventory, and generated file plan; delete docs/rrd-python-client-v1.md; no Python/Rust runtime, package manifest, dependency lock, generated endpoint map, wire fixture, endpoint, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion behavior changed
contract or behavior changed: none; the canonical record fixes Python as one outward sync/async client family of the one RrdEngine, corrects the stale 34-operation claim to a synchronous generic 33-HTTP-operation surface with no async/WebSocket/remote profile, and freezes exact runtime-validation, credential, retry/certainty, cancellation, transport, packaging, interpreter, MX/KV, and full-engine cross-surface requirements without claiming SDK or engine qualification
smallest test command and result: uv --directory sdks/python run --frozen pytest -q — 4 mock-focused tests passed before and after editing
owning package command and result: uv lock --check — resolved 24 locked packages; generator check passed; Ruff lint and format checks passed over 8 files; strict mypy passed over 7 source files; pytest passed 4 tests; an offline Hatchling build produced a 7,019-byte pure-Python wheel and 47,763-byte source distribution before and after editing under Python 3.14.4 and uv 0.11.21
cross-boundary command and result: python3 scripts/ci/run_sdk_conformance.py — exited 0 and all six current language entries reported the shared corpus SHA-256 b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; this is one real direct-seeded rrflowKV daemon fixture, not installed-engine qualification. Generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; deterministic inventory reported 843 current/generated/planned records; documentation policy, workspace architecture, workflow policy, frozen 1.0.0 version policy, Ruff, Python compilation, Cargo formatting, workspace all-target check, and diff checks passed
failure/crash/differential evidence: with RRD_SDK_CONFORMANCE_MANIFEST removed, the Python conformance entry exited 1 before executing a scenario. A direct HTTPX probe replayed an invalid unpinned query twice, accepted 201 text/plain with an invalid success payload, and serialized the synthetic bearer through dataclasses.asdict. The first offline-build inspection successfully built and listed both artifacts but its cleanup missed the hidden build-output .gitignore and exited 1; the exact temporary file/directory was inspected and removed, and the corrected offline build/cleanup exited 0. A mistaken --help argument to the conformance runner launched its all-language behavior because the script has no argument parser; the process was observed to completion and the exact no-argument command was subsequently rerun deliberately and exited 0. One restaging command repeated the already-staged deleted flat pathname, so Git rejected that path before newer journal edits were staged; the generator output from that sequence was discarded, only existing paths were staged, and identical generation then exited 0. An additional repository-wide Ruff format check identified three unchanged CI scripts that the current formatter would rewrite; this package did not modify unrelated files, while Ruff lint, Python compilation, and the changed inventory generator's focused format check passed. No storage crash/reopen or rrflowMX/rrflowKV differential was created
not run and reason: minimum Python 3.11 through current-version and Linux/Windows/macOS matrices, downstream PEP 561 consumer, HTTPS/mTLS/mesh rotation, async/WebSocket/server-cancellation, complete operation/fault/resource corpus, D-01 installation, full workspace tests, persistent graph/BM25/vector/RRF/reasoning/Arrow/DataFusion semantic corpus, Connectome, clean cache-empty offline dependency installation, reproducible/signature comparison, benchmarks, and release qualification do not prove this documentation-only KB-05 classification and remain owned by A-07/B-04/D/H/J
remaining known errors: 8 KB-05 records remain; A-06/A-07 are incomplete; POAM-011 remains; Python still has a monolithic synchronous runtime, unvalidated operation payloads, permissive status/media/result/error handling, public mutable secret dictionaries, broad immediate retry, local-only deadlines, ambient HTTPX trust/resource defaults, no async/remote/WebSocket/W3C path, direct-seeded label-only conformance, missing py.typed/release metadata, and no supported interpreter/platform/clean-offline package qualification
roadmap checkbox changed: no
```

##### `rrd-go-client`

```text
gate/package: A-06 / KB-05 / rrd-go-client
revision: parent f081d88; result is the commit containing this entry
baseline files/digests: docs/rrd-go-client-v1.md=ac0e3f2f8f7bd8bf20ac66fc513d6f138e6ca190ec87a06b60e0487306d8dc4f; go.mod=786d23b4a9d1467dfed7e6545db8305ff28eaf208ef5a5590b5ede777b939daf; client.go=687e1da78c40597dbfd6cf6ee5968ed069e0d89b954c781ecbbd43b7792b1b01; models.go=fba33f05e241a8a009d4dd3b1cbca99a51e692cc618ffd35308435f71b3e6be0; endpoints_gen.go=28ad4b65efc33ff4ed7c0cb3321d2919b3801a463ddc367d389ba91629d1d66b; client_test.go=17dabb45e36d2038718e1e13757ef825c97d8e3fabb21c8cd3534c53e8224f93; generator=7f642d5c9f3f8988d9a48ff4e8e58d00393ad91aefc5320599a02da847af7057; conformance command=541bff222ab9170d1668b6072f410920b692463fa95dd703d9f476b5028fec38; runner=5870f73b9bad92c14befd94b9bbab4f396f8ddd664c56cec0ce7e3a64c0b1a39; checker=fcd235248e027a4587444438480f1061f1314ba15e95a82c1068afe84fbad783; SDK corpus=b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; live harness=8fab285eb4a3a7081031cbcf858e1d604571346b9bb2ec1701c6807a888cf158; corpus contract=69e24d5a6299b257a819dce1eba1df6df656401fc6ce51719a7982ab16fb663b; inventory generator=c530ba0cffccb67ef663414c3f1c7af5a9518172bd066b790203c77bf0c1a69c; execution map=91b0df48114fa11334c4e5e73982e86017b3383c5f0b6a4a5dc84e1132aad4b7
files read in full: AGENTS.md; root README; flat Go-client record; SDK index and public-contract reference; relevant canonical-roadmap, POA&M, execution-map owner/traceability/queue/A-07/journal sections; complete three-line module file; all 498 lines of client.go; all 57 lines of models.go; all 79 generated endpoint lines; all 177 unit-test lines; all 151 generator lines; and all 199 conformance-command lines. Canonical Rust/TypeScript/Python SDK references, the complete deterministic inventory generator and execution map, shared corpus, live harness, corpus contract, generated-surface checker, and conformance runner were reused only after their digests matched the immediately preceding complete SDK reviews and every Go-relevant span was revalidated. Current Go context, net/http/Transport/httptrace, toolchain selection/support, nested-module source/tag/archive, module-release, structured-secret-redaction, RFC 9110/6455, and W3C Trace Context constraints were inspected from their primary publishers
files changed/created/deleted/moved: create docs/reference/sdk/go.md; update the SDK index, public-contract surface table, POAM-011, this implementation inventory/traceability/direct-convergence/resolved-review/queue/A-07/journal map, deterministic planned-file inventory, and generated file plan; delete docs/rrd-go-client-v1.md; no Go/Rust runtime, module manifest, generated endpoint map, wire fixture, endpoint, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion behavior changed
contract or behavior changed: none; the canonical record fixes Go as one outward concurrent client family of the one RrdEngine, corrects the stale 34-operation claim to a context-aware synchronous generic 33-HTTP-operation surface with no WebSocket/remote profile, and freezes exact runtime-validation, credential, transport, retry/certainty, caller/server-cancellation, module/toolchain, MX/KV, and full-engine cross-surface requirements without assigning Go orchestration or claiming SDK/engine qualification
smallest test command and result: go test -count=1 ./... — the three mock-focused package tests passed; both command packages compiled and reported no test files before and after editing
owning package command and result: go test -race -count=1 ./... passed the same package under the race detector; go vet ./..., go run ./cmd/generate --check, gofmt listing, and go mod tidy -diff exited 0; GOTOOLCHAIN=local GOPROXY=off GOSUMDB=off go test -count=1 ./... passed with the current dependency-free module under Go 1.26.0 on Linux/amd64 before and after editing
cross-boundary command and result: python3 scripts/ci/run_sdk_conformance.py — exited 0 and all six current language entries reported shared corpus SHA-256 b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; this is one real direct-seeded rrflowKV daemon fixture, not installed-engine qualification. Generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; deterministic inventory reported 857 current/generated/planned records; documentation, workflow, frozen 1.0.0 version, Cargo formatting, 16-test workspace architecture, workspace all-target, and diff checks passed
failure/crash/differential evidence: with RRD_SDK_CONFORMANCE_MANIFEST removed, go run ./cmd/conformance exited nonzero before executing a scenario. A temporary same-package adversarial probe, removed after execution, attempted an invalid unpinned query twice, accepted 201 text/plain with an invalid success payload, and exposed the synthetic bearer through both encoding/json and fmt formatting. The first inventory regeneration failed because the unstaged deletion remained in the Git-derived current-file list; only this bounded package was staged, deterministic regeneration then wrote 857 records, and the failure is retained here. No storage crash/reopen or rrflowMX/rrflowKV differential was created
not run and reason: declared-minimum Go 1.24 and other supported-toolchain execution, Windows/macOS and architecture matrices, external clean module consumer/archive/tag/provenance checks, authenticated HTTPS/mTLS/mesh rotation, WebSocket/correlated server cancellation, complete operation/fault/resource corpus, D-01 installation, full workspace test suite, persistent graph/BM25/vector/RRF/reasoning/Arrow/DataFusion semantic corpus, Connectome, reproducible signed offline distribution, benchmarks, and release qualification do not prove this documentation-only KB-05 classification and remain owned by A-07/B-04/D/H/J
remaining known errors: 7 KB-05 records remain; A-06/A-07 are incomplete; POAM-011 remains; Go still has catch-all client/models/test files, untyped operation payload/results, permissive status/media/result/error handling, exported secret-bearing structs, broad immediate replay, local-only context cancellation, ambient shared default-transport policy, no explicit close/remote/WebSocket/W3C path, direct-seeded label-only conformance, and no minimum-toolchain/platform/module-archive/clean-consumer qualification
roadmap checkbox changed: no
```

##### `rrd-java-client`

```text
gate/package: A-06 / KB-05 / rrd-java-client
revision: parent 0a3e8f5; result is the commit containing this entry
baseline files/digests: docs/rrd-java-client-v1.md=6c2f49dbd14bf083e9d6774152bfa7e44d0671e5f26cf3407f083f1648e63bf4; pom.xml=27e9cfa04afa3dea941882337d75ed04c5a3f45a1f80d5dfce9531d0b4be101a; generator=15f8447ba03177aa76ef1626a1b6f9cc8597410218a1d48ceff040f6742958c9; OperationId.java=4b36b967fd80e025d83ad3f2a2f4bc2d2751865db848dacd8a174ef7e6aa5fa1; RequestOptions.java=662656172c7f38d723994eba0525321f2d84d3bd178d46a7a7a3c1e4c8ab39cf; ResourceSegment.java=268961d37b461dc13e3bf7c401dec49a619c95ce0b05a377c41423eec990092c; RrdApiException.java=42b13f4edd9fa5fbf8fb862b79de1c20eeb020d0feadf2c0f4171415a6862b6b; RrdClient.java=358a84d77f457d57b701d586d81a3f97d3f53c8827f47ed0f76d62ac37c7137d; RrdClientException.java=17baacde2858e2db5f40efd40cb0a6e6ff29f86c2371edb291a49a5966d3b5fe; Session.java=d3725437b96dc789612134f1a3447596a8f4c914dc63b825813e7389e9d607bb; RrdClientTest.java=6fc7dc2d1905acccc3ac2cf7618fa5b974b7a20d871441a1f44a63f5fcb130cc; SdkConformanceTest.java=f5238c6cef78c7f7cdb68de626bc3b9dd5f9f67c16bec3b32fba1de7077d6928; runner=5870f73b9bad92c14befd94b9bbab4f396f8ddd664c56cec0ce7e3a64c0b1a39; checker=fcd235248e027a4587444438480f1061f1314ba15e95a82c1068afe84fbad783; SDK corpus=b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; live harness=8fab285eb4a3a7081031cbcf858e1d604571346b9bb2ec1701c6807a888cf158; corpus contract=69e24d5a6299b257a819dce1eba1df6df656401fc6ce51719a7982ab16fb663b; inventory generator=aa5479e05a309f408ca61c3a178d276db98fbfe056dc24ae625de07978ef4439; execution map=cbb652f2045326ddd23bed790531cfc8da08245f28e50b22bc5718f00f6e8a8a
files read in full: AGENTS.md; root README; flat Java-client record; SDK index and canonical Rust/TypeScript/Python/Go SDK references; public-contract reference; relevant canonical-roadmap, POA&M, and complete execution-map owner/traceability/queue/A-07/journal sections; complete Maven project file and generator; all 61 OperationId lines, 33 RequestOptions lines, 3 ResourceSegment lines, 30 RrdApiException lines, 390 RrdClient lines, 13 RrdClientException lines, 5 Session lines, 219 unit-test lines, and 294 conformance-test lines. The complete deterministic inventory generator and execution map were reused from the immediately preceding package only after their baseline digests were verified, then every Java-relevant span and the modified inventory logic were revalidated. The shared corpus, live harness, corpus contract, generated-surface checker, and conformance runner were reused only after unchanged digests matched the preceding complete SDK reviews. Current Java 21 HttpClient, WebSocket, CompletableFuture, record, JAR, Maven repository/offline/reproducibility/dependency-convergence, RFC 9110/6455, and W3C Trace Context constraints were inspected from their primary publishers
files changed/created/deleted/moved: create docs/reference/sdk/java.md; update the SDK index, public-contract surface table, POAM-011, this implementation inventory/traceability/direct-convergence/resolved-review/queue/A-07/journal map, deterministic planned-file inventory, and generated file plan; delete docs/rrd-java-client-v1.md; no Java/Rust runtime, Maven project, generated operation enum, wire fixture, endpoint, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion behavior changed
contract or behavior changed: none; the canonical record fixes Java as one outward synchronous/async client family of the one RrdEngine, corrects the stale 34-operation claim to a synchronous generic 33-HTTP-operation surface with no WebSocket or qualified remote profile, and freezes exact operation-model, protocol-validation, credential, transport, retry/certainty, interruption/caller/server-cancellation, concurrency, artifact, MX/KV, and full-engine cross-surface requirements without assigning Java engine authority or claiming SDK/engine qualification
smallest test command and result: mvn --batch-mode -f sdks/java/pom.xml -Dtest=RrdClientTest test — 3 mock-focused tests passed under OpenJDK 21.0.12 and Maven 3.9.12
owning package command and result: Java generator check passed; mvn --batch-mode -f sdks/java/pom.xml clean test — 3 unit tests passed and the one manifest-dependent conformance test was explicitly skipped, for 4 total with 0 failures and 1 skip
cross-boundary command and result: python3 scripts/ci/run_sdk_conformance.py — exited 0 and all six current language entries reported shared corpus SHA-256 b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; Java's configured run executed 1 test with no skip. This is one real direct-seeded rrflowKV daemon fixture, not installed-engine qualification. Workspace architecture — 16 passed; cargo check --workspace --all-targets --locked — passed; deterministic inventory — 874 records; documentation policy — 89 statuses/81 coordinates; generated-surface parity — 33 HTTP operations/OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow policy, frozen 1.0.0 version policy, focused Ruff, Python compilation, Cargo formatting, and diff checks passed
failure/crash/differential evidence: a temporary JUnit characterization, removed immediately after execution, made an invalid unpinned query attempt twice after the first connection drop, accepted 201 text/plain with an invalid success payload, and exposed the synthetic bearer through Session.toString and Jackson serialization. Two clean unchanged Maven package runs produced different JAR SHA-256 values e1c90e371fda61195e10961d67c507fa4ed2a523f1542b6b8bc3cdf5597a22d0 and 17e7efd438f69f9bd699c62b1857c82ade97a7e2e6d00cdf11675ce29e549876. A fresh empty local repository with Maven offline failed before compilation while resolving maven-resources-plugin; the built archive has no explicit module descriptor and derives rrd.client@1.0.0. One focused Ruff format check found the newly extended planned-path tuple noncanonical; the generator was formatted, regenerated, and every focused check then passed. No storage crash/reopen or rrflowMX/rrflowKV differential was created
not run and reason: alternate supported JDK/Maven releases, Windows/macOS and architecture matrices, external clean Maven/JPMS consumers, byte-identical main/source/Javadoc artifacts, exact empty-cache offline closure and signatures, authenticated HTTPS/mTLS/mesh rotation, WebSocket/correlated server cancellation, complete operation/fault/resource corpus, D-01 installation, full workspace test suite, persistent graph/BM25/vector/RRF/reasoning/Arrow/DataFusion semantic corpus, Connectome, benchmarks, and release qualification do not prove this documentation-only KB-05 classification and remain owned by A-07/B-04/D/H/J
remaining known errors: 6 KB-05 records remain; A-06/A-07 are incomplete; POAM-011 remains; Java still has a catch-all synchronous client, arbitrary Object/JsonNode operation shapes, permissive status/media/result/error/correlation handling, public record credentials, broad immediate replay, local-only timeout/interruption, ambient HttpClient executor/proxy/pool behavior, no explicit close/remote/WebSocket/W3C path, direct-seeded label-only conformance, non-reproducible JAR output, derived module identity, no exact offline closure, and no supported JDK/platform/concurrency/consumer qualification
roadmap checkbox changed: no
```

##### `rrd-dotnet-client`

```text
gate/package: A-06 / KB-05 / rrd-dotnet-client
revision: parent c96196c; result is the commit containing this entry
baseline files/digests: docs/rrd-dotnet-client-v1.md=e2ee501cae5a40ce40711ec6866d1579f72a021c7ba48275256eb9649ff8c5b6; Rrflow.Rrd.slnx=7e6be1ffb40f06ac62c5c5c44bb471050c96da64303c069e75ca45d7b00aabf6; generator=31e7bb2d0cb769f9c443b43854d03062ab363e79df4a637a7c1568446977a8dc; Errors.cs=e3851b6398caa6343aa0710dbc856bc0d09fd3d070e3d98cc157eaf8de3ac0e4; Models.cs=65a3bd721f6a820b1b397427a6148f8c3cfb6f75d8af19424bd4a8d93d34f5e2; OperationId.g.cs=eadda9613340ebafc99ad49c0f6f7662f6b1414e0c1a8f3196cc0cb203ea9cc8; package README=5aac99ea9c32a7ff52fe66b247132b8937437b832b7936f4fac490db48c2b382; RrdClient.cs=3b9ba555d5245c55c78cc12ddb328aab7cd0ba3fb1bfb6a02392fa13aeb336ad; client project=289e839e3b61e98acc7f8b66d2ada723a84b96a7a563b2cf3d22ddcbd14093e4; client lock=a29c6aa8cfb81874ff8bb78dc369d7416f28c9b8cc47e99592bfc019b20c41eb; RrdClientTests.cs=7b1c8a5108dde3623bb4a04636b9bfe5183662e3bfe083ebb87e5e93153af8f7; test project=31978cbb0b320406bf112309ac0f037de27492de5628353f486d00a3c18c7222; SdkConformanceTests.cs=65a1abfce08dfbcda9fab1a8ff375363f5ae1cfcd946ac42af14a3a333080ed0; test lock=8fa544dcde231d725d444f3a204e6d9b75e5066f94cc71d66a4d5c26074ac608; SDK corpus=b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; runner=5870f73b9bad92c14befd94b9bbab4f396f8ddd664c56cec0ce7e3a64c0b1a39; live harness=8fab285eb4a3a7081031cbcf858e1d604571346b9bb2ec1701c6807a888cf158; corpus contract=69e24d5a6299b257a819dce1eba1df6df656401fc6ce51719a7982ab16fb663b; checker=fcd235248e027a4587444438480f1061f1314ba15e95a82c1068afe84fbad783; SDK index=34c963c32ba17d262c4bfaf2cb7a6a7b6a16844c2c40b87d8ae512226f994857; public contract=daf9a5fc8fc38af0d8ae23ee3d247eb3966bb3c106d2348f1254c0fd2a56427a; POA&M=716496e040073d5e0f7ce7e9b960d19bb0169966cf40707700bef1d6a8a611fd; inventory generator=f0788b6eb716ba596b11c388a4cece09749e38907845b0dc4221d533eb5aae74; execution map=2b19aa1e4639dbab52f5001fa1e8fdcd15726cbd3d6b9cfa5129e111952ce721
files read in full: AGENTS.md; root README; flat .NET-client record; SDK index and canonical Rust/TypeScript/Python/Go/Java client references needed for the shared rules; public-contract reference; relevant canonical-roadmap, POA&M, and complete execution-map owner/traceability/queue/A-07/journal sections; complete solution, generator, both project files, both lock files, all 33 Errors lines, 34 Models lines, 97 generated-operation lines, 7 package-README lines, 455 RrdClient lines, 258 unit-test lines, and 323 conformance-test lines. The complete deterministic inventory generator and execution map were reused from the immediately preceding SDK package only after their parent digests were verified, then every .NET-relevant span and all modified inventory logic were re-read. The unchanged shared corpus, live harness, corpus contract, generated-surface checker, and conformance runner were reused only after their digests matched. Current .NET 10 HttpClient/SocketsHttpHandler, ClientWebSocket, System.Text.Json source generation, NuGet locked restore/package authoring/determinism/validation/signing, trimming/Native AOT, supported-runtime, cross-platform, RFC 9110/6455, and W3C Trace Context constraints were inspected from their primary publishers
files changed/created/deleted/moved: create docs/reference/sdk/dotnet.md; update the SDK index, public-contract surface table, POAM-011, this implementation inventory/traceability/direct-convergence/resolved-review/queue/A-07/journal map, deterministic planned-file inventory, and generated file plan; delete docs/rrd-dotnet-client-v1.md; no .NET/Rust runtime, solution/project/lock file, generated operation enum, wire fixture, endpoint, engine, storage, query, graph, index, vector, reasoning, Arrow, or DataFusion behavior changed
contract or behavior changed: none; the canonical record fixes .NET as one outward asynchronous client family of the one RrdEngine, corrects the stale 34-operation claim to a generic 33-HTTP-operation surface with no WebSocket or qualified remote profile, and freezes exact workspace, class, operation-model, protocol-validation, credential, handler/socket, retry/certainty, caller/server-cancellation, concurrency, NuGet/offline/toolchain, MX/KV, and full-engine cross-surface requirements without assigning .NET engine authority or claiming SDK/engine qualification
smallest test command and result: dotnet run --project tests/Rrflow.Rrd.Client.Tests/Rrflow.Rrd.Client.Tests.csproj --no-restore -- -method Rrflow.Rrd.Client.Tests.RrdClientTests.PublicNegotiationRetriesTransportLossAndValidatesIdentity — 1 focused test passed under .NET SDK 10.0.111/runtime 10.0.11 on Ubuntu 26.04 x64
owning package command and result: locked solution restore, .NET generator check, Release solution build, and dotnet format verification passed; build reported zero warnings/errors; the full xUnit executable reported 4 passed, but only 3 are meaningful local mock tests because the manifest-absent conformance method returns successfully rather than reporting a skip
cross-boundary command and result: python3 scripts/ci/run_sdk_conformance.py — exited 0 and all six current language entries reported shared corpus SHA-256 b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2; .NET's configured run executed the live method. This remains one real direct-seeded rrflowKV daemon fixture, not installed-engine qualification. Workspace architecture — 16 passed; cargo check --workspace --all-targets --locked — passed; deterministic inventory — 900 current/generated/planned records; documentation policy — 89 statuses/82 coordinates; generated-surface parity — 33 HTTP operations/OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow policy, frozen 1.0.0 version policy, focused Ruff, Python compilation, Cargo formatting, and final diff checks passed
failure/crash/differential evidence: a temporary xUnit characterization, removed immediately after execution, made an invalid unpinned query attempt twice after the first HttpRequestException, accepted 201 text/plain with an invalid success payload, and exposed the synthetic bearer through Session.ToString and System.Text.Json serialization. Two clean unchanged Release builds produced byte-identical DLL SHA-256 e7b649c6ea363f36a5bf3241278175da6dd03f8b5e36d7fb59b9a4c4b6ce3abb and PDB SHA-256 d73c75f59453a5f42adeb882987f85da84aff812949915f94df208b8e3f0a966, but consecutive NuGet packages differed at 41eede3ed8ffc00c7507faa6b788231b6aea638c1c92eac253fae10630445619 and 9c859969c08f4845b734ae75097eeeb152efe7ef9a959d5c706f3f72d7391916 because their OPC core-property relationship/part identity and archive timestamps differed. Supplying DeterministicTimestamp still yielded different packages 5283790c726215f7f4c66410f8c9cdb16f424d6a6dc91107a8d6f5f479921c0a and 98671044d23f7757ca2fb9fabc397d617346d5e8a168467fd3bf2e82c9afa11e. dotnet nuget verify --all failed NU3004 because the package is unsigned; an empty file-backed NuGet source/cache restored the dependency-free runtime project but failed the solution resolving xUnit. An initial package-inspection command was rejected before execution because it proposed recursive deletion; the replacement used a validated temporary directory and bounded file deletion. One focused-test invocation selected zero tests due an incorrect namespace; listing methods exposed the exact name and the corrected invocation passed one. No storage crash/reopen or rrflowMX/rrflowKV differential was created
not run and reason: alternate supported .NET SDK/runtime patches, Windows/macOS and architecture matrices, external clean NuGet consumer, package validation, trimming/Native-AOT consumers, byte-identical signed main/symbol/source packages, exact empty-cache offline feed closure, authenticated HTTPS/mTLS/mesh rotation, WebSocket/correlated server cancellation, complete operation/fault/resource corpus, D-01 installation, full workspace test suite, persistent graph/BM25/vector/RRF/reasoning/Arrow/DataFusion semantic corpus, Connectome, benchmarks, and release qualification do not prove this documentation-only KB-05 classification and remain owned by A-07/B-04/D/H/J
remaining known errors: 5 KB-05 records remain; A-06/A-07 are incomplete; POAM-011 remains; .NET still has catch-all model/client files, arbitrary object/JsonElement operation shapes, permissive status/media/result/error/correlation handling, public record credentials, broad immediate replay, conflated local timeout cancellation, ambient handler proxy/cookie/pool/header policy plus independent client timeout, no qualified remote/WebSocket/W3C path, direct-seeded label-only conformance with a manifest-absent false pass, non-reproducible unsigned placeholder NuGet output, no pinned SDK/offline closure, and no supported runtime/platform/concurrency/external/trimmed/Native-AOT consumer qualification
roadmap checkbox changed: no
```

##### `qdrant-capability-inventory`

```text
gate/package: A-06 / KB-05 / qdrant-capability-inventory
revision: parent 902b96f; result is the commit containing this entry
baseline files/digests: docs/qdrant-capability-inventory.md=8d6489a61f824e05b9594a2b8cd630384a06c1218a0653c1401a0e9cb2b5fc25; research index=5f0c3c8857f76ec315ff4d8a2435294f847228bfcdc59ba98db81453bdd60cbb; convergence research=0fb0e99b95ed1947b9fd6b4925a1eea60727b175d0944b5e18b68122d80aaef3; vector index=7b76d509325ccbcd07ae2306a2c03a16c7a28f2e90254bceaf7dd7973e801310; collections reference=3a03428bcca4e623d03c462a558caf42525109da2f389965e0f528c7a3de544d; search reference=fb9a789539e36a9104dfe58d7235c7b9d6c9c3a8f8293b6a7bd8042f00d3f7fc; HNSW reference=7f2566c3315eab4f137f79a3b67b1941d81f4dcd00a31ba8bc04384128398ace; quantization reference=3093deb53226555dc3c6fe1a844cc0d1720db8f37fa3868757e47c3fe100de6f; memory-tier reference=ff99145b6bee7376006f6075b4f6786bef18111db1a386d5822737c047040172; retrieval reference=b9a225e1ee244769634ed1bc54d420c711505ecc33e521c60aacc4f0943477c9; query catalogue=7631204f28cf838f9f7bbf5cb5c3c175c2075d3a934806a9bc4c4ece43645302; engine flow=d7b93cfe8b6a29c95e6822aa397bf98f39dc8dc69fa68765e9be3c4fe9702784; execution map=a731b0abb17aad944d3b402d9ccc81da13cd9664ac73009ae38d278d455b9dcb; generated file plan=2feaa45bedbd61dbaf47fb8291fff8aae1b721efaf89c5a0996df3cf089edc18; POA&M=d36e6f1ae248ad0d985d3475294d5fc8a7f93580e86bbc2e45ea5a3377c00f63; inventory generator=23e70dd9f2f19f2b921da63a2d32e13667e5c51da3120f89b30bbbeeac4d532b; engine vector root=d5b8469c48ec78e6b34e09395711ae26dfff51a30f911cb3fb83d4c0c4d7fb7b; engine retrieval=38f34992e59cf7aa4580591cb3faaa25f250d3d949de88b368967257089630eb; retrieval query=27d1c84d4d35e63493f0394f3639fc4bf6857c8284bfd5d51be4d13609c73a27; HNSW source=0cd8df8848d237094176b9ee6a745ed4550e9f4c03e4370d33e31836e111ff5b
files read in full: AGENTS.md; root README; the 381-line flat Qdrant inventory; complete research index and convergence-research record; complete vector index, collections, search, HNSW, quantization, memory-tier, and compact-artifact references; complete context-retrieval, query-index-catalogue, and engine-data-flow owners; relevant canonical roadmap and POA&M rows; complete execution-map classification, traceability, queue, resolved-review, and journal owner; complete engine vector root and collection/index/point/quantization/search modules; complete engine retrieval and recursive-query modules; and complete native HNSW implementation. The new canonical record and every changed section were reread after authoring
external primary references reviewed: Qdrant v1.19.1 and v1.19.0 releases; exact v1.19.1 source tag and commit; collection update worker; segment manifest; universal collection query; payload-index optimizer; HNSW graph builder; sparse IDF; quantized-vector implementations; shard-transfer modes; edge library; and official collections, points, storage, indexing, filtering, hybrid-query, quantization, memory-tier, multitenancy, distribution, consistency, snapshot, administration, configuration, interface, security, monitoring, inference, and edge documentation. GitHub's tag API independently reported the annotated tag object `de333e3c04660fe475d6275e9efc9fb9f54138fe` as validly signed and resolving to commit `6ab21cac18ebb6f4ae29102c7f8f5cc11affd5de`
files changed/created/deleted/moved: create docs/research/qdrant-capability-inventory.md; update the research index, convergence-research Qdrant source pins, this implementation traceability/resolved-review/queue/journal map, and generated file plan; delete docs/qdrant-capability-inventory.md; no Rust source, public contract, fixture, endpoint, SDK, engine, storage, query, graph, index, vector, reasoning, Arrow, DataFusion, install, or attunement behavior changed
contract or behavior changed: research ownership and deterministic file-planning behavior changed; runtime behavior did not. Qdrant is now an exact-revision behavior, algorithm, workload, and failure oracle rather than a product model or hidden engine. Any later adaptation must journal the exact upstream symbol and rejected assumptions, map it into RRFlow identities/transactions/Arrow schemas/budgets, retain an exact oracle, pass independent semantic/fault/resource/reopen evidence, and remove experimental compatibility paths. No ease, performance, durability, recall, or superiority claim exists before J-04
smallest test command and result: cargo test -p rrd-vector --test online_hnsw --locked — 3 generation/overlay/filter tests passed; cargo test -p rrd-engine --lib engine::tests::vector_index --locked — 10 current vector/retrieval characterization tests passed; cargo test -p rrd-engine --lib engine::tests::context --locked — 3 same-stamp/context/reopen tests passed. These characterize existing behavior and do not accept the C/E/F targets
owning package command and result: deterministic inventory check reported 899 current, generated, and planned records; documentation policy reported 89 statuses and 83 classified coordinates with parent indexes and local links intact; generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715
cross-boundary command and result: workspace architecture — 16 passed; cargo check --workspace --all-targets --locked — passed; frozen 1.0.0 version policy, workflow policy, focused Ruff, Python compilation, Cargo formatting, and diff checks passed
failure/crash/differential evidence: the first workspace-architecture run failed only because four new metadata lines contained forbidden trailing horizontal whitespace; those bytes were removed, the hash inventory was regenerated, and the complete 16-test gate passed. Current source review proved repeated cursor-zero history reconstruction, deterministic JSON HNSW generations that clone on advance, schema-only payload-index admission, Rust-collection retrieval fusion, a successful TurboQuant compatibility adapter, and eager DataFusion MemorySource materialization. No RRFlow storage crash, restart, semantic differential, recall comparison, or benchmark was created by this documentation package
not run and reason: the full workspace test suite, complete vector exact/ANN/quantization/accelerator matrix, rrflowMX/rrflowKV semantic differential, native graph/scalar/BM25/vector atomicity corpus, streamed Arrow/DataFusion resource/fault corpus, reasoning-tree persistence, D-01 installation/attunement, SDK/transport conformance, Connectome, clean offline rollout, and comparative benchmarks do not prove a KB-05 research classification and remain owned by C through J
remaining known errors: 4 KB-05 records remain; A-06/A-07 and POAM-013/014 remain incomplete; RRFlow still lacks accepted canonical rrflowKV transaction/native-read convergence, atomic native graph/scalar/BM25/vector projections, streamed read-stamped Arrow/DataFusion execution, persisted reasoning/context feedback, qualified installation/attunement, and like-for-like Qdrant performance or rollout evidence
roadmap checkbox changed: no
```

##### `surrealdb-capability-inventory`

```text
gate/package: A-06 / KB-05 / surrealdb-capability-inventory
revision: parent 0f83ff9; result is the commit containing this entry
baseline files/digests: docs/surrealdb-capability-inventory.md=37c193f80dd7d01128717b945c9b028944b4b19eace03283ee4b950532ccdee8; research index=aa2a24fa253e74cbc7fb2187d162d88d2aad8f6a9c628548799f2cbd7ccd337c; convergence research=e23e2c837d4a147bbd36656f6344db39afdd587ab9138562a847cd758cfa852a; system overview=9d5fb6f660e79b58730ff1cb21357d31474316e4657203df3d84716c48ed4111; engine flow=d7b93cfe8b6a29c95e6822aa397bf98f39dc8dc69fa68765e9be3c4fe9702784; current format=1f1872961c6e56c0210c8d623b98376781d6e9c6c8a2ef99104d5624aaa58ca9; multi-model owner=fcf1cf4c8f130dea72ccd654727d232cee41cd451eb388ba9ce535951456ece7; schema owner=d45eadef555002bd2539eba874e3e7fd17b0fc7c1a8b90b05a522f51f12ce777; query owners=0b929a07d584c7244ac72003eaa855352e3b5c7769c6b76882ce44a494bb13de,0f405e76e30cc7ea39be3ba9970da247d0e7cb47a42efed11d025ff793fd9d45,29533b25660422093038e86d1ac44e3ccca5db5f2416ed6976d4aadf962e0d19,7631204f28cf838f9f7bbf5cb5c3c175c2075d3a934806a9bc4c4ece43645302; execution map=3c29421e5465d9d4a6e1da752f1c30dc9281b99da35c99f81015624417de2d19; generated file plan=6bd6bea5408447ea449faec120711150f52feecf6d04fdd566cec472d5764826
files read in full: AGENTS.md; root README; the 364-line flat SurrealDB inventory; research index and convergence-research record; system overview; current rrflowKV format; multi-model and schema owners; all four query owners; relevant complete roadmap, POA&M, execution-map classification/traceability/queue/resolved-review/journal sections; all 1,957 lines of the storage port and rrflowMX implementation; all 2,774 lines of rrflowKV; the complete keyspace codec; complete engine transaction, query-transaction, query, and subscription files; complete query execute, DataFusion, index, and live-query files; and the complete MCP authority adapter. The unchanged engine-data-flow owner was reused only after its digest matched the immediately preceding complete review and every SurrealDB-relevant flow span was revalidated. The new canonical record and every changed section were reread after authoring
external primary references reviewed: SurrealDB 3.2 stable and 3.3 preview release lines; exact v3.2.4 and v3.3.0-beta.3 tag commits; complete recursive stable source-tree manifest; targeted stable `dbs`, `doc`, graph-key, transaction, index/build, RPC, and MCP source anchors; and official architecture, native data-model, geospatial, files, transaction, live-query, changefeed, security, permission, capability, extensions, RPC, SDK, operations/observability, self-hosted monitoring, Surrealist, agent-setup, and coding-agent-memory documentation. The v3.2.4 lightweight tag resolved independently to verified commit `93ab219d69f09d8f999851b0359c80ebe6726102`; preview beta.3 resolved to `6dce5c84e29ff6c12b73c401b2251566e1aeca60`
files changed/created/deleted/moved: create docs/research/surrealdb-capability-inventory.md; update the research index, convergence-research SurrealDB source pins, this implementation traceability/resolved-review/queue/journal map, and generated file plan; delete docs/surrealdb-capability-inventory.md; no Rust source, public contract, fixture, endpoint, SDK, engine, storage, query, graph, index, vector, reasoning, Arrow, DataFusion, install, or attunement behavior changed
contract or behavior changed: research ownership and deterministic file-planning behavior changed; runtime behavior did not. SurrealDB is now an exact-revision behavior, layout, lifecycle, failure, security, protocol, and readiness oracle rather than a feature scorecard or hidden engine model. Any later adaptation must journal the exact upstream symbol and rejected assumptions, map it into RRFlow identities/transactions/keys/pages/Arrow schemas/budgets, retain an independent oracle, pass semantic/fault/security/resource/reopen evidence, and remove experimental compatibility paths. No ease, performance, durability, correctness, or superiority claim exists before J-04
smallest test command and result: cargo test -p rrd-store --test unified_data --locked — 4 rrflowMX/rrflowKV semantic, rollback, object-failure, reopen, and idempotent-retry characterization tests passed; cargo test -p rrd-store rrflow_kv_multi_family_transaction_recovers_all_or_none_at_every_wal_boundary --locked — 1 crash/storage-full WAL-boundary test passed. These characterize current JSON/keyspace behavior and do not accept C
owning package command and result: cargo test -p rrd-query --test query --test index_catalogue --test live_query --locked — 24 query, stamp, profile/reopen, graph, index, corruption, uniqueness, BM25, DataFusion-analysis, and live-delta characterization tests passed. Five focused rrd-engine context/index/subscription/DataFusion/query-transaction tests passed individually. These prove the documented rough-draft behavior, not native adjacency, incremental index commits, or streamed Arrow pages
cross-boundary command and result: deterministic inventory — 898 current/generated/planned records; documentation policy — 89 statuses/84 coordinates with parent indexes and local links intact; generated-surface parity — 33 HTTP operations/OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workspace architecture — 16 passed; cargo check --workspace --all-targets --locked — passed; frozen 1.0.0 version, workflow, focused Ruff/format, Python compilation, Cargo formatting, and diff checks passed
failure/crash/differential evidence: existing storage tests exercised rollback, object publication failure, profile equivalence, durable reopen/retry, and injected crash/storage-full before WAL append and after WAL sync. Source review proved current relation-ID-only persistence, JSON row/keyspace values and snapshot artifacts, a broad storage port with separately committed catalogue state, cursor-zero reconstruction, eager Vec-to-Arrow `MemorySource` materialization, unsupported filter pushdown, relation-scan traversal, full old/new index reconciliation, and two-snapshot live polling. Four tool invocations were rejected before mutation: three used malformed working-directory or file-path strings, and one restaging command included the already-staged deleted path; their exact corrected commands, patch, and staging passed. No new RRFlow runtime fault, differential, or benchmark evidence was created by this documentation package
not run and reason: full workspace test suite, complete storage model/soak/compaction/corruption matrix, native graph/scalar/BM25/vector atomicity corpus, streamed Arrow/DataFusion buffer/resource/fault corpus, reasoning-tree and feedback persistence, D-01 installation/attunement, SDK/transport conformance, Connectome, clean offline rollout, and SurrealDB differential/benchmark do not prove a KB-05 research classification and remain owned by C through J
remaining known errors: 3 KB-05 records remain; A-06/A-07 and POAM-013/014 remain incomplete; RRFlow still lacks the accepted final key/Arrow-page format, narrow effect-complete transaction port, native direct graph/scalar/BM25/vector reads and atomic projections, streamed read-stamped Arrow/DataFusion execution, persisted reasoning/context feedback, qualified installation/attunement, and like-for-like SurrealDB performance or rollout evidence
roadmap checkbox changed: no
```

##### `rrflow-surrealdb-differential`

```text
gate/package: A-06 / KB-05 / rrflow-surrealdb-differential
revision: parent ecc817f; result is the commit containing this entry
baseline files/digests: AGENTS.md=b8209c1e2dcb70304099a4f5be21a1e0ae68fcca89e31aa36c62f0cf29335021; root README=1abccad070815efde97ed13af8a220a9931abd36550c6b02cad4831becbe437b; canonical roadmap=df4cdd67098c51fc18214c667b196518becf35613f8a78890fe22f9ec780a894; POA&M=d36e6f1ae248ad0d985d3475294d5fc8a7f93580e86bbc2e45ea5a3377c00f63; research index=93bb9bd242aafff517c0facf02196ff1ce58975229cab99e71e3f9146131ae3f; SurrealDB reference=a73b157ba8e33bd54e9c8b009cfd1b627104993d9c323c998c608c24c5505562; differential record=1ee3f1d588172fddc54cbe490c027d92b9476ebf5ac1fa782a79a5500cfd7758; Python comparator=7a45f80d809e01a717bcaf646c9a563e440eacc7a50a49ca673927568438c91f; stored result=12fef5d64171294783c00268e113036e8db9317fa3953996a0a3fb8193b5659b; benchmark evidence test=d6b5dde53538abea0111e91070dfe080bd79a662de588a07936b2d47a0270b63; current Rust benchmark=7c4d3d7942685a8890f31071be642e0f33b7a43b1c0ea38ea4792aedeb2897c9; execution map=81566d5a286f2fe38969dae78baa8ae3b8ec22d142f5c2cbf709b25c6eb33240; generated file plan=bfa7cf079b0e0dab068d591ccedcc0aad4d0ad8ab1a78b68dec8db820a7fd31f; inventory generator=23e70dd9f2f19f2b921da63a2d32f13667e5c51da3120f89b30bbbeeac4d532b
files read in full: AGENTS.md; root README; the 60-line differential record; all 591 lines of its Python comparator; all 640 lines of its stored result; all 115 baseline lines of the benchmark-evidence test; all 808 lines of the current Rust benchmark; the complete 478-line canonical roadmap; the complete 309-line baseline SurrealDB reference; the complete research index; the complete 60-physical-line POA&M; and the complete execution-map classification, queue, resolved-review, journal, and J-04 sections needed to resolve this package. The entire 1,861-line execution map was not claimed as newly read; only the named complete governing sections and their dependencies were used. Every changed source section was re-read after authoring
local references reviewed: the installed `/home/wardenop/.local/bin/surreal` reported `3.0.5 for linux on x86_64` and SHA-256 a9a5e9e36e4f6fe922e1991a4fb0ea1ee4fe90819c5e3a8dce238a56666e8cec; the canonical reference remains source-pinned SurrealDB v3.2.4 commit 93ab219d69f09d8f999851b0359c80ebe6726102. The current `engine_benchmark` child contract and per-file histories were inspected, and the planned J-04 comparator plus baseline manifest were verified absent rather than represented as working code
files changed/created/deleted/moved: update the canonical SurrealDB research record, research index, POAM-013, retained rrd-store benchmark-evidence test, this classification/resolved-review/queue/journal map, and generated file plan; delete docs/rrflow-surrealdb-differential.md, eval/surrealdb_claim_differential.py, and eval/results/2026-08-23-rrflow-surrealdb-3.0.5-claim-diagnostic-v1.json; no engine, rrflowMX, rrflowKV, rrflowDB, rrflowQL, DataFusion, Arrow, graph, index, vector, reasoning, protocol, installation, attunement, endpoint, or SDK runtime behavior changed
contract or behavior changed: no runtime behavior changed. An unsupported active competitive claim and its parse-only assertion were removed. The canonical SurrealDB record now retains the actionable comparison requirements: exact executable/source/dependency closure, one independently verified semantic corpus, equal timing/resource/lifecycle boundaries, failure cells, declared statistical treatment, and complete RRFlow workloads. One future comparator remains planned under J-04 only after its C through J-03 prerequisites; no compatibility path or evidence archive was created
smallest baseline command and result: cargo test -p rrd-store --test benchmark_evidence --locked — 2 passed before the edit, but one test only parsed checked-in JSON and asserted selected stored ratios. A current debug `engine_benchmark` was then built successfully; invoking the advertised Python comparator with one eight-operation trial exited 1 before trial zero because it supplied obsolete `--child native --path ...` arguments to a benchmark whose current contract is `--child --path ...`. After removal, cargo test -p rrd-store --test benchmark_evidence --locked — 1 retained storage-characterization test passed
owning package command and result: cargo test -p rrd-store --locked --quiet — exited 0; 139 tests passed across the package targets, with zero failed, ignored, or measured. This proves the removal did not discard executable rrd-store behavior; it does not prove the C storage target or a SurrealDB comparison
cross-boundary command and result: deterministic inventory — 895 current/generated/planned records; documentation policy — 88 statuses/84 classified coordinates with parent indexes and local links intact; generated-surface parity — 33 HTTP operations/OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; CI workflow policy — 7 substantive jobs, 5 cohesive engine suites, 20 default-feature packages, and 5 optional-feature packages; frozen version policy — 1.0.0; workspace architecture — 16 passed; cargo check --workspace --all-targets --locked — passed. Ruff lint over all four CI policy/inventory scripts, focused Ruff formatting for the only package-relevant retained generator, Python compilation, Cargo formatting, staged diff integrity, exact deletion checks, and the search for surviving active ratios, deleted-harness identities, or the host-specific Cargo cache path all passed
failure/crash/differential evidence: the fresh comparator failure before trial zero was retained as disposition evidence rather than hidden. The removed result omitted RRFlow revision, executable digest, build profile, dependency closure, and release manifest; used SurrealDB 3.0.5 rather than pinned 3.2.4; compared embedded RRFlow with SurrealDB HTTP/SQL and unlike readiness phases; used different session values and unequal field verification; accounted for processes, resources, and maintenance asymmetrically; ran three small trials without fixed resource controls, warm-up, concurrency, confidence treatment, long-duration drift, conflict/failure injection, or graph/BM25/vector/Arrow/DataFusion/context workloads; and asserted derived ratio direction without executing or independently recomputing the evidence. One initial history command used `git log --follow` with multiple paths and a misspelled benchmark path; exact per-file history commands passed. One read command had an invalid working directory and was rejected before execution. A policy-rejected temporary-file cleanup was replaced by exact non-recursive `unlink`; both temporary capture files were removed. The first inventory regeneration failed without writing because its Git-index enumeration still saw the unstaged deleted files; exact package staging followed by regeneration passed at 895 records. A deliberately broad Ruff-format probe exposed pre-existing formatting drift in unchanged `check_generated_surfaces.py` and `check_workflow.py`; focused formatting of the retained package-relevant generator passed, and no unrelated rewrite was added. No workspace mutation resulted from the failed diagnostic commands
not run and reason: no replacement SurrealDB benchmark was run because the pinned 3.2.4 executable, RRFlow release manifest, canonical equal semantic corpus, fixed resource envelope, and planned J-04 comparator/baseline do not exist. Full workspace tests, storage model/soak/compaction/corruption matrices, rrflowMX/rrflowKV semantic differential, native graph/scalar/BM25/vector atomicity, streamed Arrow/DataFusion resource/fault execution, persisted reasoning/context feedback, D-01 installation/attunement, SDK/transport conformance, Connectome, offline rollout, and release comparison remain owned by C through J and are not evidence for this documentation/evidence-removal package
remaining known errors: 2 KB-05 supporting records remain; A-06/A-07 and POAM-012 through POAM-014 remain incomplete; the planned J-04 comparator and baseline manifest are absent; RRFlow still lacks accepted native Arrow-page rrflowKV convergence, atomic graph/scalar/BM25/vector projections, streamed read-stamped Arrow/DataFusion execution, persisted reasoning/context feedback, qualified installation/attunement, and any like-for-like SurrealDB performance or rollout proof
roadmap checkbox changed: no
```

##### `anytype-connectome-client`

```text
gate/package: A-06 / KB-05 / anytype-connectome-client
revision: parent e660c24; result is the commit containing this entry
baseline files/digests: AGENTS.md=b8209c1e2dcb70304099a4f5be21a1e0ae68fcca89e31aa36c62f0cf29335021; root README=1abccad070815efde97ed13af8a220a9931abd36550c6b02cad4831becbe437b; docs/anytype-ui-research.md=85e941be75bbc1f3cde084aa3eb1ffa9fdff88acf1b7fe9c9a2cae6a83fdf003; system overview=9d5fb6f660e79b58730ff1cb21357d31474316e4657203df3d84716c48ed4111; engine flow=d7b93cfe8b6a29c95e6822aa397bf98f39dc8dc69fa68765e9be3c4fe9702784; instance topology=8d8d92d0bef5ca474eb92db7085d9f7a3b55cf0f745bf3dd2823193bf97086cf; reference index=4b4221f6d35fe66b91e0c29a207074cd4c89821a0c68c10c23280028f0843e9f; public contract=c42a4b22a70d2f33cffb0c52d558c0ae697d73819087bbfadd4d092922deac08; canonical roadmap=df4cdd67098c51fc18214c667b196518becf35613f8a78890fe22f9ec780a894; POA&M=fcb65afe5f6861691fa11e66ea6c8f8ad03fef4b9621d8639b7ab3c041281624; execution map=36f9cc516fba37cbd980b03d232c3c396ffd97bc757797e0924aaae0d545c8ad; inventory generator=23e70dd9f2f19f2b921da63a2d32e13667e5c51da3120f89b30bbbeeac4d532b; generated file plan=9983b4101af83a3775b8fca773b4240ad19f6b9f2309b7a3bf456562b503fdbd
separate-client baseline: clean ../connectome commit 38f68ce7adda9d03501f3591165f0e14996899ec; README=539a09a98e2bf745257cfdabff429b232e256d67af5e2764a95d604dca3d8d9a; REVIEW=7fe35b98d4aa3fde05dabf90addb040957459a0a5a5344c871e7f2f9260bd9c8; package=f5ed572a0de6caa85e0f607f210451158db30b0608fa87580633edef976cd67b; runtime=19a174180fd9d2f89a90534f6ae22f78ec8324c7c4088230649af93be6509275; attunement=66c221506930a7bbe34f39bb0ad7e251da6c9ba722c95e0582e0b2b3f173dda9; diagnostics=5a424674fe0548a627e7b7dc6de3ba31ebe1aeeb0fd1df8f96ad6d1b2fd068fc; control-plane=53f321a2dd1c62b3d7b4130ec45d35fefa964cc3d9c86edf6841e8558ed9a5a0; native module=000815881686439c71ee5d8a44aa1d6fb0d2402916b87b56069f6daf9d9e9d95; native client=daa20c483cc14360e8d78aef61fd53a00e73a17b612f1986d54e9e8c34e5a01e; native transport=21b4cad1594cd506c6e39b62505cad5e69541ba1dd947b469ffb47cf8ab7f4a4; browser smoke=a130a2ae3e104ef9290b6fe826d437ffc4880b99577b349d577829437d97e552; playwright=88c16cdc470ac465ae8143f82d5641a3273d8015c59e0208fe3092b4e5f48645
files read in full: AGENTS.md; root README; the 125-line flat Anytype/workbench record and its complete Git history; documentation and reference indexes; system overview; engine data flow; instance topology; public contract; canonical roadmap; complete physical POA&M; execution-map classification, queue, resolved-review, journal, H, J, and generated-path sections used by this package. In the separate Connectome checkout: README, REVIEW, package manifest, native RRD module/client/transport, runtime contract, attunement definitions, diagnostics protocol, control-plane client, root context provider, runtime-connection screen, browser smoke, and Playwright configuration. Every new or changed documentation body and index was reread after authoring
external primary references reviewed: the Anytype v0.55.4 annotated tag object f0d70edb4b3521d2b8e5da55299c014955ece21d independently dereferenced to commit f4677a073e41be6bfcf34a21b433027a3b3851aa; that exact commit's architecture guide, 74-line browser/native boundary, and 40-line graph-renderer record plus official view documentation were reviewed. Only projection, UI/backend separation, selected/global graph, composable-inspector, explicit platform-profile, and measured worker/WebGL patterns were retained; no Anytype code, identity, topology, middleware, protocol, or storage design was adopted
files changed/created/deleted/moved: create docs/reference/client/README.md and docs/reference/client/connectome.md; update root and reference portals, system-overview client pointer, H-06 required change/evidence, POAM-011, this classification/resolved-review/queue/journal map, inventory generator, and generated file plan; delete docs/anytype-ui-research.md; no Connectome source and no RRFlow engine, storage, query, graph, index, vector, Arrow, DataFusion, reasoning, install, attunement, protocol, endpoint, SDK, or automation runtime behavior changed
contract or behavior changed: documentation ownership and H-06 acceptance changed; runtime behavior did not. Connectome is now explicitly a separate projection client over generated public RRD operations. The target fixes fail-closed candidate/probe/capability/session sequencing, structured deployment truth, exact instance/resource/ReadStamp/cursor/evidence/completeness bindings, live resume/ACK/gap handling, engine-owned graph/context/index/DataFusion decisions, previewed actions, native/browser credential separation, typed transparent presentation, local-only preferences, bounded rendering, accessibility, multi-instance isolation, direct convergence, and installed rrflowMX/rrflowKV qualification. It rejects client-owned database/query/retrieval/reasoning/attunement/automation/diagnostics/control truth and claims no H-06 completion
smallest current-client command and result: in clean ../connectome, pnpm run check — Biome checked 569 files without fixes, the RRFlowQL identity gate checked 758 files, and TypeScript emitted no error; cargo test --manifest-path src-tauri/Cargo.toml rrd --locked — 4 native URL/request/session/context tests passed and 1 unrelated test was filtered. These characterize a partial client, not shared-contract or real-engine conformance
owning client command and result: pnpm run build — passed after 3,501 modules and emitted the documented unresolved/browser-externalized SurrealDB QL/Wasm and oversized-chunk warnings; pnpm run test:smoke — 2 Chromium tests passed against the production bundle and a mocked liveness/readiness/capabilities handshake. No installed RRD process, authentication, WebSocket, graph/index/DataFusion/context/reasoning flow, restart, denial, or resource boundary was exercised
cross-boundary command and result: deterministic inventory — 896 current/generated/planned records after intended-tree staging; documentation policy — 89 statuses/86 classified coordinates with parent indexes and local links intact; generated-surface parity — 33 HTTP operations/OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; CI workflow policy — 7 substantive jobs, 5 cohesive engine suites, 20 default-feature packages, and 5 optional-feature packages; frozen version policy — 1.0.0; workspace architecture — 16 passed; cargo check --workspace --all-targets --locked — passed. Focused Ruff lint/format, Python compilation, Cargo formatting, exact retired-identity search, diff integrity, and both worktree checks passed
failure/crash/differential evidence: no new engine failure/crash/differential evidence was created. The separate build's SurrealDB Wasm resolution, browser-externalized Node modules, oversized chunks, and mock-only handshake were retained as gaps. Before correction, inventory check failed without writing because the staged index still contained the deleted flat file; generation passed after exact package staging. A guessed nonexistent version script and guessed nonexistent Cargo package each exited before relevant verification; repository search identified and the exact commands passed. The first exact workspace-architecture run passed 15 tests and failed the retired-identity guard because three new audit records repeated the prohibited identifier; all occurrences were generalized without hiding the file/protocol role, and the complete 16-test rerun passed. One search included a nonexistent Connectome root Cargo.toml but still found the actual Tauri manifest; it was corrected. One exact restaging attempt also named the already staged deleted flat path and failed before changing the index; restaging the remaining explicit package paths passed. A staged diff-integrity pass then rejected one extra blank line at the new client-index EOF; that line was removed before the complete rerun. Earlier read-only calls included four rejected malformed working-directory/path strings and one nonexistent-path `wc`; none mutated either checkout
not run and reason: no installed RRFlow/Connectome cross-process corpus, remote HTTPS/mTLS, multiplexed WebSocket, endpoint rotation, renderer credential adversary, graph/timeline scale, accessibility, multi-instance isolation, rrflowMX/rrflowKV semantic differential, rrflowKV restart, native graph/scalar/BM25/vector transaction, streamed Arrow/DataFusion, persisted reasoning/context/feedback, installation/attunement, offline artifact, or release proof was run because the prerequisites remain open and this package changes documentation only. Full RRFlow workspace tests and full Connectome Tauri/desktop suites do not establish this KB-05 classification or H-06
remaining known errors: 1 KB-05 record remains; A-06/A-07 and POAM-001/011/013/014 remain incomplete; Connectome remains partial and non-conforming; RRFlow still lacks accepted native Arrow-page rrflowKV convergence, atomic graph/scalar/BM25/vector projections, streamed read-stamped Arrow/DataFusion execution, persisted reasoning/context feedback, qualified installation/attunement, generated cross-surface clients, and clean deployment proof
roadmap checkbox changed: no
```

##### `ci-operations`

```text
gate/package: A-06 / KB-05 / ci-operations
revision: parent dceb448; result is the commit containing this entry
baseline files/digests: .github/CODEOWNERS=fe0700b00106dc321ceaa2c07f49f0aa48553c19e065a424ae7779b8a0f2afc3; candidate caller=489b9e60da94adb6b6cf65348674013634181985b4820fd61fc5363813d9b54b; reusable candidate=eee1ed22ffdfb50acd0b32db8d040a8954c97dd91bfe13c46b3676d36651ba22; scheduled benchmark=3c15ab7c3c4a361b2ef44e8736cafe11668d378d490251a724da7e667089df81; ARC installer=6f991cc1e6dcb0d56346f3010115fc548e3fe8cad2e5ddc199c9b15518a2b005; workflow policy=e7b5806529436db5a5075e62db4315b3d3b4fe815e0c6ab5b64dd72a56dea82d; controller values=d49cd5cd58b78b0c6e02c82d907ec5d0157c75a55aaf850e054e17d4df385160; heavy values=218045e514813e242c959c71f5320c8bfeec7c1bee4d8cfb1ea8b7d852d94b44; standard values=0efca3997d1fa6d1e174c1329dbc35729d257319e22d45c2710edd39263036e2; root README=d40f306175c80287fb2cface16ce0c7ace890bdcb4997121be52398e6214462b; docs index=3e1e3d016c85a1303dd64ba953088b9e26ebb997bc0cc8d7999332a34a78af6; CI operations=024b9760c2fb726ec0d0bc8167e7c5f3db6c868563a4b937726976d612075ebb; storage index=69adb21d4c53238786361217c6a5a7d39dc7ec287a867e550597a93d1186ef5a; benchmark owner=91d17c1d5f2f5c1cde6891878f9f77d659449685a408336d36b3506bac5159e3; convergence research=e1b2edb7766b7977ebf658f3ee4d82e06a32fa4149dcdc6f109ce2885953bde4; POA&M=95e79fcd2b70635b3c97584ca655a77bbbd47650b08fbf2467cc7a0127e7b01f; roadmap=5a7c34f46b5946d06ee0eabe113b4c74d884ced559da81474fcc925e7da4d148; execution map=3d6d7fdb5d467b295d13ad77b9fe7434f641733001bef61c820e82fd5da83527; inventory generator=062e05f3e80746fe13abf32e3a246e1133eea3a56c2f5d9202c6f0643b384b23; generated file plan=a7215aff6b21418c6566f0e0139dd0ada0f7518b841b502349d06dd993d6ad41
files read in full: root README; docs knowledge portal; the 134-line CI operations record and its complete six-commit history; candidate caller and reusable workflow; scheduled benchmark workflow; workflow-policy checker; ARC installer and all three values files; CODEOWNERS; SDK conformance orchestrator; rrflowKV benchmark owner and storage index; retained benchmark-evidence test; knowledge exporter, exporter test, documentation checker, Rust knowledge contract/test/golden; the 486-line system-convergence research record; canonical A-06/KB-05 roadmap and evidence sections; execution-map KB-02 through KB-05, queue, resolved-review, journal, repository-run, and stop-condition sections; POAM-001 and every CI/benchmark/release gap reached from these owners. Every changed complete documentation body, script, workflow, and index was reread after authoring
external primary references reviewed: GitHub's secure-use, workflow-syntax, repository Actions-policy, and self-hosted-runner/ARC guidance confirm least privilege, full-length action SHA pinning, protected workflow review, fork isolation, and ephemeral autoscaling boundaries. The official actions/upload-artifact v4.6.2 tag was independently resolved to ea165f8d65b6e75b540449e92b4886f43607fa02; existing candidate action revisions were reused rather than guessed. No external workflow, runner, database, or lifecycle authority was adopted
files changed/created/deleted/moved: create docs/operations/README.md; update docs/operations/ci.md, root/docs portals, storage benchmark owner/index, system-convergence sequencing text, canonical A-06/KB-05 roadmap evidence, POAM-001, this classification/queue/resolved-review/journal map, workflow CODEOWNERS, candidate topology display name, scheduled diagnostic workflow, ARC installer, repository workflow policy, and generated file inventory; no file is deleted and no RRFlow engine, storage, query, graph, scalar/BM25/vector, Arrow/DataFusion, reasoning, install, attunement, public contract, endpoint, SDK, or adapter runtime behavior changed
contract or behavior changed: documentation ownership and repository CI behavior changed; RRFlow engine behavior did not. The operations record is now coordinated and indexed; all checked-in workflows are discovered and must use full-SHA actions, digest-pinned service images, read-only contents permission, no pull_request_target, and checkout credential non-persistence. The storage workflow is pinned but remains diagnostic. Workflow ownership covers the directory. The ARC installer resolves an explicit/current OWNER/REPOSITORY and rejects malformed or public targets instead of defaulting to an unrelated repository. Candidate topology naming no longer claims Connectome is started. A-06/KB-05 close only the deterministic checkout memory package
smallest test command and result: before editing, python3 scripts/ci/check_workflow.py passed while direct source inspection found eight floating action uses and two credential-persisting checkouts in the uninspected scheduled workflow. After editing, the policy reported 3 workflows, 7 substantive jobs, 5 cohesive suites, 20 default-feature packages, and 5 optional-feature packages; three in-memory negative probes rejected a floating action, missing checkout credential policy, and pull_request_target. bash -n accepted the installer; PyYAML parsed all three workflows; focused Ruff lint/format passed
owning package command and result: python3 scripts/knowledge/test_export.py — 11 passed; cargo test -p rrd-contract --test knowledge_contract --locked — 7 passed; cargo test -p rrd-contract --all-targets --locked — 58 passed; cargo clippy -p rrd-contract --all-targets --locked -- -D warnings — passed; cargo test -p rrd-store --test benchmark_evidence --locked — 1 retained historical-storage parser passed and produced no current benchmark result
cross-boundary command and result: the first complete intended-tree export at Git tree 9922b821cc32f8b70bc0c919891f67863a1fd400 produced byte-identical packages with 91 manifested sources, 89 coordinated records, two explicit exclusions (`README.md` and `SPEC.md`), package digest 72b43d077f16d10937b51d5bf0540b83b6d00cf688e5735f0a602a84731e11cf, and encoded-file SHA-256 13168e46beb9945752f50b0f060ce9aa54552d6a8a0dd10670453fcb036ff779. The complete verification pass then reported 90 document statuses, 88 classified coordinates, 896 current/generated/planned path records, 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715, 3 workflows, 7 substantive jobs, 5 cohesive suites, 20 default-feature packages, 5 optional-feature packages, frozen version 1.0.0, 16 workspace-architecture tests, and a passing workspace all-target check. Focused Ruff lint/format, Python compilation, workflow YAML parsing, shell syntax, Cargo formatting, and staged/unstaged diff integrity also passed. Because this journal is itself included in the content-addressed package, the final post-journal tree/package digest pair is carried in the resulting commit metadata and verified again after commit rather than copied recursively into the content it hashes
failure/crash/differential evidence: no RRFlow engine failure/crash/differential evidence was created. Full-file review exposed the unpoliced scheduled workflow, floating action tags, persisted checkout credentials, narrow CODEOWNERS entry, hardcoded repository fallback, false Connectome topology label, stale SurrealDB-artifact description, and stale A-06 sequencing text. A first formatter check exposed pre-existing Ruff drift across the now-owned policy file; the complete file was formatted and rechecked. Seven malformed read-only launchers and one recursive-cleanup launcher were rejected before execution and changed no state. The first exact cleanup call supplied two operands to this platform's one-file unlink and removed neither; two explicit unlink calls plus rmdir then removed the resolved temporary directory. The first final-export invocation used the rejected `.json` suffix; rerunning the same staged-tree revision with the required `.jsonl` suffix passed. A static-check launcher used unavailable `python3 -m ruff`, and its later standalone Cargo command could still run; the fail-fast rerun resolved the installed `ruff` executable, passed Ruff 0.16.4, and repeated every remaining static check successfully
not run and reason: no GitHub candidate or scheduled workflow was dispatched; no branch-protection/organization policy, ARC installation, runner identity, Kubernetes cluster, secret path, or chart download was mutated or externally verified. No storage benchmark, full workspace test suite, SDK/Connectome release conformance, rrflowMX/rrflowKV semantic differential, rrflowKV crash matrix, native graph/scalar/BM25/vector transaction, streamed Arrow/DataFusion, persisted reasoning/context/feedback, project installation/attunement, offline distribution, or release qualification was run because this A-06 package owns documentation packaging and CI policy, not those later runtime/evidence gates
remaining known errors: A-07 and POAM-001/012/013/014 remain; external GitHub/runner state is unproved; the current SDK and topology jobs remain characterization only; RRFlow still lacks accepted Arrow-page rrflowKV convergence, atomic graph/scalar/BM25/vector projections, streamed read-stamped Arrow/DataFusion execution, persistent reasoning/context/feedback, qualified installation/attunement, generated cross-surface clients, and clean deployment proof
roadmap checkbox changed: yes — A-06 and KB-05 only
```

### A-07.0 — implementation traceability before structural edits

Precondition: KB-05 is complete, which closes the checkout-authoring portion
of A-06. The later KB-06 through KB-08 persistence milestones are deliberately
not prerequisites for source convergence. Re-run the generated file inventory,
read every current package manifest and module root, and refresh the
[implementation-requirements traceability](#implementation-requirements-traceability)
against the exact starting revision. For each capability family, record:

- current source modules and public symbols;
- characterization tests, fixtures, examples, and benchmarks;
- behavior and invariants that the accepted architecture requires;
- one canonical destination and owning roadmap gate;
- the equal-or-stronger replacement evidence; and
- every conflicting file or symbol removed after that evidence passes.

This is a documentation-only work package. It changes no runtime behavior and
does not mark A-07 complete. A-07.1 cannot begin with an unaccounted source,
test, fixture, or behavior.

#### A-07.0 evidence journal

```text
gate/package: A-07 / A-07.0 / implementation-requirements traceability
revision: parent b5ec39162771ced38a83dd512dda5b0720aef714; result is the commit containing this entry
baseline files/digests: starting tree=3c8d11fe697cd88645179f15441a7485d934f240; Cargo.lock=bafc36ae835b2b7af47fd560140f721f7282a9d1bee0d18c28c802025538cd7b; root README=a86b158bf16abd21ec9a2e867442ed601c1038fa9d2de288063e48a211a8e9bc; canonical roadmap=df4fe77a7ae0f88bd054e6a0a376df7b35f14102fc2ddc860eff3b0f9576727b; execution map=677b3a1cf67be55666e899c72b2812a4b352e3ea66c781bab05e16d802cedcbc; POA&M=8ad40fdfe5878c6dc0fabd2dd8b9bb29ccacce13ab8a0c96df2484c2ebc23b6d; generated file plan=cd6dfa2b0f45bac85f58ccfdf1b8cef5d0184c430fd16d4e098c3e09a028b269; ordered Rust-manifest hash ledger=0f0a3fea21ad278fed66472a97c89d24490b7625625f6fd3ea85f62f2ee29b86; ordered module-root hash ledger=bd6b041ce9ac9a4da64883937670b44b3e0645ad8be8d770fa286bcdaa094a88; ordered non-workspace-manifest hash ledger=2e2b04c1cffab3516869c1738a3562d4553567b70eed3900ea9d46cc44a098f0
files read in full: AGENTS.md; root README; canonical roadmap; complete 60-physical-line POA&M; A-07.0 authority, complete implementation-requirements traceability and direct-convergence tables, package-journal requirements, and relevant A-07.1/A-07.2 execution-map sections; root Cargo.toml, all 20 workspace Cargo.toml files, Cargo.lock, and the non-workspace Rust fixture manifest; every one of the 32 Cargo library/binary module roots; all five generated SDK package/solution/project manifests and their TypeScript, Python, Go, Java, and .NET public source roots; both evaluation fixture manifests; workspace architecture test; and all 1,684 lines of the deterministic inventory generator. Every changed documentation body/section was reread after authoring; the regenerated inventory was parsed and independently checked in full
files changed/created/deleted/moved: update root README, canonical roadmap, POA&M-014, this execution map, and the generated file plan; no file is created, deleted, or moved; no Rust, SDK, fixture, protocol, persisted format, endpoint, configuration, install template, or runtime source changes
contract or behavior changed: documentation traceability and the current next-package pointer changed; product/runtime behavior did not. The exact 20-package dependency/public-root map and five generated-SDK roots now join the existing capability and direct-convergence maps. Existing code remains rough-draft inventory: RrdEngine is the target sole authority, while direct-store security, estate, maintenance, and cluster paths, eager query/Arrow materialization, competing vector catalogues, startup installation, markers, SDK gaps, and other named conflicts remain scheduled for direct removal only after equal-or-stronger canonical evidence
smallest test command and result: python3 scripts/ci/build_execution_inventory.py --check — 896 records passed after deterministic regeneration; python3 scripts/ci/check_documentation.py — 90 document statuses, 88 classified coordinates, parent indexes, and local links passed
owning package command and result: cargo metadata --format-version=1 --locked --no-deps — resolved the same 20 workspace packages and repository-local first-party paths; cargo test -p rrd-engine --test workspace_architecture --locked — all 16 dependency, source, boundary, terminology, and tracked-file tests passed
cross-boundary command and result: python3 scripts/knowledge/test_export.py — 11 passed; generated-surface parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; frozen version policy remained 1.0.0; cargo fmt --all -- --check, git diff --check, and cargo check --workspace --all-targets --locked all passed
failure/crash/differential evidence: no new engine failure, crash, semantic differential, recall, or performance evidence was created. Full-source review proved direct StorageEngine dependencies in rrd-security, rrd-estate, and rrd-maintenance; direct rrd-lsm/rrd-store authority in rrd-cluster; eager row-to-Arrow/DataFusion paths; alternate vector catalogues; startup-created instance/security/process state; and incomplete SDK/runtime boundaries. Several malformed read-only launcher paths, including one post-review continuation and one parallel verification launcher, were rejected before process creation and changed no state; every intended read or check was rerun from the verified repository working directory
not run and reason: full workspace tests, optional-feature suites, SDK language/package/live conformance, Connectome, rrflowMX/rrflowKV semantic differential, rrflowKV crash/reopen and final-format rejection, native graph/scalar/BM25/vector atomicity, streamed Arrow/DataFusion resource/fault execution, persisted reasoning/context/evidence/feedback, installation/attunement, offline clean deployment, and competitive benchmarks were not run because this package changes documentation traceability only and none can qualify A-07.0
remaining known errors: A-07.1 and A-07.2 remain before A-07 can close; all C-through-J engine acceptance gaps and POAM-001 through POAM-024 remain open or sequenced as recorded. In particular, RRFlow does not yet have accepted one-port MX/KV semantics, final Arrow-compatible rrflowKV pages, atomic native graph/scalar/BM25/vector projections, bounded read-stamped DataFusion streaming, persisted reasoning/recall feedback, qualified install/attunement, complete SDK/Connectome parity, or clean-release proof
roadmap checkbox changed: no
```

### A-07.1 — freeze package and type vocabulary

Read all 20 `Cargo.toml` files, root `Cargo.toml`, `Cargo.lock`, every crate
`lib.rs`/`main.rs`, and `workspace_architecture.rs`. Produce the reviewed
dependency table in the A-07 commit before any physical move.

A-07.1 executes as bounded commits so each direct move can expose and repair
its own compiler, boundary, and conformance failures. A checked subpackage is
accepted evidence inside A-07.1; it does not check A-07 in the canonical
roadmap.

| Done | Subpackage | Exact boundary |
|---|---|---|
| [x] | A-07.1a | Governed-function catalogue/binding direct names, closed golden contract, `engine/function/` split, retired-source guard, and first rrflowMX/rrflowKV/reopen corpus. |
| [x] | A-07.1b | Rust SDK direct responsibility split and secret-safe public session boundary, preserving current characterized behavior. |
| [x] | A-07.1c | TypeScript SDK direct responsibility split and direct conformance-test filename convergence. |
| [ ] | A-07.1d | Python SDK direct responsibility split, removal of the catch-all model module, and typed-package marker. |
| [ ] | A-07.1e | Go SDK idiomatic responsibility split with one generated operation projection and no orchestration authority. |
| [ ] | A-07.1f | Java SDK responsibility split and package documentation without a second registry or engine. |
| [ ] | A-07.1g | .NET workspace policy and responsibility split with one generated operation projection. |
| [ ] | A-07.1h | Remaining case-insensitive package/type/path vocabulary, repository-closure, dependency-direction, generated-inventory, and all-target acceptance for the complete A-07.1 candidate. |

Inventory names case-insensitively across packages and crate namespaces,
binaries, modules, public types, CLI commands, protocol and MCP operations,
SDKs, configuration paths, environment variables, persisted markers, fixture
identities, digest/media domains, benchmarks, and documentation coordinates.
Classify every hit as the accepted direct name, a third-party interoperability
term, or superseded RRFlow residue with an owning deletion gate. No allowlist
may hide first-party residue; upstream dependency identifiers are recorded at
their adapter boundary rather than copied into RRFlow vocabulary.

Verify these direct names, with compiler errors allowed between edits but not
at the package commit:

- `rrd_store::Engine` -> `StorageEngine` without a forwarding re-export;
- `NativeEngine` -> `RrflowKvStore`;
- `RrflowMxEngine` -> `RrflowMxStore`;
- `EngineBox` -> `StorageProfile`; and
- `AutomationCatalogue` -> `FunctionCatalogue`,
  `ReplaceAutomationCatalogue` -> `ReplaceFunctionCatalogue`,
  `ListAutomationCatalogue` -> `ListFunctionCatalogue`, and private
  `AutomationHead` -> `FunctionCatalogueHead`;
- `FunctionTrigger` -> `TransactionFunctionBinding`,
  `FunctionTriggerMutation` -> `TransactionMutationKind`, and
  `FunctionTriggerEffect` -> `TransactionFunctionEffect`, including direct
  `trigger_id`/`triggers` to `binding_id`/`transaction_bindings` field changes;
  and
- any module/file name claiming an authority it does not own is moved directly,
  with no re-export or transitional alias.

`rrd-contract/src/lib.rs` (currently 6,333 lines), `rrd-core/src/runtime.rs`,
`rrd-store/src/rrflow_kv.rs`, and other monoliths are split only along already
accepted responsibilities. Splitting is mechanical first; behavioral changes
remain in their later gates.

For functions, retain `rrd-contract/src/function.rs` as the public contract
owner, create its closed `fixtures/function-contract-v1.json` and
`tests/function_contract.rs`, and directly replace
`rrd-engine/src/engine/automation.rs` with
`rrd-engine/src/engine/function/{mod,catalogue,execution,javascript,webassembly,transaction_binding}.rs`.
The split preserves characterized execution while isolating the private-control
storage and split-audit defects for C-03; it does not create event, trigger, or
routine behavior. `fixtures/rrd-function-conformance-v1.json` and
`rrd-engine/tests/function_conformance.rs` become the one engine/profile/surface
corpus extended by C/H/J. No `automation` module, old field decoder, forwarding
method, or type re-export survives the A-07 commit.

For the Rust SDK, A-07.1b directly replaced the 1,235-line
`rrd-client/src/lib.rs` implementation body with
`rrd-client/src/{client,endpoint,error,operation,retry,session,subscription,transport}.rs`
and retained `lib.rs` only as the narrow public export root. Current typed
operations and real-server behavior remain characterized; the public session
boundary no longer exposes or formats the bearer-bearing lease. Reserve the
planned `operation_coverage`, `protocol_validation`, and `transport_faults`
test seams for the gates that implement their real behavior rather than adding
empty success surfaces. B-04 owns multiplexing/cancellation, H-04 owns complete
catalogue binding and protocol validation, and H-07 owns resolution and
rotation; A-07 does not conceal those gaps behind forwarding methods or a new
client authority.

For the TypeScript SDK, A-07.1c directly replaced the runtime body in
`sdks/typescript/src/index.ts` with the public export root and moved every
existing responsibility into
`src/{client,endpoint,error,operation,retry,session,transport}.ts`. No
`subscription.ts` was created because the audited package has no WebSocket
behavior to move; B-04 creates that module only with its real bounded state
machine and carriage tests. Generated `validators.ts` and the planned
operation-coverage, protocol-validation, transport-fault, package-consumer,
and browser-conformance tests remain reserved for their assigned B/H/J gates.
`tests/sdk_conformance.ts` was directly renamed to `tests/sdk-conformance.ts`,
and its sole orchestrator caller was updated in the same commit. Current
generation, typing, loopback, envelope construction, byte-limit, abort, and
fail-closed harness behavior remain characterized. A-07 does not claim the
H-04 validation/retry/package behavior, B-04 socket, H-07 network transport,
or J qualification complete and leaves no old implementation or forwarding
fallback.

For the Python SDK, split the current `sdks/python/src/rrd_client/client.py`
responsibilities directly across
`{client,endpoint,error,operation,retry,session,transport}.py`, retain
`__init__.py` only as the narrow public export root, and remove the old
catch-all `models.py` after every type has one destination. Preserve current
generation, loopback and redirect denial, envelope construction, byte limit,
context-manager closure, fail-closed harness input, and mock behavior
mechanically. Add `py.typed` during that package. Reserve
`async_client.py`, `subscription.py`, generated `models.py`, and the planned
operation, protocol, transport, async, and package-consumer tests for their
assigned B/H/J gates rather than creating empty success surfaces. A-07 does
not claim runtime validation, async/WebSocket behavior, remote transport,
semantic retry/cancellation, installed-engine conformance, or release package
qualification and leaves no forwarding model/client module.

For the Go SDK, keep one idiomatic `rrd` package and split the current
`sdks/go/{client,models}.go` responsibilities directly across
`{doc,config,client,endpoint,errors,operation,retry,session,transport}.go`.
Retain `endpoints_gen.go` as the catalogue-derived projection and keep the two
commands as generator and conformance entrypoints, not runtime services.
Preserve context-first calls, generation, loopback and redirect denial,
envelope construction, identical retry bytes, response byte limits,
fail-closed harness input, and current `client_test.go` unit/race behavior
mechanically. Remove the catch-all `models.go` after every type has one
destination and retain `client.go` only as the narrow facade. Reserve the
planned operation/protocol/transport tests, `subscription.go`, `models_gen.go`,
subscription/package-consumer tests, concrete operation validation, remote
transport, server cancellation, and module/release qualification for their
assigned B/H/J gates. No Go orchestration, storage, installation, attunement,
or lifecycle authority and no forwarding API survives this package.

For the Java SDK, keep one Maven artifact under the `io.rrflow.rrd` client
namespace and split the current `RrdClient.java` responsibilities directly
across `ClientConfig`, `EndpointResolver`, `HttpTransport`,
`OperationBinding`, `OperationExecutor`, `ProtocolCodec`, and `RetryPolicy`.
Add `package-info.java`; retain `OperationId.java` as the catalogue-derived
projection and the existing small public files at their one responsibility.
Preserve generation, loopback and redirect denial, canonical envelope and
resource construction, mutation idempotency, identical retry bytes, response
byte limits, interrupt restoration, explicit manifest-absent skip, and current
`RrdClientTest` characterization mechanically. Reserve `RrdCall`,
`Subscription`, `WebSocketTransport`, generated operation models, the planned
operation/protocol/transport/subscription/concurrency/package-consumer tests,
remote transport, server cancellation, credential redesign, and
artifact/release qualification for their assigned B/H/J gates. A-07 neither
claims those behaviors nor leaves a catch-all duplicate implementation,
forwarding class, Java-side engine, or second operation registry.

For the .NET SDK, keep one `Rrflow.Rrd.Client` NuGet artifact and
`Rrflow.Rrd` client namespace. Add a pinned `global.json` plus shared
`Directory.Build.props` and `Directory.Packages.props`; move the narrow README
to the .NET workspace root and `OperationId.g.cs` under `Generated`; split the
current `Models.cs` and `RrdClient.cs` responsibilities directly across
`ClientOptions`, `EndpointResolver`, `HttpTransport`, `OperationBinding`,
`OperationExecutor`, `ProtocolCodec`, `RequestOptions`, `ResourcePath`,
`RetryPolicy`, and `Session`. Retain `RrdClient` only as the narrow immutable
public facade. Preserve generation, loopback and redirect denial, canonical
envelope/resource construction, mutation idempotency, identical retry bytes,
absolute-deadline calculation, response byte limits, disposal, explicit
manifest-absent behavior, and current `RrdClientTests` characterization
mechanically. Reserve `RrdCall`, `Subscription`, `WebSocketTransport`,
generated operation models/JSON metadata, the planned operation/protocol/
transport/subscription/concurrency/package-consumer tests, remote transport,
server cancellation, credential redesign, deterministic package builder, and
artifact/release qualification for their assigned B/H/J gates. A-07 neither
claims those behaviors nor leaves a catch-all duplicate, forwarding type,
.NET-side engine, nested README, or second operation registry.

Acceptance commands:

```text
cargo metadata --format-version=1 --locked
cargo test -p rrd-contract --test function_contract --locked
cargo test -p rrd-engine --test function_conformance --locked
cargo test -p rrd-engine --test workspace_architecture --locked
cargo check --workspace --all-targets --locked
rg -n 'NativeEngine|RrflowMxEngine|EngineBox|rrd_store::Engine|AutomationCatalogue|FunctionTrigger|automation_catalogue' crates
```

Extend `workspace_architecture.rs` in this same package to inspect Cargo
metadata plus tracked paths. It rejects a local dependency or target outside
the workspace root, a tracked escaping symlink, a submodule, a Git dependency
used as first-party implementation, and host-specific absolute build/install
inputs. Registry dependencies remain permitted only when locked. The final
symbol search must be empty. A-07 stops if any outward adapter imports a
physical crate, any RRFlow code is supplied by another checkout, or the
mechanical split silently changes public schema.

#### A-07.1a evidence journal

```text
gate/package: A-07 / A-07.1a / governed-function catalogue and transaction-binding vocabulary
revision: parent 0dfbab45a2216b361c760c49006cdb1e9e01387b, tree ff4f0dd84680aef32bdd40a4fb66c5ad4b83e84a; result is the commit containing this entry
baseline files/digests: rrd-contract/function.rs=cca64b3eb22fd039c3ead75f6cd2aef836198cbb8ba0549cf97f5cf68223bf1c; rrd-engine/automation.rs=e587ad19d39a0e203f187397b3c9d47c5fbadefa9780fd5c056054ba20754f08; engine automation test=c8d4f3a0247c8689d62adfd115420e23e6ddfa59b14672a652ef283e255a6e8f; resulting rrd-contract/function.rs=41f3a606ecc2adb725e06bb1d534069b96922358f76dff7f66732be16fb0be99; function-contract fixture=4a27230c3f7b6c45929f2a27f2ac2cd4d1d147cb4ee04151e6cd0470cfbe2e2b; profile fixture=21995d4377799247724caa74ca99f34f2db650cd5dc2fdca1f009f744df572e2
files read in full: AGENTS.md; root README; governed-function reference; engine-data-flow owner; canonical roadmap; complete 60-physical-line POA&M; A-07.0/A-07.1 authority, implementation-traceability rows, mandatory convergence rows, evidence template, and stop conditions in the execution map whose complete file had been reviewed in the immediately preceding unchanged traceability package; complete function contract and root exports; complete prior engine automation implementation and test before the split; complete engine module root, transaction, error, capabilities, and affected test root; all six resulting engine/function modules; complete golden contract test and fixture; complete shared profile conformance test and fixture; and the complete relevant workspace-architecture guard. Every changed source and documentation section was reread after formatting
files changed/created/deleted/moved: directly rename the public function catalogue and transaction-binding types/fields plus engine methods/errors/commit-intent coordinate; replace the private key prefix with function-catalogue and provide no former-key reader; delete engine/automation.rs and create engine/function/{mod,catalogue,execution,javascript,webassembly,transaction_binding}.rs; rename the focused engine test module; add the closed contract fixture/test and shared rrflowMX/rrflowKV function fixture/test; extend the workspace source guard; update capability text, the deterministic inventory generator/file plan, README current status, governed-function reference, engine-flow current boundary, POAM-008, canonical roadmap evidence, and this execution map
contract or behavior changed: the function catalogue and pre-commit binding schema now has one direct canonical spelling and rejects former binding field shapes; private persisted catalogue keys use only function-catalogue and intentionally do not read the former prefix. Characterized JavaScript/WebAssembly execution and transaction-binding behavior is mechanically preserved under a function-only module. No engine event, post-commit trigger, routine, skill, install, adapter, endpoint, SDK operation, Arrow/DataFusion operator, graph/index behavior, or new authority was added
smallest test command and result: cargo test -p rrd-contract --test function_contract --locked — 2 passed; cargo test -p rrd-engine --lib engine::tests::function --locked — 4 passed; cargo test -p rrd-engine --test function_conformance --locked — 1 passed; cargo test -p rrd-engine --test workspace_architecture --locked — 17 passed
owning package command and result: cargo test -p rrd-contract --all-targets --locked — 60 tests passed across the package; cargo test -p rrd-engine --all-targets --locked — 110 tests passed across unit and integration targets; cargo clippy -p rrd-contract -p rrd-engine --all-targets --locked -- -D warnings — passed
cross-boundary command and result: the shared fixture installed and read one catalogue, executed one JavaScript function, and committed one transaction-bound assertion through both rrflowMX and rrflowKV with equal catalogue/output/runtime-commit/cursor/claim evidence; rrflowKV then closed/reopened and returned the same catalogue and invocation output/digest. Documentation policy passed with 90 statuses and 88 classified coordinates; deterministic inventory passed with 895 current/generated/planned records; all 11 knowledge-export tests passed; generated parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; version policy remained 1.0.0; locked Cargo metadata resolved; focused Ruff passed; and cargo check --workspace --all-targets --locked passed
failure/crash/differential evidence: the first golden run failed only on the deliberately unset expected catalogue digest and supplied f4652b6819ee73e2e1e2480c43a63c7fc148eb7d5cf04579d2338fcd830a7c36; setting that exact reviewed value made both golden tests pass. The first architecture run rejected the two new untracked test targets and found retired spellings embedded literally in the negative-test source; exact staging and runtime construction of rejected field names repaired both, after which all 17 checks passed. The shared profile corpus proves selected semantic equality and ordinary close/reopen only; it is not the C-03 crash/effect-gap/runtime-upgrade/resource matrix
not run and reason: the complete workspace test suite, optional-feature suites, generated SDK conformance, Connectome, installation/attunement, full JavaScript/Wasm supported-target/runtime-build corpus, function prepare/WAL/commit/acknowledgement fault matrix, ENOSPC/size/resource matrix, public-surface conformance, Arrow/DataFusion streaming, native graph/scalar/BM25/vector atomicity, persisted reasoning/feedback, clean offline deployment, and benchmarks were not run because this bounded package changes only the governed-function vocabulary/module seam and its first characterization fixture. Those later gates cannot be qualified by expanding this test batch
remaining known errors: A-07.1b through A-07.1h and A-07.2 remain before A-07 can close. Governed functions still use inline source/module bytes, a private monolithic JSON catalogue, direct StorageEngine access, split pre-domain allowed audits, runtime-build-unbound recovery, incomplete deterministic profiles, and no schema/artifact/prepared-receipt/install/cross-language/public-surface proof. POAM-008 remains sequenced; all rrflowKV hybrid pages, native graph/index atomicity, streamed stamped Arrow/DataFusion, persistent reasoning/recall, installation/attunement, SDK/Connectome, and release evidence remain open
roadmap checkbox changed: no; only supporting A-07.1a subpackage status changed
```

#### A-07.1b evidence journal

```text
gate/package: A-07 / A-07.1b / Rust SDK responsibility and public-session boundary
revision: parent 2bed9d9b2fee5fef8c317cd18d6ad454e40dc68e, tree 1a44881ce80e3e751a67c363b443155a529fbef4; result is the commit containing this entry
baseline files/digests: former rrd-client/src/lib.rs=8261bb3bd65554a9b1437a85709b90acc943ba2c4e6ba8d987cd5b7d43fd43ff; resulting ordered lib/client/endpoint/error/operation/retry/session/subscription/transport SHA-256 ledger=207ea2ddd921e1c619be50fac75a933ac8ffdb6a40befa0762c81c5be4f41dd3
files read in full: AGENTS.md; root README; canonical roadmap, complete POA&M, Rust SDK reference, and relevant complete A-07.0/A-07.1 package map, behavior trace, mandatory-convergence, execution, evidence, and stop-condition sections; rrd-client manifest; the complete former 1,235-line lib.rs before the split; all resulting lib/client/endpoint/error/operation/retry/session/subscription/transport source files; complete real_server and sdk_conformance tests; and the complete sdk_conformance_server example. Every changed source file and documentation section was reread after formatting
files changed/created/deleted/moved: replace the rrd-client implementation body in lib.rs with a 22-line policy/export root; move client identity/discovery, endpoint construction, closed error handling, typed operations/envelope outcomes, retry configuration, session lifecycle, durable-subscription socket behavior, and HTTP carriage directly into their eight named modules; make Session fields private, add non-secret metadata accessors, and add a manual redacted Debug plus focused unit test; update the Rust SDK reference, root status, canonical roadmap evidence, POAM-011, execution-map package/behavior/convergence rows, A-07.1 subpackage status, and deterministic file plan
contract or behavior changed: the public Session no longer exposes its principal or bearer-bearing SessionLease fields and no longer derives a formatter capable of printing the bearer. Six read-only accessors expose only principal/session identity, lease timestamps, and limits; transport alone can access the token. Every former public method remains and all existing typed-operation, HTTP, mutual-TLS, WSS, ACK, reconnect, retry, and response-limit mechanics were otherwise moved mechanically. No operation, route, wire shape, storage profile, engine transaction, graph/index, Arrow/DataFusion, reasoning, install, provider, or lifecycle behavior was added
smallest test command and result: cargo test -p rrd-client --lib session::tests::session_debug_redacts_the_bearer_credential --locked — 1 passed and proves the synthetic bearer is absent from Session Debug output; the pre/post public-method inventory found all 41 former methods and only the six intended non-secret accessors as additions
owning package command and result: cargo test -p rrd-client --all-targets --locked — the one unit test and three real-server tests passed; the sdk_conformance target also reported one pass in 0.00 seconds because no manifest was supplied, so that invocation executed no scenario and is not accepted conformance evidence. cargo clippy -p rrd-client --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-client --test real_server --locked — 3 passed across real loopback HTTP, mutual-TLS server identity, WSS, and current durable-subscription behavior; cargo test -p rrd-engine --test workspace_architecture --locked — 17 passed, including the implementation-free client boundary and tracked-source guards; locked Cargo metadata resolved and cargo check --workspace --all-targets --locked passed. Documentation policy passed with 90 statuses and 88 classified coordinates; deterministic inventory passed with 895 current/generated/planned records; all 11 knowledge-export tests passed; generated parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; version policy remained 1.0.0; and focused Ruff passed
failure/crash/differential evidence: no implementation check failed and this mechanical/API-safety package creates no engine crash, storage-profile differential, recall, or performance evidence. The public-method comparison made the exact API delta visible: no former method disappeared and the six additions are non-secret Session metadata accessors. The existing manifest-absent conformance false success remains deliberately recorded as a defect rather than reclassified as evidence
not run and reason: a manifest-backed live SDK conformance run, complete 33-operation protocol corpus, transport fault injection, generated SDKs, Connectome, installation/attunement, rrflowMX/rrflowKV semantic comparison, rrflowKV crash/reopen, graph/scalar/BM25/vector atomicity, stamped Arrow/DataFusion streaming, persisted reasoning/context/feedback, clean offline deployment, and benchmarks were not run because this bounded package changes only Rust source responsibility and the public session-secret boundary. Those later B through J gates require real behavior before their reserved test seams can exist or qualify anything
remaining known errors: A-07.1c through A-07.1h and A-07.2 remain before A-07 can close. The Rust client still binds only 28 of 33 HTTP operations; its bearer is an ordinary CorrelationId internally; request/success validation, exact status/media enforcement, semantic retry/uncertainty, correlated server cancellation, bounded multiplexed frames, W3C propagation, authenticated endpoint rotation, fail-closed conformance configuration, D-01 installation, and complete MX/KV cross-surface proof remain open under B/D/H/J and POAM-011. All rrflowKV hybrid pages, native graph/index atomicity, stamped Arrow/DataFusion, persistent reasoning/recall, installation/attunement, Connectome, and release evidence remain open
roadmap checkbox changed: no; only supporting A-07.1b subpackage status changed
```

#### A-07.1c evidence journal

```text
gate/package: A-07 / A-07.1c / TypeScript SDK responsibility and conformance-entry boundary
revision: parent e74261ad8e6720b83d4154ad6784f0db5c1559e7, tree 28eeadb128a152df5766f967ba2b0e4b36130ee0; result is the commit containing this entry
baseline files/digests: former sdks/typescript/src/index.ts=6ecc0ede843c3d1347d815d45d5acd201392ef0bb4eecc964ff890633fc8e3fd; tests/sdk_conformance.ts=b5c5a54401a9041814c6f70c26bf7787372ca297f4a6bd486a30bbb4d21d72c0; conformance runner=5870f73b9bad92c14befd94b9bbab4f396f8ddd664c56cec0ce7e3a64c0b1a39; inventory generator=420ad223cc0c2d2f38594c892ef2908970c7b0fe5550e6e15a395001772e2214; resulting ordered index/client/endpoint/error/operation/retry/session/transport SHA-256 ledger=d494c084ce3df388840fd22bddaca0ac27403dec17162bc66b89f98f3c018d7e; renamed conformance entry retains b5c5a54401a9041814c6f70c26bf7787372ca297f4a6bd486a30bbb4d21d72c0; resulting runner=18f63809fc675c8e3fec9f8a15a9d95d4d74793bb1a2019a5f3c9a95bea267fe; resulting inventory generator=b1990a9ecc2f15227e3e66ef961182bf466e8fa3768cccfc78752e876aff9dca
files read in full: AGENTS.md; root README; canonical roadmap, complete POA&M, TypeScript SDK reference, and relevant complete A-07.0/A-07.1 package map, behavior trace, mandatory-convergence, execution, evidence, and stop-condition sections; TypeScript package, Biome, workspace, and TypeScript configuration; complete 692-line pnpm lock; generator; complete generated endpoint map; complete former 394-line runtime; complete mock and conformance tests before/after rename; and complete 190-line shared conformance orchestrator. The 24,523-line generated OpenAPI projection was verified byte-for-byte through the generator/checker, its first/last/export topology and all 33 descriptor bindings were inspected, and its unchanged digest was recorded rather than treating generated output as authored prose. Every changed source file and documentation section was reread after formatting
files changed/created/deleted/moved: replace the implementation body in src/index.ts with a 15-line public export root; move client construction/discovery/dispatch, loopback endpoint validation/configuration, current error classes, generated operation typing/request-coordinate validation, broad retry/deadline helpers, the plain session shape, and bounded Fetch/envelope decoding directly into client/endpoint/error/operation/retry/session/transport.ts; directly rename tests/sdk_conformance.ts to tests/sdk-conformance.ts without changing its bytes and update the sole orchestrator caller; keep subscription.ts and future validator/test files absent until their real B/H/J behavior exists; remove A-07 from the inventory generator's planned subscription gate because no A-07 behavior owns that file; update the TypeScript SDK reference, root status, canonical roadmap evidence, POAM-011, execution-map package/behavior/convergence rows, A-07.1 subpackage status, and deterministic file plan
contract or behavior changed: no public operation or runtime semantic was intentionally changed. All twelve former root exports, the RrdClient constructor, its six public fields, and its five public methods remain; generated all-33 HTTP typing, request bytes, loopback restriction, partial ArkType envelope validation, response byte cap, broad immediate retry, local AbortSignal behavior, plain enumerable credentials, and fail-closed conformance input were moved mechanically. The conformance filename is the only execution-entry change. No WebSocket/subscription module, runtime validator, HTTPS/mesh resolver, opaque credential, package artifact, engine transaction, storage profile, graph/index, Arrow/DataFusion, reasoning, install, provider, or lifecycle behavior was added
smallest test command and result: TypeScript compiler API inspection of src/index.ts — exact expected 12 public exports and 11 RrdClient properties/methods passed; pnpm --dir sdks/typescript test — 4 mock-focused tests passed
owning package command and result: pnpm --dir sdks/typescript check — generated schema/endpoint bytes matched, Biome checked 14 files without fixes, strict no-emit type checking passed, and all 4 tests passed under Node 24.16.0 and pnpm 11.22.0
cross-boundary command and result: python3 scripts/ci/run_sdk_conformance.py — the hermetic example harness executed the shared corpus through Rust, TypeScript at its renamed entry, Python, Go, Java, and .NET; every language reported corpus SHA-256 b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2 and the runner completed successfully. Direct TypeScript execution without RRD_SDK_CONFORMANCE_MANIFEST exited 1 at the required-manifest assertion. This is real daemon/client characterization over a direct-seeded rrflowKV fixture, not D-01 installation or full semantic coverage. Workspace architecture passed 17 tests; the locked workspace all-target check passed; documentation policy passed with 90 statuses and 88 classified coordinates; deterministic inventory passed with 894 current/generated/planned records; all 11 knowledge-export tests passed; generated parity remained 33 HTTP operations at OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; version policy remained 1.0.0; focused Ruff, Cargo formatting, and diff integrity passed
failure/crash/differential evidence: the first post-split typecheck passed; the first Biome run rejected two import-order violations, which were corrected directly before the complete package check passed. The first inventory generation stopped at the deleted underscored test path because the direct rename/new files were not yet staged; staging the exact candidate topology made the tracked-path inventory resolve the hyphenated file, after which it passed. A proposed one-language harness launcher was rejected before process creation because its temporary cleanup used a prohibited removal pattern, so the reviewed repository orchestrator was used instead. This structural package creates no engine crash, MX/KV differential, recall, or performance evidence
not run and reason: operation-specific runtime validation, browser import/CORS/credential behavior, HTTPS/mTLS/mesh resolution, multiplexed WebSocket, correlated server cancellation, transport/resource fault injection, built ESM/declaration consumer, deterministic/offline package closure, D-01 installation, structural all-operation conformance, rrflowMX/rrflowKV semantic comparison, rrflowKV crash/reopen, graph/scalar/BM25/vector atomicity, stamped Arrow/DataFusion streaming, persisted reasoning/context/feedback, Connectome, and benchmarks were not run because this bounded package only moves current TypeScript responsibilities and its conformance filename. Those later B through J behaviors must exist before their reserved tests can qualify anything
remaining known errors: A-07.1d through A-07.1h and A-07.2 remain before A-07 can close. TypeScript still has erased request/success payload types at runtime, permissive success status/media/error handling, plain serializable API-key/session credentials, broad immediate replay, local-only cancellation, implicit redirect following, loopback-only HTTP, no subscription/WebSocket/W3C/remote resolver, a raw private source package, and a direct-seeded label-overstated corpus. POAM-011 remains sequenced; all rrflowKV hybrid pages, native graph/index atomicity, stamped Arrow/DataFusion, persistent reasoning/recall, installation/attunement, Connectome, and release evidence remain open
roadmap checkbox changed: no; only supporting A-07.1c subpackage status changed
```

### A-07.2 — freeze causal evidence vocabulary

Read `rrd-core/src/trace.rs`, `rrd-engine/src/runtime/trace.rs`, every current
trace caller/test named in the traceability matrix, and the HTTP/WebSocket
ingress before changing trace behavior. Record one reviewed map containing:

- low-cardinality `rrflow.<boundary>.<operation>` names for `ingress`,
  `engine`, `kv`, `ql`, `graph`, `lexical`, `vector`, `datafusion`,
  `inference`, `attunement`, `routine`, `adapter`, and `delivery`;
- the canonical typed links and bounded attributes for request correlation,
  actor/scope, authorization, `ReadStamp`, plan, projection, reasoning cursor,
  source, commit, resource, error/denial, and output evidence;
- W3C `traceparent`/`tracestate` extraction, validation, child propagation,
  asynchronous causal-link, and invalid-context behavior; and
- the exact mapping from durable `RuntimeTraceEvent` evidence to Rust
  `tracing` and optional OpenTelemetry export, including redaction and sampling
  rules that can never advance authoritative state.

This package freezes vocabulary and assigns instrumentation to its owning
C-through-I work package; it does not add a second trace store or claim H-05.
Mixed current names such as `vector.search`, `embedding.run`,
`cluster.artifact_transfer`, and `rrflow_kv.concurrent-observation` remain
direct-convergence inventory until their owning behavior changes. H-05 later
proves the complete cross-surface chain.

## Gate B work packages

### B-03 — model-manifest handshake

Create `rrd-contract/src/model_manifest.rs`; extend
`rrd-contract/src/router.rs`, its export fixture, and contract tests; add the
runtime validator beside the new `RouterBackend` in `rrd-inference/src/router.rs`.
The manifest binds model bytes, tokenizer bytes, routing-schema digest,
decision capabilities, context/output/token limits, runtime ABI, device class,
quantization, and deterministic grammar revision. Validation happens before
model load/inference. Test each mismatched digest/limit independently.

### B-04 — multiplex WebSocket protocol

Create `rrd-contract/src/websocket.rs` and one golden fixture before changing
`rrd-server/src/http/websocket.rs`, `rrd-client/src/subscription.rs`, or
subscription code. One frame envelope owns request, response, cancellation,
subscription, ACK, heartbeat, error, and backpressure coordinates. Configure
client and server message/frame/read/write limits before allocation; validate
protocol, request/subscription identity, generation, sequence, cursor, and
bounded errors on every frame; expose receive deadlines and a correlated
terminal cancellation result. Delete the dedicated socket frame enums after
server/client round trips and malicious-peer/resource tests pass. Do not add
transport state to `RrdEngine`.

### B-05 — GraphQL lowering only

Create `rrd-contract/fixtures/graphql-equivalence-v1.json`,
`rrd-query/src/graphql.rs`, and `rrd-query/tests/graphql_equivalence.rs`.
Derive the GraphQL schema from the frozen RRD schema catalogue/contract, parse
and validate into the same `Query` or transaction-program AST, then call the
same bind/plan/authorization path. Fixtures pair GraphQL with rrflowQL and
compare bound request digest, fields, scope, stamp semantics, and denial. No
GraphQL resolver may call storage. The outward HTTP adapter is deliberately
deferred to H-04, which adds `rrd-server/src/http/handlers/graphql.rs` only
after lowering equivalence is established.

## Gate C work packages

### C-01 — one ordered binary key codec

Files: `rrd-core/src/key.rs`, `rrd-store/src/keyspaces.rs`,
`rrd-store/src/rrflow_kv.rs`, new `rrd-store/src/key_codec.rs`, their unit tests,
and a new frozen hex fixture.

Tuple fields are length/type encoded and escape-safe. The prefix is
`format / tenant / scope / family`; families cover current, temporal,
outgoing edge, incoming edge, scalar, unique, term dictionary/stat/posting,
vector, projection delta, catalogue, runtime commit, outbox, and audit. Version
ordering is explicit and tested. The catalogue family includes separate
function artifact, definition, transaction binding, membership/head, and
invocation-receipt subfamilies; it never makes executable bytes a public key or
repeats them in a head record. `prefix_end` has property tests over all byte
values. Conceptual `*`, `~`, and `+` family notation is documentation only,
never a raw delimiter contract.

C-01 freezes the codec and switches fresh rrflowKV writes/reads to it in one
reviewed batch. Existing pre-1.0 readers are removed in C-05; there is no dual
write and no new data is emitted in an earlier codec.

### C-02 — transaction port and MX/KV conformance

Create `rrd-lsm/src/transaction.rs` and `rrd-store/src/transaction.rs`; reduce
the renamed `rrd-store/src/engine.rs::StorageEngine` contract to point,
bounded range, put, delete, commit, rollback, and conflict semantics.
Implement a transaction write set, read snapshot, point/range reads including
read-your-writes, rollback, and commit conflict detection in rrflowMX and
rrflowKV.

The same corpus runs against both profiles: repeatable read, phantom/range
policy, write-write conflict, delete/update conflict, idempotent retry,
rollback, read-only commit, and bounded range ordering. The accepted alpha
contract is snapshot isolation with write conflict detection. Explicitly test
write skew so the code does not accidentally claim strict serializability.

### C-03 — one semantic mutation batch

Refactor `RrflowKvCommitPlan` in `rrd-store/src/rrflow_kv.rs` into repository
plans under `rrd-store/src/access/`. The engine validates the plan; the store
encodes it; one `StorageEngine` commit writes all current/temporal data,
both adjacency directions, synchronous indexes, runtime entry, projection
deltas, outbox, audit, and cursor. Mirror semantics in rrflowMX.

For governed functions, add typed function artifact, definition, transaction
binding, catalogue-membership/head, and prepared invocation-receipt families.
The catalogue contains references and digests rather than inline repeated
source/base64 bytes. Replace the private `server/state/*/automation/*` control
records and direct `StorageEngine` calls. A transaction binding commits its
validated prepared receipt and proposal with the domain/index/event/outbox and
allowed audit effects; recovery reuses the receipt or known commit outcome and
cannot execute against a different runtime build. Add the function cases to
`rrd-store/tests/semantic_commit_atomicity.rs` and
`rrd-engine/tests/function_conformance.rs`, including the current one-MiB
control-value contradiction as a positive final-size and negative over-limit
boundary.

Failure injection must cover before WAL append, after append/before sync,
after sync/before manifest visibility, and reopen. Any partially visible family
fails the gate.

### C-04 — direct current and temporal reads

Move record, relation, vector, and runtime access to `rrd-store/src/access/`.
Replace `runtime_read_changes(..., 0, ...)` in normal query/vector paths with
direct point/prefix/version reads. Retain log scans only for explicit replay,
changefeed, archive, recovery verification, and bounded diagnostic operations.
Add physical counters proving the selected key ranges and decoded values.

### C-05 — remove alternate pre-release execution

The Fjall dependency, `Store`, backend selector, migration command/API, and
format-upgrade executor must remain absent. Remove all earlier batch, segment, and
manifest read branches still present in `rrd-lsm`. Unknown or pre-1.0 physical
bytes fail with one explicit unsupported-format error. Fresh-database,
corrupt-format, and rrflowKV close/reopen tests replace alternate-path success
tests.

### C-06 — hybrid immutable segment format

Split `rrd-lsm/src/segment.rs` into the planned `segment/` modules. A manifest
pins one segment generation containing:

- ordered key/version spine with tombstone/value/page references;
- per-column page descriptors with logical/physical type, encoding,
  compression, row interval, offsets, checksum, and statistics;
- aligned immutable buffers where practical; and
- schema/codec/page-format digests.

The WAL and mutable memtable remain key/version optimized. Flush transforms a
sorted immutable memtable into the hybrid layout; compaction merges versions
and rebuilds pages under a byte as well as row budget. Point/range reads use
the spine without DataFusion. Analytical scans project only needed pages.

Use one generic page contract with explicit data-family and statistics
descriptors. Test mixed control, causal-stream, temporal-entity, search-metadata,
and vector-shaped traffic before adding family-aware page grouping, restart
compression, filters, pinning, or cache admission. Retain a specialization only
when fixed-corpus evidence improves its declared access pattern without
regressing correctness, memory bounds, or another family; otherwise keep the
simpler shared policy.

Do not claim Lance compatibility. Freeze RRFlow-owned binary vectors, fuzz
decoders, differential reads against the memtable oracle, and report borrowed,
read, decoded, copied, allocated, and decompressed bytes.

### C-07 — durability and lifetime matrix

Extend existing WAL, manifest, compaction, snapshot, failure-matrix, tiered-I/O,
and memory tests. Add mapped-buffer pinning tests where compaction deletes an
old generation while Arrow still owns a batch. Add ENOSPC/short-write,
checksum, torn current pointer, orphan cleanup, concurrent pinned snapshot,
and repeated crash/reopen cases. Exercise bounded automatic flush/compaction,
write stalls and backpressure, global write-buffer accounting, mixed-family
compaction/cache interference, and sustained/long-duration RSS. Expose the
write-buffer, cache, disk, compaction, stall, and failure counters needed to
explain each result. No following gate begins while an acknowledged write can
be lost, memory can grow without its declared bound, or a mapped buffer can
dangle.

## Gate D work packages

### D-01 — previewable project bootstrap

Create `rrd-engine/src/engine/install.rs`, `rrflow-cli/src/install.rs`, and the
CLI install conformance test before adding the `install` dispatch to
`rrflow-cli/src/command.rs`. The versioned template manifest and files under
`rrflow-cli/templates/project-v1/` own minimal `.rrflow/config.toml`, estate
identity, native rrflowKV placement, attunement profile, `AGENTS.md`, and
supported forwarding-only provider files. The specialization manifest selects
the primary seat and allowed provider-representation candidates. The generic
template has no hardcoded persona or provider; this repository's specialization
selects Clyffy explicitly. During development, inputs are
embedded in the CLI or resolved relative to an explicitly supplied, locally
verified candidate-bundle root; J-03 assembles the complete release candidate
and J-05 signs the reproducible distribution. Apply has no download or
sibling-discovery branch. The bundle manifest also accounts for every embedded
function runtime/build, sealed built-in registry entry, default portable
function artifact, schema, and golden vector; D-01 installs only the generic
inactive foundation, while I-06 owns optional project function/binding
activation. `AGENTS.md` is the one instruction body. Existing
user files are never overwritten without an exact previewed action and
explicit apply.

`InstallationActionKind::InitializeInstance` is the only cold-start security
operation. Its input digest covers the exact installed topology, storage
profile, initial principals/roles/grants, typed credential-verifier policy,
credential generation or opaque source/sink descriptor, validity/rotation
policy, and bundle/configuration/specialization digests. Preview is pure: it
does not open a store, read or generate a secret, contact a provider, or start
RRD. Apply acquires an exclusive create-new installation lease, proves the
target has no installed binding/security/canonical state, uses an engine clock,
and commits installed binding, security state, action checkpoint, audit,
initial seat definition, outbox/commit evidence, and cursor in one `RrdEngine`
transaction. Provider representation is activated only from an approved
binding and stores neither credential nor plaintext provider subject. Once an
installed binding exists, cold start remains unavailable even if policy is
missing or damaged.

Secrets are references or planned generation actions, not values. The engine
accepts a bounded secret capability from an injected adapter, never an
absolute path. A file adapter opens a normalized relative entry below an open
capability root and validates the opened handle; Kubernetes link/mode behavior
and Windows ACL behavior remain adapter-specific conformance. Generated API
keys use the OS cryptographic random source and a previewed create-new sink.
Raw buffers are zeroized after delivery and typed-verifier creation and never
enter canonical data, arguments, environment, logs, traces, errors, or stdout.
External delivery uses prepared/effect/observation/receipt state so replay
cannot generate or deliver another credential.

Empty-project and existing-application-project golden fixtures run with
network denied and sibling paths absent and prove pure preview, exact apply,
user-file preservation, one credential outcome, authentication, and rrflowKV
close/reopen. A boundary matrix interrupts before and after credential
prepare/delivery, engine commit, locator publication, attunement-job creation,
acknowledgement, and cleanup. Negative cases cover changed plan/source
revision, partial state, stale lease, pre-existing sink, path/link/mount
replacement, Unix mode, Windows ACL, Kubernetes projection, already-installed
cold start, plaintext accounting, and listener readiness. Only after this
behavior passes may an executable installation guide be added. The current
`RrdEngine::bootstrap_security_store`, `rrd-security-bootstrap` binary,
`security-bootstrap.json`, CLI supervisor path, Kubernetes init path, and old
success fixtures are removed in the same direct-convergence package.

### D-02 — persisted job executor

Create `rrd-engine/src/engine/attunement.rs`. Persist B-01 jobs, leases, and
checkpoints through ordinary authorized commits. Events/traces observe that
state; they never infer it. Kill the process at each state transition and
prove idempotent resume/cancel/stale-lease handling.

### D-03 and D-04 — pure inventory and parsing

Create `rrd-attunement` with no store, engine, transport, provider, or secret
dependency. Implement inventory and parse as separate commits in the canonical
order. Each phase accepts bounded inputs plus digest/revision metadata and
returns a deterministic proposal. `RrdEngine` commits the proposal and
checkpoint.

- inventory: `crates/compute/rrd-attunement/src/inventory.rs` owns the pure
  `SourceTreeSnapshot`, `SourceTreeEntry`, `SourceTreeChangeSet`,
  `InventoryPolicy`, `InventoryError`, normalization, ordered Merkle digest,
  classification, and previous-snapshot diff logic. It receives enumerated
  metadata/content chunks; it never opens a path. The existing planned
  `crates/authority/rrd-engine/src/engine/attunement.rs` owns the authorized
  two-pass filesystem read, lease/cancellation/resource enforcement, proposal
  validation, and atomic snapshot/containment/change-set/checkpoint/runtime-log/audit
  commit. `crates/compute/rrd-attunement/tests/inventory.rs` owns deterministic
  fixtures for Git precedence, tracked ignored files, non-Git roots, hidden
  source, secret/generated/vendor/cache exclusion, symlink/mount escape,
  unreadable/vanished/racing files, add/edit/remove/rename, traversal order,
  resource bounds, cancellation, reopen, and zero-content-read no-work runs;
- parse: `crates/compute/rrd-attunement/src/parse.rs` consumes an exact
  committed snapshot/change-set digest and owns Tree-sitter grammar
  digest/revision, incremental edit, and retained `ERROR`/`MISSING` evidence.
  `crates/compute/rrd-attunement/tests/incremental_parse.rs` proves parsing
  cannot begin before the inventory commit receipt and preserves unaffected
  identities across edit and restart.

D-05 is intentionally deferred until Gates E and F pass. Inventory and parsing
produce canonical source inputs; they must not manufacture temporary lexical,
vector, graph, or analytical implementations just to complete an earlier gate.

Do not split inventory into competing walkers or language-specific discovery
paths. Start with that one module and one fixture corpus; split internal modules
inside `rrd-attunement` only when file size or test seams justify it, while
retaining one `inventory` API and deleting the superseded path in the same
pre-release change. Filesystem watchers are out of D-03: Gate I may add a
removable host-event adapter, but notifications remain hints that schedule the
same authoritative inventory operation.

### D-06 — external source and project capability descriptors

Create `rrd-attunement/src/source.rs`, its fixture-driven source-discovery
test, and `rrd-operator-knowledge/src/source.rs`; then extend the inventory
phase. PostgreSQL, Turso/libSQL, Dragonfly/Redis, SQL configuration, object
stores, and other databases produce type/version/endpoint-reference/schema-
capability descriptors only. Credentials are redacted. Discovery never
installs, starts, or contacts a source. Adding an adapter and allowing contact
are explicit operator decisions and cannot change RRFlow persistence or
default readiness.

For project commands, add `rrd-contract/src/capability.rs`, its closed golden
fixture and contract test; `rrd-attunement/src/capability.rs`, its deterministic
fixture-driven discovery test; and `rrd-engine/src/engine/capabilities.rs` plus
the capability half of `rrd-engine/tests/capability_activity_conformance.rs`.
The contract keeps discovered facts, inactive candidates, immutable installed
bindings, and retirement distinct. A direct-process binding authenticates one
executable and literal argv without ambient `PATH`; a package-script binding
captures package-manager, manifest script/lifecycle closure, lockfile,
toolchain, wrappers, argument policy, and sandbox. Incomplete or unenforceable
closures fail closed. Preview performs no mutation or effect; apply accepts the
exact digest through `RrdEngine`. D-06 executes no command: I-03 owns prepared
activity dispatch and receipt acceptance, and I-06 owns optional scaffolding.

### D-07 through D-10 — placement, shared inputs, accounting, hibernation

Keep CoW, mount, and cold-tier mechanics in planned `rrflow-devforge`, split
as `placement.rs`, `mounts.rs`, `accounting.rs`, and `hibernation.rs` with one
fixture-driven test per concern.
`RrdEngine` owns quiesce/snapshot/resume authorization; rrflowKV owns durable
snapshot boundaries. Immutable lower layers and content-addressed dependency
mounts are input descriptors. The per-instance upper layer owns every mutable
RRFlow byte. Measure logical, allocated, compressed, WAL, segment, cache, and
snapshot sizes separately. Restore verifies digest, instance identity, and
commit boundary before serving.

## Gate E work packages

### E-01 — temporal adjacency

Use C-01 outgoing and incoming relation prefixes. Replace relation-log
reconstruction and linear scans in `rrd-query/src/execute.rs` with bounded
direction/type/depth expansion. Compare every result to an exact graph oracle;
report visited vertices, scanned adjacency keys, emitted edges, and budget
termination.

### E-02 — scalar and unique indexes

Move uniqueness from full current-record scans in `rrd-query/src/index.rs` to
transactional unique-key conflict. Maintain scalar entries and retirements in
C-03's batch. Differential tests cover null/missing values, composite keys,
updates, temporal visibility, conflict, crash, and reopen.

### E-03 — incremental BM25

Keep analyzer/tokenizer/scoring semantics in `rrd-query/src/bm25.rs`; persist
dictionary, document length/count, postings, positions, and tombstones at a
source cursor. Updates compute old/new term deltas in the C-03 transaction.
Corrupt/stale projections fall back to exact rebuild or fail according to
declared policy—never silently return partial rankings.

### E-04 — canonical vector plus projection delta

Canonical exact vectors commit through C-03. Immutable HNSW and quantized
generations name source cursor, collection generation, model, metric,
dimensions, payload-index digest, codec, and artifact digest. Searches merge
the active generation with exact delta overlay, apply authorized filters, and
exactly rerank final candidates. Interrupted build never replaces the active
generation.

### E-05 — cost/selectivity planner

Unify the query and vector candidate-path vocabulary. Statistics are stamped
and optional; absent/stale stats trigger correct fallback. The planner—not
LFG, GraphQL, SDK, or caller—chooses point, range, adjacency, scalar, BM25,
exact vector, or HNSW. Explain output records estimates, chosen/skipped paths,
freshness, and fallback reason.

## Gate F work packages

### F-01 — streaming rrflowKV provider

Create `rrd-query/src/provider/`. Replace `ArrowSnapshot` and `MemorySource`
construction from a complete `Vec<QueryRow>` with an execution plan that owns
one pinned read stamp and yields bounded `RecordBatch` streams from rrflowKV
memtables/spine/pages. Cancellation drops scans and page pins cleanly.

### F-02 — honest pushdown and physical accounting

Translate only supported projection, predicate, limit, and ordering into key
ranges/page selection. Return DataFusion `Exact`, `Inexact`, or `Unsupported`
accurately. Explain/metrics include keys, pages, physical bytes, decoded,
copied, allocated, borrowed, decompressed, rows in/out, and elapsed time.

### F-03 — native stamped operators

Create `rrd-query/src/physical/` for graph, BM25, vector candidate, and pure RRF
operators. Each consumes/emits a stamped Arrow schema and verifies stamp,
catalogue, and ordering compatibility. DataFusion handles relational analytical
composition; native operators retain storage-specific access and correctness.

### F-04 — one cross-operator budget

Move budgets to `rrd-query/src/budget.rs`; reserve/release through one query
context shared by storage scans, native operators, Arrow builders, DataFusion,
spill, serialization, and result delivery. Test memory, spill, time, scanned
keys, graph steps, candidates, rows, and bytes independently and in combination.

### F-05 — measured cache decision

First benchmark no cache versus a byte-bounded reference cache. If reuse is
material and safe, implement keys containing estate/scope, read stamp, schema,
catalogue, authorization projection, logical/physical plan digest, and output
shape. Mutation/schema/catalogue changes must invalidate by key/version, not a
best-effort callback. Moka is one candidate, not a requirement; record an ADR
whether it is adopted or rejected.

## Deferred D-05 work package after Gates E and F

Resume the B-01 phase order only after the native E access paths and streamed F
analytical boundary pass. Implement one phase per commit:

1. normalize produces stable typed facts with source identities and
   provenance;
2. entity-link proposes deterministic symbol/entity relationships;
3. lexical-index commits BM25 deltas through the E-03 path;
4. embed invokes a digest-bound inference backend and commits canonical exact
   vectors through `RrdEngine`;
5. vector-index consumes committed deltas and proposes an E-04 projection
   generation;
6. graph commits both E-01 adjacency directions with the canonical relations;
7. ground binds every derived fact to source, parser, model, schema, and read
   coordinates; and
8. verify queries the ordinary native and DataFusion paths and records exact
   results, unresolved errors, budgets, and output digests.

Each phase consumes the preceding committed checkpoint, returns a pure bounded
proposal from `rrd-attunement`, and is atomically committed with its checkpoint,
runtime-log entry, audit, and index/projection delta by `RrdEngine`. Kill/retry,
stale-input, digest-drift, close/reopen, exact-oracle, physical-plan, and trace
tests pass before the next phase begins. KB-06 then imports the deterministic
documentation package through this same pipeline; KB-07 proves durable
readback and recovery. No attunement-only index, graph, Arrow snapshot, or
storage path is allowed.

## Gates G through J work packages

### Gate G — LFG routing

- G-01: define `RouterBackend` in `rrd-inference/src/router.rs`; create the
  `rrflow-lfg` adapter; keep embeddings separate.
- G-02: build a bounded route packet in
  `rrd-engine/src/engine/routing.rs` from one stamp, eligible recipes, verified
  observations, permitted fields, and budgets. Bind the authenticated principal,
  visible provider-representation edge, resolved durable seat, policy revision,
  and authorization digest at that same coordinate; never trust a caller actor
  label as identity.
- G-03: grammar-constrain and post-validate the three B-02 decision variants;
  malformed or injected output creates no mutation.
- G-04: execute recipe/branch CAS through direct rrflowKV operations with no
  DataFusion plan, atomically retaining seat attribution, audit, and causal
  trace with the tree mutation.
- G-05: lower context requests to semantic rrflowQL intent; physical access
  stays engine-selected.
- G-06: add reproducible routing/task/invalid/escalation evaluation with model,
  tokenizer, runtime, quantization, hardware, and raw-sample manifests.

### Gate H — context, feedback, delivery, and Connectome

- H-01: replace fixed/parallel context assembly logic in `context.rs`,
  `retrieval.rs`, and `retrieval_query.rs` with the E/F planner and per-avenue
  selected/skipped evidence.
- H-02: centralize pure `math::rrf()`; persist only verified outcomes and new
  versioned policies for future stamps; old-stamp replay is immutable.
- H-03: feed C-03 commit impacts into predicate-specific live evaluation;
  remove two-query diffing.
- H-04: add `rrd-server/src/http/handlers/graphql.rs` and one
  `rrd-server/tests/cross_surface_conformance.rs`; drive all public surfaces
  from one operation catalogue and structurally checked cross-surface corpus;
  Rust and generated SDKs bind every available operation exactly once and
  never invent operations. Validate full request/response/status/media/
  identity/stamp/receipt contracts, redact sessions/errors, classify retry and
  uncertain outcomes by operation semantics, and make absent harness input
  fail or explicitly skip without reporting conformance. Extend
  `fixtures/rrd-function-conformance-v1.json` so function catalogue
  administration and standalone invocation compare the same denial, stamp,
  catalogue/definition/artifact/runtime identities, output digest, and receipt
  through every supported surface; do not advertise an engine-only method as
  an outward binding.
- H-05: correlate ingress, auth, plan, KV, graph, BM25, HNSW, DataFusion, LFG,
  commit, attunement, and delivery spans; redact before export.
- H-06: keep Connectome in its separate repository and use only public RRD
  health/capability/session/query/trace/subscription operations.
- H-07: create an endpoint-candidate resolver and mesh transport adapter after
  the operator supplies the Zuul Zero/shippin.ai protocol contract. Each
  candidate carries expected transport and RRD instance identities; rotation
  repeats liveness, readiness, capabilities, catalogue-digest, and application
  authentication. Network reachability never authenticates identity,
  initializes RRFlow, selects storage, or gives the mesh RRFlow state.

### Gate I — explicit automation

- I-01: freeze the one semantic `EngineEvent` contract and directly converge
  the existing kernel `RuntimeEvent` representation into it. Public submission
  lowers to `RuntimeMutation::Event`; a committed event consumed by triggers is
  that same object plus its commit receipt, never a second event database.
  Rename the transaction-function `EmitEvent` capability/effect to
  `ProposeEngineEvent`; `RrdEngine` supplies and validates every authoritative
  event coordinate before the proposal joins the semantic commit.
- I-02: persisted triggers match committed events and may request only an
  authorized engine operation. Rename the existing synchronous proposed-
  transaction function bindings so `Trigger` has no second meaning. The
  function binding corpus proves it cannot consume a committed-event cursor,
  recursively rematch its own proposal, or schedule a routine.
- I-03: routines are versioned resumable operation graphs with checkpoint,
  budget, lease, idempotency, cancel, compensation, verification, and terminal
  state. Add `rrd-contract/src/activity.rs`, its golden fixture and test;
  `rrd-engine/src/engine/{routines,activities}.rs`;
  `rrd-engine/tests/capability_activity_conformance.rs`; and the injected
  `rrflow-local-process/src/activity.rs` port plus adapter conformance test.
  Model, process, network, and external MCP calls are prepared, fenced
  activities; adapters return bounded observations, while only `RrdEngine`
  accepts a receipt and advances state. Replay never repeats an effect merely
  to reconstruct state, and an effect with an unprovable outcome enters
  explicit reconciliation rather than automatic retry.
- I-04: optional `rrflow-host-events` translators for Claude, Codex, Gemini,
  and a reference host emit the same typed event only after previewed explicit
  installation. One shared conformance fixture and test compare their emitted
  envelopes. No session-start hook ships by default.
- I-05: skills are immutable identity/digest instruction-resource packages;
  resolving them at a read stamp is context retrieval, not executing
  storage/lifecycle code or granting the requested capabilities.
- I-06: `rrflow-cli/src/automation_install.rs` and its conformance test own
  preview/apply/retire/uninstall for optional function artifacts/definitions,
  transaction bindings, triggers, routines, host adapters, skills, and
  capability/activity scaffolding. They report exact artifacts, schemas,
  files, records, invocation closures, and adapter registrations; preview and
  attunement execute no function, default artifacts resolve only from the
  verified offline distribution, retirement blocks new work while prior
  receipts remain readable, and uninstall removes only accepted unreferenced
  RRFlow-owned scaffolding.
- I-07: every project-development run first binds the latest complete
  authorized project-tree snapshot; missing/stale inventory returns
  `inventory-required`, changed-since-plan entries and paths outside the root
  are denied, and committed inventory/schema/dependency/workload/failure
  changes schedule only the affected attunement phases and eligible automation
  under policy.

The existing `engine/automation.rs` function sandbox is reusable inventory.
A-07 first moves it into the exact `engine/function/` modules and applies the
direct contract names; C-03 then replaces its private control records and split
audit/commit behavior. I-01/I-02 connect its proposals to the final event
contract while events, post-commit triggers, and routines remain separate
modules. None of those mechanical or transactional steps alone completes Gate
I.

The first proof is the `error-resolution` vertical slice in the engine-flow
owner. Its routine uses semantic operation/capability references only. The
context planner—not the routine, skill, model, host, or MCP adapter—selects
graph, BM25, exact/HNSW/TurboQuant, or DataFusion work and records every
selected or skipped reason. Kill/restart at every persisted transition and
after every external activity; compare run identity, effects, stamps, digests,
and evidence across embedded, HTTP, WebSocket, SDK, MCP, and Connectome reads.

### Gate J — release proof

- J-01: remove all alternate pre-release entrypoints, readers, backend
  selectors, migration executors, transitional aliases, provider-hook paths,
  and stale generated outputs; run strict repository searches.
- J-02: run unit, property, fuzz corpus, differential, crash/reopen, ENOSPC,
  security, budget, adapter, SDK, and real-process tests; retain failures.
- J-03: use `scripts/release/assemble.py` to build a complete manifest-verified
  release-candidate bundle from tracked inputs, then use
  `scripts/release/qualify.py` to install it into an empty fixture and this
  repository with outbound network denied, sibling repositories hidden, and no
  external database service; attune, query fast/heavy paths, close/reopen, and
  rerun incrementally. This gate qualifies candidate contents and behavior;
  J-05 owns reproducibility, signing, and final clean-machine verification.
- J-04: publish fixed-hardware raw results for storage, graph, BM25,
  exact/HNSW, DataFusion, context, LFG, memory, and disk. Use
  `scripts/release/compare_deployment.py` for a pinned official
  SurrealDB/Qdrant rollout matrix driven by
  `fixtures/release/deployment-baselines-v1.toml`; record artifact and installed
  bytes, commands, elapsed time, services, ports, configuration, secrets,
  readiness, and persistent readback. “Smoke Lance,” “easier to deploy,” or any
  other comparative claim is forbidden until like-for-like data exists. Bind
  each run to the exact revision, binary, toolchain, host, filesystem, device,
  corpus, warm-up, cache state, and failed samples; report logical, apparent,
  allocated, cached, and resident bytes separately. Include mixed-family
  interference and sustained/long-duration maintenance rather than promoting a
  short local microbenchmark.
- J-05: use `scripts/release/assemble.py` and `scripts/release/verify.py` to
  produce and verify one signed reproducible default distribution containing
  all default-distribution first-party executables and linked engine
  capabilities, schemas/goldens, SDKs, project/attunement templates, default
  configuration/profile, required local model/runtime artifacts, SBOM/licenses,
  backup/restore evidence, provenance, and operator runbook. Freeze the
  manifest shape in `fixtures/release/distribution-manifest-v1.json` and test
  assembly, tamper rejection, completeness, and offline qualification in
  `scripts/release/test_distribution.py`. Verify every installed byte and run
  the complete install/readiness/commit/reopen check on a clean machine without
  a compiler, source checkout, sibling repository, local cache, package
  registry, external database/query/vector service, or outbound network.

## Repository-wide run checklist

Run the narrow command named by the package first. The widening sequence is:

```text
cargo test -p rrd-engine --lib engine::tests::context --locked
cargo test -p rrflow-cli --test operator_surface \
  identity_bind_resolve_and_readme_warp_share_the_persistent_engine --locked
cargo test -p rrflow-mcp --test stdio --test stdio_daemon --locked
cargo test -p rrd-client --test real_server \
  rust_client_negotiates_authenticates_queries_and_reads_audit --locked
```

Those are the current narrow context, warp, MCP, and client checks. The
separate Connectome repository owns `pnpm run check` and `pnpm run test:smoke`.
After the affected narrow suite passes, widen in this order:

```text
python3 scripts/ci/build_execution_inventory.py --check
python3 scripts/ci/check_documentation.py
python3 scripts/ci/check_generated_surfaces.py
python3 scripts/ci/check_workflow.py
python3 scripts/check_version.py
cargo fmt --all -- --check
cargo test -p <owning-package> --locked
cargo clippy -p <owning-package> --all-targets --locked -- -D warnings
cargo test -p rrd-engine --test workspace_architecture --locked
cargo check --workspace --all-targets --locked
```

Full workspace tests, SDK conformance, crash matrices, release installation,
and benchmarks are run only at the gates that change those surfaces; they are
not substituted with compilation.

## Evidence record template

Every completed package records:

```text
gate/package:
revision:
baseline files/digests:
files read in full:
files changed/created/deleted/moved:
contract or behavior changed:
smallest test command and result:
owning package command and result:
cross-boundary command and result:
failure/crash/differential evidence:
not run and reason:
remaining known errors:
roadmap checkbox changed: yes/no
```

## Global stop conditions

Pause the current package and update this map before continuing if any of these
occurs:

- a required file, dependency, public symbol, generated surface, or external
  protocol is missing from the plan;
- a change would add a second transaction, storage, graph, vector, query,
  context, lifecycle, or instruction authority;
- an alias or shim appears necessary to keep an earlier pre-release surface;
- the only proof is compilation, a mock, a generated file, or an emitted event;
- an approximate result lacks an exact comparison or declared fallback;
- a persistent claim lacks close/reopen and failure-boundary evidence;
- an external source requires copying credentials or taking application DB
  ownership;
- a first-party capability, release input, or default-runtime dependency lives
  outside this repository or must be fetched during install/runtime;
- “zero-copy,” latency, scale, or superiority would be claimed without measured
  physical evidence; or
- the worktree contains unexplained overlapping changes.

The package resumes only after the discrepancy is documented against the
architecture owner, canonical roadmap, POA&M, and file inventory.
