# RRFlow 1.0 code execution map

**Status:** active supporting execution map; it cannot mark a release gate complete
**Coordinate:** `rrflow://rrflow-instance/data/execution-map/rrflow-1.0`
**Owner:** file, symbol, dependency, test, and stop-condition mapping for the canonical RRFlow 1.0 roadmap
**Source baseline revision:** `79d81b0`
**Reviewed:** 2026-09-06

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

1. Verify the preceding roadmap item is complete and the worktree contains no
   unexplained changes.
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
| rrflowKV | RRFlow's local durable physical profile | `rrd-lsm` plus the native physical implementation in `rrd-store` |
| rrflowMX | RRFlow Memory Execution, the non-durable profile behind the same transaction contract | the in-memory implementation in `rrd-store`; not Dragonfly, Redis, or a cache |
| rrflowQL | parse, bind, logical plan, physical selection, native operators, and DataFusion analytical execution | `crates/compute/rrd-query` |
| Arrow substrate | the typed columnar batch/buffer interchange for analytical execution | Arrow crates used by rrflowKV page readers, native operators, rrflowQL, and DataFusion |
| DataFusion | compute-only analytical planner/executor | a dependency of `rrd-query`; no direct commit, authorization, or lifecycle authority |
| RRFlow vector subsystem | exact vectors plus payload indexes, HNSW/quantized projections, filtered candidates, and exact reranking | `rrd-vector` plus atomic canonical/index deltas in `rrd-store` |
| LFG | one replaceable local routing-model adapter | `RouterBackend` in `rrd-inference`, adapter in planned `rrflow-lfg`, orchestration in `RrdEngine` |
| external project data | PostgreSQL, Turso, Dragonfly, SQL stores, object stores, and other operator/application systems | discovered source descriptors and explicit adapters; never implicit rrflowDB persistence |

The internal trait currently named `rrd_store::Engine` overlaps mentally with
`RrdEngine`. A-07 renames it directly to `StorageEngine`; C-02 then narrows
that same port to transactional point/range primitives plus semantic
repositories. The final code must not retain `Engine`, `NativeEngine`,
`PersistentEngine`, or `EngineBox` as aliases. The intended concrete names are
`RrflowKvStore`, `RrflowMxStore`, and `StorageProfile`.

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
│   ├── rrflow-lfg               # planned model adapter
│   ├── rrflow-host-events       # planned explicit host translators
│   ├── rrflow-mesh              # planned endpoint resolver only
│   └── rrflow-devforge          # planned CoW/mount/hibernation adapter
├── operations/
│   ├── rrd-cluster
│   ├── rrd-kubernetes
│   ├── rrd-maintenance
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
       -> inventory -> parse -> normalize -> entity-link -> lexical-index
       -> embed -> vector-index -> graph -> ground -> verify
       -> commit each digest-bound checkpoint through RrdEngine
       -> close/reopen/readback

committed project signal -> determine affected phases only
       -> policy eligibility -> explicit capability/routine/skill decision
       -> bounded resumable phase work (never a blanket reinstall loop)
