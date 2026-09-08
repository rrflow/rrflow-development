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
and maps each one back to the gates below. The
[system overview](../architecture/system-overview.md) owns the canonical
component and security-boundary map, and the
[engine data-flow record](../architecture/engine-data-flow.md) owns the detailed
transactional, storage, Arrow, DataFusion, and context flow. This roadmap owns
only dependency order, checkboxes, and accepted completion evidence.

The supporting [RRFlow 1.0 code execution map](rrflow-1.0-execution-map.md)
binds the unchecked gates to current files, symbols, planned paths, commands,
and stop conditions. Its generated
[file plan](rrflow-1.0-file-plan.jsonl) covers the complete repository
baseline. Neither supporting record may change the completion ledger here.

Roadmap completion currently stands at:

| Gate | Purpose | Complete |
|---|---|---:|
| A | authority, naming, documentation memory, and repository-contained source boundaries | 5 / 7 |
| B | public, install, routing, model, WebSocket, and GraphQL contracts | 2 / 5 |
| C | sole hybrid persistent rrflowKV substrate | 0 / 7 |
| D | per-project install, configuration, and attunement | 0 / 10 |
| E | native graph, scalar, BM25, and vector access paths | 0 / 5 |
| F | streamed Arrow/DataFusion analytical execution | 0 / 5 |
| G | LFG routing through the engine | 0 / 6 |
| H | dynamic context, feedback, delivery, tracing, and Connectome | 0 / 7 |
| I | explicit engine events, triggers, routines, host-event adapters, and skills | 0 / 7 |
| J | self-contained release and real deployment proof | 0 / 5 |

## RRFlow 1.0 execution checklist

This checklist is the release order, not an inventory of aspirations. Work may
not skip a gate because a later subsystem already has partial code. A checkbox
changes to `[x]` only in the same reviewed change that supplies its required
behavioral evidence.

Checklist rules:

- execute the dependency spine below; gate letters classify release outcomes
  and are not a false total order;
- keep one checklist item per coherent commit unless two items cannot be tested
  independently;
- update **Current status** when an item changes observable product behavior;
- ship one accepted RRFlow 1.0 path for each operation and physical format; do
  not retain alternate pre-release entrypoints, forwarding aliases, backend
  selectors, migration executors, or dual writes;
- recognize no legacy/deprecation class during pre-release convergence:
  classify inventory as accepted target behavior or superseded residue,
  absorb every required behavior into its canonical owner, and remove the
  competing path plus its successful old-shape fixtures in the owning gate;
- trace every affected current behavior, module, test, and fixture into its
  canonical boundary before a deletion, move, merge, or rewrite; Git ancestry,
  merge status, and compilation are not consolidation evidence;
- do not count compilation, mocked UI state, generated schemas, or an artifact
  file existing as behavioral proof;
- require exact/reference comparison before enabling an approximate index;
- require close/reopen evidence for persisted state and crash/failure evidence
  for acknowledged writes;
- require every public surface to reach the same `RrdEngine` operation; and
- stop at the first failed gate, repair it, and rerun the smallest owning test
  before continuing.

### Executable dependency spine

This is the one implementation order. It removes two circular or temporary
paths from the earlier alphabetical sequence: knowledge import cannot block
the storage and attunement code it requires, and D-05 cannot build temporary
lexical/vector/graph paths before Gates E and F establish their canonical
access and analytical boundaries.

| Wave | Work | Required exit before continuing |
|---:|---|---|
| 0 | Finish A-06 through KB-05, then execute A-07 | One deterministic checkout knowledge package; every current behavior/file/test/fixture mapped; source, package, public, and trace vocabulary frozen. |
| 1 | B-03 through B-05 | Model handshake, multiplexed WebSocket, and GraphQL lowering are provider-neutral contracts with golden equivalence. |
| 2 | C-01 through C-07 | One rrflowMX/rrflowKV transaction contract, one accepted physical reader, atomic multi-model batch, direct stamped reads, hybrid segments, and crash/lifetime proof. |
| 3 | D-01 through D-04 | Offline preview/apply, persisted jobs, deterministic committed project tree, and incremental parse evidence exist. |
| 4 | E-01 through E-05 | Graph, scalar/unique, BM25, exact vector, HNSW/quantized candidate, and planner paths are persistent, incremental, stamped, and exact-oracle checked. |
| 5 | F-01 through F-05 | rrflowKV streams bounded Arrow batches into native/DataFusion operators with honest pushdown, one resource budget, and a measured cache decision. |
| 6 | D-05 through D-10, then KB-06 and KB-07 | Every attunement phase uses the accepted C/E/F paths; external sources remain adapters; placement/accounting/hibernation pass; the checkout knowledge package imports and survives readback/recovery. |
| 7 | G-01 through G-06 | LFG can propose bounded routing decisions while `RrdEngine` retains predicates, physical planning, authorization, CAS, and persistence. |
| 8 | H-01 through H-07 | Dynamic context, pure RRF feedback, live deltas, all public surfaces, complete traces, Connectome, and mesh resolution observe one engine. |
| 9 | I-01 through I-07 | Canonical events, triggers, routines, skills, and optional host adapters are explicit persisted capabilities, not hooks or a parallel runtime. |
| 10 | J-01 through J-03, then KB-08, then J-04 and J-05 | Alternate paths are absent; failure and clean-install qualification pass; rrflowDB becomes the normal warp path; comparative evidence and the signed distribution close release. |

