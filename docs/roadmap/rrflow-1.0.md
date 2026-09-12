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
| A | authority, naming, documentation memory, and repository-contained source boundaries | 7 / 7 |
| B | public, install, routing, model, WebSocket, and GraphQL contracts | 5 / 5 |
| C | sole hybrid persistent rrflowKV substrate | 5 / 7 |
| D | per-project install, operation, repair, configuration, and attunement | 0 / 11 |
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

This is the one implementation order. It removes three circular or temporary
paths from an alphabetical sequence: knowledge import cannot block the storage
and attunement code it requires; D-05 cannot build temporary lexical/vector/
graph paths before Gates E and F establish their canonical access and
analytical boundaries; and final storage qualification cannot postpone the
first real installed binary until after every internal subsystem. C-06 finishes
the active physical-format package first, then D-01 establishes the walking
product spine. C-07 subsequently qualifies that installed path rather than a
checkout-only constructor.

| Wave | Work | Required exit before continuing |
|---:|---|---|
| 0 | Finish A-06 through KB-05, then execute A-07 | One deterministic checkout knowledge package; every current behavior/file/test/fixture mapped; source, package, public, and trace vocabulary frozen. |
| 1 | B-03 through B-05 | Model handshake, multiplexed WebSocket, and GraphQL lowering are provider-neutral contracts with golden equivalence. |
| 2 | Finish C-06 | One accepted hybrid segment reader, selective projected pages, physical counters, property/fuzz and mixed-family evidence, and measured format-policy decisions exist. |
| 3 | D-01 | One natively executable `rrflow`/`rrflow.exe` command tree can plan/apply installation, serve, prove authenticated readiness, commit, close, reopen, and perform baseline read-only verification without checkout-built companions. |
| 4 | C-07 | WAL/manifest recovery, bounded maintenance, write backpressure, pinned generations, storage-full behavior, and sustained lifetime pass through the D-01-installed composition. |
| 5 | D-02 through D-04 | Persisted jobs, deterministic committed project tree, and incremental parse evidence exist through the installed engine. |
| 6 | E-01 through E-05 | Graph, scalar/unique, BM25, exact vector, HNSW/quantized candidate, and planner paths are persistent, incremental, stamped, and exact-oracle checked. |
| 7 | F-01 through F-05 | rrflowKV streams bounded Arrow batches into native/DataFusion operators with honest pushdown, one resource budget, and a measured cache decision. |
| 8 | D-05 through D-11, then KB-06 and KB-07 | Every attunement phase uses accepted C/E/F paths; external sources remain adapters; placement/accounting/hibernation and complete verify/repair/restore/uninstall pass; the checkout knowledge package imports and survives readback/recovery. |
| 9 | G-01 through G-06 | LFG can propose bounded routing decisions while `RrdEngine` retains predicates, physical planning, authorization, CAS, and persistence. |
| 10 | H-01 through H-07 | Dynamic context, pure RRF feedback, live deltas, all public surfaces, complete traces, Connectome, and mesh resolution observe one engine. |
| 11 | I-01 through I-07 | Canonical events, triggers, routines, skills, and optional host adapters are explicit persisted capabilities, not hooks or a parallel runtime. |
| 12 | J-01 through J-03, then KB-08, then J-04 and J-05 | Alternate paths are absent; failure and clean-install qualification pass on every supported native artifact; rrflowDB becomes the normal warp path; comparative evidence and the signed distribution close release. |

A-07 freezes the operation-name, causal-link, attribute, and propagation
vocabulary. Each later work package adds the trace and physical counters for
its behavior in the same change. H-05 proves complete cross-surface
correlation, export, and redaction; it does not postpone instrumentation until
Wave 8.

The active executable item is **C-06**. Its completed slices have replaced
row-record immutable segments with segment v4's ordered key/version spine plus
six Arrow-layout page buffers, manifest-authenticated format identities, safe
mmap ownership, page-level physical counters, and a generated mixed-family
MVCC/reopen/compaction differential with malformed-byte rejection. C-06g now
adds the pinned, bounded, selective projected storage stream and separates
segment-open, startup-reconciliation, and query evidence. C-06 remains
unchecked while C-06h adversarial/property/fuzz qualification and C-06i
revision-bound physical-policy measurements remain open.
C-05e removed vector artifact catalogue v1 and its alternate identity digest;
C-05f requires one explicit nonempty schema table map and removes all
missing-table model inference; C-05g requires every persisted vector,
inference job, source delta, commit identity, and derived artifact to carry its
canonical collection plus named-vector address; and C-05h removes generic
quantized publication, the TurboQuant ensure adapter, and reconstruction-time
suppression while retaining the explicit quantization lifecycle. C-01's physical-key
package, C-02's shared
transaction/repository package, C-03's effect-complete semantic mutation
batch, C-04's direct current/temporal read package, C-05's lower physical
reader/dependency packages, and every Gate B contract package are complete.
C-05 is accepted for the single-node alpha composition; C-06 is in progress.
After C-06 closes, the next item is D-01, not another documentation or
component-only detour. D-01 must produce the first walking installed-product
proof before C-07 widens the durability/lifetime matrix through that real
composition.
A-07 is
complete: A-07.0 mapped current requirements to code and evidence; A-07.1a
through A-07.1h directly converged package/type/path vocabulary, SDK
responsibility boundaries, dependency direction, and repository-contained
source; and A-07.2 froze the 13 trace boundaries, 53 canonical operations, 14
typed causal-link forms, 46 type-checked canonical attributes, W3C propagation
rules, diagnostic projection, and exact shrinking current-producer inventories.
This is vocabulary and traceability completion, not engine-capability
qualification. B-01 through B-05 are completed contract/admission work. B-04
does not claim H-04 generic operation dispatch or generated-SDK parity, and
B-05 does not claim a public GraphQL endpoint or a second query executor.

### Gate A — freeze authority, names, and boundaries

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [x] | A-01 | Define RRFlow, RRD, `RrdEngine`, rrflowDB, rrflowKV, rrflowMX, rrflowQL, Arrow substrate, DataFusion execution, RRFlow vector and inference subsystems, LFG, and Connectome exactly once. | `README.md`, system overview, ADR-0001 | Terminology, execution topology, and authority boundaries use one meaning for every term and prohibit parallel database, model, client, or compute authority. |
| [x] | A-02 | Define generic `reasoning_tree`, `reasoning_node`, typed `reasoning_edge`, recipe, active cursor, decision evidence, and verification-result semantics. | `rrd-contract`, `rrd-core` | Versioned schema and golden round trips reject unknown fields, invalid edges, and unverifiable cursor advances. |
| [x] | A-03 | Resolve the pending reasoning-ledger removal against A-02 without restoring a hard-coded universal reasoning lifecycle or deleting reusable semantics. | `rrd-core`, `rrd-engine`, CLI | Golden/API diff proves reusable data moved to the generic contract, contains no forced Goal→Plan→Attempt sequence, and focused core, engine, and CLI tests pass. |
| [x] | A-04 | Move all existing crates into one non-duplicated grouped source tree, remove the empty `rrd-graph` boundary, and remove `connectome-ui` after its public-client behavior is present in the separate Connectome repository. | workspace | `cargo metadata`, dependency-direction check, and repository search show the declared layout and no second graph, memory, routing, lifecycle, UI, or provider authority. |
| [x] | A-05 | Remove stale documentation claims or mark supporting documents historical where they describe another architecture. | documentation | Repository link/terminology check finds no supporting document presented as current authority. |
| [x] | A-06 | Establish the documentation memory topology: the root and each major source-boundary README are warp maps into one owning `docs/<subject>/` record set; classify every flat document without duplicating content; generate a deterministic content-addressed manifest/JSONL bootstrap package for later authorized rrflowDB ingestion. | documentation | CI proves every active record has status, owner, stable coordinate, one inbound owner link, valid local fallback links, and no duplicate roadmap or architecture body; repeated packaging produces byte-identical ordered records and digests with an explicit inclusion/exclusion ledger and no silently omitted eligible record. |
| [x] | A-07 | Audit the actual dependency graph, public vocabulary, implementation-requirements traceability, and causal evidence vocabulary; then freeze industry-aligned directory, crate, module, test, fixture, binary, command, configuration, environment, wire, persisted marker, digest/media domain, low-cardinality operation, typed-link, and trace-attribute names. Directly rename the overloaded function `AutomationCatalogue` and pre-commit `FunctionTrigger*` family to the canonical function-catalogue and transaction-function-binding vocabulary, and split their implementation from later committed-event triggers and routines. Keep every first-party build/install/runtime input inside this repository and converge overlapping pre-release boundaries directly with no forwarding aliases or parallel execution paths. | workspace | The reviewed traceability matrix accounts for every affected current behavior, source module, test, fixture, and planned destination; a frozen trace map assigns ingress, engine (including governed function execution), KV, QL, graph, lexical, vector, DataFusion, inference, attunement, routine, adapter, and delivery work to one naming/coordinate scheme; the function contract has one golden closed-schema fixture and no old name/field decoder; case-insensitive terminology, `cargo metadata`, dependency-direction, tracked-path, and owning-suite checks prove every package has one responsibility, every dependency points inward, no successful old-shape reader/default/alias remains, every local dependency/target is under the workspace root, and no tracked submodule, escaping symlink, host-specific absolute path, sibling checkout, or Git dependency supplies RRFlow code. |

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
| [x] | KB-05 | Resolve remaining flat supporting documents one complete file at a time: retain a record only when it owns current knowledge, merge accepted material into its existing owner, and remove the redundant source. Do not create another archive for unresolved or duplicate pre-release material. | Each reviewed file has one current owner or is removed after accepted content is integrated; retained records have one coordinate and index entry, with no copied authority body and passing link/terminology checks. |

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

