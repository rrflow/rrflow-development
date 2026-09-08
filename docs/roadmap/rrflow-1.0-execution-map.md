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
│   ├── rrflow-lfg               # planned model adapter
│   ├── rrflow-host-events       # planned explicit host translators
│   ├── rrflow-mesh              # planned endpoint resolver only
│   └── rrflow-devforge          # planned CoW/mount/hibernation adapter
├── operations/
│   ├── rrd-cluster
│   ├── rrd-kubernetes
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
| `RrdEngine` | one opening/composition authority, security/session/query/data/vector/context/retrieval/subscription/function operations | add persisted attunement/routing/events/routines/skills; make all paths use the final transaction/index/provider contracts; do not add transport imports |
| transport/adapters/SDKs | HTTP, subscriptions, public client, MCP, CLI, five generated SDK surfaces | freeze multiplex WS and GraphQL lowering; prove cross-surface conformance only after storage/query semantics |
| estate/cluster/operations | substantial desired/observed, process, backup, recovery, cluster, Kubernetes, and operator-source foundations; estate state currently uses a public direct-store repository and monolithic JSON control value; `rrd-maintenance` is an unused standalone direct-store state-machine draft | preserve the fenced reconciliation/recovery semantics while making estate values pure and commits engine-owned; reconcile the overlapping operational hierarchy; converge reusable maintenance validation into generic routines; remove both private storage authorities rather than wrapping them |

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
| Deterministic embedding, vector search, compact artifacts, HNSW, quantization, and accelerator admission | `rrd-inference/src/{lib,fastembed_local}.rs`; `rrd-vector/src/{contract,catalog,exact,filter,plan,segment,compact,hnsw,quantization,accelerator,runtime}.rs`; `rrd-engine/src/engine/{inference,vector}.rs` | `rrd-inference/tests/pipeline.rs`; `rrd-vector/tests/{golden,exact_model,engine_differential,model_binding,compact_dense,online_hnsw,quantization_matrix,accelerator,recall_gate}.rs`; `rrd-engine/src/engine/tests/{vector_index,native_inference}.rs` | Keep deterministic model/provenance binding and exact oracles; commit canonical vectors and index deltas atomically, bind projections to one source cursor, filter candidates, and exact-rerank before results become authoritative. | D-05, E-04, E-05, F-03, H-01, J-04 |
| Edge packaging and public delivery | `rrd-engine/src/edge.rs`; `rrflow-edge/src/main.rs` | `rrflow-edge/tests/{offline,evidence}.rs`; `rrd-client/tests/real_server.rs`; `rrd-server/tests/http_process.rs`; `rrflow-mcp/tests/{stdio,stdio_daemon}.rs` | Retain deterministic offline artifact and provenance checks as outward packaging evidence; all reads and mutations continue through public RRD capabilities with no edge-owned engine state. | H-04, H-07, J-03, J-05 |
| Temporal graph, BM25, hybrid retrieval, and context evidence | `rrd-query/src/{bm25,index,execute,plan}.rs`; `rrd-engine/src/engine/{context,retrieval,retrieval_query}.rs` | `rrd-query/tests/{query,index_catalogue,golden}.rs`; `rrd-engine/src/engine/tests/{context,index_foundation}.rs`; `rrd-engine/tests/{runtime_query_trace,runtime_data_plane_trace}.rs` | Replace broad snapshot reconstruction with transactional adjacency/BM25/vector access paths, cost-selected at one stamp and fused with bounded deterministic evidence. | E-01, E-02, E-03, E-05, F-03, H-01, H-02, H-05 |
| Arrow/DataFusion analytical execution | `rrd-query/src/{arrow,fusion,execute,pipeline,plan}.rs` | `rrd-query/tests/{golden,query,index_catalogue}.rs` and the DataFusion-focused unit tests inside the listed source modules | Replace complete `Vec<QueryRow>` materialization with a pinned stamped provider; push supported work into rrflowKV, compose native operators, enforce one resource budget, and report every read/decode/copy/allocation. | F-01 through F-05 |
| Generic reasoning trees, routing, and governed mutation | `rrd-core/src/reasoning_tree.rs`; `rrd-contract/src/{reasoning_tree,router}.rs`; `rrd-engine/src/engine/{context,transaction}.rs` | `rrd-core/tests/{reasoning_tree_contract,reasoning_trace_link}.rs`; `rrd-contract/tests/{reasoning_tree_contract,router_contract}.rs`; `rrd-engine/src/engine/tests/{context,transaction_stamp}.rs` | Preserve the accepted generic tree and three bounded routing decisions; add persisted CAS execution, the model-manifest handshake, constrained LFG dispatch, and engine-selected physical work without a fixed lifecycle. | B-03, G-01 through G-05, H-01, H-05 |
| Context projection maintenance draft | `rrd-maintenance/src/lib.rs` | no focused package test and no caller outside the package; the exhaustive preservation/disposition matrix is owned by `docs/reference/context/context-maintenance.md` | Preserve source-cut inventory/accounting, proposal and evidence completeness, review attribution, optimistic conflicts, digest lineage, atomic publication, generation ownership, observation, and compensating rollback through new generic-routine acceptance tests. Replace the direct `StorageEngine`, private scope/event/repository, cursor-zero replay, JSON-wrapper records, fixed seven-stage lifecycle, hardcoded classes/reduction/token policy, and package-local API with generic I-03 routine state and authorized `RrdEngine` operations; remove the standalone crate only after every preserved/generalized matrix row is covered, with no wrapper. | A-07, C-03, C-04, H-02, H-05, I-01, I-03, J-01 |
| Installation, attunement, and explicit automation | `rrd-contract/src/attunement.rs`; `rrd-engine/src/engine/automation.rs`; `rrflow-cli/src/{command,dev}.rs` | `rrd-contract/tests/attunement_contract.rs`; `rrd-engine/src/engine/tests/{automation,deployment_conformance,lifecycle,recovery}.rs`; `rrflow-cli/tests/operator_surface.rs` | Build bundle-resident preview/apply, installed credential references, persisted phase jobs, incremental project specialization, canonical events, resumable routines, digest-bound skills, and optional host translators under the one `RrdEngine` authority. Estate provisioning and local authorization remain traced in their dedicated estate/security rows rather than duplicated here. | A-07, D, H-04, I, J-01, J-03, J-05 |
| Installed project, estate, instance, environment, and physical topology | `rrd-engine/src/runtime/{instance,mod}.rs`; `rrd-contract/src/{platform,lib}.rs`; `rrd-estate/src/authority.rs`; `rrd-server/src/{main,http/server}.rs`; `rrd-cluster/src/{lib,contract}.rs` | `rrd-engine/tests/runtime_instance.rs`; `rrd-contract/tests/platform_terminology.rs`; `rrd-estate/tests/authority_catalogue.rs`; `rrd-cluster/tests/contracts.rs`; initialization callers inventoried across server, CLI, MCP, Rust client fixtures, and engine tests | Preserve canonical IDs, strict format/input rejection, exact-root containment, foreign-store denial, digest validation, desired/observed and cluster placement/snapshot/transfer safety. Replace the three competing hierarchies with one project ↔ estate ↔ instance relationship graph, operation-specific resource paths, a minimal D-01 locator, and one engine-persisted installed-estate binding shared by rrflowMX and rrflowKV. Remove startup-created manifests, historical word-order authority, private JSON binding, and duplicate operational topology with no compatibility reader. | A-07, B-04, C-02, C-03, D-01 through D-03, D-06, H-04, H-07, J-01, J-03, J-05 |
| Local RRD process launch, readiness, observation, and shutdown | `rrd-estate/src/local_process.rs`; `rrd-estate/src/commands/rrd-deployment-catalog.rs`; `rrd-engine/src/engine/estate_control.rs`; `rrflow-cli/src/{command,dev/supervisor}.rs`; `rrflow-cli/src/bin/rrd-estate-controller.rs`; `rrd-server/src/main.rs` | local-process unit test; `rrd-estate/tests/deployment_catalog.rs`; `rrflow-cli` supervisor/controller unit tests; `rrd-server/tests/local_estate_driver.rs`; all catalogue, process-record, marker, initializer, and debug-hold callers/fixtures | Preserve typed no-shell invocation, strict relative-path validation, PID/start/image reauthentication, bounded graceful/forced stop, child reaping, prepared/effect lost-ack convergence, create-new durability patterns, and retained data. Move host code into one injected outward adapter; resolve only the installed artifact/effect plan; bind verified bytes to the executed image; authenticate challenge-bound readiness/control; commit typed plans/receipts through `RrdEngine`; bound/redact diagnostics and resources; remove both supervisors, catalogue/controller binaries, direct-store/static entry points, process JSON, marker authority, startup initialization, plaintext environment state, and all successful prior shapes. | A-07, C-02, C-03, D-01, D-02, H-04, H-05, J-01 through J-03, J-05 |
| Estate desired/observed control and external-effect reconciliation | `rrd-estate/src/{lib,authority,reconcile,backup_job,backup_reconcile,recovery}.rs`; `rrd-store/src/control.rs`; `rrd-engine/src/engine/{estate,estate_control}.rs`; estate controller adapters and server estate-read handler | `rrd-estate/tests/{authority_catalogue,estate_authority,reconciler_recovery,backup_jobs,backup_reconciliation,recovery}.rs`; `rrd-store/tests/control_journal.rs`; `rrd-engine/tests/engine_authority.rs`; `rrd-server/tests/http_process.rs`; `rrflow-cli/tests/backup_controller.rs` | Preserve monotonic desired/observed state, fencing, prepared-before-effect, receipts, lost-ack convergence, activity, backup, and recovery invariants. Make `rrd-estate` pure; split the JSON aggregate into native typed records/relations/indexes; reconcile the duplicate operational-authority hierarchy; and execute every boundary through one installed, authenticated, stamped `RrdEngine` transaction. | A-07, C-01 through C-04, C-06, C-07, D-01, D-02, E-01, E-02, F-01, F-05, H-04, H-05, J-01 through J-05 |
| Durable trace, context-path evidence, and lifecycle-named residue | `rrd-core/src/trace.rs`; `rrd-engine/src/runtime/{mod,trace}.rs`; `rrd-engine/tests/runtime_trace.rs`; `docs/package-workflows.md` | trace schema/golden tests; concurrent writer and native-reopen tests in `runtime_trace.rs`; corrected exact-argv/freshness/verification/redaction target policy in the package-workflow note | Preserve correlated start/finish/annotation evidence, incomplete-span visibility, deterministic identities, data classes, typed causal links, redaction, exact argv policy, freshness, and observable verification. Route durable emission through authorized engine operations; add selected/skipped source, cache/model-context-compaction, work/byte/token/latency, contribution, and outcome evidence sufficient to detect repeated, stale, duplicate, conflicting, missed, or lost context under a fixed comparison rubric. Reserve traces for evidence, converge lifecycle/workflow-named code directly, and move the corrected supporting policy to its KB-05 canonical path without restoring hooks or inventing hidden reasoning. | A-07, H-05, I, J-01 |

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
| `.rrflow/instance.toml`, `InstanceManifest`, `InstanceMode`, `ProjectAuthorityBinding`, `RrdEngine::{open_project_store,open_bound,bind_project_authority}`, `rrd-server initialize`, and startup/test `ensure_dedicated*` callers | preserve strict version/unknown-field rejection, canonical IDs, create-new publication, exact-root and store containment, digest checking, foreign-store denial, and no silent rebind; replace the manifest/private JSON control value with the sole D-01 `.rrflow/config.toml` locator plus an engine-persisted installed-estate binding shared by rrflowMX and rrflowKV, route fixtures through that contract, and remove every initializer/reader/export with no alias or compatibility path | A-07, C-02, C-03, D-01 through D-03, H-04, J-01, J-03, J-05 |
| `LocalProcessDriver`, `LocalDeploymentCatalog`, `rrd-deployment-catalog`, `rrd-estate-controller`, `rrflow-cli::dev::supervisor`, `supervisor.json`, process-record JSON, and ready/shutdown marker-file authority | preserve only typed no-shell arguments, strict path bounds, authenticated PID/start/image identity, child reaping, bounded graceful/forced stop, retained data, and effect-gap replay; implement them once in the outward local-process adapter over an immutable installed/fenced plan and typed engine receipt, close the artifact-digest-to-executed-image race, authenticate readiness/control, bound/redact diagnostics and resources, then delete every listed implementation/state/binary/shape with no wrapper | A-07, C-02, C-03, D-01, D-02, H-04, H-05, J-01 through J-03, J-05 |
| `rrd-contract::PLATFORM_TERMS`, generic `ResourcePath` ordering, historical vocabulary order test, `EstateAuthorityResourceKind`, `rrd-estate::AuthorityResourceKind`, and separate cluster identity/scope strings | preserve bounded typed identifiers, duplicate rejection, strict parent checks where canonical, desired/observed and receipt lineage, and cluster placement/snapshot/transfer invariants; freeze one project/estate/instance topology and operation-specific resource grammar, split job/health/secret/security/cluster concepts into their sole owners, unify identity binding, and remove active dependence on the historical table and duplicate catalogue | A-07, B-04, C-03, D-01, D-02, H-04, H-07, J-01 |
| `rrd-estate::{LocalOperatorPolicy,LocalOperatorAuthorization,LocalEstatePermission}` and `RrdEngine::*_estate_*_store` path-opening administration | preserve the seven exact least-privilege effects, strict bounds, policy-derived actor, and denial before unauthorized creation; absorb them into canonical principal grants, installed bindings, engine-observed time, and one stamped invocation/transaction path, then delete the file-policy types and arbitrary-path entry points without a wrapper | A-07, C-02, C-03, D-01, H-04, H-05, J-01 |
| `rrd-estate::EstateRepository`, storage-parameterized reconcilers, `EstateDocument`, `server/state/estate/*/document`, and nested `EstateAuthorityState` | preserve the fully enumerated desired/observed, lease/fencing, prepared/effect/observation, idempotency, activity, backup, and recovery requirements in the canonical estate-control record; make the crate pure, reconcile every overlapping topology resource, replace the monolithic JSON/control-journal replacement with native typed records/relations/indexes and one stamped transaction, then delete the direct-store API and old key/shape with no forwarding reader | A-07, C-01 through C-04, C-06, C-07, D-01, D-02, E-01, E-02, F-01, F-05, H-04, H-05, J-01 through J-05 |
| rrflowKV semantic and AI-access benchmarks | current absolute rrflowKV diagnostics; add immutable source/environment provenance and fixed-hardware thresholds before release use | C-06, F-05, J-04 |
| `rrd-query/src/arrow.rs::ArrowSnapshot` | replace with stamped batch/page adapters | F-01 |
| `rrd-query/src/execute.rs::execute` eager loading | split into native access and streaming execution | F-01..F-04 |
| `rrd-query/src/live.rs::poll_live_query` two-snapshot diff | replace with commit-impact evaluation | H-03 |
| `rrd-core/tests/golden.rs` Go/bbolt/LFG parity-engine narrative | remove the provider-specific alternate-engine claim while retaining only the characterized Rust contract vectors needed until C-01 freezes the final codec; Go remains eligible only as an outward capability adapter, never an RRFlow storage or routing authority | A-07, C-01, J-01 |
| `rrd-maintenance::MaintenanceRepository` and its fixed maintenance state/event vocabulary | satisfy every preserve/generalize row in the canonical context-maintenance disposition matrix through focused generic-routine tests, then remove the crate, direct store dependency, private scope, cursor-zero replay, JSON-wrapper records, hardcoded taxonomy/reduction/token policy, and package-local API with no alias or compatibility reader | A-07, C-03, C-04, H-02, H-05, I-01, I-03, J-01 |
| `LEGACY_VECTOR_ARTIFACT_CATALOG_VERSION`, its alternate encoder/decoder, `VectorRuntime::suppress_legacy_turboquant`, and the `ensure_vector_index` compatibility adapter | preserve projection provenance, lifecycle restoration, exact oracle/rerank, mmap, bounded-memory, and recall evidence in one source-stamped vector catalogue and planner; then delete the older catalogue/version/suppression/ensure path and its successful fixtures | E-04, E-05, H-01, J-01 |
| defaulted older estate substate in `rrd-estate::EstateDocument`, optional recovery-policy decoding in `rrd-estate::{backup_job,recovery}`, and their successful missing-field fixtures | preserve current desired/observed, backup, recovery-policy, receipt, lease, and recovery-point semantics in one required canonical estate schema; remove decoding behavior and success tests that exist only for pre-release documents/jobs and add negative old-shape rejection vectors | A-07, C-05, D-02, D-10, J-01, J-02 |
| `claim-transactions` and other public transport paths described as compatibility/fallback surfaces | preserve any distinct bounded operation semantics only through the canonical multi-model transaction, subscription, and WebSocket contracts; delete duplicate capability/handler/client success paths after cross-surface conformance passes | B-04, H-03, H-04, J-01 |
| earlier cluster adapter domains | retain only explicit fail-closed format rejection evidence; no opener or migration path may accept the superseded domain | C-05, J-01 |
| `rrd-contract::AutomationCatalogue` | directly rename/narrow to the function catalogue after full contract/surface inventory; it cannot imply ownership of routines, skills, or event triggers | A-07, I-03 |
| `rrd-contract::FunctionTrigger*` | directly rename as proposed-transaction function binding types; reserve `Trigger` for post-commit canonical engine-event predicates | A-07, I-01, I-02 |
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
| `docs/package-workflows.md` | `docs/reference/automation/package-workflows.md`; reconcile with I-01..I-07 before calling active |
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
| 1 | `docs/rrd-deployment-modes-v1.md` | Author deployment profiles without creating alternate engines or persistence authorities. |
| 2 | `docs/rrd-cluster-m7.md` | Preserve useful distributed contracts, remove milestone authority, and expose unimplemented cluster behavior. |
| 3 | `docs/rrd-kubernetes-v1alpha1.md` | Keep only real Kubernetes API semantics and subordinate reconciliation to the engine contract. |
| 4 | `docs/rrd-security-bootstrap-v1.md` | Merge stable requirements into current security/install owners and remove the flat source; do not publish a task guide for unimplemented D-01 behavior. |
| 5 | `docs/clyffy-kernel-alpha.md` | Preserve only provider-neutral routing/orchestration requirements under RRFlow and remove the parallel-product framing. |
| 6 | `docs/package-workflows.md` | Reconcile package workflow semantics with the canonical event, trigger, routine, skill, and adapter boundaries. |
| 7 | `docs/rrd-functions-v1.md` | Preserve bounded function semantics while preventing a function runtime from becoming lifecycle authority. |
| 8 | `docs/rrd-rust-client-v1.md` | Establish the reference SDK behavior from the implemented Rust client and record open conformance gaps. |
| 9 | `docs/rrd-typescript-client-v1.md` | Reconcile the generated TypeScript projection against the shared contract and Rust reference behavior. |
| 10 | `docs/rrd-python-client-v1.md` | Reconcile the generated Python projection against the same contract and evidence. |
| 11 | `docs/rrd-go-client-v1.md` | Reconcile the generated Go projection without assigning Go orchestration authority. |
| 12 | `docs/rrd-java-client-v1.md` | Reconcile the generated Java projection against the shared SDK conformance boundary. |
| 13 | `docs/rrd-dotnet-client-v1.md` | Reconcile the generated .NET projection and close the SDK documentation set. |
| 14 | `docs/qdrant-capability-inventory.md` | Retain a source-pinned capability/reference inventory without importing Qdrant's product model. |
| 15 | `docs/surrealdb-capability-inventory.md` | Retain a source-pinned capability/reference inventory without importing SurrealDB's authority model. |
| 16 | `docs/rrflow-surrealdb-differential.md` | Preserve only reproducible claim-differential inputs and results after both source inventories are canonical. |
| 17 | `docs/anytype-ui-research.md` | Merge useful public-client/Connectome interaction requirements and remove UI product or lifecycle authority. |
| 18 | `docs/operations/ci.md` | Re-read the retained CI owner, verify its index and commands, and record the final KB-05 supporting-file disposition. |

