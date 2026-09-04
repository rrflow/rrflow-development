# RRFlow 1.0 release roadmap

**Status:** active canonical release plan; 1.0.0 is frozen and the alpha baseline is not established
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0`
**Owner:** release planning; linked from the repository root `README.md`

This is the single detailed RRFlow 1.0 execution record. The repository root
[README](../../README.md) is the bootstrap knowledge map and owns product
identity and current status. Supporting design, research, and evidence records
may inform this roadmap but cannot silently change its gates or completion.

## Current alpha objective and remediation

The [RRFlow 1.0 alpha objective](../objectives/rrflow-1.0-alpha.md) owns the
measurable outcome and its current evidence classification. The
[RRFlow 1.0 alpha POA&M](../poam/rrflow-1.0-alpha.md) owns verified deficiencies
and maps each one back to the gates below. This roadmap owns only dependency
order, checkboxes, and accepted completion evidence.

Roadmap completion currently stands at:

| Gate | Purpose | Complete |
|---|---|---:|
| A | authority, naming, documentation memory, and source boundaries | 5 / 7 |
| B | public, install, routing, model, WebSocket, and GraphQL contracts | 2 / 5 |
| C | sole hybrid persistent rrflowKV substrate | 0 / 7 |
| D | per-project install, configuration, and attunement | 0 / 10 |
| E | native graph, scalar, BM25, and vector access paths | 0 / 5 |
| F | streamed Arrow/DataFusion analytical execution | 0 / 5 |
| G | LFG routing through the engine | 0 / 6 |
| H | dynamic context, feedback, delivery, tracing, and Connectome | 0 / 7 |
| I | explicit triggers, routines, hook adapters, and skills | 0 / 7 |
| J | clean release and real deployment proof | 0 / 5 |

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

The next executable item is **A-06**. B-01 and B-02 remain completed contract
work, but no further Gate B work proceeds until A-06 and A-07 correct the
pre-release documentation and source-boundary assumptions.

### Gate A — freeze authority, names, and boundaries

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [x] | A-01 | Define RRFlow, RRD, RRFlow kernel, rrflowKV, rrflowMX, RRFlowQL, Arrow/DataFusion analytical path, LFG, and Connectome exactly once. | `README.md` | Terminology table, execution topology, and ownership table use one meaning for every term. |
| [x] | A-02 | Define generic `reasoning_tree`, `reasoning_node`, typed `reasoning_edge`, recipe, active cursor, decision evidence, and verification-result semantics. | `rrd-contract`, `rrd-core` | Versioned schema and golden round trips reject unknown fields, invalid edges, and unverifiable cursor advances. |
| [x] | A-03 | Resolve the pending reasoning-ledger removal against A-02 without restoring a hard-coded universal reasoning lifecycle or deleting reusable semantics. | `rrd-core`, `rrd-engine`, CLI | Golden/API diff proves reusable data moved to the generic contract, contains no forced Goal→Plan→Attempt sequence, and focused core, engine, and CLI tests pass. |
| [x] | A-04 | Move all existing crates into one non-duplicated grouped source tree, remove the empty `rrd-graph` boundary, and remove `connectome-ui` after its public-client behavior is present in the separate Connectome repository. | workspace | `cargo metadata`, dependency-direction check, and repository search show the declared layout and no second graph, memory, routing, lifecycle, UI, or provider authority. |
| [x] | A-05 | Remove stale documentation claims or mark supporting documents historical where they describe another architecture. | documentation | Repository link/terminology check finds no supporting document presented as current authority. |
| [ ] | A-06 | Establish the documentation memory topology: the root and each major source-boundary README are warp maps into one owning `docs/<subject>/` record set; classify every flat document without duplicating content and preserve stable coordinates for later rrflowKV ingestion. | documentation | CI proves every active record has status, owner, stable coordinate, one inbound owner link, valid local fallback links, and no duplicate roadmap or architecture body. |
| [ ] | A-07 | Audit the actual dependency graph and public vocabulary, then freeze industry-aligned directory, crate, module, test, fixture, and binary names; rename or remove overlapping pre-release boundaries directly with no aliases or compatibility shims. | workspace | Reviewed boundary map plus `cargo metadata`, dependency-direction tests, repository terminology search, and owning suites prove every package has one responsibility and every dependency points inward. |

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
  the workspace architecture test compares every package to its declared
  manifest path and rejects any additional top-level crate group. A-07 must
  still validate and, where necessary, replace those group names.
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

- Every supporting semantic, design, implementation, evidence, research, and
  history document declares status in its first 12 lines; the root README owns
  product identity and the knowledge map, while this linked record alone owns
  the detailed RRFlow 1.0 release checklist.
- Superseded runtime, in-repository Connectome, provider-flight, Clyffy alpha,
  and architecture-triage documents are explicitly historical. Active status
  lines cannot use the retired F/G/M/Q milestone scheme.
- Stale alternate-authority claims, the removed UI/graph layout, obsolete CI
  package counts/topology, a hard-coded branch, and five broken local document
  links were corrected.
- `scripts/ci/check_documentation.py` validates the README knowledge map,
  designated roadmap ownership, supporting status, prohibited alternate
  authority/layout claims, and repository-local links in CI. The documentation policy, CI policy, Ruff, `git diff --check`,
  and all 16 workspace-architecture tests passed.

Gate A exits only when the worktree contains one architecture, one navigable
documentation memory, an industry-aligned and dependency-checked source tree,
the generic tree contract, and no obsolete lifecycle implementation.

### Gate B — freeze public and model-neutral contracts

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [x] | B-01 | Define install plan, installation result, attunement plan, job, phase checkpoint, status, resume, cancel, and verification envelopes. | `rrd-contract` | Golden JSON and generated schema tests cover every state transition and reject skipped phases or mismatched digests. |
| [x] | B-02 | Define `RouterBackendDescriptor`, `RouteStepRequest`, and the `select_recipe`, `advance_branch`, and `request_context` decision variants. | `rrd-contract` | Golden vectors prove model/provider neutrality, strict fields, bounded inputs, and stable digests. |
| [ ] | B-03 | Define the LFG model-manifest handshake: model/tokenizer digests, routing schema digest, capabilities, limits, runtime, and quantization. | `rrd-contract`, `rrd-inference` | Mismatched contract, model, tokenizer, or resource declarations fail before inference. |
| [ ] | B-04 | Define one multiplexed WebSocket frame protocol for authenticated request/response, cancellation, subscription, ACK, and backpressure. | `rrd-contract` | Codec golden tests prove correlation, ordering, limits, unknown-frame rejection, and reconnect resume coordinates. |
| [ ] | B-05 | Define GraphQL as a schema-derived ingress adapter that lowers into the same bound RRFlow query representation. | `rrd-contract`, `rrd-query` | Equivalence fixtures show GraphQL and RRFlowQL produce the same authorized logical request without a second executor. |

B-01 evidence (2026-09-04):

- `rrd-contract` defines strict provider-neutral installation plans/results and
  attunement plans, jobs, leases, phase checkpoints, status, resume, cancel,
  and verification payloads. They contain no provider hook, secret, local
  path, or client-owned lifecycle state.
- The canonical phase order is connect, inventory, parse, normalize,
  entity-link, lexical-index, embed, vector-index, graph, ground, and verify.
  Checkpoints must be an exact prefix, bind the plan configuration and runtime
  coordinates, and chain each input digest to the preceding output digest.
- `install-attunement-v1.json` freezes every payload shape, all seven job
  states, and all 12 permitted transitions. The transition matrix test accepts
  valid snapshots for those 12 transitions and rejects every other one of the
  49 possible state pairs; rejection tests cover phase skips, content,
  configuration, chain, revision, retry, and verification digest drift.
- All 45 `rrd-contract` tests, strict package Clippy, the implementation-free
  architecture test, and `cargo check --workspace --all-targets --locked`
  passed. No engine, persistence, endpoint, SDK, CLI, or Connectome behavior
  changed in B-01.

B-02 evidence (2026-09-04):

- `RouterBackendDescriptor` declares only a canonical identity, revision,
  supported decision kinds, hard dispatch limits, and a content digest. Model,
  tokenizer, runtime, and quantization bindings remain owned by B-03.
- `RouteStepRequest` reuses the A-02 reasoning cursor, recipe, edge, and
  condition types. It supplies bounded scalar signals, candidate-closed
  decisions, an exact deadline, and an optional context allowance whose scope
  must equal the cursor read scope.
- `select_recipe` can return bounded typed parameters only for an offered
  recipe revision; `advance_branch` can select only an offered outbound edge
  and cannot claim condition evaluation; `request_context` can only narrow
  offered seeds and resource budgets and cannot select a storage key, field,
  index, vector backend, authorization, physical plan, or mutation.
- `router-contract-v1.json` freezes the descriptor, stamped request, all three
  decisions, and their SHA-256 bindings. Six focused tests cover closed
  generated schemas, unknown-field rejection, provider/model neutrality,
  encoded-byte and nesting bounds, candidate/budget containment, request
  binding, and deadlines.
- All 51 `rrd-contract` tests, strict package Clippy, the implementation-free
  architecture test, and `cargo check --workspace --all-targets --locked`
  passed. No inference runtime, engine, endpoint, SDK, CLI, or Connectome
  behavior changed in B-02.

Gate B exits only when other languages and LFG can implement the contracts from
golden vectors without importing Rust internals.

### Gate C — make rrflowKV the only local persistent substrate

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | C-01 | Freeze one ordered binary key codec for current records, temporal versions, outgoing/incoming edges, scalar values, term postings, vectors, projection deltas, catalogue state, and runtime commits. | `rrd-core`, `rrd-store` | Ordering/golden tests prove prefix boundaries, round trips, tenant separation, and malformed-key rejection. |
| [ ] | C-02 | Expose the minimal snapshot transaction primitives required by the semantic store: point read, bounded range scan, put, delete, commit, rollback, and conflict. | `rrd-lsm`, `rrd-store` | rrflowKV and rrflowMX conformance suites agree on read-your-writes, repeatable reads, range ordering, and write conflicts. |
| [ ] | C-03 | Commit canonical record, relation, both adjacency directions, synchronous index changes, runtime log entry, and durable projection deltas as one write batch. | `rrd-store`, `rrd-engine` | Failure injection at every WAL/batch boundary proves all-or-nothing behavior after reopen. |
| [ ] | C-04 | Serve current and temporal reads from direct versioned keys at one `ReadStamp`; remove normal-path whole-log reconstruction. | `rrd-store` | Physical counters and plan evidence show bounded point/range reads while exact snapshot comparisons remain equal. |
| [ ] | C-05 | Remove Fjall selection, compatibility readers, migration-only runtime paths, legacy format branching, and associated dependencies from the 1.0 executable. | `rrd-store`, workspace | Fresh native database tests pass; repository search and dependency metadata contain no Fjall/compatibility execution path. |
| [ ] | C-06 | Replace row-record immutable segments with the hybrid rrflowKV layout: an ordered key/version spine plus Arrow-compatible column pages, explicit encoding/compression metadata, and safe buffer lifetimes. Keep point/range/CAS reads independent of DataFusion. | `rrd-lsm`, `rrd-store` | Frozen format vectors, property tests, exact differential reads, selective-scan counters, and comparative benchmarks prove the new layout; eligible uncompressed/aligned pages borrow buffers while all decoded, copied, and allocated bytes are reported. |
| [ ] | C-07 | Prove WAL recovery, manifest recovery, pinned-snapshot compaction, Arrow-page lifetime safety, checksums, storage-full behavior, and acknowledged-write durability. | `rrd-lsm` | Crash matrix, reader/compaction concurrency, and reopen suite pass repeatedly with no lost acknowledged write, dangling mapped buffer, or exposed partial batch. |

Gate C exits only when rrflowKV is the sole local persistent implementation and
its correctness is demonstrated below the semantic engine.

### Gate D — install, configure, and attune one real estate

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | D-01 | Implement `rrflow install` by resolving a versioned generic project-bootstrap template, provider-neutral specialization manifest, and attunement profile, with explicit new/existing-project modes, preview/apply behavior, and a minimal `.rrflow/config.toml` locator containing no canonical mutable state or plaintext secret. | `rrflow-cli`, `rrd-engine` | Template golden tests prove `AGENTS.md` is the single instruction body and any supported provider files are forwarding stubs; fresh and existing project tests preserve user content, initialize, authenticate, close, and reopen the same instance; preview performs no writes. |
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
| [ ] | F-01 | Replace pre-materialized `Vec<QueryRow>` snapshots with a stamped `RrflowKvTableProvider` streaming bounded Arrow `RecordBatch` values from rrflowKV memtables and immutable segment pages. DataFusion receives borrowed buffers only when the physical encoding is eligible and receives pool-owned decoded buffers otherwise. | `rrd-query`, `rrd-store` | Provider tests prove batch streaming, buffer lifetime safety, fixed memory bounds on data larger than query memory, and exact results across borrowed and decoded paths. |
| [ ] | F-02 | Push supported projection, predicate, limit, and ordering requirements into rrflowKV key/page scans; report unsupported predicates honestly and account for physical I/O, decoded, copied, and allocated bytes. | `rrd-query`, `rrd-store` | Explain and physical-counter tests show less I/O and decoding for selective queries while results equal the unoptimized oracle; no test equates memory mapping with universal zero-copy. |
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
| [ ] | I-07 | React to committed inventory, schema, dependency, workload, and failure signals by scheduling only the required incremental attunement phases and evaluating eligible capability, routine, and skill activation under the estate's explicit policy. | `rrd-engine`, `rrd-attunement` | Adding one language, framework, data source, or recurring failure triggers the minimal bounded work, survives restart, records its decision evidence, and never performs a blanket reinstall or unauthorized activation. |

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