KB-05/A-06 evidence (2026-09-08):

- Every queued flat supporting record was read and resolved in its own
  journaled commit. `docs/README.md` is now the only top-level `docs/*.md`
  record; every other current or historical record is nested under one
  taxonomy owner and nearest parent index. No compatibility archive or copied
  authority body was created.
- The final retained CI record now has the stable
  `rrflow://rrflow-instance/data/operations/ci` coordinate and a canonical
  operations index. Source tracing also corrected its unqualified workflow
  security claims: all three workflows are checked for full-SHA actions,
  digest-pinned service images, read-only permission, absent
  `pull_request_target`, and non-persisted checkout credentials. The scheduled
  storage workflow remains explicitly diagnostic rather than engine or release
  evidence.
- Two independent real exports of the final candidate tree were byte-identical
  and contained 91 manifested Markdown sources partitioned into 89 coordinated
  records and two explicit exclusions. The manifest, record coordinates,
  source membership, inclusion/exclusion partition, and package digest were
  independently checked.
- Eleven Python exporter/drift tests, seven Rust knowledge-contract tests, all
  58 `rrd-contract` tests, strict `rrd-contract` Clippy, documentation policy,
  workflow policy plus three negative workflow-security probes, inventory,
  version, generated-surface, workspace-architecture, formatting, and complete
  workspace all-target checks passed. The package journal records exact
  commands and corrections. The resulting commit binds the final tree and
  package digests because an included journal cannot contain its own final
  content digest without changing it; post-commit export verifies that pair.
- A-06 qualifies the deterministic checkout bootstrap package only. It does
  not claim knowledge import, persistence, readback, normal public warp
  resolution, or any rrflowKV/graph/index/Arrow/DataFusion/reasoning capability;
  KB-06 through KB-08 and the runtime gates remain open.

A-07.0 evidence (2026-09-08):

- The reviewed starting point is commit
  `b5ec39162771ced38a83dd512dda5b0720aef714`, tree
  `3c8d11fe697cd88645179f15441a7485d934f240`. The deterministic execution
  inventory contains 896 current, generated, and planned records.
- Every root/workspace Rust manifest, locked dependency edge, and all 32
  library/binary module roots were read completely, as were all generated-SDK
  package boundaries and public source roots. The execution map now records the
  20-package dependency/public-surface baseline and the five SDK boundaries.
- The package map, capability-family matrix, and mandatory direct-convergence
  table jointly bind current modules and public symbols to characterization
  tests/fixtures/examples/benchmarks, accepted invariants, one canonical
  destination, owning gates, equal-or-stronger proof, and each conflicting
  authority or successful old surface that must later be removed.
- Deterministic inventory and documentation checks, all 11 knowledge-exporter
  tests, locked Cargo metadata, all 16 workspace-architecture tests, generated
  parity for 33 HTTP operations, frozen-version policy, Cargo formatting,
  diff integrity, and the complete workspace all-target check passed.
- This was documentation-only traceability. It did not rename, move, delete,
  or qualify runtime code; it did not prove rrflowMX/rrflowKV equivalence,
  rrflowKV durability, graph/index atomicity, streamed Arrow/DataFusion
  execution, or persisted reasoning/recall. At that revision A-07 remained
  unchecked and A-07.1 was the next package.

A-07.1a governed-function vocabulary evidence (2026-09-08):

- The public and engine type/field/method family now uses
  `FunctionCatalogue`, `TransactionFunctionBinding`,
  `TransactionMutationKind`, `TransactionFunctionEffect`, `binding_id`,
  and `transaction_bindings` directly. The former source symbols and wire
  fields have no alias or successful decoder.
- The overloaded `engine/automation.rs` authority is absent. Catalogue,
  execution, JavaScript, WebAssembly, and transaction-binding responsibilities
  are isolated under `engine/function/`; engine events, post-commit triggers,
  routines, skills, installation, and external activities were not invented by
  this move.
- A closed `function-contract-v1.json` fixture and two golden/schema tests
  freeze the current canonical shape. A shared engine fixture additionally
  proves the selected JavaScript catalogue read/invocation/transaction result
  equal on rrflowMX and rrflowKV, then closes/reopens rrflowKV and repeats
  catalogue read and invocation.
- All 17 workspace-architecture tests, two function-contract tests, four
  engine function tests, and the one focused cross-profile/reopen test passed.
  The architecture guard finds no former function-authority symbol in
  executable contract/engine source.
- This is structural convergence plus a first characterization corpus, not
  capability qualification. Private monolithic control JSON, direct storage
  access, split allowed audits, unbound runtime replay, inline artifacts,
  incomplete runtime profiles, and absent install/public/cross-language proof
  remain owned by C/D/H/I/J and POAM-008. A-07 and A-07.1 remain unchecked;
  A-07.1b follows as the next bounded subpackage.

A-07.1b Rust SDK responsibility-boundary evidence (2026-09-08):

- The former 1,235-line `rrd-client/src/lib.rs` implementation body is absent.
  Its existing responsibilities now live directly in `client`, `endpoint`,
  `error`, `operation`, `retry`, `session`, `subscription`, and `transport`;
  `lib.rs` contains only crate policy, module declarations, and public exports.
- Every pre-split public method remains. The intentional pre-release API change
  removes direct access to the bearer-bearing `SessionLease`: `Session` exposes
  only non-secret identity, expiry, and limit metadata, keeps the bearer
  crate-private, and uses a focused, tested redacted `Debug` implementation.
- The package all-target corpus passed, including the redaction unit test and
  three real-server tests covering loopback HTTP, mutual TLS, WSS, and current
  durable subscription ACK/reconnect behavior. Strict package Clippy and the
  workspace architecture suite also passed.
- This is a direct responsibility split and public secret-boundary correction,
  not SDK conformance. The client still implements 28 of 33 HTTP operations;
  complete request/response/status/media validation, semantic retry and
  uncertainty, correlated cancellation, bounded multiplexed frames, W3C
  propagation, endpoint rotation, secret zeroization, installed MX/KV
  conformance, and fail-closed harness behavior remain owned by B/D/H/J and
  POAM-011. A manifest-absent conformance invocation still reports a Cargo
  success without executing a scenario and is explicitly not evidence.
- A-07 and A-07.1 remain unchecked; A-07.1c follows as the next bounded
  subpackage.

A-07.1c TypeScript SDK responsibility-boundary evidence (2026-09-08):

- The former 394-line `sdks/typescript/src/index.ts` implementation body is
  absent. Existing construction/dispatch, loopback endpoint, error, generated
  operation typing/request coordinates, attempt/deadline, plain session, and
  bounded Fetch/envelope behavior now live directly in their named modules;
  `index.ts` is only the public export root.
- All twelve former root exports, the `RrdClient` constructor, and its five
  public methods remain. The current broad replay rule, partial ArkType
  response validation, serializable credentials, loopback-only transport, and
  other characterized defects were exposed in their direct owners rather than
  silently changed or hidden.
- `tests/sdk_conformance.ts` was directly renamed to
  `tests/sdk-conformance.ts`, and the sole shared orchestrator caller now uses
  that path. Missing manifest input still fails immediately; the shared real
  daemon corpus ran successfully through Rust, TypeScript, Python, Go, Java,
  and .NET and reported the expected corpus digest for each.
- Generation freshness, Biome over 14 files, strict no-emit type checking, and
  all four mock-focused tests passed. Generated parity remains 33 HTTP
  descriptors at the unchanged OpenAPI digest.
- This is structural convergence, not TypeScript SDK qualification. No
  `subscription.ts` exists because there is no current socket behavior to
  move; B-04 owns the real multiplexed protocol. Generated runtime validators,
  opaque credentials, exact status/media/payload enforcement, semantic
  retry/uncertainty, server cancellation, W3C propagation, HTTPS/mesh endpoint
  resolution, browser proof, built ESM/declarations, and installed MX/KV
  conformance remain owned by B/D/H/J and POAM-011.
- A-07 and A-07.1 remain unchecked; A-07.1d is next.

A-07.1d Python SDK responsibility-boundary evidence (2026-09-08):

- The former catch-all `sdks/python/src/rrd_client/models.py` is absent. The
  synchronous facade remains in `client.py`; current loopback endpoint policy,
  errors, request/resource construction, broad retry/deadline policy, plain
  session shape, and bounded HTTP/envelope decoding now live directly in their
  named modules. `__init__.py` is only the eight-symbol public export root.
- All eight former root exports and all six public `RrdClient` methods remain.
  Existing request bytes, 33-descriptor dispatch, loopback/redirect denial,
  connection reuse, context-manager closure, partial response validation,
  immediate retry behavior, and mutable bearer-bearing session dictionaries
  are characterized rather than silently changed. No async or WebSocket file
  was created.