A-07 freezes the operation-name, causal-link, attribute, and propagation
vocabulary. Each later work package adds the trace and physical counters for
its behavior in the same change. H-05 proves complete cross-surface
correlation, export, and redaction; it does not postpone instrumentation until
Wave 8.

The next executable item remains **A-06**, specifically **KB-05**. B-01 and
B-02 remain completed contract work, but no further Gate B work proceeds until
A-06 and A-07 correct the pre-release documentation and source-boundary
assumptions.

### Gate A — freeze authority, names, and boundaries

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [x] | A-01 | Define RRFlow, RRD, `RrdEngine`, rrflowDB, rrflowKV, rrflowMX, rrflowQL, Arrow substrate, DataFusion execution, RRFlow vector and inference subsystems, LFG, and Connectome exactly once. | `README.md`, system overview, ADR-0001 | Terminology, execution topology, and authority boundaries use one meaning for every term and prohibit parallel database, model, client, or compute authority. |
| [x] | A-02 | Define generic `reasoning_tree`, `reasoning_node`, typed `reasoning_edge`, recipe, active cursor, decision evidence, and verification-result semantics. | `rrd-contract`, `rrd-core` | Versioned schema and golden round trips reject unknown fields, invalid edges, and unverifiable cursor advances. |
| [x] | A-03 | Resolve the pending reasoning-ledger removal against A-02 without restoring a hard-coded universal reasoning lifecycle or deleting reusable semantics. | `rrd-core`, `rrd-engine`, CLI | Golden/API diff proves reusable data moved to the generic contract, contains no forced Goal→Plan→Attempt sequence, and focused core, engine, and CLI tests pass. |
| [x] | A-04 | Move all existing crates into one non-duplicated grouped source tree, remove the empty `rrd-graph` boundary, and remove `connectome-ui` after its public-client behavior is present in the separate Connectome repository. | workspace | `cargo metadata`, dependency-direction check, and repository search show the declared layout and no second graph, memory, routing, lifecycle, UI, or provider authority. |
| [x] | A-05 | Remove stale documentation claims or mark supporting documents historical where they describe another architecture. | documentation | Repository link/terminology check finds no supporting document presented as current authority. |
| [ ] | A-06 | Establish the documentation memory topology: the root and each major source-boundary README are warp maps into one owning `docs/<subject>/` record set; classify every flat document without duplicating content; generate a deterministic content-addressed manifest/JSONL bootstrap package for later authorized rrflowDB ingestion. | documentation | CI proves every active record has status, owner, stable coordinate, one inbound owner link, valid local fallback links, and no duplicate roadmap or architecture body; repeated packaging produces byte-identical ordered records and digests with an explicit inclusion/exclusion ledger and no silently omitted eligible record. |
| [ ] | A-07 | Audit the actual dependency graph, public vocabulary, implementation-requirements traceability, and causal evidence vocabulary; then freeze industry-aligned directory, crate, module, test, fixture, binary, command, configuration, environment, wire, persisted marker, digest/media domain, low-cardinality operation, typed-link, and trace-attribute names. Directly rename the overloaded function `AutomationCatalogue` and pre-commit `FunctionTrigger*` family to the canonical function-catalogue and transaction-function-binding vocabulary, and split their implementation from later committed-event triggers and routines. Keep every first-party build/install/runtime input inside this repository and converge overlapping pre-release boundaries directly with no forwarding aliases or parallel execution paths. | workspace | The reviewed traceability matrix accounts for every affected current behavior, source module, test, fixture, and planned destination; a frozen trace map assigns ingress, engine, KV, QL, graph, lexical, vector, DataFusion, inference, function, attunement, routine, adapter, and delivery work to one naming/coordinate scheme; the function contract has one golden closed-schema fixture and no old name/field decoder; case-insensitive terminology, `cargo metadata`, dependency-direction, tracked-path, and owning-suite checks prove every package has one responsibility, every dependency points inward, no successful old-shape reader/default/alias remains, every local dependency/target is under the workspace root, and no tracked submodule, escaping symlink, host-specific absolute path, sibling checkout, or Git dependency supplies RRFlow code. |

#### A-06 knowledge-bootstrap sequence

KB-01 through KB-05 are the incremental work packages inside A-06. A-06 is
complete only when KB-05 passes and the accepted checkout knowledge remains a
deterministic, content-addressed package. Importing that package cannot be an
A-gate prerequisite because it requires later persistence, attunement, and
client behavior.

