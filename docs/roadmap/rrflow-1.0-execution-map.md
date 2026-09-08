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
| transport/adapters/SDKs | catalogue-derived HTTP/OpenAPI, durable subscriptions, a Rust client implementing 28 of 33 HTTP operations plus the dedicated socket, TypeScript, synchronous Python, and context-aware synchronous Go generic calls over all 33 HTTP descriptors with partial runtime enforcement and no qualified socket/remote/package surface, MCP, CLI, and two other generated SDK projections | split clients by accepted responsibility; close operation/validation/secret/retry/cancellation/frame/resolver/package/browser/interpreter/toolchain/concurrency gaps; freeze multiplex WS and GraphQL lowering; make missing harness configuration fail or explicitly skip; prove every surface only against the same installed engine semantics after storage/query behavior exists |
| estate/operations | substantial desired/observed, process, backup, recovery, and operator-source foundations; estate state currently uses a public direct-store repository and monolithic JSON control value; `rrd-maintenance` is an unused standalone direct-store state-machine draft | preserve the fenced reconciliation/recovery semantics while making estate values pure and commits engine-owned; reconcile the overlapping operational hierarchy; converge reusable maintenance validation into generic routines; remove both private storage authorities rather than wrapping them |
| `rrd-cluster` | useful placement/stamp/consistency/transfer/reshard contracts, OpenRaft storage and TLS mechanics, bounded snapshot/artifact transfer, telemetry, simulation, and one-host tests; current code also owns parallel IDs/topology/schema, opens storage directly, accepts raw commits, persists private JSON transfer state, and is failing against the current rrflowKV application format | retain the safety semantics through pure contracts and injected distributed execution ports beneath `RrdEngine`; remove the parallel authorities and old-format paths directly; keep `clustered_server` unavailable until a later roadmap amendment schedules and accepts independent-host full-engine qualification |
| `rrd-kubernetes` | namespaced structural CRD, deterministic five-resource rendering, digest-pinned image validation, retained one-replica StatefulSet/PVC, restricted container settings, PDB/NetworkPolicy, server-side apply, status/finalizer controller, checked manifests, and four local tests; current code also invents desired RRFlow state, runs two parallel bootstrap paths, force-takes fields, equates TCP/ready replica with application readiness, watches cluster-wide, and deletes without engine intent/receipt proof | preserve the useful Kubernetes mechanics in planned `rrflow-kubernetes`, an outward adapter consuming sealed engine install/effect plans and returning bounded observations/receipts; use standard conditions, authenticated readiness, explicit field ownership, engine-fenced deletion/retention, and real API-server/clean-install/full-engine evidence; remove the old operations package with no forwarding crate |

## Implementation-requirements traceability

This is the initial current-tree accounting required by A-07 and POAM-014. It
is updated in a documentation-only change before a listed implementation file
moves or a newly discovered behavior expands a gate. A row identifies work
that must be carried into the one target system; it does not mark that work
complete.