- Lock and generator freshness, Ruff over 13 Python files, strict mypy over 10
  source files, and all four mock-focused tests passed. A locked offline build
  produced a 9,294-byte wheel and 48,486-byte sdist; both include `py.typed`
  and exclude the removed model module. Direct manifest-absent conformance
  exited 1, while the shared real-daemon corpus passed through all six SDKs at
  the expected corpus digest.
- This is structural and packaging-topology convergence, not Python SDK or
  engine qualification. Runtime request/result models, opaque credentials,
  exact status/media/identity enforcement, semantic retry/uncertainty, native
  async, multiplexed WebSocket/cancellation, W3C propagation, HTTPS/mesh
  resolution, supported interpreter/platform consumers, signed offline
  closure, and installed MX/KV conformance remain owned by B/D/H/J and
  POAM-011.
- A-07 and A-07.1 remain unchecked; A-07.1e is next.

A-07.1e Go SDK responsibility-boundary evidence (2026-09-08):

- The former 498-line `sdks/go/client.go` responsibility monolith is now a
  114-line public facade. Current construction/configuration, loopback endpoint
  policy, API errors, operation/resource/envelope construction, broad
  retry/deadline behavior, plain credentials/sessions, and bounded HTTP
  response decoding live directly in `config.go`, `endpoint.go`, `errors.go`,
  `operation.go`, `retry.go`, `session.go`, and `transport.go`; `doc.go`
  declares the package boundary.
- The catch-all `models.go` is absent with no alias or forwarding file. All ten
  former exported types and all six public `Client` methods remain, the
  generated 33-operation projection is byte-stable, and future
  `subscription.go`, `models_gen.go`, and qualification tests remain absent
  until their B/H/J behavior exists.
- Go test, race, vet, generator drift, module-tidy, and network-disabled
  local-toolchain checks passed on Go 1.26.0/Linux amd64. Direct
  manifest-absent conformance failed closed, while the shared real-daemon
  corpus passed through all six SDKs at corpus digest
  `b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2`.
- This is structural source convergence, not Go SDK or engine qualification.
  Concrete operation models, exact status/media/payload/identity validation,
  opaque credentials, semantic retry/uncertainty, correlated server
  cancellation, explicit bounded HTTP/WebSocket carriage, W3C propagation,
  HTTPS/mesh resolution, supported toolchain/platform/consumer artifacts, and
  installed MX/KV conformance remain owned by B/D/H/J and POAM-011.
- A-07 and A-07.1 remain unchecked; A-07.1f is next.

A-07.1f Java SDK responsibility-boundary evidence (2026-09-08):

- The former 390-line `RrdClient.java` responsibility monolith is now a
  72-line public facade. Current validated construction, loopback endpoint and
  route resolution, bounded HTTP carriage, common operation binding,
  synchronous execution, partial protocol codec, and broad retry/deadline
  behavior live directly in package-private `ClientConfig`,
  `EndpointResolver`, `HttpTransport`, `OperationBinding`,
  `OperationExecutor`, `ProtocolCodec`, and `RetryPolicy` classes;
  `package-info.java` declares the client-only boundary.
- Both public constructors, all five public `RrdClient` methods, and every
  existing public record, enum, and exception signature remain. The generated
  33-operation enum, existing public value classes, tests, generator, and POM
  are byte-stable; future async, WebSocket, generated-model, and qualification
  classes remain absent until their B/H/J behavior exists.
- Maven test passed three current loopback tests and preserved the explicit
  manifest-absent conformance skip under JDK 21.0.12/Maven 3.9.12. Generation,
  compilation, package creation, public/package-private bytecode inspection,
  and the shared six-SDK real-daemon corpus passed at corpus digest
  `b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2`.
- This is structural source convergence, not Java SDK, artifact, or engine
  qualification. Concrete operation models, exact status/media/payload and
  causal validation, opaque credentials, semantic retry/uncertainty, async and
  correlated server cancellation, bounded closeable HTTP/WebSocket carriage,
  W3C propagation, HTTPS/mesh resolution, reproducible/offline signed Maven
  artifacts, toolchain/platform consumers, and installed MX/KV conformance
  remain owned by B/D/H/J and POAM-011.
- A-07 and A-07.1 remain unchecked; A-07.1g is next.

A-07.1g .NET SDK responsibility-boundary evidence (2026-09-08):

- SDK `10.0.111` is selected exactly with roll-forward disabled. Common
  target/compiler/analysis/deterministic-compilation/lock-generation policy
  and the xUnit 3.2.2 version now have one workspace-level owner. The package
  README lives at the .NET root, the byte-identical 33-operation projection
  lives only under `Generated`, and the generator writes only that path.
- The former 455-line `RrdClient.cs` responsibility monolith is an 89-line
  public facade over package-internal validated options, endpoint resolution,
  bounded HTTP carriage, operation binding/execution, partial protocol coding,
  and broad retry/deadline seams. `Models.cs` is absent; each existing public
  option, resource, and session record has one responsibility file. No future
  call, subscription, WebSocket, generated-model, or package-builder surface
  was fabricated.
- A temporary isolated reflection audit compared the committed and candidate
  Release assemblies: all 140 exported type/member signatures matched, with
  zero additions or removals. Locked restore, warning-clean build, all three
  current mock-focused tests, generator drift, and formatting passed under
  .NET SDK 10.0.111/runtime 10.0.11 on Linux x64. The fourth locally reported
  xUnit result remains the documented manifest-absent false pass and is not
  counted as engine conformance.
- Release pack observations produced 24,983-byte and 24,982-byte six-entry
  NuGet packages with the relocated root README, preserving a concrete
  byte-reproducibility failure. The shared six-SDK real-daemon corpus passed at
  corpus digest
  `b3977c57c8d268f861e9d5158e5609bf3e7d2e1db01f914a12911b95c3404cb2`.
  These establish package topology plus direct-seeded rrflowKV client
  characterization, not deterministic/offline/signed artifact or installed
  engine evidence.
- Concrete operation models, exact status/media/payload/causal validation,
  opaque credentials, semantic retry/uncertainty, correlated server
  cancellation, bounded production HTTP/WebSocket carriage, W3C propagation,
  authenticated HTTPS/mesh resolution, external consumers, supported
  toolchain/platform matrices, D-01 installation, rrflowMX/rrflowKV parity,
  crash/reopen, native graph/index atomicity, stamped Arrow/DataFusion,
  persistent reasoning/context/feedback, and Connectome remain owned by their
  B through J gates and POAM-011.
- At the A-07.1g revision, A-07 remained unchecked and A-07.1h was the final
  package/type/path closure pass.

A-07.1h repository-closure and vocabulary evidence (2026-09-08):

- The contextual vocabulary is now explicit: product prose uses `RRFlow`,
  `RRD`, `RrdEngine`, rrflowDB, rrflowKV, rrflowMX, rrflowQL, Arrow, and
  DataFusion; Rust, frozen wire/path values, and generated languages retain
  only their required identifier casing. Third-party names remain confined to
  locked dependencies, outward adapters, comparisons, or provenance records.
- Workspace architecture now rejects Cargo target or path-dependency inputs
  outside the repository, non-member local dependencies, direct or transitive
  Git dependencies, tracked submodules, escaping or unresolved tracked
  symlinks, case-insensitive tracked-path/package collisions, noncanonical
  package descriptions, the retired uppercase query-language spelling in
  active first-party prose, and host-specific
  absolute paths in build or install inputs. Six focused detector/closure
  checks increased that suite from 17 to 23 tests.
- Locked Cargo metadata, both governed-function contract/profile fixtures, all
  23 workspace-architecture tests, the exact retired-symbol search, and the
  complete locked workspace all-target check passed. Focused query parser,
  inference pipeline, public-contract, query-trace, and optional cluster/
  operator-knowledge compilation checks also passed for the files whose public
  diagnostics or examples received canonical spelling.
- Deterministic inventory, documentation, workflow, version, generated-surface,
  knowledge-export, formatting, lint, and diff checks passed as recorded in the
  supporting execution-map journal. The public contract retains the same 33
  operation descriptors; only its human-facing unavailable-surface reason uses
  canonical rrflowQL spelling.
- This proved A-07.1 source/package/type/path closure, not an engine
  capability. At that revision A-07 remained unchecked pending A-07.2; the
  following A-07.2 package now closes it.
  rrflowMX/rrflowKV equivalence and durability, native graph/index atomicity,
  streamed stamped Arrow/DataFusion execution, persisted reasoning/recall,
  installation/attunement, SDK/Connectome conformance, and release proof remain
  owned by their open gates.

A-07.2 causal-evidence vocabulary (2026-09-08):

- `rrd-core` now owns 13 exact `TraceBoundary` values and 53 closed
  `TraceOperation` names. Operation/boundary mismatch, unknown operations, the
  former `domain` field, extra event/link fields, and the retired provider,
  workflow, and operator-knowledge link shapes fail closed with no alias.
- Fourteen generic `TraceLink` forms carry request, actor/scope,
  authorization, read/snapshot, plan/projection, reasoning, source, commit,
  resource, asynchronous-causation, and output coordinates. Forty-six
  `TraceAttribute` names enforce their boolean, unsigned, finite-decimal,
  digest, or bounded lowercase-token value type before persistence.
