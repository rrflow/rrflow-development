# Gate A — freeze authority, names, and boundaries

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/gate-a`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [x] | A-01 | Define RRFlow, RRD, `RrdEngine`, rrflowDB, rrflowKV, rrflowMX, rrflowQL, Arrow substrate, DataFusion execution, RRFlow vector and inference subsystems, LFG, and Connectome exactly once. | `README.md`, system overview, ADR-0001 | Terminology, execution topology, and authority boundaries use one meaning for every term and prohibit parallel database, model, client, or compute authority. |
| [x] | A-02 | Define generic `reasoning_tree`, `reasoning_node`, typed `reasoning_edge`, recipe, active cursor, decision evidence, and verification-result semantics. | `rrd-contract`, `rrd-core` | Versioned schema and golden round trips reject unknown fields, invalid edges, and unverifiable cursor advances. |
| [x] | A-03 | Resolve the pending reasoning-ledger removal against A-02 without restoring a hard-coded universal reasoning lifecycle or deleting reusable semantics. | `rrd-core`, `rrd-engine`, CLI | Golden/API diff proves reusable data moved to the generic contract, contains no forced Goal→Plan→Attempt sequence, and focused core, engine, and CLI tests pass. |
| [x] | A-04 | Move all existing crates into one non-duplicated grouped source tree, remove the empty `rrd-graph` boundary, and remove `connectome-ui` after its public-client behavior is present in the separate Connectome repository. | workspace | `cargo metadata`, dependency-direction check, and repository search show the declared layout and no second graph, memory, routing, lifecycle, UI, or provider authority. |
| [x] | A-05 | Remove stale documentation claims or mark supporting documents historical where they describe another architecture. | documentation | Repository link/terminology check finds no supporting document presented as current authority. |
| [x] | A-06 | Establish the documentation memory topology: the root and each major source-boundary README are warp maps into one owning `docs/<subject>/` record set; classify every flat document without duplicating content; generate a deterministic content-addressed manifest/JSONL bootstrap package for later authorized rrflowDB ingestion. | documentation | CI proves every active record has status, owner, stable coordinate, one inbound owner link, valid local fallback links, and no duplicate roadmap or architecture body; repeated packaging produces byte-identical ordered records and digests with an explicit inclusion/exclusion ledger and no silently omitted eligible record. |
| [x] | A-07 | Audit the actual dependency graph, public vocabulary, implementation-requirements traceability, and causal evidence vocabulary; then freeze industry-aligned directory, crate, module, test, fixture, binary, command, configuration, environment, wire, persisted marker, digest/media domain, low-cardinality operation, typed-link, and trace-attribute names. Directly rename the overloaded function `AutomationCatalogue` and pre-commit `FunctionTrigger*` family to the canonical function-catalogue and transaction-function-binding vocabulary, and split their implementation from later committed-event triggers and routines. Keep every first-party build/install/runtime input inside this repository and converge overlapping pre-release boundaries directly with no forwarding aliases or parallel execution paths. | workspace | The reviewed traceability matrix accounts for every affected current behavior, source module, test, fixture, and planned destination; a frozen trace map assigns ingress, engine (including governed function execution), KV, QL, graph, lexical, vector, DataFusion, inference, attunement, routine, adapter, and delivery work to one naming/coordinate scheme; the function contract has one golden closed-schema fixture and no old name/field decoder; case-insensitive terminology, `cargo metadata`, dependency-direction, tracked-path, and owning-suite checks prove every package has one responsibility, every dependency points inward, no successful old-shape reader/default/alias remains, every local dependency/target is under the workspace root, and no tracked submodule, escaping symlink, host-specific absolute path, sibling checkout, or Git dependency supplies RRFlow code. |

## A-06 knowledge-bootstrap sequence

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

## Later knowledge-persistence sequence

These milestones preserve the original knowledge-package identifiers but are
owned by the later runtime waves. They do not block A-07, Gate B, or Gate C.

| Done | ID | Bounded change | Acceptance evidence |
|---|---|---|---|
| [ ] | KB-06 | Import a verified KB-03 package through persisted attunement checkpoints and authorized `RrdEngine` mutations. | D-02 and D-05 evidence proves digest-bound resume, idempotency, authorization, atomic mutation, and rejection of package or configuration drift. |
| [ ] | KB-07 | Prove close/reopen, readback, warp resolution, incremental update, rollback, and recovery against rrflowDB. | Durable tests reproduce every imported record and relationship at its committed read stamp after restart and failure injection. |
| [ ] | KB-08 | Make authorized rrflowDB warp resolution the normal client path after import while retaining the already-thin bootstrap READMEs as recovery maps. | H and J evidence shows clients resolve the same authorized records through public operations; local fallbacks remain sufficient for recovery without duplicating mutable state. |


## [Accepted evidence](gate-a-evidence.md)

## Exit condition

Gate A exits only when the worktree contains one architecture, one navigable
documentation memory, an industry-aligned and dependency-checked source tree,
the generic tree contract, and no obsolete lifecycle implementation.
