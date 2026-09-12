# RRFlow 1.0 alpha objectives

**Status:** active measurable outcome set; no objective is complete
**Coordinate:** `rrflow://rrflow-instance/data/objective/rrflow-1.0-alpha`
**Owner:** first usable RRFlow alpha outcome

The RRFlow 1.0 version is frozen while this baseline is established. The
repository root [README](../../README.md) owns product identity and current
maturity. The [engine data-flow architecture](../architecture/engine-data-flow.md)
owns the detailed transactional and analytical flow, the
[roadmap](../roadmap/rrflow-1.0.md) owns delivery order, and the
[POA&M](../poam/rrflow-1.0-alpha.md) owns observed deficiencies. This record
owns only the measurable result.

## Objective

Deliver one self-contained, installable per-project RRFlow estate that can
drive development, maintenance, governance, reasoning, and recall through one
persistent engine. After its signed release bundle is acquired, the default
estate installs, starts, persists, recovers, and verifies without a source
checkout, sibling repository, package registry, external database/query/vector
service, or outbound network access. The estate must remain provider-neutral:
Codex, Claude, Gemini, Grok, LFG, and future callers consume the same public
operations, authorization, read stamps, context evidence, routines, and
skills.

The default operator entry point is the natively built `rrflow` executable on
Linux/macOS and `rrflow.exe` on Windows. That one command tree must plan and
apply project installation, serve the engine, prove authenticated readiness,
verify without mutation, plan and apply bounded offline repair, verify backup
and restore to an absent target, and remove only RRFlow-owned project
integration. A checkout supervisor, companion-binary build, source-tree
diagnostic, PID, port, marker, or compile result cannot satisfy this outcome.

rrflowDB owns AI knowledge, temporal graph state, reasoning state,
indexes, evidence, automation state, and analytical storage. Application and
operator databases such as PostgreSQL, Turso, SQLite, Dragonfly, and hosted
services remain governed external sources unless an operator explicitly adds
an adapter. They never become an implicit RRFlow persistence backend.

## Required outcomes

An outcome is `complete` only after its end-to-end proof passes at an exact
revision. `Partial` means useful implementation exists but the named proof has
not passed. A type, fixture, mock, compile, or document alone is not proof.