- Every reviewed current trace producer now uses the exact boundary field and
  generic source link. Its remaining mixed operation and subsystem-specific
  attribute names are admitted only by private sorted inventories that cannot
  overlap the canonical catalogues and can only shrink in their assigned
  C-through-I behavior gates.
- The engine data-flow owner freezes W3C extraction/invalid-context/child and
  asynchronous propagation rules, durable-to-Rust-`tracing`/OpenTelemetry
  mapping, redaction, nonsampling of authoritative evidence, and the physical
  instrumentation each later gate must add. This package does not implement or
  claim H-05.
- The full `rrd-core` suite passed 58 unit tests plus all integration/golden
  suites; the rrflowMX/rrflowKV trace differential passed; the complete
  `rrd-engine` suite passed; and the focused cluster artifact contract passed
  all 10 tests. Strict default-feature Clippy passed for every touched package,
  and all-feature cluster Clippy passed. The feature-gated OpenRaft transport
  test still fails before its trace assertion because application index 8
  reaches the already-recorded unsupported rrflowKV application format; that
  retained storage/cluster defect is not counted as A-07 evidence.
- A-07 is complete, but no C-through-J capability checkbox changes. RRFlow
  still lacks accepted final rrflowKV pages, atomic native graph/BM25/vector
  paths, bounded stamped Arrow/DataFusion streaming, persisted reasoning and
  recall feedback, qualified installation/attunement, public-surface parity,
  Connectome conformance, and clean deployment proof. B-04 is next after the
  completed B-03 model-admission contract.

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
| [x] | B-03 | Define the LFG model-manifest handshake: model/tokenizer digests, routing schema digest, capabilities, limits, runtime, and quantization. | `rrd-contract`, `rrd-inference` | Mismatched contract, model, tokenizer, or resource declarations fail before inference. |
| [x] | B-04 | Define one multiplexed WebSocket frame protocol for authenticated request/response, cancellation, subscription, ACK, and backpressure. | `rrd-contract` | Codec golden tests prove correlation, ordering, limits, unknown-frame rejection, and reconnect resume coordinates. |
| [x] | B-05 | Define GraphQL as a schema-derived ingress adapter that lowers into the same bound RRFlow query representation. | `rrd-contract`, `rrd-query` | Equivalence fixtures show GraphQL and rrflowQL produce the same bound logical request; the shared engine authorization path is unchanged and no second executor exists. |

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

- `RouterBackendDescriptor` declares a canonical identity, revision, supported
  decision kinds, hard dispatch limits, and content digest. B-03 now extends
  that current descriptor with an exact model-manifest identity, revision, and
  digest without changing B-02's three proposal semantics.
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

B-03 evidence (2026-09-08):

- `RouterModelManifest` is a closed provider-neutral contract for immutable
  model, tokenizer, runtime, and constrained-decoding grammar artifacts. It
  binds canonical media/format revisions, exact byte lengths and SHA-256
  digests, the generated `RouteStepDecision` schema digest, supported decision
  kinds, backend and model resource limits, runtime ABI/device/configuration,
  quantization, and grammar source/revision. It contains no provider, path,
  URL, endpoint, credential, or secret authority.
- `RouterBackendDescriptor` now binds one manifest identity/revision/digest.
  `RouterModelHandshake` independently reports the runtime's observed
  manifest, backend, artifacts, schema, capabilities, limits, ABI/device,
  quantization, and grammar; its own domain-separated digest cannot repair a
  stale manifest or backend digest.
- `rrd-inference::load_router_model_after_handshake` invokes its loader closure
  only after the manifest and handshake validate and the supplied model,
  tokenizer, runtime, and grammar byte slices match both declared lengths and
  digests. The returned admission token is opaque and grants no route,
  authorization, storage, or mutation capability. No `RouterBackend`, LFG
  adapter, registry, dispatch, persistence, endpoint, or provider integration
  was created; those remain G-01 and later work.
- `model-manifest-v1.json` freezes the complete manifest/backend/handshake
  chain. Six focused model-contract tests independently reject contract,
  manifest, backend, model, tokenizer, schema, capability, backend-limit,
  model-limit, ABI, device, quantization, and grammar drift. Three inference
  admission cases prove a matching loader runs exactly once while every
  declaration or byte mismatch leaves its call count at zero. The full
  `rrd-contract` package passed 66 tests, `rrd-inference` passed 10 integration
  cases, both packages passed strict all-target Clippy, and all 23 workspace
  architecture checks passed.
- The required full-file inference review also corrected the malformed
  pre-release embedding-job digest domain directly from
  `rrd-inferenceding-job-v1` to `rrflow-embedding-job-v1`; a byte-level test
  freezes the corrected identity and no compatibility branch remains.

B-04 evidence (2026-09-08):

- `rrd-contract` now owns one closed `WebSocketFrame` protocol with exact
  protocol/version, connection, direction-local sequence, request,
  cancellation, subscription, cumulative ACK, heartbeat, error, and
  backpressure coordinates. Validated negotiated limits bound message, frame,
  buffer, request, subscription, timeout, heartbeat, JSON, error, and retry
  inputs before application allocation; unknown members, wrong-direction
  frames, sequence gaps, identity substitution, and oversized inputs fail
  closed.
- The authenticated generic `GET /v1/ws` adapter and Rust `RrdWebSocket`
  reference carriage use that contract. One connection multiplexes durable
  changefeed and live-query subscriptions while `RrdEngine` alone owns resume
  cursor, connection generation, lease, outstanding delivery window, ACK, and
  close semantics. Reconnect clears only the interrupted delivery window and
  replays from the last durable ACK. Cursor-only progress is deliverable and
  ACK-able, so an unchanged live-query result cannot loop forever at an older
  read stamp.
- Generic request, response, and correlated cancellation shapes are frozen,
  but the server deliberately returns `failed_precondition` until H-04 binds
  them to the shared operation dispatcher. TypeScript, Python, Go, Java, and
  .NET document the same dependency and do not claim a WebSocket
  implementation. No transport connection state was added to `RrdEngine` and
  no second lifecycle authority was created.
- The golden protocol suite passed 8 cases; the focused engine subscription
  suite passed 6; the Rust real-server suite passed 3 loopback/mTLS/WSS cases;
  and 3 transport-fault tests covered six malicious-peer scenarios. The full
  four-package run passed 74 contract, 117 engine, 29 server, and 8 Rust-client
  cases. Strict all-target Clippy, locked workspace all-target compilation,
  all 23 architecture checks, documentation/inventory/generated-surface/
  knowledge/workflow/version policies, Cargo formatting, and diff integrity
  passed. This evidence does not qualify persistence, native indexes,
  Arrow/DataFusion execution, reasoning, installation, generated-SDK parity,
  Connectome, or release.

B-05 evidence (2026-09-08):

- `rrd-query` now owns a bounded GraphQL query-ingress adapter. It parses with
  a standards-based GraphQL parser, selects exactly one query operation and
  one root source, derives permitted source/field definitions from the
  catalogue at the requested historical cursor, validates declared scalar
  variables and schema membership, and lowers into the existing `Query` plus
  `Parameters` representation before calling the ordinary `bind` path.
- The adapter supports the seven existing rrflowQL source families, temporal
  coordinates, filters, projection, bounded limits, explain modes, directed
  traversal, and one bounded equi-join. Mutations, subscriptions, fragments,
  directives, undeclared or unused variables, unknown arguments, unknown
  fields/sources, invalid GraphQL names, future cursors, oversized documents,
  and excess variables fail closed. It contains no resolver, storage trait,
  executor, transport, authorization decision, or mutation authority.
- `graphql-equivalence-v1.json` freezes five record/event/traversal/join/claim
  pairs and nine denial cases. Each accepted pair produces the same complete
  `BoundQuery` and bound digest from GraphQL and rrflowQL; the derived schema
  digest is also frozen. A historical-schema test and typed-default test prove
  the adapter cannot invent a second field or parameter model.
- `cargo test -p rrd-query --locked` passed 50 tests across all package
  targets, including all four GraphQL equivalence/denial cases. Strict
  all-target Clippy passed for `rrd-query` and `rrd-engine`; all 23 workspace
  architecture checks passed; and `cargo check --workspace --all-targets
  --locked` passed all 20 packages. This closes Gate B only. H-04 still owns
  the public GraphQL HTTP adapter and cross-surface authorization/semantic
  proof; C-02 now resumes the engine dependency spine.

Gate B exits only when other languages and LFG can implement the contracts from
golden vectors without importing Rust internals.