After row 18, run the complete KB-05/A-06 acceptance corpus and change the
canonical roadmap checkbox only if it passes. Then execute A-07.0 traceability,
A-07.1 package/type vocabulary, and A-07.2 causal evidence vocabulary as
separate journaled packages. B-03 is the next implementation package only
after A-06 and A-07 are complete.

Resolved full-file reviews:

| Baseline record | Canonical record | Review result |
|---|---|---|
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
| `docs/rrd-security-v1.md` | `docs/reference/security/authority.md` | Preserved bounded principal/role/grant validation, API-key and bounded JWT identity, exact third-party binding, credential-revision invalidation, row/field query policy, independent audit lineage/export, and current mutual-TLS evidence. Rejected `rrd-security` as an independent policy/storage authority, obsolete RRO provisioning, Fjall compatibility claims, and missing-policy access as an accepted mode. Exposed its public direct-store repository, separate policy/data observations, operation-wide mutation authorization, standalone bootstrap, split domain/audit commits, and incomplete production transport/identity proof as POAM-016 work owned by A/C/D/F/H/J. |
| `docs/local-estate-authorization-v1.md` | `docs/reference/security/local-estate-authorization.md` | Preserved strict bounded file decoding, exact estate/action/time/key scope, seven real least-privilege effect distinctions, denial before unauthorized database creation, policy-derived journal actor, typed results, idempotent replay, quiesced backup admission, and fenced identity-based prune/restore safeguards. Rejected the file as policy truth, caller-selected authorization time and physical paths, static store-opening methods as canonical engine operations, and broad allowed audit as authorization. Proved the supposed one-authority test actually succeeds through a second local policy without a canonical estate-admin grant; POAM-017 owns direct absorption. The owning suite also exposed successful older estate/backup shapes, now tracked by POAM-018. |
| `docs/rrd-public-contract.md` | `docs/reference/protocol/public-contract.md` | Preserved the strict `rrd` protocol identity/version, explicit resource and correlation coordinates, request/response/error envelopes, mutation idempotency requirement, typed multi-model/query/vector/delivery vocabulary, deployment and capability discovery, generated endpoint/OpenAPI authority, golden samples, and shared conformance inputs. Corrected the stale 34-operation claim to the executable 33-operation catalogue and `memory` to the actual `rrflow_mx` wire value; separated exported Rust types, catalogued operations, runtime capability discovery, and behavioral proof; and mapped every contract family to the required rrflowMX/rrflowKV, native graph/BM25/vector, rrflowQL/Arrow/DataFusion, reasoning, attunement, and cross-surface gates. Full review exposed a nonexistent backup route in the frozen sample, retired gate labels in runtime capability text, and successful alternate transaction/vector branches; POAM-015 now owns their direct convergence rather than this documentation move silently changing behavior. |
| `docs/rrd-server-v1.md` | `docs/reference/protocol/server.md` | Preserved the generated catalogue/router authority, loopback and mutual-TLS boundaries, typed envelopes, exact authorization, durable idempotency and lifecycle state, restart reconciliation, WebSocket subscription semantics, and real-process evidence. Removed the copied endpoint inventory, retired milestone/status claims, stale deployment link, and any implication that callable routes prove target storage or recall behavior. The later subscription review corrected this reference to recognize the existing real-process missing-client-certificate, wrong-server-name, HTTPS, and WSS coverage while leaving untrusted-client-chain, rotation, revocation, external identity, and deployment qualification open. Exposed claim-only transactions, the non-atomic three-transition commit, one-request TLS connections, bounded follow/two-snapshot polling, incomplete trace propagation, and absent multiplexed WebSocket/GraphQL/SDK qualification as direct B/C/H/J convergence work. |
| `docs/rrd-live-subscriptions-v1.md` | `docs/reference/protocol/subscriptions.md` | Preserved the implemented immutable stream definition, one runtime-cursor order, durable ACK and outstanding-delivery state, cumulative ACK, generation fencing, reconnect/restart replay, retention floor, lease renewal, bounded backpressure, dual authorization, changefeed/live-query frames, Rust client, and WSS behavior. Removed the milestone title, compatibility language, and implication that durable transport completes live-query execution. Documented the current JSON control record, heartbeat-driven single-subscription socket, session-bound ownership, logical rather than physical retention floor, and two-snapshot live-query cost as B-04/H-03/J work; corrected the adjacent server reference against its existing mutual-TLS real-process test. |