| Capability family | Current implementation that must be read in full | Characterization inventory that must be preserved or strengthened | Canonical convergence | Gates |
|---|---|---|---|---|
| rrflowKV WAL, MVCC, manifests, recovery, compaction, hot reads, block filtering, and decoded-block caching | `rrd-lsm/src/{wal,memtable,database,manifest,segment}.rs`; `rrd-store/examples/ai_hotset_benchmark.rs` | `rrd-lsm/tests/{wal,mvcc,manifest,failure_matrix,compaction,segment,snapshot_memory}.rs`; `rrd-store/tests/{durability,snapshot,benchmark_evidence}.rs` | Keep the useful durability, snapshot, bounded-cache, filter, and physical-counter behavior while replacing the port and row-only segment format; prove the final key spine, Arrow pages, buffer lifetime, and measured cache decision. | C-02, C-04, C-06, C-07, F-05, J-04 |
| rrflowMX/rrflowKV semantic equivalence | `rrd-store/src/{engine,rrflow_kv,ds}.rs` | `rrd-store/tests/{engine,snapshot,runtime,unified_data,rrflow_kv_operator,rrflow_kv_model_soak}.rs`; vector engine differential tests | Narrow the one `StorageEngine` port, then run the same transaction, point/range, snapshot, graph, index, and query corpus against `RrflowMxStore` and `RrflowKvStore`; durability assertions apply only to rrflowKV. | A-07, C-02, C-03, C-04 |
| Stamped transactions, identity, authorization, and audit | `rrd-core/src/runtime.rs`; `rrd-store/src/{engine,rrflow_kv,control}.rs`; `rrd-engine/src/engine/{transaction,query_transaction,session,security,invocation,control}.rs`; `rrd-security/src/lib.rs` | `rrd-store/tests/{control_journal,durability,snapshot}.rs`; `rrd-engine/src/engine/tests/{query_transaction,security,transaction_stamp,recovery}.rs`; `rrd-security/tests/security_authority.rs`; `rrd-server/tests/http_process.rs` | Preserve authenticated `ReadStamp`, `DataTransaction`, conflict, idempotency, audit-chain, and policy semantics while making one transaction port and one cross-surface authorization path. | C-02, C-03, H-04, H-05, J-02 |
| Initial security and credential installation | `rrd-engine/src/engine/security_bootstrap.rs`; `rrflow-cli/src/bin/rrd-security-bootstrap.rs`; `rrflow-cli/src/dev/supervisor.rs`; `rrd-kubernetes/src/lib.rs`; `rrd-security/src/lib.rs::SecurityRepository::{initialize,load}` | `rrflow-cli/tests/security_bootstrap.rs`; bootstrap/reopen path in `rrd-engine/tests/engine_authority.rs`; initialization, replay, drift, redaction, rotation, and MX/KV audit cases in `rrd-security/tests/security_authority.rs`; supervisor and Kubernetes rendering assertions | Preserve strict decoding, bounded non-empty regular inputs, unique identities, full policy validation, verifier-only persistence, atomic initial policy/audit, exact replay, drift denial, and no network-listener bootstrap. Replace the database/absolute-path/caller-time helper and adapter-authored policies with D-01's exact-plan `initialize_instance`: exclusive fresh-target proof, engine clock, capability-scoped/versioned secret sources and sinks, typed verifiers, atomic installed binding/policy/checkpoint/audit, prepared external-effect receipts, restart-safe credential delivery, and rejection of partial or already-installed cold start. Delete the standalone binary, static engine opener, private bootstrap JSON, all callers, and successful old shape after equal-or-stronger conformance passes. | A-07, C-02, C-03, D-01, D-02, H-05, J-01 through J-03, J-05 |
| Deterministic embedding, vector search, compact artifacts, HNSW, quantization, and accelerator admission | `rrd-inference/src/{lib,fastembed_local}.rs`; `rrd-vector/src/{contract,catalog,exact,filter,plan,segment,compact,hnsw,quantization,accelerator,runtime}.rs`; `rrd-engine/src/engine/{inference,vector}.rs` | `rrd-inference/tests/pipeline.rs`; `rrd-vector/tests/{golden,exact_model,engine_differential,model_binding,compact_dense,online_hnsw,quantization_matrix,accelerator,recall_gate}.rs`; `rrd-engine/src/engine/tests/{vector_index,native_inference}.rs` | Keep deterministic model/provenance binding and exact oracles; commit canonical vectors and index deltas atomically, bind projections to one source cursor, filter candidates, and exact-rerank before results become authoritative. | D-05, E-04, E-05, F-03, H-01, J-04 |
| Edge packaging and public delivery | `rrd-engine/src/edge.rs`; `rrflow-edge/src/main.rs` | `rrflow-edge/tests/{offline,evidence}.rs`; `rrd-client/tests/real_server.rs`; `rrd-server/tests/http_process.rs`; `rrflow-mcp/tests/{stdio,stdio_daemon}.rs` | Retain deterministic offline artifact and provenance checks as outward packaging evidence; all reads and mutations continue through public RRD capabilities with no edge-owned engine state. | H-04, H-07, J-03, J-05 |
| Rust SDK operation, transport, credential, retry, cancellation, subscription, and conformance behavior | `rrd-client/src/lib.rs`; `rrd-client/Cargo.toml`; `rrd-contract/src/{lib,sdk_conformance}.rs`; executable endpoint catalogue and WebSocket descriptor | `rrd-client/tests/{real_server,sdk_conformance}.rs`; `rrd-client/examples/sdk_conformance_server.rs`; `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py` | Preserve the implementation-free normal dependency, typed calls, loopback restriction, explicit TLS, identity checks, four-MiB HTTP limit, real server/mTLS/WSS fixtures, and durable ACK/reconnect behavior. Split the monolith into the planned modules; bind all 33 operations exactly once; validate complete payload/envelope/status/media/identity contracts; make sessions opaque/redacted; replace boolean immediate retry and local-drop cancellation with semantic certainty and correlated cancellation; bound/validate multiplexed frames; add W3C propagation and authenticated endpoint rotation; make conformance configuration and scenario coverage fail closed; seed only through D-01/public operations and compare full rrflowMX/rrflowKV engine evidence across surfaces. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| TypeScript SDK generation, HTTP transport, credential, retry, cancellation, packaging, browser, and conformance behavior | `sdks/typescript/{package.json,pnpm-lock.yaml,pnpm-workspace.yaml,biome.json,tsconfig.json}`; `sdks/typescript/scripts/generate.ts`; `sdks/typescript/src/{index,generated/endpoints,generated/rrd-openapi}.ts` | `sdks/typescript/tests/{client.test,sdk_conformance}.ts`; `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py`; shared live harness | Preserve deterministic repository-local generation, all-33 HTTP descriptor typing, loopback restriction, canonical ID/resource construction, idempotency requirement, bounded body reading, external abort, fail-closed manifest input, locked tools, and current mock/live characterization. Split the runtime into the planned seams; generate full runtime validators; enforce exact payload/envelope/status/media/identity/stamp/receipt contracts; make credentials opaque; replace broad immediate retry/local abort with semantic certainty/server cancellation; add bounded multiplexed WebSocket and authenticated Node/browser transports; build deterministic ESM JavaScript/declarations; and prove package/offline/browser plus installed rrflowMX/rrflowKV cross-surface behavior. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| Python SDK generation, sync/async HTTP transport, credential, retry, cancellation, packaging, and conformance behavior | `sdks/python/{pyproject.toml,uv.lock}`; `sdks/python/scripts/generate.py`; `sdks/python/src/rrd_client/{__init__,client,models,generated/__init__,generated/endpoints}.py` | `sdks/python/tests/{test_client,sdk_conformance}.py`; `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py`; shared live harness | Preserve deterministic repository-local generation, all-33 HTTP operation literals/descriptors, loopback and redirect denial, canonical ID/resource construction, mutation-idempotency requirement, bounded body reading, fail-closed manifest input, locked tools, buildable wheel/sdist, and current mock/live characterization. Split the runtime into the planned seams; generate full runtime models; enforce exact payload/envelope/status/media/identity/stamp/receipt contracts; make credentials opaque; replace broad immediate replay/local timeout with semantic certainty/server cancellation; add native async plus bounded multiplexed WebSocket and authenticated remote transports; ship `py.typed` and reproducible signed offline artifacts; and prove dependency, interpreter, platform, consumer, and installed rrflowMX/rrflowKV cross-surface behavior. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| Go SDK generation, context-aware HTTP transport, credential, retry, cancellation, concurrency, module distribution, and conformance behavior | `sdks/go/go.mod`; `sdks/go/{client,models,endpoints_gen,client_test}.go`; `sdks/go/cmd/{generate,conformance}/main.go` | `fixtures/rrd-sdk-conformance-v1.json`; `scripts/ci/{run_sdk_conformance,check_generated_surfaces}.py`; shared live harness | Preserve deterministic repository-local generation, all-33 operation constants/descriptors, first-argument contexts, loopback and redirect denial, canonical ID/resource construction, mutation-idempotency requirement, identical attempt bytes, bounded body reading, fail-closed manifest input, dependency-free HTTP baseline, and current unit/race/live characterization. Split the package into the planned idiomatic Go files; generate concrete request/result bindings; enforce exact payload/envelope/status/media/identity/stamp/receipt contracts; make credentials opaque; replace broad immediate replay/local-only context cancellation with semantic certainty and correlated server cancellation; own explicit bounded concurrent HTTP/WebSocket transports and endpoint identity; and prove module archive, dependency, toolchain, OS/architecture/race, external-consumer, and installed rrflowMX/rrflowKV cross-surface behavior. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| Temporal graph, BM25, hybrid retrieval, and context evidence | `rrd-query/src/{bm25,index,execute,plan}.rs`; `rrd-engine/src/engine/{context,retrieval,retrieval_query}.rs` | `rrd-query/tests/{query,index_catalogue,golden}.rs`; `rrd-engine/src/engine/tests/{context,index_foundation}.rs`; `rrd-engine/tests/{runtime_query_trace,runtime_data_plane_trace}.rs` | Replace broad snapshot reconstruction with transactional adjacency/BM25/vector access paths, cost-selected at one stamp and fused with bounded deterministic evidence. | E-01, E-02, E-03, E-05, F-03, H-01, H-02, H-05 |
| Arrow/DataFusion analytical execution | `rrd-query/src/{arrow,fusion,execute,pipeline,plan}.rs` | `rrd-query/tests/{golden,query,index_catalogue}.rs` and the DataFusion-focused unit tests inside the listed source modules | Replace complete `Vec<QueryRow>` materialization with a pinned stamped provider; push supported work into rrflowKV, compose native operators, enforce one resource budget, and report every read/decode/copy/allocation. | F-01 through F-05 |
| Generic reasoning trees, routing, and governed mutation | `rrd-core/src/reasoning_tree.rs`; `rrd-contract/src/{reasoning_tree,router}.rs`; `rrd-engine/src/engine/{context,transaction}.rs` | `rrd-core/tests/{reasoning_tree_contract,reasoning_trace_link}.rs`; `rrd-contract/tests/{reasoning_tree_contract,router_contract}.rs`; `rrd-engine/src/engine/tests/{context,transaction_stamp}.rs` | Preserve the accepted generic tree and three bounded routing decisions; add persisted CAS execution, the model-manifest handshake, constrained LFG dispatch, and engine-selected physical work without a fixed lifecycle. | B-03, G-01 through G-05, H-01, H-05 |
| Durable seat identity, provider representation, and routing attribution | `rrd-contract/src/memory_estate.rs`; `rrd-engine/src/engine/memory_estate.rs`; `rrd-engine/src/operator.rs`; `rrd-core/src/claim.rs`; `rrflow-cli/src/command.rs` | `rrd-engine/src/engine/tests/memory_estate.rs`; `rrflow-cli/tests/operator_surface.rs::identity_bind_resolve_and_readme_warp_share_the_persistent_engine`; `rrd-contract/tests/router_contract.rs`; claim/store golden and grounding tests | Preserve subject-digest redaction, temporal provider-to-seat representation, unrepresented denial, replacement, persistent reopen, and warp-to-context behavior. Rename the ambiguous `MemoryEstate*` surface directly; make D-01 install the specialization-selected seat without a generic Clyffy default; bind the authenticated provider identity, visible representation edge, seat, policy, route packet, proposal, mutation, audit, and trace at one coordinate; reject arbitrary producer actor strings as identity; replace broad snapshot resolution with canonical native access. | A-07, C-03, D-01, G-01 through G-05, H-01, H-04, H-05, J-01 |
| Context projection maintenance draft | `rrd-maintenance/src/lib.rs` | no focused package test and no caller outside the package; the exhaustive preservation/disposition matrix is owned by `docs/reference/context/context-maintenance.md` | Preserve source-cut inventory/accounting, proposal and evidence completeness, review attribution, optimistic conflicts, digest lineage, atomic publication, generation ownership, observation, and compensating rollback through new generic-routine acceptance tests. Replace the direct `StorageEngine`, private scope/event/repository, cursor-zero replay, JSON-wrapper records, fixed seven-stage lifecycle, hardcoded classes/reduction/token policy, and package-local API with generic I-03 routine state and authorized `RrdEngine` operations; remove the standalone crate only after every preserved/generalized matrix row is covered, with no wrapper. | A-07, C-03, C-04, H-02, H-05, I-01, I-03, J-01 |
| Governed functions and proposed-transaction bindings | `rrd-contract/src/{function,lib}.rs`; `rrd-engine/src/engine/{automation,transaction,security,mod}.rs`; `rrd-engine/src/capabilities.rs`; `rrd-engine/Cargo.toml`; `rrd-store/src/control.rs`; locked `rquickjs`/`wasmi` versions in `Cargo.lock` | `rrd-contract::function::tests`; `rrd-engine::engine::tests::automation`; `rrd-store::control::tests`; no function golden fixture, MX/KV differential, effect-complete audit crash test, runtime-build/target corpus, physical-size boundary, installation test, or outward conformance exists | Preserve closed digests, canonical ordering, byte/depth/item/numeric bounds, single-attempt semantics, fresh runtimes, memory/stack/interrupt/fuel controls, import denial, revision pinning, no recursive rematch, validator denial, and derived-event commit behavior. Directly rename the catalogue and transaction binding family; split `engine/automation.rs` into `engine/function/{mod,catalogue,execution,javascript,webassembly,transaction_binding}.rs`; bind closed schemas, content-addressed artifacts, explicit runtime profiles/builds, prepared receipts, and physically satisfiable limits; replace private control JSON/direct-store access with typed stamped state; commit allowed receipt/audit/event/outbox/domain/index effects atomically; keep JSON sandboxes separate from vectorized rrflowQL/DataFusion functions; add offline install, MX/KV, crash/reopen/upgrade, resource/security, and real-surface proof. | A-07, C-01 through C-04, D-01, F-03, H-04, H-05, I-01, I-02, I-06, J-01 through J-05 |
| Installation, attunement, and explicit automation | `rrd-contract/src/attunement.rs`; `rrd-engine/src/engine/automation.rs`; `rrflow-cli/src/{command,dev}.rs` | `rrd-contract/tests/attunement_contract.rs`; `rrd-engine/src/engine/tests/{automation,deployment_conformance,lifecycle,recovery}.rs`; `rrflow-cli/tests/operator_surface.rs` | Build bundle-resident preview/apply, installed credential references, persisted phase jobs, incremental project specialization, canonical events, resumable routines, digest-bound skills, and optional host translators under the one `RrdEngine` authority. Estate provisioning and local authorization remain traced in their dedicated estate/security rows rather than duplicated here. | A-07, D, H-04, I, J-01, J-03, J-05 |
| Project command discovery, installed capability bindings, and external activities | `rrd-contract/src/function.rs`; `rrd-engine/src/engine/automation.rs`; `rrd-core/src/{runtime,trace}.rs`; `rrd-engine/src/runtime/trace.rs`; `rrd-estate/src/local_process.rs`; `rrflow-cli/src/dev/supervisor.rs`; `rrflow-eval/src/main.rs` | function contract/unit and engine automation tests; engine lifecycle/trace tests; local-process characterization; CLI supervisor tests; no command-capability, discovery, activity, adapter, or cross-profile conformance test exists | Retain the bounded function sandbox under function-only names, durable causal trace evidence, and the useful no-shell/path/identity/timeout/effect-gap process safety. Keep provider CLI invocation confined to evaluation. Add pure discovered-fact/candidate/binding/activity/observation/receipt contracts; deterministic discovery; engine-owned prepared dispatch, accepted receipt, reconciliation, and re-inventory; and one outward local activity adapter. Distinguish authenticated direct-process argv from package-script closures that may invoke a shell; deny incomplete closures, ambient authority, adapter-authored completion, and direct storage/query/index access. | A-07, C-03, C-04, D-03, D-06, H-05, I-01, I-03, I-06, I-07, J-01 through J-05 |
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
| `rrd-client/src/lib.rs` monolith, five missing catalogue operations, public bearer-bearing `Session` `Debug`, manual route/mutation/retry flags, partial response validation, generic successful-status handling, library-default WebSocket limits far above RRD, and manifest-absent conformance success | preserve the implementation-free dependency direction, typed surface, loopback restriction, explicit Rustls transport, expected-instance negotiation, four-MiB HTTP cap, durable ACK/reconnect, and real loopback/mTLS/WSS characterization. Split directly into the planned client/endpoint/error/operation/retry/session/subscription/transport modules; derive exact bindings from the catalogue; validate every payload/envelope/status/media/identity; make credentials opaque and redacted; implement operation-semantic retry/uncertainty and correlated cancellation; configure bounded multiplexed frames; make missing harness input fail or explicitly skip; replace label-only corpus coverage and direct-store fixture installation with structural D-01/public-operation MX/KV cross-surface evidence. No old module, route table, debug shape, dedicated socket protocol, or permissive test path survives its owning gate. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| TypeScript `src/index.ts` monolith, compile-time-only payload types, partial ArkType envelope, any-`2xx` success, ignored media, plain serializable credentials, broad immediate retry, local-only abort, implicit redirect following, loopback-only HTTP, absent WebSocket/W3C, and private raw-source package | preserve deterministic local generation, the 33-operation generic type surface, canonical request/resource construction, mutation idempotency requirement, response byte cap, fail-closed conformance input, locked toolchain, and current mock/live characterization. Split directly into client/endpoint/error/operation/retry/session/subscription/transport modules; generate one complete runtime binding per descriptor; reject invalid request/response/status/media/identity; make credentials opaque; implement semantic certainty/cancellation; reject redirects; add authenticated bounded Node/browser transport; and ship deterministic ESM JavaScript/declarations through the signed offline distribution. Replace the direct-seeded label-only harness with D-01-installed structural MX/KV/browser/cross-surface evidence. No source-export, old monolith, permissive envelope, or second browser lifecycle survives. | A-07, B-04, D-01, H-04, H-05, H-07, J-01 through J-05 |
| earlier cluster adapter domains | retain only explicit fail-closed format rejection evidence; no opener or migration path may accept the superseded domain | C-05, J-01 |
| `rrd-contract::AutomationCatalogue`, `ReplaceAutomationCatalogue`, `ListAutomationCatalogue`, and private `AutomationHead` | directly rename to `FunctionCatalogue`, `ReplaceFunctionCatalogue`, `ListFunctionCatalogue`, and `FunctionCatalogueHead`; `automation` cannot remain a wire field, key family, module authority, or catalogue that implies ownership of routines, skills, or event triggers | A-07, C-01, C-03, J-01 |
| `rrd-contract::FunctionTrigger*`, `trigger_id`, and catalogue `triggers` | directly rename to `TransactionFunctionBinding`, `TransactionMutationKind`, `TransactionFunctionEffect`, `binding_id`, and `transaction_bindings`; reserve `Trigger` exclusively for post-commit canonical engine-event predicates and reject every successful old shape | A-07, I-01, I-02, J-01 |
| `server/state/*/automation/{head-v1,revision/*}` plus whole-catalogue JSON/control-journal replacement | replace with typed function artifacts, definitions, transaction bindings, immutable catalogue membership, prepared invocation receipts, and one CAS head through `RrdEngine`; catalogue references never repeat executable bytes and public maxima must fit the physical transaction | C-01, C-03, C-04, J-01, J-02 |
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
| `docs/anytype-ui-research.md` | merge any still-valid interaction requirements into the current Connectome/public-client owner, then remove |
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
| `docs/rrflow-surrealdb-differential.md` | `docs/evidence/comparisons/rrflow-surrealdb-claim-differential.md`; preserve exact revision/harness metadata |
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