### Gate C — make rrflowKV the only local persistent substrate

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [x] | C-01 | Freeze one ordered binary key codec for current records, temporal versions, outgoing/incoming edges, scalar values, term postings, vectors, projection deltas, catalogue state, and runtime commits. | `rrd-core`, `rrd-store` | Ordering/golden tests prove prefix boundaries, round trips, tenant separation, and malformed-key rejection. |
| [x] | C-02 | Expose the minimal snapshot transaction primitives required by the semantic store: point read, bounded range scan, put, delete, commit, rollback, and conflict. | `rrd-lsm`, `rrd-store` | rrflowKV and rrflowMX conformance suites agree on read-your-writes, repeatable reads, range ordering, and write conflicts. |
| [x] | C-03 | Commit canonical record, relation, both adjacency directions, synchronous index changes, runtime log entry, durable projection deltas, function invocation receipt and derived proposal, effect-complete audit, and outbox entry as one write batch. Replace the private monolithic function-catalogue control record with typed definitions, bindings, content-addressed artifacts, immutable membership revisions, and one compare-and-swap head under the same transaction authority. | `rrd-store`, `rrd-engine` | The shared rrflowMX/rrflowKV corpus plus failure injection at every prepare/WAL/batch/acknowledgement boundary proves all-or-nothing behavior; an allowed function audit cannot survive a failed domain commit, advertised catalogue limits fit physical limits, and rrflowKV reopens without re-executing a prepared function under another runtime build. |
| [x] | C-04 | Serve current and temporal reads from direct versioned keys at one `ReadStamp`; remove normal-path whole-log reconstruction. | `rrd-store` | Physical counters and plan evidence show bounded point/range reads while exact snapshot comparisons remain equal. |
| [x] | C-05 | Keep Fjall selection, migration-only runtime paths, and alternate stores absent; remove every pre-1.0 reader and alternate format branch from the 1.0 executable. | `rrd-store`, workspace | Fresh rrflowKV database and format-rejection tests pass; repository search and dependency metadata contain one rrflowKV opener and one accepted physical-format reader. |
| [ ] | C-06 | Replace row-record immutable segments with the hybrid rrflowKV layout: an ordered key/version spine plus Arrow-compatible column pages, explicit encoding/compression metadata, and safe buffer lifetimes. Keep point/range/CAS reads independent of DataFusion. The current v4 slice provides the common uncompressed page contract; compression, key/value separation, persisted filters, family grouping, and cache policy remain measured choices rather than assumed architecture. | `rrd-lsm`, `rrd-store` | Frozen format vectors, property/fuzz tests, exact differential reads, selective projection/scan counters, mixed-family interference tests, and comparative benchmarks prove the new layout; eligible uncompressed/aligned pages borrow buffers while all read, decoded, decompressed, copied, allocated, cached, and open-validation bytes are reported. Compare common pages against adaptive per-page codecs and, where value size/update workloads justify it, WiscKey-style separated values; retain a specialization only when its declared workload improves without correctness, recovery, GC, snapshot, or other-family regression. |
| [ ] | C-07 | Prove WAL recovery, manifest recovery, bounded maintenance and write backpressure, pinned-snapshot compaction, Arrow-page lifetime safety, checksums, storage-full behavior, and acknowledged-write durability. | `rrd-lsm` | Crash matrix, reader/compaction concurrency, sustained-write/maintenance/RSS runs, and repeated reopen suite pass with no lost acknowledged write, unbounded write-buffer growth, dangling mapped buffer, or exposed partial batch. |

C-01 evidence (2026-09-08):

- `rrd-store` now owns one strict `RRKV0001` application-key grammar with typed
  `format / tenant / scope / family / tuple` coordinates. It freezes current,
  temporal, outgoing/incoming edge, scalar, unique, term dictionary/statistic/
  posting, vector, projection-delta, catalogue, runtime-commit, outbox, audit,
  engine-event, and system families plus six distinct function-catalogue
  subfamilies. Variable-width values are delimiter-free memcomparable groups;
  numeric and descending-version order is explicit.
- Every existing `RrflowKvStore` read, write, prefix, seek, snapshot inspection,
  and reopen path now uses the typed codec. The manifest authenticates
  `RRKV0001`; absent or different application identities fail closed. There is
  no dual write or earlier application-key reader. Physical-key construction,
  parsing, and its error variant were removed from `rrd-core`, leaving the
  kernel storage-independent.
- The frozen codec fixture covers every C-01 data family and catalogue
  subfamily; tag and round-trip tests additionally freeze engine-event and
  system families. Unit properties cover every possible final prefix byte,
  trailing `0xff` carry,
  embedded NUL/slash/`0xff`, component and tenant/scope isolation, signed and
  unsigned ordering, descending versions, malformed tags/types/padding/UTF-8,
  truncation, and exact round trips. A black-box fixture opens the underlying
  LSM after a real store commit, compares the exact persisted claim, sequence,
  and watermark bytes, then reopens through `RrflowKvStore` and reads the same
  claim.
- `cargo test -p rrd-core --locked`, `cargo test -p rrd-store --locked`, and
  strict all-target Clippy for both packages passed. The focused black-box
  codec test passed, all 23 workspace-architecture checks passed, and
  `cargo check --workspace --all-targets --locked` passed all 20 packages.
- This package closed only C-01; it supplied no C-02 transaction-parity,
  C-03 atomic multi-model write, C-04 direct stamped access, C-05 lower-level
  pre-1.0 reader-removal,
  C-06 hybrid Arrow-compatible pages, C-07 crash/lifetime proof, native graph/
  lexical/vector indexes, streamed DataFusion execution, and persistent
  reasoning/recall evidence.

C-02 evidence (2026-09-09):

- `rrd-lsm` and `rrd-store` expose one consumed transaction with snapshot
  reads, point and half-open bounded range access, read-your-writes, put,
  delete, commit, rollback, typed write conflict, and snapshot pinning.
  rrflowMX and rrflowKV run the same corpus; only rrflowKV promises reopen.
- Claim, control, projection, runtime, and invocation semantics now live in
  concrete repositories over a borrowed transaction port. `StorageEngine`
  retains only transaction creation, repository access, and bounded physical
  evidence; rrflowMX's duplicate semantic maps and rrflowKV's parallel direct
  semantic implementations were removed rather than forwarded.
- Repository conformance proves identical stored semantics on both profiles
  and authoritative rrflowKV readback after reopen. Concurrent disjoint
  control writes replan boundedly around the shared hash-chained journal while
  true compare-and-swap changes still fail; logical snapshot leases create and
  reconcile their physical rrflowKV checkpoint with commit outcome.
- Source-boundary enforcement rejects semantic repositories that import
  rrflowKV, rrflowMX, `rrd-lsm`, or raw database writes, and rejects restoration
  of broad semantic methods on `StorageEngine`. Exact searches found no such
  write-around or former semantic trait method.
- All 157 `rrd-store` tests and all 119 `rrd-engine` tests passed, including
  MX/KV differential, conflict, rollback, snapshot, crash/reopen, semantic
  repository, engine-authority, and workspace-architecture cases. The changed
  inference and downstream package suites passed, as did strict locked
  workspace all-target Clippy. This closes C-02 only; C-03 atomic multi-model
  effects, C-04 direct stamped reads, hybrid Arrow pages, streamed DataFusion,
  and persistent reasoning/recall remain open.

C-03 evidence (2026-09-09):

- One `SemanticCommitPlan` encodes current and temporal records and relations,
  current and temporal outgoing/incoming adjacency, schema-bound scalar and
  unique changes, BM25 and vector source deltas, runtime changes, durable
  projection work, outbox, semantic audit, commit cursor/outcome, and prepared
  function receipts. Both storage profiles consume that same plan through the
  C-02 transaction port; rrflowKV alone claims reopen durability.
- Governed-function catalogues now publish content-addressed binary artifacts,
  typed immutable definition/binding records, digest-only membership revisions,
  and one compare-and-swap head. Public admission caps decoded artifacts at
  3 MiB and canonical catalogue JSON at 4 MiB; the store also constructs the
  exact full publication closure and rejects it unless it fits one current
  16 MiB rrflowKV WAL batch before beginning a transaction.
- Deterministic low-level injection covers prepared, complete-WAL-appended,
  WAL-synced, and process-visible-before-acknowledgement boundaries for crash
  and storage-full modes, both borrowed and owned writes, and buffered and
  authoritative durability. The complete multi-family semantic key set is
  compared before and after reopen at every boundary; no family becomes partly
  visible, and post-WAL uncertainty requires reopen before another write.
- Function recovery rejects a validly resealed prepared receipt whose runtime
  build differs from the pinned definition. A separate lost-acknowledgement
  test makes the domain batch and receipt durable, advances the active
  catalogue to a rejecting function, reopens, and proves the exact receipt
  closes the session transaction with no second guest execution or cursor
  advance. A rejected receipt cannot leave domain state, outcome, or an orphan
  allowed receipt.
- The complete locked `rrd-contract`, `rrd-lsm`, `rrd-store`, and `rrd-engine`
  suites passed: 76, 76, 166, and 117 tests respectively. Strict affected-
  package Clippy, workspace all-target checking, architecture, documentation,
  generated-surface, inventory, formatting, and diff-integrity checks are
  recorded in the C-03d execution journal.
- This closes only C-03. C-04 still owns bounded direct reads; C-05 removes
  earlier physical readers; C-06 builds hybrid Arrow-compatible immutable
  pages; C-07 supplies final storage recovery/lifetime qualification; Gates E
  and F still own native materializers/access paths and streamed DataFusion;
  persistent reasoning/context, installation, routines/skills, Connectome,
  deployment, optimization, and release evidence remain open.

C-04 evidence (2026-09-09):

- One authenticated direct reader selects typed semantic versions through
  bounded point/prefix/version access at a supplied `ReadStamp`. Current-head
  validation uses current-state point reads with zero change reads or proof
  nodes; retained historical stamps use bounded accumulator and direct
  semantic-version proofs. Repository snapshots, transaction previews,
  retirement validation, rrflowQL catalogue/execution, query, vector,
  retrieval, context, memory, seat, router, embedding, and operator-knowledge
  paths consume that boundary rather than reconstructing normal state from the
  runtime change log.