```

## Current implementation inventory and exact disposition

| Boundary | Current code that is real | Required convergence |
|---|---|---|
| `rrd-core` | canonical IDs, scopes, runtime values/mutations, read stamps, bitemporal values, reasoning-tree contract, trace links | keep semantic types; move physical key encoding out of `key.rs` during A-07/C-01; add only event primitives proven generic |
| `rrd-lsm` | WAL, version chains, snapshots, manifest/CURRENT, immutable row-block segments, cache, compaction, snapshot bundles, I/O tiers, failure injection | add transaction conflicts; replace current segment format with key spine + column pages; delete every pre-1.0 reader; prove crash/lifetime safety |
| `rrd-store` | Fjall `Store`, native store, rrflowMX, common semantic trait, runtime commits, current projections, archives/backups/object tiers | remove Fjall and backend selectors; split the giant trait; freeze binary keys; make semantic batch/index maintenance atomic; direct stamped reads |
| `rrd-query` | parser, binder, plan, index catalogue, BM25 implementation, DataFusion execution, spill pool, live polling | replace eager `Vec<QueryRow>` and `MemorySource`; add streaming provider/pushdown/native operators; replace two-snapshot live diff |
| `rrd-vector` | exact oracle, immutable segments, HNSW, filtered planning, catalogues, compact dense artifacts, quantization, and accelerator code | bind all artifacts to canonical source cursors; persist atomic deltas; exact-rerank; remove alternate TurboQuant catalogue paths; benchmark codecs before retaining them |
| `rrd-inference` | provider-neutral embedding jobs/backend and local FastEmbed adapter | add manifest handshake and separate `RouterBackend`; LFG remains an outward adapter |
| `RrdEngine` | one opening/composition authority, security/session/query/data/vector/context/retrieval/subscription/function operations | add persisted attunement/routing/events/routines/skills; make all paths use the final transaction/index/provider contracts; do not add transport imports |
| transport/adapters/SDKs | HTTP, subscriptions, public client, MCP, CLI, five generated SDK surfaces | freeze multiplex WS and GraphQL lowering; prove cross-surface conformance only after storage/query semantics |
| estate/cluster/operations | substantial local process, backup, recovery, cluster, Kubernetes, maintenance, and operator-source foundations | retain as later-gate inventory; prevent these packages from creating semantic or storage authority |

## Implementation-requirements traceability

This is the initial current-tree accounting required by A-07 and POAM-014. It
is updated in a documentation-only change before a listed implementation file
moves or a newly discovered behavior expands a gate. A row identifies work
that must be carried into the one target system; it does not mark that work
complete.

| Capability family | Current implementation that must be read in full | Characterization inventory that must be preserved or strengthened | Canonical convergence | Gates |
|---|---|---|---|---|
| rrflowKV WAL, MVCC, manifests, recovery, compaction, hot reads, block filtering, and decoded-block caching | `rrd-lsm/src/{wal,memtable,database,manifest,segment}.rs`; `rrd-store/examples/ai_hotset_benchmark.rs` | `rrd-lsm/tests/{wal,mvcc,manifest,failure_matrix,compaction,segment,snapshot_memory}.rs`; `rrd-store/tests/{durability,snapshot,benchmark_evidence}.rs` | Keep the useful durability, snapshot, bounded-cache, filter, and physical-counter behavior while replacing the port and row-only segment format; prove the final key spine, Arrow pages, buffer lifetime, and measured cache decision. | C-02, C-04, C-06, C-07, F-05, J-04 |
| rrflowMX/rrflowKV semantic equivalence | `rrd-store/src/{engine,native,persistent,ds}.rs` | `rrd-store/tests/{engine,persistent,snapshot,runtime,unified_data,native_operator}.rs`; vector engine differential tests | Rename and narrow one `StorageEngine` port, then run the same transaction, point/range, snapshot, graph, index, and query corpus against `RrflowMxStore` and `RrflowKvStore`; durability assertions apply only to rrflowKV. | A-07, C-02, C-03, C-04 |
| Stamped transactions, identity, authorization, and audit | `rrd-core/src/runtime.rs`; `rrd-store/src/{engine,native,store,control}.rs`; `rrd-engine/src/engine/{transaction,query_transaction,session,security,invocation,control}.rs`; `rrd-security/src/lib.rs` | `rrd-store/tests/{control_journal,durability,snapshot}.rs`; `rrd-engine/src/engine/tests/{query_transaction,security,transaction_stamp,recovery}.rs`; `rrd-security/tests/security_authority.rs`; `rrd-server/tests/http_process.rs` | Preserve authenticated `ReadStamp`, `DataTransaction`, conflict, idempotency, audit-chain, and policy semantics while making one transaction port and one cross-surface authorization path. | C-02, C-03, H-04, H-05, J-02 |
| Deterministic embedding, vector search, compact artifacts, HNSW, quantization, and accelerator admission | `rrd-inference/src/{lib,fastembed_local}.rs`; `rrd-vector/src/{contract,catalog,exact,filter,plan,segment,compact,hnsw,quantization,accelerator,runtime}.rs`; `rrd-engine/src/engine/{inference,vector}.rs` | `rrd-inference/tests/pipeline.rs`; `rrd-vector/tests/{golden,exact_model,engine_differential,model_binding,compact_dense,online_hnsw,quantization_matrix,accelerator,recall_gate}.rs`; `rrd-engine/src/engine/tests/{vector_index,native_inference}.rs` | Keep deterministic model/provenance binding and exact oracles; commit canonical vectors and index deltas atomically, bind projections to one source cursor, filter candidates, and exact-rerank before results become authoritative. | D-05, E-04, E-05, F-03, H-01, J-04 |
| Edge packaging and public delivery | `rrd-engine/src/edge.rs`; `rrflow-edge/src/main.rs` | `rrflow-edge/tests/{offline,evidence}.rs`; `rrd-client/tests/real_server.rs`; `rrd-server/tests/http_process.rs`; `rrflow-mcp/tests/{stdio,stdio_daemon}.rs` | Retain deterministic offline artifact and provenance checks as outward packaging evidence; all reads and mutations continue through public RRD capabilities with no edge-owned engine state. | H-04, H-07, J-03, J-05 |
| Temporal graph, BM25, hybrid retrieval, and context evidence | `rrd-query/src/{bm25,index,execute,plan}.rs`; `rrd-engine/src/engine/{context,retrieval,retrieval_query}.rs` | `rrd-query/tests/{query,index_catalogue,golden}.rs`; `rrd-engine/src/engine/tests/{context,index_foundation}.rs`; `rrd-engine/tests/{runtime_query_trace,runtime_data_plane_trace}.rs` | Replace broad snapshot reconstruction with transactional adjacency/BM25/vector access paths, cost-selected at one stamp and fused with bounded deterministic evidence. | E-01, E-02, E-03, E-05, F-03, H-01, H-02, H-05 |
| Arrow/DataFusion analytical execution | `rrd-query/src/{arrow,fusion,execute,pipeline,plan}.rs` | `rrd-query/tests/{golden,query,index_catalogue}.rs` and the DataFusion-focused unit tests inside the listed source modules | Replace complete `Vec<QueryRow>` materialization with a pinned stamped provider; push supported work into rrflowKV, compose native operators, enforce one resource budget, and report every read/decode/copy/allocation. | F-01 through F-05 |
| Generic reasoning trees, routing, and governed mutation | `rrd-core/src/reasoning_tree.rs`; `rrd-contract/src/{reasoning_tree,router}.rs`; `rrd-engine/src/engine/{context,transaction}.rs` | `rrd-core/tests/{reasoning_tree_contract,reasoning_trace_link}.rs`; `rrd-contract/tests/{reasoning_tree_contract,router_contract}.rs`; `rrd-engine/src/engine/tests/{context,transaction_stamp}.rs` | Preserve the accepted generic tree and three bounded routing decisions; add persisted CAS execution, the model-manifest handshake, constrained LFG dispatch, and engine-selected physical work without a fixed lifecycle. | B-03, G-01 through G-05, H-01, H-05 |
| Installation, attunement, and explicit automation | `rrd-contract/src/attunement.rs`; `rrd-engine/src/engine/automation.rs`; `rrd-estate/src/{authority,reconcile,local_authorization,local_process,recovery}.rs`; `rrflow-cli/src/{command,dev}.rs` | `rrd-contract/tests/attunement_contract.rs`; `rrd-engine/src/engine/tests/{automation,deployment_conformance,lifecycle,recovery}.rs`; `rrd-estate/tests/{estate_authority,reconciler_recovery,local_authorization,recovery}.rs`; `rrflow-cli/tests/operator_surface.rs` | Build bundle-resident preview/apply, persisted phase jobs, incremental project specialization, canonical events, resumable routines, digest-bound skills, and optional host translators under `RrdEngine`. | D, I, J-03, J-05 |

The `<package>/...` shorthand resolves through the frozen target source tree;
for example, `rrd-lsm/src/wal.rs` means
`crates/persistence/rrd-lsm/src/wal.rs`. The generated JSONL contains the exact
repository path and complete byte/line span for every file. This matrix binds
the cross-file behavior that a mechanical inventory cannot infer.

### Mandatory direct convergence

| Current path/symbol | Final disposition | Gate |
|---|---|---|
| `rrd-store/Cargo.toml` `fjall` dependency | delete after native conformance baseline | C-05 |
| `rrd-store/src/store.rs::Store` | delete Fjall implementation | C-05 |
| `rrd-store/src/persistent.rs::{PersistentBackend,PersistentEngine}` | replace with one `RrflowKvStore::open`; no selector/alias | A-07, C-05 |
| `rrd-store/src/migration.rs` and `tests/migration.rs` | delete runtime migration surface and test | C-05 |
| `rrd-store/src/upgrade.rs` | delete the pre-release format upgrade executor; 1.0 tests create the accepted format directly | C-05 |
| `rrd-store/src/keyspaces.rs::NativeKeyCodec::{TextV1,TagV2}` | replace with one ordered typed tuple codec | C-01, C-05 |
| `rrd-store/src/native.rs::legacy_storage_key` and codec transcoding | delete; this is an existing symbol name, not a supported surface | C-05 |
| `rrd-lsm/src/segment.rs::decode_legacy` | delete when hybrid segment format lands; this is an existing symbol name, not a supported reader | C-05, C-06 |
| pre-1.0 version branching in `rrd-lsm/src/{batch,manifest,segment}.rs` | retain only the final 1.0 version; corrupt/unknown versions fail | C-05, C-07 |
| `rrd-store/src/engine.rs::Engine` | rename directly to `StorageEngine`, then narrow it in C-02; no alias | A-07, C-02 |
| `NativeEngine`, `RrflowMxEngine`, `EngineBox` | direct rename to `RrflowKvStore`, `RrflowMxStore`, `StorageProfile` | A-07 |
| `rrd-query/src/arrow.rs::ArrowSnapshot` | replace with stamped batch/page adapters | F-01 |
| `rrd-query/src/execute.rs::execute` eager loading | split into native access and streaming execution | F-01..F-04 |
| `rrd-query/src/live.rs::poll_live_query` two-snapshot diff | replace with commit-impact evaluation | H-03 |
| alternate TurboQuant catalogue/ensure surfaces | remove; keep a codec only if exact differential and benchmark gates justify it | E-04, J-01 |
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

| Current record | Planned destination/classification |
|---|---|
| `docs/anytype-ui-research.md` | merge any still-valid interaction requirements into the current Connectome/public-client owner, then remove |
| `docs/blueprint-triage.md` | merge every still-open verified deficiency into the POA&M, then remove |
| `docs/clyffy-kernel-alpha.md` | merge accepted provider-neutral orchestration requirements into the system overview or agent-bootstrap owner, then remove |
| `docs/context-path-profiler.md` | merge accepted context measurement requirements into the engine-flow owner and H/J gates, then remove |
| `docs/prompt-flight-experiments.md` | merge provider-neutral adapter requirements and reproducible evidence into their current owner, then remove |
| `docs/runtime-graph.md` | merge accepted temporal-graph semantics into the system and engine-flow owners, then remove |
| `docs/context-maintenance-v1.md` | `docs/reference/context/context-maintenance.md`; active maintenance contract after line-by-line authority audit |
| `docs/estate-control-v1.md` | `docs/reference/operations/estate-control.md` |
| `docs/instance-topology.md` | `docs/architecture/instance-topology.md`; it defines accepted logical and physical deployment topology |
| `docs/local-estate-authorization-v1.md` | `docs/reference/security/local-estate-authorization.md` |
| `docs/local-process-driver-v1.md` | `docs/reference/deployment/local-process-driver.md` |
| `docs/package-workflows.md` | `docs/reference/automation/package-workflows.md`; reconcile with I-01..I-07 before calling active |
| `docs/qdrant-capability-inventory.md` | `docs/research/qdrant-capability-inventory.md` |
| `docs/surrealdb-capability-inventory.md` | `docs/research/surrealdb-capability-inventory.md` |
| `docs/rrflow-surrealdb-differential.md` | `docs/evidence/comparisons/rrflow-surrealdb-claim-differential.md`; preserve exact revision/harness metadata |
| `docs/rrflow-rename-ledger.md` | remove after A-07 integrates any still-open naming requirement and the final vocabulary search passes |
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
| `docs/rrd-security-bootstrap-v1.md` | `docs/guides/installation/security-bootstrap.md` after D-01 aligns installation |
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

No row authorizes a blind move. The file must first be read in full, compared
to current code and its target owner, and then retained as the owner, merged
without duplicated text into that owner, or removed.

No new history destination is created by this sequence. Accepted current
knowledge moves into its one owner; unresolved work becomes a POA&M row; raw
reproducible results remain evidence; redundant narrative is removed.

### A-07.0 — implementation traceability before structural edits

Precondition: A-06 is complete. Re-run the generated file inventory, read every
current package manifest and module root, and refresh the
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

Direct renames, with compiler errors allowed between edits but not at the
package commit:

- `rrd_store::Engine` -> `StorageEngine` without a forwarding re-export;
- `NativeEngine` -> `RrflowKvStore`;
- `RrflowMxEngine` -> `RrflowMxStore`;
- `EngineBox` -> `StorageProfile`; and
- any module/file name claiming an authority it does not own is moved directly,
  with no re-export or transitional alias.

`rrd-contract/src/lib.rs` (currently 6,333 lines), `rrd-core/src/runtime.rs`,
`rrd-store/src/native.rs`, and other monoliths are split only along already
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
`rrd-store/src/native.rs`, new `rrd-store/src/key_codec.rs`, their unit tests,
and a new frozen hex fixture.

Tuple fields are length/type encoded and escape-safe. The prefix is
`format / tenant / scope / family`; families cover current, temporal,
outgoing edge, incoming edge, scalar, unique, term dictionary/stat/posting,
vector, projection delta, catalogue, runtime commit, outbox, and audit. Version
ordering is explicit and tested. `prefix_end` has property tests over all byte
values. Conceptual `*`, `~`, and `+` family notation is documentation only,
never a raw delimiter contract.

C-01 freezes the codec and switches fresh native writes/reads to it in one
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

Refactor `NativeRuntimeCommitPlan` in `rrd-store/src/native.rs` into repository
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

Delete Fjall dependency, `Store`, backend selection, migration command/API,
pre-1.0 storage key codec, upgrade executor, and all earlier
batch/segment/manifest read branches. Merge any valid requirement from the
migration documents into the current owner, then remove the duplicate
documents and success tests. Unknown or pre-1.0 physical bytes fail with one
explicit unsupported-format error. Fresh-database, corrupt-format, and native
close/reopen tests replace alternate-path success tests.

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

Do not claim Lance compatibility. Freeze RRFlow-owned binary vectors, fuzz
decoders, differential reads against the memtable oracle, and report borrowed,
read, decoded, copied, allocated, and decompressed bytes.

### C-07 — durability and lifetime matrix

Extend existing WAL, manifest, compaction, snapshot, failure-matrix, tiered-I/O,
and memory tests. Add mapped-buffer pinning tests where compaction deletes an
old generation while Arrow still owns a batch. Add ENOSPC/short-write,
checksum, torn current pointer, orphan cleanup, concurrent pinned snapshot,
and repeated crash/reopen cases. No following gate begins while any
acknowledged write can be lost or a mapped buffer can dangle.

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

### D-03 through D-05 — pure attunement phases

Create `rrd-attunement` with no store, engine, transport, provider, or secret
dependency. Implement one phase per commit in the canonical order. Each phase
accepts bounded inputs plus digest/revision metadata and returns a deterministic
proposal. `RrdEngine` commits the proposal and checkpoint.

- inventory: ignore/secret/generated/cache rules, classifications, content
  digests, bounded estimate;
- parse: Tree-sitter grammar digest/revision, incremental edit, `ERROR` and
  `MISSING` evidence retained;
- normalize/entity-link: stable identities and provenance;
- lexical/embed/vector/graph: canonical facts plus derived-index proposals;
- ground/verify: cited source links, inconsistency/unresolved-error records,
  package and project digests.

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

- I-01: freeze `EngineEvent` in `rrd-contract/src/engine_event.rs`.
- I-02: persisted triggers match committed events and may request only an
  authorized engine operation.
- I-03: routines are versioned resumable operation graphs with checkpoint,
  budget, cancel, compensation, verification, and terminal state.
- I-04: optional `rrflow-host-events` translators for Claude, Codex, Gemini,
  and a reference host emit the same typed event only after previewed explicit
  installation. One shared conformance fixture and test compare their emitted
  envelopes. No session-start hook ships by default.
- I-05: skills are immutable identity/digest instruction-resource packages;
  resolving them is context retrieval, not executing storage/lifecycle code.
- I-06: `rrflow-cli/src/automation_install.rs` and its conformance test own
  preview/apply/uninstall for optional trigger, routine, host-adapter, and
  skill scaffolding; they report exact files and records and leave canonical
  state readable.
- I-07: committed inventory/schema/dependency/workload/failure changes schedule
  only the affected attunement phases and eligible automation under policy.

The existing `engine/automation.rs` function sandbox is reusable inventory.
First extract function execution unchanged; then implement events, triggers,
and routines in separate modules. Do not rename the current synchronous
function trigger and pretend Gate I is complete.

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
  other comparative claim is forbidden until like-for-like data exists.
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