#### Remaining KB-05 execution queue

This is the bounded work order inside KB-05; it is not a second release plan
or completion ledger. Exactly one row is reviewed and committed at a time. A
row leaves this queue only when the same commit adds its resolved-review row,
structured package journal, canonical owner/index changes, generated inventory,
and required verification. A newly discovered dependency or conflicting
authority stops the row and updates this map before work continues.

| Order | Record | Review boundary |
|---:|---|---|
| 1 | `docs/rrd-java-client-v1.md` | Reconcile the generated Java projection against the shared SDK conformance boundary. |
| 2 | `docs/rrd-dotnet-client-v1.md` | Reconcile the generated .NET projection and close the SDK documentation set. |
| 3 | `docs/qdrant-capability-inventory.md` | Retain a source-pinned capability/reference inventory without importing Qdrant's product model. |
| 4 | `docs/surrealdb-capability-inventory.md` | Retain a source-pinned capability/reference inventory without importing SurrealDB's authority model. |
| 5 | `docs/rrflow-surrealdb-differential.md` | Preserve only reproducible claim-differential inputs and results after both source inventories are canonical. |
| 6 | `docs/anytype-ui-research.md` | Merge useful public-client/Connectome interaction requirements and remove UI product or lifecycle authority. |
| 7 | `docs/operations/ci.md` | Re-read the retained CI owner, verify its index and commands, and record the final KB-05 supporting-file disposition. |