- The shared all-model corpus covers two schema revisions, claims, records,
  relations, events, vectors, series, geo values, immutable references, typed
  retirements, valid-time corrections, and a foreign scope. Direct results
  equal the authenticated-log oracle byte for byte on rrflowMX and rrflowKV at
  retained and current stamps, and again after rrflowKV close/reopen. Retained
  reads reported 49 point reads, 10 ranges, 68 keys/values, and 15,371 decoded
  bytes; current reads reported 89 point reads, 9 ranges, 107 keys/values, and
  17,109 decoded bytes. Reopened rrflowKV reported 9 additional block loads,
  16,128 loaded bytes, and 281 filter checks.
- Query physical plans name direct version sources and expose nonzero stamped
  path evidence. Tight key budgets fail closed rather than falling back to
  replay. Event identity lookup remains bounded, and the exact query corpus
  retains replay only as an independently compared test oracle.
- A workspace source-closure test enumerates every `read_changes`, alternate
  runtime-change reader, and kernel snapshot reducer occurrence by exact path,
  kind, count, and named operation. Normal query, vector, retrieval, context,
  memory, inference, embedding admission, vector-search admission, and
  operator-knowledge admission have no causal-log source. The remaining
  production log users are explicit diagnostic, rollback, cluster artifact-
  transfer, projection-catalogue rebuild, and trace-only conflict-recovery
  operations; reducers outside tests consume directly selected versions.
- The complete locked workspace all-target test suite, complete strict
  workspace all-target Clippy, workspace all-target check, focused source-
  closure and direct-read differential/reopen tests, all 24 architecture
  guards, and documentation/generated-surface/inventory/workflow/version/
  formatting/diff policies passed as recorded in the C-04c3 execution journal.
- This closes only C-04. C-05 still removes pre-1.0 physical readers; C-06
  builds the hybrid key/version spine and Arrow-compatible pages; C-07 owns
  final durability, maintenance, and buffer-lifetime qualification; E owns
  native graph/scalar/BM25/vector access paths; F owns streamed DataFusion and
  one cross-operator resource ledger. Persistent reasoning/context,
  installation/attunement, routines/skills, Connectome, deployment,
  optimization, and release proof remain open.

C-05 accepted convergence evidence and audit correction (2026-09-09):

- C-05a at `b8f07c97775d4440df728bceb261a1128c4d816a` removes the
  pre-1.0 batch, manifest, and segment decoders and the nonempty
  cursor-zero accumulator reconstruction path. Earlier physical bytes now
  fail with an explicit unsupported-version result before another decoder or
  memtable publication; missing authenticated accumulator state at a nonzero
  cursor fails closed on rrflowMX, rrflowKV, and rrflowKV reopen.
- C-05b at `e1a855791fcc0dfcccb7ba72db10bc1804a045f3` requires
  `EstateDocument` authority and all eight backup/recovery maps and requires
  the bound recovery-policy snapshot in both internal and public backup-job
  state. The absence branches, legacy comment, and successful incomplete
  fixtures are removed. Negative tests reject omission of every named field;
  one fresh estate/backup/recovery corpus is identical on rrflowMX, rrflowKV,
  and rrflowKV reopen. This accepts the evidence required to close POAM-018.
- C-05c at `cd1bf495ece3c6698cde1cca413bfbe7be55588c` removes the unused
  `rrflow-cli` development dependency on `rrd-lsm` and adds an executable
  architecture guard over dependency kind, optionality, and feature
  resolution. The independently installable alpha has exactly one required
  production edge into `rrd-lsm`, from `rrd-store`; `rrd-cluster` defaults to
  no implementation feature and `rrd-engine` enables only its pure
  `object-transfer` contracts. The guard also pins the sole current batch v2,
  manifest v2, and segment v3 readers and rejects revival of retired reader or
  backend names.
- C-05e, in the reviewed change containing this record, removes the successful
  vector artifact catalogue v1 validation branch and its shorter identity
  tuple. One current v2 tuple includes build evidence, and an internally
  digest-consistent v1 entry fails before artifact decoding or engine replay.
  Current v2 publication remains identical on rrflowMX and rrflowKV, and the
  rrflowKV catalogue still reopens before missing artifact bytes fail closed.
- C-05f, in the reviewed change containing this record, makes the canonical
  logical table map required and nonempty in both the kernel and public schema
  contracts. Specialized record, relation, and event constraints must resolve
  to an explicit strict table of the matching family; vector, series, geo, and
  object mutations always resolve through the same map. Paired authoring
  methods publish each strict specialized schema with its table identity, and
  snapshot reconstruction has no caller-selected model fallback. Schema bytes
  and generated client projections carry that authority explicitly.
- C-05g, in the reviewed change containing this record, removes every
  collectionless runtime-vector representation. The public mutation, kernel
  value, inference job, persistent vector source, commit digest, source-delta
  key, and compact/quantized/TurboQuant artifact metadata all require the same
  canonical collection plus named-vector address. Omitted artifact metadata and
  public/kernel fields fail decoding, empty physical address parts fail before
  key encoding, and collection/name changes produce distinct commit and source
  identities. Search/list filters remain optional query selectors and are not
  persisted identity.
- C-05h, in the reviewed change containing this record, makes the generic
  artifact catalogue exact/compact/HNSW-only and makes quantization lifecycle
  restoration explicit and revision-neutral. Generic scalar/product/binary/
  TurboQuant publication fails closed; the public TurboQuant
  `ensure_vector_index` shape no longer decodes; runtime reconstruction has no
  TurboQuant suppression branch; and only an active lifecycle generation
  enters the shared planner. Explicit build/activate/search/reopen/retire,
  HNSW, hybrid RRF, memory-tier, and application-backup behavior remains.
- C-05a through C-05c and C-05e through C-05h pass their focused and owning suites, all 28 workspace
  architecture guards, the complete default workspace all-target test and
  strict Clippy matrices, and the repository inventory, documentation,
  generated-surface, workflow, version, formatting, and diff policies. Exact
  commands, counts, source probes, and scope exclusions are recorded in the
  C-05a through C-05h execution journals.
- C-05c closes only the independently installable alpha's physical dependency
  and reader slice. The first C-06 owner read then followed every active C-05
  reference and found compiled successful pre-release paths above that slice:
  vector artifact catalogue v1 beside v2; missing-table schema derivation and
  caller-selected snapshot model fallback; missing vector collection
  addresses; and suppression of the second vector/TurboQuant catalogue during
  runtime reconstruction. C-05e through C-05h directly remove those four
  items. The C-05d correction journal remains the audit trail that prevented
  lower physical closure from being mistaken for whole-executable closure.
- The optional post-alpha OpenRaft implementation still contains direct cluster
  openers and remains visible under POAM-023; it is neither enabled nor
  qualified by the alpha composition and cannot enter a release until a new
  distributed gate accepts it. The accepted single-node executable has no
  remaining confirmed C-05 alternate reader, store, migration, missing-
  authority shape, duplicate vector publication path, or suppression shim.
  C-06 may now begin the hybrid ordered-spine/Arrow-page work.

C-06 progress evidence (2026-09-10; gate remains open):

- `rrd-lsm` now writes and exclusively reads segment v4. Flush preserves one
  sorted `(key, sequence)` MVCC spine and emits Arrow-compatible key offsets,
  key data, sequence values, value validity, value offsets, and value data for
  each row group. A key's complete version chain is never split by the target
  byte or row budget. Point, range, compaction, and snapshot reads use the
  physical pages without invoking DataFusion.
- Manifest v3 authenticates each reachable segment's physical version, schema
  digest, key-codec digest, and page-format digest. Segment v1/v2/v3 and
  manifest v1/v2 have no accepted reader or migration path. The complete v4
  bytes and manifest v3 bytes are checked-in vectors.
- Every page is 64-byte aligned and independently authenticated. Explicit mmap
  returns an Arrow buffer whose allocation owner pins the mapping; bounded and
  io_uring reads allocate aligned Arrow buffers; snapshot-envelope validation
  copies. Page evidence distinguishes read, decoded, borrowed, allocated,
  copied, decompressed, cached, and filter activity. No end-to-end DataFusion
  zero-copy claim is made.
- `DatabaseOptions::segment_row_group_budget` now supplies validated byte and
  row targets to every new flush and compaction output. Each segment
  authenticates those targets in its header, so historical generations remain
  self-describing when future writer configuration changes; the checked-in
  default-format vector remains byte-identical.
- Exact point/range/MVCC comparisons, a version chain larger than one row-group
  target, page corruption, post-open tampering, cache bounds, mmap/bounded/
  io_uring ownership, snapshot, and compaction suites exercise this slice. Two
  implementation failures surfaced and were fixed: all-tombstone groups need
  a full Arrow validity bitmap, and compaction outputs must derive their actual
  maximum sequence instead of inheriting an unrelated durable watermark.