| Done | ID | Bounded change | Acceptance evidence |
|---|---|---|---|
| [x] | KB-01 | Establish the master system overview, accepted single-engine ADR, canonical component terminology, and indexed warp points. | Root and boundary portals link the owners; documentation policy checks their coordinates, required sections, terminology, indexes, and local links. |
| [x] | KB-02 | Freeze the provider-neutral knowledge-record, manifest, exclusion-ledger, and package schemas without implementing import. | Closed-schema golden vectors cover stable coordinates, source paths, content digests, classification, ordering, provenance, exclusions, and package digest calculation. |
| [x] | KB-03 | Implement the deterministic Markdown-to-package exporter using the KB-02 contract. | Two clean exports are byte-identical; every eligible document is present exactly once; excluded paths carry a reason; no generated package is treated as editable authority. |
| [x] | KB-04 | Add documentation/package drift and reproducibility enforcement to CI. | CI fails on duplicate coordinates, unindexed active records, unclassified eligible records, changed content without digest change, unstable ordering, missing exclusions, or non-reproducible output. |
| [ ] | KB-05 | Resolve remaining flat supporting documents one complete file at a time: retain a record only when it owns current knowledge, merge accepted material into its existing owner, and remove the redundant source. Do not create another archive for unresolved or duplicate pre-release material. | Each reviewed file has one current owner or is removed after accepted content is integrated; retained records have one coordinate and index entry, with no copied authority body and passing link/terminology checks. |

#### Later knowledge-persistence sequence

These milestones preserve the original knowledge-package identifiers but are
owned by the later runtime waves. They do not block A-07, Gate B, or Gate C.

| Done | ID | Bounded change | Acceptance evidence |
|---|---|---|---|
| [ ] | KB-06 | Import a verified KB-03 package through persisted attunement checkpoints and authorized `RrdEngine` mutations. | D-02 and D-05 evidence proves digest-bound resume, idempotency, authorization, atomic mutation, and rejection of package or configuration drift. |
| [ ] | KB-07 | Prove close/reopen, readback, warp resolution, incremental update, rollback, and recovery against rrflowDB. | Durable tests reproduce every imported record and relationship at its committed read stamp after restart and failure injection. |
| [ ] | KB-08 | Make authorized rrflowDB warp resolution the normal client path after import while retaining the already-thin bootstrap READMEs as recovery maps. | H and J evidence shows clients resolve the same authorized records through public operations; local fallbacks remain sufficient for recovery without duplicating mutable state. |

KB-02 evidence (2026-09-05):

- `rrd-contract` now owns a closed, provider-neutral version-one knowledge
  record, provenance, manifest disposition, exclusion ledger, independent
  source inventory, and deterministic package contract. It performs no
  Markdown discovery, import, engine mutation, persistence, or HTTP work.
- Record and package SHA-256 inputs use versioned domain separators and
  explicit unsigned 64-bit length frames. Validation binds normalized relative
  paths, canonical RRFlow coordinates, UTF-8/LF bodies, classification, owner,
  provenance, stable ordering, unique membership, exclusions, and complete
  independently discovered source inventory.
- `knowledge-package-v1.json` freezes the encoded contract. Seven focused
  tests cover golden reopen, closed generated schemas, unsafe coordinates and
  paths, line-ending and body drift, ordering and duplicate rejection,
  independently detectable omission, exclusion binding, provenance changes,
  and record/package digest substitution.
- All 58 `rrd-contract` tests, strict package Clippy, and the
  implementation-free contract architecture test passed. The OpenAPI exporter
  and fixture remain unchanged because KB-02 adds no public operation.

KB-03 evidence (2026-09-06):

- `scripts/knowledge/export.py` discovers the bootstrap product `README.md`,
  optional `SPEC.md`, and every `docs/**/*.md` source before classification.
  It includes coordinated active/historical records and emits an explicit
  reason-bound exclusion for every other discovered source; no input can
  disappear from both the manifest and exclusion ledger.
- Included records derive their classification from the documentation
  taxonomy and their owner coordinate from the nearest linked index. Missing
  UTF-8, unsafe paths, duplicate coordinates, missing index links, invalid
  metadata, source changes during export, and unignored in-repository output
  fail closed.
- Python digest calculations match the Rust KB-02 golden vectors exactly. At
  the KB-03 acceptance revision, the repository exported twice to independent
  temporary paths with byte-identical output: 79 manifested sources, 26
  included records, 53 explicit exclusions, and one package digest. Later
  records must pass the same reproducibility check instead of rewriting this
  accepted count.
- The real export exposed and repaired a KB-02 validation error: canonical
  nested coordinates such as `/data/reference/storage/<record-id>` are now
  accepted, while missing, empty, non-canonical, query, and fragment segments
  remain rejected.
- Seven focused exporter tests and all seven Rust knowledge-contract tests
  passed. Generated packages remain temporary/ignored artifacts and are not
  checked in as editable documentation authority.

KB-04 evidence (2026-09-07):