After row 7, run the complete KB-05/A-06 acceptance corpus and change the
canonical roadmap checkbox only if it passes. Then execute A-07.0 traceability,
A-07.1 package/type vocabulary, and A-07.2 causal evidence vocabulary as
separate journaled packages. B-03 is the next implementation package only
after A-06 and A-07 are complete.

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

### A-07.1 — freeze package and type vocabulary

Read all 20 `Cargo.toml` files, root `Cargo.toml`, `Cargo.lock`, every crate
`lib.rs`/`main.rs`, and `workspace_architecture.rs`. Produce the reviewed
dependency table in the A-07 commit before any physical move.

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

For the Rust SDK, directly replace the 1,235-line
`rrd-client/src/lib.rs` implementation body with
`rrd-client/src/{client,endpoint,error,operation,retry,session,subscription,transport}.rs`
and retain `lib.rs` only as the narrow public export root. Preserve the current
typed operations and real-server behavior mechanically; make the credential
boundary explicit so no secret-bearing type derives an unredacted formatter.
Add the planned `operation_coverage`, `protocol_validation`, and
`transport_faults` test seams. B-04 owns multiplexing/cancellation, H-04 owns
complete catalogue binding and protocol validation, and H-07 owns resolution
and rotation; A-07 must not conceal those gaps behind forwarding methods or a
new client authority.

For the TypeScript SDK, directly replace the runtime body in
`sdks/typescript/src/index.ts` with the public export root and
`src/{client,endpoint,error,operation,retry,session,subscription,transport}.ts`.
Reserve generated `validators.ts` and the planned operation-coverage,
protocol-validation, transport-fault, package-consumer, and browser-conformance
tests for their assigned H/J gates; rename `tests/sdk_conformance.ts` directly to
`tests/sdk-conformance.ts` and update its sole orchestrator caller in the same
commit. Preserve current generation, typing, loopback, envelope construction,
byte-limit, abort, and fail-closed harness behavior. A-07 does not claim the
H-04 validation/retry/package behavior, B-04 socket, H-07 network transport, or
J qualification complete and leaves no source-export or forwarding fallback.

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
