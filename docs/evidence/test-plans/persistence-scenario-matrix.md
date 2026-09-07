# RRFlow persistence, reasoning, and recall scenario matrix

**Status:** active acceptance test plan; current characterization exists, but no C, E, or F gate is complete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/test-plan/persistence-reasoning-recall`
**Owner:** executable persistence-to-context scenarios and their evidence state

This matrix defines how RRFlow proves one persistent AI governance, reasoning,
and recall engine. It does not treat compilation, a type, a generated artifact,
or a high-level reopen result as proof of the intended physical access path.
The [release roadmap](../../roadmap/rrflow-1.0.md) owns gate order and completion;
the [execution map](../../roadmap/rrflow-1.0-execution-map.md) owns file-level
implementation instructions.

## Evidence vocabulary

| State | Meaning |
|---|---|
| Characterization | The named test exists and protects useful current behavior. It can expose work that a rewrite must preserve, but cannot close a target gate by itself. |
| Planned | The owning roadmap gate requires the scenario, but its accepted implementation and evidence do not exist yet. |
| Accepted | The owning roadmap row cites retained output at an exact revision. No row in this matrix currently has this state. |

Every scenario runs at least one test in its own process with zero failures and
no hidden retry. A filtered-out invocation is not a pass. Failure artifacts and
resource counters are retained. Platform evidence applies only to the platform
that executed it; fixed-hardware performance and competitive results belong to
J-04, never to a correctness test.

## Current characterization corpus

These tests are real and valuable. Their limitations are equally important.

| ID | Existing test | Behavior protected | Why it is not target completion |
|---|---|---|---|
| CHAR-01 | `crates/persistence/rrd-store/tests/rrflow_kv_open.rs::missing_paths_create_rrflow_kv_and_reopen_by_authenticated_marker` | A missing root initializes rrflowKV, persists a claim, and reopens through `CURRENT`. | Does not reject every pre-1.0 batch, manifest, and segment reader required by C-05. |
| CHAR-02 | `crates/persistence/rrd-store/tests/unified_data.rs::unified_transaction_and_evidence_match_across_storage_profiles` | rrflowMX and rrflowKV agree on one transaction containing claims, records, relations, events, dense vectors, series, geo, and immutable-object references. | Current semantic planning still uses JSON keyspaces and reconstructed runtime state; native adjacency/index keys are not proved. |
| CHAR-03 | `crates/persistence/rrd-store/tests/unified_data.rs::rrflow_kv_unified_evidence_survives_reopen_and_retry` | The rrflowKV commit outcome, projection outbox, audit stamp, immutable payload reference, and idempotent retry survive reopen. | Does not prove that every authoritative family and synchronous index update share the C-03 physical batch. |
| CHAR-04 | `crates/authority/rrd-engine/src/engine/tests/recovery.rs::commit_reopens_replays_and_does_not_duplicate_claims` | `RrdEngine` session and transaction state reopens and exact commit retry does not duplicate a claim or journal transition. | Protects current transaction recovery, not future reasoning-tree or attunement-job persistence. |
| CHAR-05 | `crates/persistence/rrd-store/tests/bitemporal.rs::a_correction_at_the_same_valid_from_preserves_the_claim_it_corrects` | Transaction-time corrections retain superseded valid-time state. | The normal query path can still reconstruct history from the runtime log instead of bounded version-key reads. |
| CHAR-06 | `crates/persistence/rrd-store/tests/snapshot.rs::{retained_prefix_proofs_survive_later_commits_on_all_engines,rrflow_kv_snapshot_leases_pin_physical_manifests_until_release_or_expiry}` | Authenticated read stamps behave the same on rrflowMX/rrflowKV, and durable leases pin rrflowKV manifests. | The proof method currently performs full hash-chain replay, and no Arrow-page lifetime exists to pin. |
| CHAR-07 | `crates/compute/rrd-query/tests/index_catalogue.rs::rrflow_kv_catalogue_reopens_and_invalid_fields_fail_before_control_state_changes` | A query-index definition and artifact reopen; invalid definitions do not change catalogue state. | The artifact is a rebuildable projection over eager rows, not a C-03/E transactional native index. |
| CHAR-08 | `crates/compute/rrd-query/tests/live_query.rs::semantic_deltas_are_identical_across_every_engine` | rrflowMX and rrflowKV produce identical bounded semantic add/update/remove results. | The implementation compares two materialized snapshots; H-03 still requires commit-impact deltas. |
| CHAR-09 | `crates/authority/rrd-engine/src/engine/tests/vector_index.rs::persistent_retrieval_indexes_and_hybrid_fusion_survive_reopen_and_staleness` | HNSW, TurboQuant, BM25/vector fusion, exact overlay, staleness handling, and deterministic replay survive reopen. | Candidate discovery can scan the runtime log, BM25/HNSW are separate projection paths, and this test still exercises a compatibility-named TurboQuant ensure route scheduled for direct convergence. |
| CHAR-10 | `crates/authority/rrd-engine/src/engine/tests/vector_index.rs::unified_retrieval_algebra_executes_multimodal_late_interaction_and_analytics` | Dense, sparse, named image, and multi-vector MaxSim branches compose through nested RRF, exact/model rerank, score boost, MMR, grouping, facets, matrix output, and reopen. | Fusion and result shaping are eager in-memory engine work; there is no graph leaf, persisted reasoning-tree execution, or one native Arrow/DataFusion physical plan. |
| CHAR-11 | `crates/authority/rrd-engine/src/engine/tests/vector_index.rs::application_backup_restores_turboquant_payload_before_instance_activation` | An engine-authorized application backup restores canonical state, catalogue state, and required immutable TurboQuant bytes before a restored root is opened. | Backup/recovery is not the online index or DataFusion path and does not retain derived Arrow, BM25, or HNSW pages. |
| CHAR-12 | `crates/persistence/rrd-store/tests/{logical_archive,logical_archive_resilience,backup_catalogue}.rs` | Stable-cut logical replay, every typed runtime family, interruption resume, corruption denial, and the optional object/catalogue closure are tested. | Projections are explicitly rebuild-required; invocation telemetry, leases, and physical storage pages are excluded. |
| CHAR-13 | `crates/authority/rrd-security/tests/security_authority.rs::policy_is_persistent_exact_scope_and_deny_by_default` | Credentials, exact resource scope, policy revision, and default denial survive rrflowKV reopen. | Cross-surface `RrdEngine` authorization and field/row enforcement remain H-04/H-05 work. |

The removed `native_format_upgrade` and Fjall `migration` scenarios are not
carried forward. Their tests and runtime entrypoints no longer exist, and C-05
and J-01 require direct rejection of pre-release formats rather than a
compatibility reader or migration executor.

## Required persistent-substrate proof

All rows below are `Planned` until the owning roadmap checkbox cites exact
evidence.

| Gate | Scenario | Planned test owner | Required observation |
|---|---|---|---|
| C-01 | Ordered key codec | `crates/persistence/rrd-store/tests/key_codec.rs` | Golden/property corpus proves tenant-safe round trip and lexicographic prefix/range boundaries for record versions, both edge directions, scalar values, terms, vectors, projection work, catalogue state, and commits; malformed keys fail. |
| C-02 | rrflowMX/rrflowKV snapshot transaction conformance | `crates/persistence/rrd-store/tests/storage_profile_conformance.rs`; `crates/persistence/rrd-lsm/tests/transaction_conformance.rs` | Point/range, read-your-writes, repeatable read, conflict, delete, rollback, and write-skew-policy results agree; only rrflowKV is expected to reopen. |
| C-03 | One semantic write batch | `crates/persistence/rrd-store/tests/semantic_commit_atomicity.rs` | Record/version, relation, outgoing/incoming adjacency, scalar/BM25/vector index changes, runtime entry, projection delta/outbox, audit, and cursor are all visible or all absent at each injected WAL/sync/publication failure. |
| C-04 | Direct versioned access | `crates/persistence/rrd-store/tests/direct_read_paths.rs` | Current, valid-time, transaction-time, record, relation, vector, and runtime reads touch bounded ordered ranges at one `ReadStamp`; normal execution performs no cursor-zero reconstruction. |
| C-05 | One accepted physical reader | `crates/persistence/rrd-store/tests/rrflow_kv_open.rs`; `crates/persistence/rrd-lsm` format tests | Dependency/symbol searches find no alternate store, selector, upgrader, or migration executor; old batch/manifest/segment bytes fail with one unsupported-format error. |
| C-06 | Hybrid immutable generation | `crates/persistence/rrd-lsm/tests/hybrid_segment.rs` | A sorted memtable flushes to one ordered key/version spine plus typed Arrow-compatible pages; point reads use the spine, projected scans read only selected pages, and exact results equal the memtable oracle. |
| C-07 | Recovery and buffer lifetime | `crates/persistence/rrd-lsm/tests/mapped_page_lifetime.rs`; existing failure/maintenance suites | WAL/manifest/compaction/reopen, ENOSPC, short write, checksum, torn pointer, and orphan cases lose no acknowledged batch; a pinned Arrow buffer remains valid after old-generation reclamation; maintenance/backpressure stays bounded. |

## Required native graph and recall proof

| Gate | Scenario | Planned test owner | Required observation |
|---|---|---|---|
| E-01 | Temporal graph traversal | `crates/compute/rrd-query/tests/native_operators.rs`; `crates/persistence/rrd-store/tests/direct_read_paths.rs` | Directed, typed, valid/known-time, depth/step-bounded traversal equals the exact graph oracle and scans in/out adjacency proportional to visited edges. |
| E-02 | Scalar and unique indexes | `crates/persistence/rrd-store/tests/semantic_commit_atomicity.rs`; `crates/compute/rrd-query/tests/native_operators.rs` | Insert/update/retire/conflict/reopen cannot expose an index-record mismatch; unique denial is checked against the prospective transaction. |
| E-03 | Incremental BM25 | `crates/compute/rrd-query/tests/native_operators.rs` | Dictionary, document length/statistics, postings, positions, and tombstones advance from one source cursor; update/delete/reopen equals a clean exact rebuild. |
| E-04 | Native vector candidate path | existing `crates/compute/rrd-vector/tests/` exact/HNSW/quantization suites plus `crates/compute/rrd-query/tests/native_operators.rs` | Canonical vector and index delta commit together; immutable HNSW plus exact delta overlay, payload filtering, and final exact rerank meet declared recall across update/delete/stale/reopen/interrupted-build cases. |
| E-05 | Engine-selected access | `crates/compute/rrd-query/tests/native_operators.rs` | Stable explain plans choose point, range, scalar, BM25, exact vector, or HNSW from stamped statistics; absent/stale projections fall back correctly, and callers cannot select storage internals. |

## Required Arrow/DataFusion proof

| Gate | Scenario | Planned test owner | Required observation |
|---|---|---|---|
| F-01 | Stamped streaming provider | `crates/compute/rrd-query/tests/provider_streaming.rs`; `crates/persistence/rrd-lsm/tests/mapped_page_lifetime.rs` | `RrflowKvTableProvider` yields bounded `RecordBatch` streams from one pinned stamp for an input larger than query memory; eligible buffers are borrowed and decoded buffers are pool-owned with safe lifetime. |
| F-02 | Projection/filter/limit pushdown | `crates/compute/rrd-query/tests/provider_streaming.rs` | Selective queries read and decode fewer keys/pages/columns while matching the unoptimized oracle; counters distinguish read, mapped, borrowed, decoded, decompressed, copied, and allocated bytes. |
| F-03 | One hybrid physical plan | `crates/compute/rrd-query/tests/native_operators.rs` | Graph, BM25, exact/HNSW vector, and `math::rrf()` operators exchange stamped Arrow batches inside one plan, preserve deterministic rank/tie order, and perform final exact rerank without an external database. |
| F-04 | End-to-end resource budget | `crates/compute/rrd-query/tests/resource_budgets.rs` | Memory, spill, elapsed time, scanned keys, graph steps, candidates, and output bytes are enforced across native and DataFusion operators with deterministic denial or explicitly marked truncation. |
| F-05 | Stamp-safe caches | `crates/compute/rrd-query/tests/resource_budgets.rs`; engine query tests | Read-stamp/query/projection reuse is byte-bounded and keyed by scope, cursor, schema, catalogue, plan, and parameters; mutation or schema change cannot return a stale batch. |

DataFusion computes and transforms; it never commits around `RrdEngine`.
rrflowKV remains the authoritative persistent substrate, and rrflowMX remains
the volatile semantic-conformance profile. Immutable media bytes remain in the
content-addressed object boundary while their references and provenance commit
with multi-model state.

## Required persistent reasoning and context proof

| Gate | Scenario | Planned test owner | Required observation |
|---|---|---|---|
| G-04 | Fast-path reasoning-tree advance | `crates/authority/rrd-engine/tests/routing_conformance.rs` | An authorized recipe/branch decision performs bounded rrflowKV reads and one CAS mutation, survives reopen, and creates no DataFusion plan. |
| G-05 | Analytical request lowering | `crates/authority/rrd-engine/tests/routing_conformance.rs` | LFG supplies semantic context intent only; `RrdEngine` authenticates and stamps it, and rrflowQL selects physical work. |
| H-01 | Dynamic context routing | `crates/authority/rrd-engine/tests/context_flow.rs` | One request selects or skips seed, graph, BM25, vector, and cached avenues by measured budget/selectivity and records each reason, source cursor, work count, and contribution. |
| H-02 | Pure RRF plus learned future policy | `crates/authority/rrd-engine/tests/context_flow.rs` | Query-time fusion does not mutate weights; a verified outcome commits a versioned policy for later stamps, while old-stamp replay remains byte-identical and regression/rollback gates fail closed. |
| H-05 | Correlated persistent evidence | `crates/authority/rrd-engine/tests/{runtime_query_trace,runtime_data_plane_trace}.rs` | Ingress, authorization, planning, KV/page reads, graph, BM25, HNSW, DataFusion, reasoning decision, commit, and delivery share bounded correlation coordinates; traces observe persisted state and never become state. |

The first end-to-end qualifying corpus must start from an installed existing
project, commit a project-tree snapshot, ingest textual and immutable multimodal
inputs, build native graph/BM25/vector projections, close and reopen rrflowKV,
then execute both:

1. a deterministic fast-path tree advance with no DataFusion plan; and
2. an analytical context request whose single stamped plan combines graph,
   BM25, vector, and RRF work through streamed Arrow batches.

The result must include source identities, projection generations, read stamp,
plan digest, selected/skipped route evidence, resource accounting, and
verification outcome. Restarting after any committed boundary must resume
without duplicate mutation, index drift, or a client-owned lifecycle state.

## Execution discipline

For each roadmap row:

1. run the smallest named test process and prove at least one test executed;
2. fix the first failure without enabling a fallback, alias, or second reader;
3. run the owning package tests, strict Clippy, format, documentation, and
   generated-surface checks;
4. widen to the locked workspace and platform matrix only after the local gate
   is green; and
5. retain the exact revision, command, environment, counts, counters, and
   failure artifacts with the roadmap acceptance record.

Approximate recall paths additionally require an exact oracle, declared corpus
digest, recall@k/quality thresholds, and adversarial filter/update/delete cases.
Performance runs additionally require fixed hardware, warm-up/cache policy,
concurrency, percentiles, long-duration RSS, and separate logical, apparent,
allocated, cached, resident, read, decoded, copied, and allocated-byte totals.