- The documentation policy loads the KB-03 exporter as the single eligibility,
  classification, ownership, and package-construction implementation. It
  exports the repository twice with fixed provenance, compares canonical
  bytes, and independently validates manifest coverage, ordering, unique
  coordinates and paths, body/record/package digests, and the inclusion versus
  exclusion partition.
- Active or historical records under every exporter-recognized documentation
  taxonomy fail when excluded or unclassified. The prior smaller hard-coded
  policy directory list was removed so `guides`, `operations`, and `evidence`
  cannot silently bypass the exporter rules.
- The existing CI authority performs two real exports at the checked-out Git
  revision into independent temporary files, requires byte identity, and runs
  the focused exporter/policy corpus. Generated output remains temporary and
  cannot become editable documentation authority.
- Eleven Python tests cover Rust/Python golden-digest parity, repeated export,
  complete independent inventory, duplicate coordinates, missing owner links,
  unclassified eligible records, changed bodies with stale digests, unstable
  record ordering, missing exclusions, non-UTF-8 input, and unsafe tracked
  output. All passed along with Ruff, documentation policy, CI policy, two real
  80-source exports (27 included and 53 explicitly excluded), the seven Rust
  knowledge-contract tests, strict `rrd-contract` Clippy, version policy,
  formatting, and generated-surface parity.

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
  lookup, and CLI record/show surface are absent; the pre-A-02 golden entry was
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
  Biome check, and aggregate `pnpm check` passed. That evidence established
  repository separation and a partial native-client characterization only. The
  [current Connectome audit](../reference/client/connectome.md#current-separate-checkout-audit)
  records the remaining parallel contract, lifecycle, inherited-runtime,
  transport, and real-engine conformance gaps; it does not satisfy H-06.
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
| [ ] | B-05 | Define GraphQL as a schema-derived ingress adapter that lowers into the same bound RRFlow query representation. | `rrd-contract`, `rrd-query` | Equivalence fixtures show GraphQL and rrflowQL produce the same authorized logical request without a second executor. |

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
| [ ] | C-03 | Commit canonical record, relation, both adjacency directions, synchronous index changes, runtime log entry, durable projection deltas, function invocation receipt and derived proposal, effect-complete audit, and outbox entry as one write batch. Replace the private monolithic function-catalogue control record with typed definitions, bindings, content-addressed artifacts, immutable membership revisions, and one compare-and-swap head under the same transaction authority. | `rrd-store`, `rrd-engine` | The shared rrflowMX/rrflowKV corpus plus failure injection at every prepare/WAL/batch/acknowledgement boundary proves all-or-nothing behavior; an allowed function audit cannot survive a failed domain commit, advertised catalogue limits fit physical limits, and rrflowKV reopens without re-executing a prepared function under another runtime build. |
| [ ] | C-04 | Serve current and temporal reads from direct versioned keys at one `ReadStamp`; remove normal-path whole-log reconstruction. | `rrd-store` | Physical counters and plan evidence show bounded point/range reads while exact snapshot comparisons remain equal. |
| [ ] | C-05 | Keep Fjall selection, migration-only runtime paths, and alternate stores absent; remove every pre-1.0 reader and alternate format branch from the 1.0 executable. | `rrd-store`, workspace | Fresh rrflowKV database and format-rejection tests pass; repository search and dependency metadata contain one rrflowKV opener and one accepted physical-format reader. |
| [ ] | C-06 | Replace row-record immutable segments with the hybrid rrflowKV layout: an ordered key/version spine plus Arrow-compatible column pages, explicit encoding/compression metadata, and safe buffer lifetimes. Keep point/range/CAS reads independent of DataFusion. | `rrd-lsm`, `rrd-store` | Frozen format vectors, property tests, exact differential reads, selective-scan counters, mixed-family interference tests, and comparative benchmarks prove the new layout; eligible uncompressed/aligned pages borrow buffers while all read, decoded, decompressed, copied, allocated, and cached bytes are reported. Family-specific page or cache policy is retained only when the declared workload improves without correctness or other-family regression. |
| [ ] | C-07 | Prove WAL recovery, manifest recovery, bounded maintenance and write backpressure, pinned-snapshot compaction, Arrow-page lifetime safety, checksums, storage-full behavior, and acknowledged-write durability. | `rrd-lsm` | Crash matrix, reader/compaction concurrency, sustained-write/maintenance/RSS runs, and repeated reopen suite pass with no lost acknowledged write, unbounded write-buffer growth, dangling mapped buffer, or exposed partial batch. |

Gate C exits only when rrflowKV is the sole local persistent implementation and
its correctness is demonstrated below the semantic engine.

### Gate D — install, configure, and attune one real estate

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | D-01 | Implement `rrflow install` by resolving only bundle-resident, versioned project-bootstrap templates, provider-neutral specialization manifests, and attunement profiles, with explicit new/existing-project modes, preview/apply behavior, and a minimal `.rrflow/config.toml` locator containing no canonical mutable state or plaintext secret. The specialization names the primary seat and allowed provider-representation candidates; generic templates contain no hardcoded persona or provider, while this repository's specialization explicitly selects Clyffy. The plan enumerates every RRFlow-owned file, record, process invocation, adapter binding, initial principal/role/grant, credential generation or opaque source/sink descriptor, verifier policy, and validity/rotation policy. `initialize_instance` is the only local cold-start security path: under an exclusive fresh-target lease and engine-observed time, `RrdEngine` atomically commits the installed estate binding, initial seat definition, initial security authority, exact action checkpoint, audit, and commit evidence before application readiness. The apply path performs no download, sibling-repository discovery, arbitrary secret-path interpretation, undeclared generator or harness invocation, implicit adapter activation, or unauthenticated network bootstrap. | `rrflow-cli`, `rrd-engine` | Template golden tests prove `AGENTS.md` is the single instruction body, generic templates contain no Clyffy/provider default, a selected specialization produces the exact primary-seat plan, and any supported provider files are forwarding stubs; with outbound network denied and sibling directories absent, fresh and existing application-project tests preserve user content, preview without writes or secret reads, apply the exact digest, deliver one high-entropy credential only through the previewed sink, authenticate, bind an approved provider representation without persisting its subject or credential, close, and reopen the same rrflowKV instance. Boundary interruption tests cover credential prepare/delivery, engine commit, locator publication, acknowledgement, and cleanup; they prove exact replay or fail-closed recovery, no second credential/policy/audit outcome, no listener before security readiness, and no plaintext in canonical state, logs, traces, errors, process arguments, or environment. Drift, path/link/mount replacement, ACL/mode, provider revision, partial-state, stale-lease, and already-installed cold-start attempts are rejected. |
| [ ] | D-02 | Persist attunement jobs and checkpoints through `RrdEngine`; implement status, resume, cancel, leases, idempotency, and phase input/output digests. | `rrd-engine` | Kill/restart tests at each transition resume committed work once and never infer completion from emitted events. |
| [ ] | D-03 | Implement only deterministic project-tree inventory: engine-authorized metadata/content reads, pure proposal building, Git-correct ignore rules, secret/generated/vendor/cache exclusions, stable root-relative identities, Merkle tree digest, containment edges, incremental change set, explicit errors, and bounded work accounting. | `rrd-attunement` through `rrd-engine` | This repository produces the same committed `SourceTreeSnapshot` digest under different traversal schedules; excludes `.git`, `target`, `node_modules`, rrflowDB files, generated bulk data, and secret payloads; handles tracked ignored files, non-Git roots, hidden source, symlink/mount escape, file races, cancellation/restart, and close/reopen; an unchanged rerun reads zero content bytes and commits a no-work checkpoint. |
| [ ] | D-04 | Add incremental Tree-sitter parsing pinned to one committed source-tree snapshot/change-set digest and parser/language revisions; preserve incomplete syntax and errors as evidence. | `rrd-attunement` through `rrd-engine` | Parse is denied before the D-03 snapshot/checkpoint commit; edit-one-file reparses only the changed source, preserves unaffected identities, records `ERROR`/`MISSING` nodes, and resumes after process restart. |
| [ ] | D-05 | After Gates E and F pass, implement normalize, entity-link, lexical-index, embed, vector-index, graph, ground, and verify one at a time against the accepted persistent/index/Arrow paths; add no temporary snapshot index or parallel attunement store. | `rrd-engine` plus owning subsystem | Every phase has an exact fixture, durable checkpoint, failure/retry case, output digest, physical-plan/trace evidence, and independent acceptance test before the next phase begins; lexical/vector/graph output is readable through the same native and analytical paths used outside attunement. |
| [ ] | D-06 | Classify project generators, package scripts, build/test/evaluation harnesses, CI/deployment tools, SQL, PostgreSQL, Turso, Dragonfly, object stores, models, meshes, and other application/operator systems as optional external capabilities or sources. Discovery creates an inactive typed candidate; only an exact previewed immutable binding plus explicit configuration, policy, and authorization may activate it. A command binding captures either one authenticated direct-process argv or the complete package-script/toolchain/lockfile/wrapper/lifecycle closure; literal package-manager argv alone is insufficient. Never install, start, execute, or select an external system as RRFlow persistence or lifecycle authority implicitly. | `rrd-contract`, `rrd-attunement`, `rrd-engine` | Fresh/existing fixtures prove RRFlow reaches default readiness without an external capability; discovery executes nothing; preview performs no writes or effects; exact apply/retirement is durable and idempotent; a generator or harness binding is source- and digest-bound and declares its permissions, sandbox, budgets, cancellation, uncertainty, verification, and output policy without invoking it; mutable/incomplete closures and unavailable enforcement fail closed; credentials are never copied; and clean uninstall removes only RRFlow-owned integration state. Project-command execution and output re-inventory are accepted later by I-03/I-07. |
| [ ] | D-07 | Bind installation to the DevForge CoW placement contract: immutable tools/models may live in the shared lower layer; workspace changes and all rrflowKV WAL, manifest, segment, catalogue, and graph state live in the writable upper layer. | `rrflow-cli`, DevForge adapter | Two clones share the same lower digest while independent writes, crash recovery, and deletion in one upper layer cannot affect the other. |
| [ ] | D-08 | Recognize content-addressed dependency mounts as shared immutable inputs rather than copying or attuning dependency caches into each estate. | `rrd-attunement`, DevForge adapter | Rust, Go, Node, and model-cache fixture proves stable mount digests, zero duplicate ingestion, and correct invalidation when a mounted digest changes. |
| [ ] | D-09 | Measure logical size, allocated blocks, compression, WAL growth, and snapshot size separately; never infer zero-byte or sparse-allocation savings from logical file size. | `rrd-lsm`, release harness | Fresh clone and sustained-write reports account for lower, upper, cache, WAL, segment, and snapshot bytes with reproducible filesystem commands. |
| [ ] | D-10 | Add a hibernation preparation/restore contract that quiesces writes, captures a verified rrflowKV snapshot boundary, exports to the configured cold tier, and resumes without changing estate identity. | `rrd-engine`, DevForge adapter | Interrupted export, corrupt object, restore, rollback, and hot-to-cold-to-hot tests prove no acknowledged-write loss and no split authority. |

Gate D exits only when a fresh and an existing project can be installed from
bundle-resident inputs, attuned, interrupted, resumed, verified, and reopened
through public engine operations with the network denied and no external
database service.

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
| [ ] | G-02 | Build a bounded route packet from one stamped tree, eligible recipes, verified observations, and allowed query fields. Bind the authenticated principal, current provider-representation edge, resolved durable seat, policy revision, and authorization digest at that same read coordinate; a caller-supplied actor label is never identity evidence. | `rrd-engine` | Golden packet and denial corpus prove representation establishes attribution rather than permission, stale/revoked/foreign representations fail closed, and the model sees no raw KV keys, secrets, hidden reasoning, unauthorized fields, or unbounded workspace content. |
| [ ] | G-03 | Grammar-constrain LFG to the three routing decisions and validate again after decoding. | LFG adapter | Corpus includes valid, malformed, unknown-recipe, unauthorized-query, stale-cursor, and prompt-injection cases; invalid decisions produce no mutation. |
| [ ] | G-04 | Execute recipe selection and branch navigation on the fast path without rrflowQL/DataFusion; keep deterministic predicates and CAS mutation in `RrdEngine`, and commit the resolved seat attribution with the resulting tree state, audit, and causal trace. | `rrd-engine`, `rrd-store` | Trace and physical-plan evidence show bounded rrflowKV operations, no DataFusion plan, conflict denial, no actor-string impersonation, and correct attributed state after reopen. |
| [ ] | G-05 | Lower `request_context` into semantic rrflowQL/context intent while leaving physical access selection to the engine. | `rrd-engine`, `rrd-query` | LFG cannot select an index/backend; resulting plan is authorized, stamped, budgeted, and equivalent to a typed SDK request. |
| [ ] | G-06 | Publish model and storage latency separately with task-success, routing-accuracy, invalid-decision, and escalation metrics. | evaluation harness | Reproducible hardware/model manifest and raw samples support every reported latency or quality claim. |

Gate G exits only when the trained LFG artifact passes the conformance corpus
and can steer a persisted tree without direct storage or planner authority.

### Gate H — prove context flow, feedback, live delivery, and Connectome

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | H-01 | Route each context request dynamically across eligible seed, BM25, vector, graph, and cached paths using the captured catalogue and budgets. | `rrd-engine` | Plan evidence states selected/skipped reason, source cursor, work, and contribution for every avenue. |
| [ ] | H-02 | Keep query-time RRF pure; persist explicit verified outcomes and learn versioned weight policies only for later read stamps. | `rrd-engine`, `rrd-query` | Replay at an old stamp is unchanged; feedback update, rollback, cold-start, and quality-regression tests pass. |
| [ ] | H-03 | Replace two-snapshot live-query diffing with commit-impact evaluation and predicate-specific deltas. | `rrd-query`, `rrd-engine` | Ordered update/delete/reconnect/backpressure tests emit each matching committed delta once without full-query rescans. |
| [ ] | H-04 | Serve HTTP, multiplexed WebSocket, Rust SDK, generated SDKs, CLI, MCP, and GraphQL adapter through the same catalogue-derived operations and authorization semantics. Every supported SDK binds every available catalogue descriptor exactly once; validates the complete request, response, media, status, protocol, identity, stamp, and receipt contract; redacts credential-bearing state; and applies one explicit operation-semantic retry, cancellation, and uncertain-outcome policy. A missing conformance harness configuration cannot report success. | transport/adapters | A D-01-installed real-process corpus structurally proves each operation and fault scenario against rrflowMX and rrflowKV where applicable, then sends the same requests through every surface and compares status, denial, stamp, digest, receipt, result, trace, restart, and resource evidence. Catalogue/router/OpenAPI/Rust/generated bindings have no missing or extra operation; credential/error/frame-limit adversarial cases pass. |
| [ ] | H-05 | Complete the per-gate trace work as one bounded causal chain across ingress, authorization, planning, selected/skipped context avenues, KV/page scans, graph, BM25, HNSW, DataFusion, LFG, cache and model-context compaction effects, model/tool attempts, verification, commit, attunement, and delivery; add W3C propagation plus redacted diagnostic export without creating another authority. | `rrd-engine`, adapters | Canonical-name, propagation, completeness, crash, export, redaction, and fixed-rubric before/after tests retain stamps, plan/projection/source identities, work/byte/token/latency accounting, contributions, and outcomes; repeated work, stale/duplicate/conflicting context, route misses, and model-context compaction loss are detectable; traces observe authoritative job/state records rather than becoming lifecycle state or hidden chain-of-thought. |
| [ ] | H-06 | Implement the separate [Connectome client contract](../reference/client/connectome.md) completely on catalogue-derived public RRD operations. The client validates exact endpoint, transport, protocol, instance, deployment, catalogue, session, resource, stamp, cursor, evidence, completeness, and receipt coordinates; renders only engine-issued state/plans/traces/trees/context/deltas/proposals; and contains no alternate storage, query, retrieval, reasoning, attunement, automation, diagnostics, control, provider, or presentation authority. | separate Connectome repository | A clean Connectome artifact consumes the generated public-contract/SDK digest and runs against D-01-installed rrflowMX and rrflowKV processes. HTTP bootstrap, authenticated session, multiplexed WebSocket resume/ACK/backpressure, graph/BM25/vector/RRF/Arrow/DataFusion/context/reasoning evidence, previewed mutations, denial/uncertainty/cancellation, restart, multi-instance isolation, credential redaction, bounded rendering, and accessibility pass with correlated identities/stamps/digests/traces. Mocks, screenshots, builds, and open ports remain non-qualifying. |
| [ ] | H-07 | Resolve loopback or Zuul Zero/shippin.ai mesh-reached network endpoint candidates through an outward transport adapter, carrying the expected RRD instance and transport-security identities independently of reachability. Re-negotiate liveness, readiness, protocol, instance, catalogue digest, and authentication after rotation; no resolver initializes RRFlow or selects storage. | `rrd-client`, mesh adapter | Laptop/phone/devspace fixture proves bounded resolution, TLS identity, exact instance/capability negotiation, endpoint rotation, stale/foreign/offline denial, and no mesh-owned database, authorization, installation, or lifecycle state. |

Gate H exits only after one prompt can be followed from ingress through LFG or
analytical routing, storage/index work, fused context, mutation, live delivery,
and Connectome using correlated evidence from one engine.

### Gate I — add explicit automation scaffolding without automatic hooks

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | I-01 | Define one canonical engine-event envelope with producer, action, target, scope, stamp, idempotency key, provenance, and authorization coordinates; directly converge the current kernel `RuntimeEvent` representation into that vocabulary without a parallel event log or compatibility type. | `rrd-contract`, `rrd-core` | Golden, lowering, commit, and replay tests reject ambiguous identity, duplicate mismatches, unbounded payloads, and events outside the authenticated estate while proving public and persisted forms are one semantic event. |
| [ ] | I-02 | Implement triggers as persisted conditions over canonical committed events; a trigger may request an authorized operation but cannot commit independently. Keep transaction function bindings as the separate pre-commit validation/proposal mechanism defined by the governed-function contract, with no `Trigger` type, field, operation, or persisted marker shared between them. | `rrd-engine` | Match/non-match, denial, duplicate, ordering, recursion-depth, and restart tests prove deterministic bounded behavior; direct-convergence tests reject every old `FunctionTrigger*` shape and show transaction bindings cannot consume committed-event cursors or schedule routines. |
| [ ] | I-03 | Implement routines as versioned resumable graphs of authorized engine operations with explicit inputs, checkpoints, budgets, cancellation, verification, and terminal status. Model, process, network, and external MCP effects are prepared fenced activities: adapters return bounded observations, while `RrdEngine` alone accepts receipts and advances state. Express error resolution and context projection maintenance as routine definitions, not operation-specific repositories or state machines. | `rrd-contract`, `rrd-engine`, outward activity adapters | Generic kill/restart, retry, compensation, stale-input, denial, maximum-step, prepared/effect/receipt crash-gap, lost-ack, uncertain-outcome reconciliation, redaction, and resource-limit tests plus both template corpora prove no busy loop, silently repeated mutation/effect, direct storage access, private event log, adapter-authored completion, or client-owned state. |
| [ ] | I-04 | Implement host-event adapters as stateless translators that submit typed events only when explicitly installed and configured; ship no editor/provider-owned automatic hook. | outward adapters | Claude/OpenAI/reference adapter conformance produces the same envelope; uninstall removes the adapter cleanly and leaves canonical state readable. |
| [ ] | I-05 | Implement skills as versioned instruction/resource packages referenced by identity and digest, resolved through governed context rather than executed as storage or lifecycle code. | `rrd-contract`, `rrd-engine` | Install/resolve/update/retire tests prove provenance, authorization, version pinning, prompt-budget enforcement, and no implicit mutation. |
| [ ] | I-06 | Add previewable install/configure/retire/uninstall scaffolding for governed functions, transaction function bindings, triggers, routines, host-event adapters, skills, and optional capability/activity bindings after their individual contracts pass. Default function runtimes/artifacts come only from the manifest-verified distribution; attunement may propose an inactive project function but cannot compile, fetch, activate, or execute it. | `rrflow-cli`, adapters | Fresh/existing project tests show exact planned artifacts/schemas/files/records/invocation closures, explicit consent, preview without execution, idempotent apply, retirement that blocks new work without deleting receipts, clean uninstall of only unreferenced RRFlow-owned scaffolding, offline readiness, and no session-start loop. |
| [ ] | I-07 | Require every project-development routine to bind the latest complete authorized project-tree snapshot, then react to committed inventory, schema, dependency, workload, and failure signals by scheduling only the required incremental attunement phases and evaluating eligible capability, routine, and skill activation under estate policy. | `rrd-engine`, `rrd-attunement` | Missing/stale inventory returns `inventory-required`; changed-since-plan entries and paths outside the root are denied; adding one language, framework, data source, or recurring failure triggers the minimal bounded work, survives restart, records its decision evidence, and never performs an ad hoc client scan, blanket reinstall, or unauthorized activation. |

Gate I exits only when automation is explicit, bounded, replayable, removable,
and subordinate to `RrdEngine`; installation alone is never evidence that a
trigger, routine, adapter, or skill worked.

The Gate I end-to-end corpus must include the generic `error-resolution`
vertical slice defined by the
[automation flow](../architecture/engine-data-flow.md#first-complete-automation-proof).
It must prove one committed diagnostic can activate, resolve governed skills
and context, select or reject graph/BM25/vector/TurboQuant/DataFusion work,
invoke an attuned verification capability, survive restart without repeating
effects, and expose the same persisted result through MCP and the other public
surfaces.

### Gate J — RRFlow 1.0 release proof

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | J-01 | Remove every superseded pre-release entrypoint, format/catalogue reader, missing-field fallback, backend selector, migration executor, editor/provider-owned automatic hook, duplicate source of truth, transitional alias, and stale generated artifact. | workspace | Strict warning/dependency/terminology searches, negative format fixtures, and all-target builds prove only the accepted RRFlow 1.0 surfaces remain and superseded RRFlow state fails closed. |
| [ ] | J-02 | Run unit, property, fuzz corpus, differential, crash/reopen, storage-full, security denial, resource-budget, adapter, and real-process suites. | workspace | Release evidence records commands, versions, passed/failed counts, and retained failure artifacts. |
| [ ] | J-03 | Assemble a complete manifest-verified release-candidate bundle from tracked inputs; install and attune both an empty fixture and this existing repository from that candidate; then restart and repeat representative fast/heavy queries. | release harness | With outbound network denied, sibling repositories absent, and no external database/query/vector service, both estates verify with stable digests; unchanged rerun is incremental and no manual database repair is needed. This qualifies contents and behavior before J-05 signs the reproducible default distribution. |
| [ ] | J-04 | Publish fixed-hardware rrflowKV, graph, BM25, exact/HNSW, DataFusion, context, LFG, end-to-end, and clean-rollout benchmarks against pinned declared baselines. | evaluation harness | Revision-bound raw data, hardware/toolchain/filesystem provenance, configuration, warmup, concurrency, percentiles, failures, long-duration RSS, logical/apparent/allocated bytes, mixed-family interference, recall/quality metrics, and a SurrealDB/Qdrant deployment matrix report artifact and installed bytes, required commands, elapsed time, services, ports, configuration, secrets, readiness, and persistent readback. No ease or superiority claim is allowed until like-for-like evidence passes. |
| [ ] | J-05 | Produce one reproducible signed default distribution containing all default-distribution first-party executables and linked engine capabilities, SDKs, schemas/goldens, project and attunement templates, default configuration/profile, required local inference assets, SBOM/licenses, artifact verifier, backup/restore rehearsal, and operator runbook. | release tooling | After artifact acquisition, a clean supported machine with no compiler, source checkout, sibling repository, repository-local cache, package registry, external database/query/vector service, or outbound network runs preview and one explicit apply operation, reaches authenticated readiness, commits data, closes, reopens, verifies persistence, and identifies every installed byte from the signed manifest. |

RRFlow 1.0 is releasable only when every Gate J item and every prerequisite is
checked. Until then the repository may describe implemented and measured
behavior, but it must not claim the complete target system is production-ready.