No row authorizes a blind move. The file must first be read in full, compared
to current code and its target owner, and then retained as the owner, merged
without duplicated text into that owner, or removed.

No new history destination is created by this sequence. Accepted current
knowledge moves into its one owner; unresolved work becomes a POA&M row; raw
reproducible results remain evidence; redundant narrative is removed.

#### KB-05 package journal

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
- any module/file name claiming an authority it does not own is moved directly,
  with no re-export or transitional alias.

`rrd-contract/src/lib.rs` (currently 6,333 lines), `rrd-core/src/runtime.rs`,
`rrd-store/src/rrflow_kv.rs`, and other monoliths are split only along already
accepted responsibilities. Splitting is mechanical first; behavioral changes
remain in their later gates.

Acceptance commands:

```text
cargo metadata --format-version=1 --locked
cargo test -p rrd-engine --test workspace_architecture --locked
cargo check --workspace --all-targets --locked
rg -n 'NativeEngine|RrflowMxEngine|EngineBox|rrd_store::Engine' crates
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
`rrd-server/src/http/websocket.rs`, `rrd-client/src/lib.rs`, or subscription
code. One frame envelope owns request, response, cancellation, subscription,
ACK, heartbeat, error, and backpressure coordinates. Delete parallel frame
enums after server/client round trips pass. Do not add transport state to
`RrdEngine`.

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
ordering is explicit and tested. `prefix_end` has property tests over all byte
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
supported forwarding-only provider files. During development, inputs are
embedded in the CLI or resolved relative to an explicitly supplied, locally
verified candidate-bundle root; J-03 assembles the complete release candidate
and J-05 signs the reproducible distribution. Apply has no download or
sibling-discovery branch. `AGENTS.md` is the one instruction body. Existing
user files are never overwritten without an exact previewed action and
explicit apply. Secrets are references, not values. Empty-project and
existing-project golden fixtures run with network denied and sibling paths
absent and prove preview, apply, preservation, authentication, and
close/reopen behavior.

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

### D-06 — external source descriptors

Create `rrd-attunement/src/source.rs`, its fixture-driven source-discovery
test, and `rrd-operator-knowledge/src/source.rs`; then extend the inventory
phase. PostgreSQL, Turso/libSQL, Dragonfly/Redis, SQL configuration, object
stores, and other databases produce type/version/endpoint-reference/schema-
capability descriptors only. Credentials are redacted. Discovery never
installs, starts, or contacts a source. Adding an adapter and allowing contact
are explicit operator decisions and cannot change RRFlow persistence or
default readiness.

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
  observations, permitted fields, and budgets.
- G-03: grammar-constrain and post-validate the three B-02 decision variants;
  malformed or injected output creates no mutation.
- G-04: execute recipe/branch CAS through direct rrflowKV operations with no
  DataFusion plan.
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
  from one operation catalogue and cross-surface corpus; generated SDKs never
  invent operations.
- H-05: correlate ingress, auth, plan, KV, graph, BM25, HNSW, DataFusion, LFG,
  commit, attunement, and delivery spans; redact before export.
- H-06: keep Connectome in its separate repository and use only public RRD
  health/capability/session/query/trace/subscription operations.
- H-07: create a mesh endpoint resolver/authenticated-transport adapter after
  the operator supplies the Zuul Zero/shippin.ai protocol contract. Network
  reachability never authenticates identity and mesh never stores RRFlow data.

### Gate I — explicit automation

- I-01: freeze the one semantic `EngineEvent` contract and directly converge
  the existing kernel `RuntimeEvent` representation into it. Public submission
  lowers to `RuntimeMutation::Event`; a committed event consumed by triggers is
  that same object plus its commit receipt, never a second event database.
- I-02: persisted triggers match committed events and may request only an
  authorized engine operation. Rename the existing synchronous proposed-
  transaction function bindings so `Trigger` has no second meaning.
- I-03: routines are versioned resumable operation graphs with checkpoint,
  budget, lease, idempotency, cancel, compensation, verification, and terminal
  state. Model, process, network, and external MCP calls are recorded
  activities; replay never repeats their effect merely to reconstruct state.
- I-04: optional `rrflow-host-events` translators for Claude, Codex, Gemini,
  and a reference host emit the same typed event only after previewed explicit
  installation. One shared conformance fixture and test compare their emitted
  envelopes. No session-start hook ships by default.
- I-05: skills are immutable identity/digest instruction-resource packages;
  resolving them at a read stamp is context retrieval, not executing
  storage/lifecycle code or granting the requested capabilities.
- I-06: `rrflow-cli/src/automation_install.rs` and its conformance test own
  preview/apply/uninstall for optional trigger, routine, host-adapter, and
  skill scaffolding; they report exact files and records and leave canonical
  state readable.
- I-07: every project-development run first binds the latest complete
  authorized project-tree snapshot; missing/stale inventory returns
  `inventory-required`, changed-since-plan entries and paths outside the root
  are denied, and committed inventory/schema/dependency/workload/failure
  changes schedule only the affected attunement phases and eligible automation
  under policy.

The existing `engine/automation.rs` function sandbox is reusable inventory.
First extract function execution unchanged; then implement events, triggers,
and routines in separate modules. Do not rename the current synchronous
function trigger and pretend Gate I is complete.

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