| ID | Outcome | Current state | Completion evidence | Roadmap gates |
|---|---|---|---|---|
| OBJ-01 | One authority and vocabulary | Partial: `RrdEngine`, grouped crates, and canonical terms exist; documentation classification, implementation-requirements traceability, and the final boundary audit remain open. | Repository policy proves one identity/status owner, one objective owner, one roadmap, one POA&M, inward dependencies, complete current-source/test/fixture traceability into the accepted boundaries, and no competing graph, memory, query, lifecycle, or provider authority. | A |
| OBJ-02 | One hybrid rrflowKV persistence engine | Partial: WAL, MVCC memtable, manifest v3, segment v5's ordered key/version spine, six Arrow-layout buffers, and authenticated persisted row-group filters now survive reopen under one physical reader. Recovery, compaction, authenticated configurable row-group targets, a bounded page cache, prior-format rejection, a frozen segment vector, and explicit filter/read/borrow/allocation/copy evidence exist. C-06g adds an owned pinned read generation, selective projected-page stream, bounded Arrow-compatible output, exact mixed-family MVCC/reopen/compaction differential, active-manifest GC retention, and distinct segment-open/startup-reconciliation/query evidence; C-06h adds finite adversarial/property/fuzz proof. C-06i compression/value-placement/mixed-workload/cache integration and C-07 installed-path lifetime/recovery qualification remain open. | Atomic crash/reopen corpus proves WAL → MVCC memtable → immutable key/version spine plus Arrow-compatible column pages → manifest publication with one backend and one accepted physical reader. | C, J |
| OBJ-03 | Equivalent volatile and persistent semantics | Partial: rrflowMX and rrflowKV share one transaction port and C-03 semantic plan; rrflowKV recovers the exact multi-family write closure all-or-none; C-04 proves equivalent authenticated, budgeted direct semantic-version reads without normal runtime-log reconstruction; C-05 removes the accepted pre-1.0/alternate execution shapes; and C-06g preserves exact rrflowKV snapshot results through its new physical projected stream. C-06h/C-06i, final installed-path recovery/lifetime qualification, and the complete native graph/index/query corpus remain open. | The same transaction, snapshot, graph, index, and query corpus passes for rrflowMX and rrflowKV; rrflowKV additionally survives every recovery boundary. | C |
| OBJ-04 | Atomic multi-model records, graph, and indexes | Partial: C-03 atomically commits records, temporal graph state in both directions, scalar/unique entries, BM25/vector source deltas, runtime state, projection work, receipts, audit, and outbox. Incremental BM25 postings, native vector generations, bounded reads, and exact-oracle access evidence remain E work. | One transaction atomically changes records, both graph directions, scalar/unique state, BM25 postings, vectors, index deltas, and the runtime log; exact oracles remain equal after reopen. | C, E |
| OBJ-05 | Real Arrow/DataFusion analytical execution | Partial: DataFusion executes stamped Arrow batches allocated from materialized `QueryRow` values. rrflowKV segment v5 can own or borrow Arrow-compatible physical pages, persists row-group filters for point reads, and C-06g exposes a bounded synchronous projected storage stream with selective page-family and per-operation read/borrow/copy/decode/allocation evidence. The filter does not itself connect storage to DataFusion. No stamped DataFusion `TableProvider`/`ExecutionPlan` consumes that stream or emits eligible borrowed `RecordBatch` buffers yet. | A stamped provider streams bounded batches from rrflowKV, pushes eligible work down, proves conditional zero-copy with read/borrow/copy/decode/allocation counters, and enforces memory, spill, scan, time, candidate, and output limits. | F |
| OBJ-06 | Dynamic governed reasoning and recall | Partial: bounded lexical, exact-vector, graph, fusion, context evidence, reasoning-tree, and router contracts exist; native indexes, persisted tree execution, adaptive routing, and LFG dispatch do not. | One authorized request chooses fast or analytical work, retrieves through eligible graph/BM25/vector paths, fuses deterministically, advances a persisted tree through CAS, and reproduces its evidence at the same stamp after reopen. | E, G, H |
| OBJ-07 | Install, operate, repair, and attune a new or existing project | Contract only: provider-neutral install/attunement envelopes and the phase state machine exist; no primary `rrflow`/`rrflow.exe` command tree yet performs project install, serve, authenticated readiness, read-only verification, digest-bound repair, safe restore, or ownership-aware uninstall. Engine job persistence, deterministic project-tree inventory, real phase executors, bundle-resident templates, and the offline distribution also do not exist. | From one verified platform bundle and its single primary executable, plan/apply, authenticated readiness, deterministic tree snapshot, interrupt/resume, commit/query, close/reopen, read-only quick/full verify, torn-tail repair plan/apply, backup verification, restore to an absent target, and ownership-safe uninstall pass with outbound network denied and sibling repositories absent for empty and existing projects. The committed snapshot proves exact exclusions, errors, incremental changes, and bounded work; no credentials, caches, generated output, application databases, or provider lifecycle files become canonical engine state. | B, C, D, J |
| OBJ-08 | Adaptive routines, triggers, skills, and adapters | Partial foundation: synchronous governed functions and transaction bindings exist. An unused standalone context-maintenance state-machine draft also exists but is not an accepted routine implementation because it directly owns storage, scope, events, and replay. Canonical engine events, persisted event triggers, generic resumable routines, skill packages, explicit host-event adapters, and capability activation do not. | Generic packages are versioned, digest-bound, authorized, budgeted, observable, restartable, removable, and activated only from verified estate signals and policy; error resolution and context projection maintenance execute through the same routine authority with no operation-specific repository. | I |
| OBJ-09 | Provider-neutral client operation | Partial: HTTP, current WebSocket behavior, SDKs, CLI/MCP, and a separate Connectome bootstrap exist at different levels of conformance. | Every supported client and model adapter produces equivalent operation, authorization, stamp, digest, denial, trace, and restart results without embedding engine logic. | B, H |
| OBJ-10 | Reproducible competitive evidence | Not established: local component evidence exists, but no complete fixed-hardware engine or clean-rollout corpus proves the alpha. | Published raw results cover correctness, p50/p95/p99 latency, throughput, memory, copied/decoded/allocated bytes, disk/WAL/compaction, recall@k, filtered quality, restart, failure behavior, and clean-machine deployment steps/time/bytes/services against pinned declared baselines, including SurrealDB and Qdrant where the compared capability exists. | J |

## Alpha exit

The baseline exists only when all ten objectives are complete and their linked
roadmap gates carry reproducible evidence. Optimization continues within the
same frozen version until the repository owner makes an explicit release and
version decision.