- `hybrid_segment.rs` now runs four fixed-seed histories (384 generated
  mutations) across control, graph-edge, record, lexical-term, and vector key
  families. An independent ordered model checks point, batched-point, full,
  bounded, and disjoint multi-range reads at every committed snapshot before
  and after reopen, then at protected snapshots after pruning compaction and a
  second reopen. Four row/byte budgets remain authenticated, reopen performs no
  semantic-page reads before a query, and 75 deterministic malformed or
  truncated segment files fail closed without a parser panic.
- C-06g captures one sequence, manifest, `Arc<Memtable>`, and eligible
  `Arc<Segment>` set without retaining a database borrow. A bounded registry
  pins the complete manifest closure for garbage collection; later writes use
  copy-on-write memtable generations, and flush/compaction can publish newer
  segments without changing the captured view.
- Its synchronous projected stream validates sorted disjoint half-open ranges
  and all request/resource ceilings, merges one forward cursor per eligible run
  by key, rejects equal key/sequence ambiguity, applies the greatest visible
  version, and suppresses a winning tombstone. It loads key offsets/data and
  sequences first, then only the winning validity page; value offsets and data
  are absent from keys-only evidence and deferred for key-value output.
- Output is emitted in bounded Arrow-compatible `i64` offset/data buffers.
  Stream-local evidence distinguishes page families, cache work, actual mmap/
  io_uring/bounded reads, logical and physical bytes, ownership, decode,
  allocation, copy, output, batches, cancellation, failure, and completion.
  Immutable rrflowKV open evidence separately reports whole-file segment
  validation and startup checkpoint reconciliation; later queries cannot
  mutate that record.
- The independent four-seed history also consumes projected streams with one-
  and three-row batches for keys-only and key-value projections at retained
  snapshots before/after reopen and protected compaction. Focused tests cover
  every declared request/operation/batch ceiling, cancellation/drop lease
  release, duplicate-sequence denial, and old-view survival through flush,
  compaction, and garbage collection. The complete `rrd-lsm` and `rrd-store`
  suites and strict affected-package Clippy pass at the C-06g candidate.
- C-06 is not accepted. Remaining evidence is C-06h broader adversarial,
  property, fuzz, fault, and mixed-family qualification, followed by C-06i
  fixed-hardware comparison of no compression against adaptive page codecs,
  optional value separation, persisted filters, and page-cache policies.
  F-01 still owns the stamped asynchronous DataFusion provider; C-06g does not
  emit a DataFusion `RecordBatch` or claim end-to-end zero-copy.

Gate C exits only when rrflowKV is the sole local persistent implementation and
its correctness is demonstrated below the semantic engine.

### Gate D — install, configure, and attune one real estate

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | D-01 | Implement the [installed lifecycle](../reference/deployment/installed-lifecycle.md) foundation and first walking product spine. The default release exposes one primary `rrflow`/`rrflow.exe` command tree. `rrflow install plan` resolves only bundle-resident, versioned project-bootstrap templates, provider-neutral specialization manifests, and attunement profiles; is byte-deterministic and side-effect free; and covers every file, record, process, adapter candidate, principal/role/grant, credential action, permission, budget, digest, and removal rule. `install apply` accepts the exact digest, explicit fresh/existing mode, and a minimal `.rrflow/config.toml` locator with no canonical mutable state or plaintext secret. Split rrflowKV create-new, open-existing, and read-only inspection so normal open never creates or repairs an estate. `initialize_instance` is the only cold-start path and atomically commits the installed binding, initial seat/security authority, checkpoint, audit, and commit evidence before readiness. Compose `serve`, authenticated `ready`, `version`, and baseline read-only `verify` through the same executable and `RrdEngine`; reusable server/MCP code is linked as libraries rather than checkout-built readiness prerequisites. Remove the old supervisor, standalone bootstrap, server initializer, create-on-open/startup binding, private marker/JSON state, and their successful fixtures in the same direct-convergence package after replacement evidence passes. | `rrflow-cli`, `rrd-engine`, `rrd-store`, `rrd-server`, `rrflow-mcp` | Native real-process tests for Linux and the available platform development runners prove `version`; deterministic plan; no-write preview; exact apply; one high-entropy credential through the previewed sink; authenticated readiness; one commit/query; close/reopen; and baseline read-only verification through the primary executable, with outbound network denied and sibling directories absent. Fresh and existing projects preserve user content and any managed instruction edit is ownership/digest bound. Interruption tests cover staging, credential prepare/delivery, engine commit, locator publication, acknowledgement, and cleanup with no duplicate credential/policy/audit outcome. Drift, path/link/mount replacement, ACL/mode, provider revision, partial state, stale lease, unsupported format, missing installed binding, and already-installed cold start fail closed. `rrflow dev doctor`, a PID/port/marker, source topology, and separate binaries cannot satisfy or substitute for this proof. |
| [ ] | D-02 | Persist attunement jobs and checkpoints through `RrdEngine`; implement status, resume, cancel, leases, idempotency, and phase input/output digests. | `rrd-engine` | Kill/restart tests at each transition resume committed work once and never infer completion from emitted events. |
| [ ] | D-03 | Implement only deterministic project-tree inventory: engine-authorized metadata/content reads, pure proposal building, Git-correct ignore rules, secret/generated/vendor/cache exclusions, stable root-relative identities, Merkle tree digest, containment edges, incremental change set, explicit errors, and bounded work accounting. | `rrd-attunement` through `rrd-engine` | This repository produces the same committed `SourceTreeSnapshot` digest under different traversal schedules; excludes `.git`, `target`, `node_modules`, rrflowDB files, generated bulk data, and secret payloads; handles tracked ignored files, non-Git roots, hidden source, symlink/mount escape, file races, cancellation/restart, and close/reopen; an unchanged rerun reads zero content bytes and commits a no-work checkpoint. |
| [ ] | D-04 | Add incremental Tree-sitter parsing pinned to one committed source-tree snapshot/change-set digest and parser/language revisions; preserve incomplete syntax and errors as evidence. | `rrd-attunement` through `rrd-engine` | Parse is denied before the D-03 snapshot/checkpoint commit; edit-one-file reparses only the changed source, preserves unaffected identities, records `ERROR`/`MISSING` nodes, and resumes after process restart. |
| [ ] | D-05 | After Gates E and F pass, implement normalize, entity-link, lexical-index, embed, vector-index, graph, ground, and verify one at a time against the accepted persistent/index/Arrow paths; add no temporary snapshot index or parallel attunement store. | `rrd-engine` plus owning subsystem | Every phase has an exact fixture, durable checkpoint, failure/retry case, output digest, physical-plan/trace evidence, and independent acceptance test before the next phase begins; lexical/vector/graph output is readable through the same native and analytical paths used outside attunement. |
| [ ] | D-06 | Classify project generators, package scripts, build/test/evaluation harnesses, CI/deployment tools, SQL, PostgreSQL, Turso, Dragonfly, object stores, models, meshes, and other application/operator systems as optional external capabilities or sources. Discovery creates an inactive typed candidate; only an exact previewed immutable binding plus explicit configuration, policy, and authorization may activate it. A command binding captures either one authenticated direct-process argv or the complete package-script/toolchain/lockfile/wrapper/lifecycle closure; literal package-manager argv alone is insufficient. Never install, start, execute, or select an external system as RRFlow persistence or lifecycle authority implicitly. | `rrd-contract`, `rrd-attunement`, `rrd-engine` | Fresh/existing fixtures prove RRFlow reaches default readiness without an external capability; discovery executes nothing; preview performs no writes or effects; exact apply/retirement is durable and idempotent; a generator or harness binding is source- and digest-bound and declares its permissions, sandbox, budgets, cancellation, uncertainty, verification, and output policy without invoking it; mutable/incomplete closures and unavailable enforcement fail closed; credentials are never copied; and clean uninstall removes only RRFlow-owned integration state. Project-command execution and output re-inventory are accepted later by I-03/I-07. |
| [ ] | D-07 | Bind installation to the DevForge CoW placement contract: immutable tools/models may live in the shared lower layer; workspace changes and all rrflowKV WAL, manifest, segment, catalogue, and graph state live in the writable upper layer. | `rrflow-cli`, DevForge adapter | Two clones share the same lower digest while independent writes, crash recovery, and deletion in one upper layer cannot affect the other. |
| [ ] | D-08 | Recognize content-addressed dependency mounts as shared immutable inputs rather than copying or attuning dependency caches into each estate. | `rrd-attunement`, DevForge adapter | Rust, Go, Node, and model-cache fixture proves stable mount digests, zero duplicate ingestion, and correct invalidation when a mounted digest changes. |
| [ ] | D-09 | Measure logical size, allocated blocks, compression, WAL growth, and snapshot size separately; never infer zero-byte or sparse-allocation savings from logical file size. | `rrd-lsm`, release harness | Fresh clone and sustained-write reports account for lower, upper, cache, WAL, segment, and snapshot bytes with reproducible filesystem commands. |
| [ ] | D-10 | Add a hibernation preparation/restore contract that quiesces writes, captures a verified rrflowKV snapshot boundary, exports to the configured cold tier, and resumes without changing estate identity. | `rrd-engine`, DevForge adapter | Interrupted export, corrupt object, restore, rollback, and hot-to-cold-to-hot tests prove no acknowledged-write loss and no split authority. |
| [ ] | D-11 | Complete the installed operational lifecycle: offline read-only quick/full verification; digest-bound offline repair plan/apply; authenticated backup verification; restore into an absent staging target plus explicit publication; explicitly non-authoritative salvage; and ownership-aware uninstall that retains rrflowDB/backups and reports `RETAINED`. Repair may truncate only a proven incomplete final WAL frame, delete only unreachable physical objects, rebuild derived graph/BM25/vector/TurboQuant/analytical projections from canonical semantic state and exact source cursors, or reconcile durable install/attunement receipts. It never guesses manifest, sequence, transaction, authorization, or acknowledged semantic truth, and it never edits the live root in place. | `rrd-engine`, `rrd-store`, `rrd-lsm`, `rrflow-cli` | On the D-01-installed binary, corruption/fault fixtures prove verify changes no bytes; every plan is deterministic and stale-plan rejection occurs before mutation; apply requires an exclusive offline lease, authenticated snapshot, exact preconditions, an absent reflink/clone or fully accounted same-filesystem candidate, audit receipt, full candidate verification, one durable atomic locator replacement, and retention/quarantine of the prior root. Complete-frame/canonical corruption routes to absent-target restore or labeled salvage rather than reset; projection rebuild matches exact oracles; backup restore never overlays live state; and uninstall removes only unchanged RRFlow-owned integration while preserving project state and retained data. |

Gate D exits only when a fresh and an existing project can be installed from
bundle-resident inputs through the single primary executable, attuned,
interrupted, resumed, served, authenticated, verified, repaired/restored under
the explicit safety contract, reopened, and uninstalled without damaging
project-owned state, with the network denied and no external database service.

### Gate E — make graph and indexes native incremental access paths

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | E-01 | Replace graph reconstruction and linear relation scans with temporal outgoing/incoming adjacency prefix scans. | `rrd-store`, `rrd-query` | Directed/typed/depth-bounded traversal matches the exact graph oracle and physical evidence scales with visited edges, not estate size. |
| [ ] | E-02 | Persist scalar and unique indexes transactionally with record mutations. | `rrd-store`, `rrd-query` | Insert/update/retire/conflict/reopen differential proves index and authoritative record cannot drift. |
| [ ] | E-03 | Persist incremental BM25 dictionary, document statistics, postings, positions, and tombstones at a declared source cursor. Select posting-partition codecs from measured raw/bit-packed, partitioned Elias-Fano, and PFOR candidates; codec choice is authenticated metadata and cannot change lexical semantics. | `rrd-query`, `rrd-store` | Incremental results equal a full exact rebuild across update/delete/reopen/corruption fixtures; adversarial sparse/dense/clustered postings prove exact seek/iteration while bytes, decode work, latency, and update/compaction amplification justify each retained codec. |
| [ ] | E-04 | Commit canonical vectors with an atomic index delta; search an immutable HNSW generation plus exact delta overlay and exact-rerank final candidates. Evaluate TurboQuant_prod (MSE quantizer plus one-bit QJL residual) as an authenticated candidate representation and evaluate an LSM-VEC-style disk graph only after the exact/HNSW baseline exists; neither is an assumed default. | `rrd-vector`, `rrd-store`, `rrd-engine` | Exact oracle, recall@k, estimator error/bias, filtered search, update/delete, stale generation, reopen, interrupted-build, build/compaction amplification, RSS, and latency tests pass on declared RRFlow corpora. Approximate candidates never become authoritative results and exact reranking/fallback remains available. |
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

H-05 is executed as five bounded packages rather than one late observability
rewrite:

1. **H-05a — build and signal identity:** add the closed metric contract,
   release-semantic diagnostic Cargo profile, machine-readable build identity,
   runtime diagnostic levels, and automated release/diagnostic feature,
   catalogue, format, result, receipt, and limit parity. This prerequisite must
   exist before D-01 qualifies installed binaries or later packages emit new
   signal families.
2. **H-05b — causal context:** replace direct-store trace emission with one
   `RrdEngine` path and implement bounded W3C ingress/egress continuation,
   asynchronous links, terminal outcomes, crash-visible incomplete starts, and
   cross-request isolation.
3. **H-05c — correlated diagnostics:** project the same operation catalogue to
   structured logs, OpenTelemetry traces, aggregatable histograms/counters/
   gauges, trace exemplars, bounded queues/cardinality, and exporter
   self-telemetry. Diagnostic loss never changes authoritative state.
4. **H-05d — physical path coverage:** complete stage/work/resource evidence
   with the owning C-through-I behavior for rrflowKV, graph, BM25, vector/
   TurboQuant candidates, rrflowQL/DataFusion, context, LFG, routines,
   attunement, commit, and delivery. DataFusion metrics are inputs to RRFlow
   evidence, not another authority.
5. **H-05e — capture and proof:** add the sanitized manifest-bound diagnostic
   bundle and fixed corpus for propagation, redaction, exporter failure,
   overhead/cardinality, deterministic faults, crash/reopen, repeated/missing
   work, release/diagnostic parity, and the complete prompt-to-delivery chain.

The exact instruments, allowed dimensions, build modes, timing boundaries,
capture contents, and fault lanes are owned by the
[engine data-flow architecture](../architecture/engine-data-flow.md#runtime-modes-build-profiles-and-build-identity).
No average-only report, development-profile timing, unbound debug log, or
passing exporter test qualifies H-05.

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

Incremental pre-release source and evidence are published only to the private
`rrflow/rrflow-development` repository. The private `rrflow/rrflow`
repository receives no development branch, tag, binary, artifact, or release;
it is an explicit promotion destination only after every Gate J prerequisite
passes and the repository owner approves the exact revision and artifacts.
Promotion records the source and destination URLs, refs, commit and tree
digests, manifest digest, artifact digests, verification evidence, and owner
decision. A force push or history rewrite is never a release mechanism.

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | J-01 | Remove every superseded pre-release entrypoint, format/catalogue reader, missing-field fallback, backend selector, migration executor, editor/provider-owned automatic hook, duplicate source of truth, transitional alias, and stale generated artifact. | workspace | Strict warning/dependency/terminology searches, negative format fixtures, and all-target builds prove only the accepted RRFlow 1.0 surfaces remain and superseded RRFlow state fails closed. |
| [ ] | J-02 | Run unit, property, fuzz corpus, differential, recorded-seed state-machine, bounded concurrency-schedule, Miri/supported-sanitizer, crash/reopen, storage-full, security denial, resource-budget, exporter-failure, adapter, release/diagnostic-parity, and real-process suites. | workspace | Release evidence records exact build identity, commands/tool versions/targets, seeds and fault schedules, passed/failed/skipped counts, declared tool limitations, post-reopen verification, and retained failure artifacts. |
| [ ] | J-03 | Assemble complete manifest-verified native release-candidate bundles from tracked inputs; use only each bundle's primary `rrflow`/`rrflow.exe` to install and attune an empty fixture and this existing repository; then restart, verify, and repeat representative fast/heavy queries. | release harness | On native Linux, Windows, and macOS qualification runners, with outbound network denied, sibling repositories absent, and no compiler or external database/query/vector service, both estates reach authenticated readiness and verify with stable digests; unchanged rerun is incremental; the repair and restore rehearsal passes; and no checkout helper or manual database repair is needed. This qualifies contents and behavior before J-05 signs the reproducible default distribution. |
| [ ] | J-04 | Publish fixed-hardware rrflowKV, graph, BM25, exact/HNSW, DataFusion, context, LFG, end-to-end, and clean-rollout benchmarks against pinned declared baselines. | evaluation harness | Revision-bound raw histograms name the exact client/server/engine/stage timing boundary and retain failed samples; evidence records build identity, hardware/toolchain/filesystem/device/clock provenance, configuration, corpus/seed, offered-load model, warmup, cache state, concurrency, p50/p95/p99/p99.9, success/error separation, long-duration RSS/CPU/I/O/queue saturation, logical/apparent/allocated bytes, mixed-family interference, recall/quality metrics, and a SurrealDB/Qdrant deployment matrix with artifact and installed bytes, commands, elapsed time, services, ports, configuration, secrets, readiness, and persistent readback. Closed-loop tests correct coordinated omission. No ease or superiority claim is allowed until like-for-like correctness/quality evidence passes. |
| [ ] | J-05 | Produce reproducible signed default distributions whose one primary operator executable is `rrflow` on Linux/macOS and `rrflow.exe` on Windows, containing or manifest-binding every default engine capability, SDK, schema/golden, project/attunement template, configuration/profile, required local inference asset, SBOM/licence, artifact verifier, backup/restore tool, and runbook. Publish archive/executable checksums, a complete RRFlow byte manifest, build provenance/SBOM attestations, and the native platform signature required by the supported target, including Authenticode for Windows. | release tooling | After artifact acquisition, a clean native supported machine with no compiler, source checkout, sibling repository, repository-local cache, package registry, external database/query/vector service, or outbound network uses only the primary executable to run version, install plan/apply, serve, authenticated ready, commit/query/context, close/reopen, quick/full verify, repair rehearsal, backup/restore, and ownership-safe uninstall; every installed byte and runtime dependency resolves from the signed manifest. |

RRFlow 1.0 is releasable only when every Gate J item and every prerequisite is
checked. Until then the repository may describe implemented and measured
behavior, but it must not claim the complete target system is production-ready.
