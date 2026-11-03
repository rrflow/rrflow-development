# Installed project indexing: implementation and evidence plan

**Status:** active supporting implementation proposal; no runtime or gate accepted
**Coordinate:** `rrflow://rrflow-instance/data/design/installed-project-indexing-plan`
**Owner:** [Gate D project workflow](gate-d.md#project-workflow-build-packages); [alpha objective](../../objectives/rrflow-1.0-alpha.md#observable-engine-objective); [POAM-006](../../poam/rrflow-1.0-alpha/poam-006.md)
**Reviewed:** public main `3a72a6b4adc43c842f67a59f4fc91d725eac7580`; parser draft [#241](https://github.com/rrflow/rrflow-development/pull/241) `63ace16fa5155fc0d91ca57e594368549779922b`; inherited-directory draft [#247](https://github.com/rrflow/rrflow-development/pull/247) remains separate and incomplete.

This is an implementing-agent handoff for the first real project-to-query cycle. It refines the [adaptive developer design](adaptive-developer-build-design.md), not the release order, alpha objective, POA&M status, or a fixed runtime sequence. Refresh the revisions, PRs, their CI and source references before implementing each package. A local test or a proposed symbol below is not evidence that an installed workflow works.

## Exact result to build

A person selects an existing project, sees the proposed estate path, exclusions, inherited access, work limits, optional capabilities and exact effects, and accepts the preview. The installed engine creates or reopens its own project identity and map, reads permitted source without moving files or changing inherited ACLs, and reports background progress. It durably publishes an inventory and indexed source spans with provenance. After a restart, an authenticated rrflowQL/context request returns the same permitted source evidence and read stamp; Connectome displays it using the public engine contract. An edited or deleted file invalidates only dependent current results and can be reprocessed without losing unaffected committed work. A second project cannot see the first project's records.

The project map is a versioned, replaceable discovery projection of the
engine-owned project identity, source bindings and committed inventory, not a
second planning or lifecycle owner. The installation preview must name its
exact path and replacement/removal policy before an authorized write; a missing
map can be regenerated from the engine, and a modified or foreign file cannot
silently change engine authority. The implementing PR must first identify
whether an existing template or public operation already owns this projection;
the inspected installation paths did not establish a current automatic map.
Minimize upfront questions to the project boundary and decisions genuinely
needed for the selected effect. Continue permitted scanning and indexing in the
background with visible progress; an unanswered optional provider question
must not stop local inventory. Idle/soak work uses configurable foreground
resource ceilings and must pause or yield when those ceilings require it.

The first joined proof may run without a model or vector provider: lexical and graph retrieval with source citations must work, and unavailable vector/inference capability is reported. The complete alpha objective additionally requires vector access, configurable reasoning and other named gates; a lexical slice cannot close it. Knowledge-path connectors, LFG, SWAG and provider capture are explicitly configured integrations; the default installed estate cannot depend on them. Parquet is analytical interchange, not the canonical commit log or a prerequisite for a small local project.

RRFlow's intended fast prompt cycles and long-running multi-day work both use
the same authority, persistent state and configurable effort/resource policies.
Long jobs need durable checkpoints, bounded leases and heartbeat/recovery,
interruptible execution, provider timeout/uncertainty reconciliation,
accumulated budgets and evidence readback; merely leaving a model call running
for days is not a durable task. LFG and SWAG connect through explicit engine
contracts when available. Their planned presence does not demonstrate reasoning
quality, latency or parity with SurrealDB; measure those separately on defined
workloads after the installed project loop works.

## Evidence map: what exists and what is missing

| Boundary | Inspected implementation and limitation | Change destination |
|---|---|---|
| Project admission | `crates/authority/rrd-engine/src/engine/installation.rs` and `installation/inventory.rs` scan under a compiled policy. `AnchoredPath` and observations constrain source access. The current scan retains entries and is not an atomic filesystem snapshot. The existing-project branch #247 still has unresolved interrupted staging and ACL re-admission. | Keep preview/apply, source anchoring and installed identity in `RrdEngine`; reconcile retained directories without altering project ACLs, moving existing entries or replacing user data. |
| Default plan | `assets/install/default-profile-v3.json` and `clyffy-profile-v3.json` contain the same eleven phases; `installation.rs` copies the profile absent a supplied proposal. `rrd-contract/src/attunement.rs` validates a linear phase chain ending in Verify. | Replace fixed default selection with available, configured engine capability/effect selection and versioned dependency-bound records. Preserve traceability of existing formats and characterize them before removal. |
| Inventory | `attunement/inventory.rs`, `inventory/records.rs`, `publication.rs`, and `reuse.rs` bind proposal, source recheck, publication groups, lease and head. The final transaction links job checkpoint, receipt and head to the runtime read stamp. | Reuse the same canonical runtime and transaction authority; prove partial groups are not presented as a complete head. Make large-snapshot reads bounded rather than reconstructing the entire proposal. |
| Parse | Main's `engine/attunement.rs` executes Connect and Inventory and returns `phase-unavailable` for other phases. Draft #241 adds `rrd-attunement/src/parse.rs` and `worker.rs`, a selected policy, bound source reading and worker response validation. It does **not** supply the native supervisor, publication or installed Parse execution. Its `records::load_for_parse` currently reconstructs the whole Inventory proposal with a size bound. | Complete #241 or carry its characterized behavior into the adaptive capability contract. Do not activate until native worker admission, source/lease recheck, transaction publication, replay and the real installed test pass. |
| Query and prompt context | `engine/query.rs`, `engine/context.rs`, `engine/retrieval.rs`, `engine/vector/search.rs`, and `rrd-query` contain real query/retrieval components. The draft `parse_source_is_bound_to_the_committed_inventory_after_reopen` traverses the committed Inventory relation after reopening; it does not publish parsed spans. `context.rs` currently fixes RRF constant and seed/text/vector/graph weights in code. | Read the same persisted source/span identities through scoped rrflowQL and context, then make ranking/graph selection a versioned bounded policy with effective values and origins visible. Check actual D-05/E/F prerequisite evidence before claiming access paths qualify. |
| Parquet | `engine/query_export.rs` rejects truncated results and currently bounds the export at 1,024 rows. `rrd-query/tests/parquet_export.rs` characterizes typed bounded files. | Keep the bounded export honest. D-06/F own larger streaming, import and external interoperability; none is proof of native WAL or project indexing. |
| UX and integrations | The pinned Connectome's `src-tauri/src/rrd/client.rs` and `estate.rs` contain client/preview paths; static phase declarations in `src/rrflow/attunement.ts` do not prove a live screen. The pinned UI has not been fully qualified for this journey. | Audit the exact pin and wire existing/public generated operations to engine state. Add client-facing status only where the authoritative operation exists; no UI-owned lifecycle or plan. |

## Source and effect model

The engine owns a versioned project identity and installation record, an admitted root/knowledge-path binding, a policy revision, a source snapshot/read stamp and durable job/receipt records. Record these distinct identities in every derivation: `estate_id`, `project_id`, `source_binding_id` and revision, root-relative path and stable entry ID, content digest, snapshot/change-set digest, parser/grammar revision, effective configuration digest, authorization revision, plan/effect ID and read/commit coordinates. Do not use filesystem path text alone as a durable authorization token.

Source spans use bounded byte offsets and the content digest of the exact source observed. Syntax ERROR/MISSING nodes remain evidence, not silently repaired source. Extracted entities and relationships must distinguish deterministic syntax facts from model-derived proposals, carry provenance and model/configuration identity, and avoid representing guesses as verified facts. New source-derived rows, relations, index entries and invalidations go through the existing engine mutation/transaction owner. An index is derived from the canonical records and rebuildable; neither a local cache nor a vector service is a second semantic authority.

Treat an effect's dependency set as more than file hashes when it depends on directory membership, a negative lookup, policy, access grants, tool version, configuration, a question answer or a query range. Re-observe those inputs and the lease immediately before each publication or external mutation. A mismatch refuses that effect without publishing it; record why, propose a successor linked to the old plan, and retain committed receipts. An unrelated edit must not invalidate an effect whose complete declared dependencies are unchanged. If dependency completeness cannot be established, refresh the relevant boundary conservatively and report the weaker guarantee. Do not claim filesystem check-and-write is atomic when the OS boundary does not provide it: use anchored handles/identity checks and test the residual race.

Groups may be committed incrementally only with a durable job-bound cursor, idempotent group identity and bounded aggregate input/output/resource accounting across retries. Publish the new complete head only after all groups and their dependencies are verified. Intermediate groups must be unreachable as a complete current index. On restart, reconcile committed groups and the existing head before resuming; stale workers fail their fence. For delete or changed permissions, retire/tombstone dependent current results at an authorized transaction boundary, retain historical read-stamp behavior as supported, and deny newly unauthorized reads. No unaudited fallback to old current results.

## Handoff shape and enforcement

The following is a **semantic example**, not an implemented wire schema. Derive the eventual versioned schema and SDKs from the executable contract owner. A schema alone cannot force an external agent to obey a plan; only engine-mediated effects can be rejected at admission, and an independent checker must compare actual external changes and receipts.

```json
{
  "version": 1,
  "estate_id": "example-estate",
  "project_id": "example-project",
  "plan_id": "example-plan",
  "plan_digest": "<canonical digest>",
  "predecessor_digest": null,
  "configuration_revision": "<digest>",
  "read_stamp": "<engine-issued coordinates>",
  "questions": [{"id": "q1", "state": "answered", "answer_digest": "<digest>"}],
  "effects": [{
    "id": "effect-a", "capability": "<registered name and version>",
    "inputs": [{"kind": "source", "identity": "<entry id>", "digest": "<content digest>"}],
    "depends_on": [], "required_questions": ["q1"],
    "authorization": "<grant revision>", "budgets": {"input_bytes": 1, "output_bytes": 1, "elapsed_ms": 1},
    "idempotency_key": "<unique key>", "verification": "<observable predicate>"
  }]
}
```

The numbers are placeholders, not recommended budgets. Validate structural bounds, canonical encoding/digest, supported capability and typed parameters, acyclic dependencies, scope, authorization, relevant source observations, questions, aggregate budgets and expected revision at the actual effect boundary. The list contains only effects selected for that request; empty and differing selections are valid when no effect is necessary. Exploratory reading and private reasoning are not forced into this schema. Unknown effects fail closed, while unfamiliar non-effect ideas remain representable. Durable result records bind the actual outcome, source/read/commit coordinates, actor, memory changes, errors and successor lineage; a sampled trace is not an audit ledger. A lost external acknowledgement is uncertain until idempotency or readback establishes the result.

For implementing-agent handoff, each PR description and linked journal must
also identify: owning alpha outcome and roadmap package; baseline revision and
current behavior; intended public behavior; exact affected source, existing
characterization tests and canonical destination; input/effect and provenance
model; failing oracle and exact commands; trace and resource decisions; test
results and unrun checks; explicit stop conditions; and per-effect evidence or
reason for a successor plan. This is a repository authoring record, not a new
runtime lifecycle. An agent must compare each proposed change with that record
and update it before changing scope. The reviewer independently compares the
actual diff, command output and persisted receipts with the record. A prose or
JSON document cannot itself prevent an actor with direct filesystem access
from bypassing the engine; do not describe it as such.

## Implementation packages and first failing oracles

These packages are review/merge units following the existing D-02/D-03/D-04/D-05 and H-06 owners, not required runtime phases. Before code, write each PR's brief, exact source/test map, format-compatibility decision, smallest red test, trace/resource decision, stop condition and linked journal; characterize existing behavior and retain it at the canonical boundary. Reconcile overlapping branches rather than starting competing edits to generated surfaces.

1. **Admission and adaptive work selection (D-02/D-03).** Characterize current default profiles, jobs, serialized digests and actual installed preview/apply. Map and replace the closed phase assumptions in `rrd-contract/src/attunement.rs`, `rrd-engine/src/engine/installation.rs`, `engine/attunement.rs`, their tests and generated public projections. Use one engine-owned versioned capability owner, selected dependencies and per-effect source/read identity. Define what happens to an existing pre-release estate/job before changing its format; do not invent a second lifecycle. First red oracle: two different projects or policies select different available work, unknown effect is rejected, no provider is activated, and an unrelated edit does not trap a completed independent effect. Preserve exact-preview authorization and do not silently change ACLs. Coordinate #247's staging/ACL recovery before relying on retained directories.
2. **Bounded snapshot access and incremental freshness (D-03/D-04).** Characterize `installation/inventory.rs`, `attunement/inventory/{records,publication,reuse}.rs` and `rrd-attunement/src/inventory.rs`. Design a bounded, authenticated, ordered/paged membership traversal at the committed stamp; authenticate each page/head, charge keys/bytes cumulatively, and preserve snapshot completeness and existing receipt/head atomicity. Avoid materializing every record or assuming the whole snapshot fits one write transaction. First red oracle: a deliberately small per-group budget processes a multi-group fixture, restarts between groups without duplicate publication, and refuses head substitution or changed relevant input. Then measure the roadmap's 100,000-record profile on an actual installed engine; no large-scale claim follows from a small fixture.
3. **Isolated parser and source publication (D-04).** Build on reviewed #241 bindings and framing; `engine/attunement.rs` remains the authority. Pin the shipped worker executable and digest, admit only explicit inherited handles, apply an OS-enforced memory limit before worker code runs, enforce time/cancel by terminating and reaping the exact child, and reject truncated/late/forged/out-of-budget responses. Windows needs suspended creation plus job assignment before resuming and a tested parent-death cleanup strategy; Linux needs a pre-exec limit and separately proven parent-death cleanup. Do not claim `RLIMIT_AS` is resident-memory accounting. Unsupported platforms fail closed until qualified. Re-read the authorized source/checkpoint and fence before publishing bounded span/relationship groups. First red oracle: the real primary executable indexes one source, kills a worker mid-request and between publication groups, restarts, and returns exactly one final source/span head; substitution, revoked access and source edits publish no stale result. An orphan between process creation and containment is a stop condition, not a tolerated race.
4. **Source-derived retrieval and configuration (D-05/E/F/H-09).** Map actual `rrd-query` graph/BM25/vector paths and `engine/{query,context,retrieval}.rs` before changing them. Publish versioned source chunks/spans and deterministic structural relations under the same rrflowKV transaction authority; link lexical and graph indexes to those identities and the committed read stamp. Make retrieval avenues, fusion weights, graph depth, limits and optional vector/model selection explicit versioned effective configuration. Preserve authorization before candidate generation. First red oracle: an installed engine indexes two files, a distinctive query returns the correct source identity and byte span after reopen, a deleted or forbidden file no longer appears at current scope, and two valid policies expose different effective selection without inventing an unavailable vector result. Qualify native vector/embedding and TurboQuant separately against exact filtered search and held-out tasks; do not equate a vector library test with source-ingestion proof.
5. **Job, question and evidence surfaces (D-02a/H-04/H-05).** Extend the existing operation catalogue and engine job/receipt stores with the authorized list/read/status/resume/cancel, question and receipt capabilities required by OBJ-14. A question blocks only effects that name it; background progress exposes counts, budgets, exclusions, error/paused state and the last committed head. Record memory reads/writes and uncertainty in durable readback, correlated by plan/effect/job/source IDs. Traces can be sampled and contain no source or credentials by default. First red oracle: kill the installed process before/after a committed job transition, replay an identical request, and compare visible job state and receipts with actual mutations; no duplicate effect or missing acknowledged memory receipt.
6. **Installed joined proof and Connectome (H-06/J).** Add an executable integration driver to `crates/adapters/rrflow-cli/tests/installed_lifecycle.rs` or its existing support modules; the precise new test name/command is a deliverable, not an existing command. It must launch the built `rrflow` binary, use public authenticated operations, and retain exact binary/UI/source/configuration digests, platform, request/receipt IDs, read stamps and raw results. Run the fixture, ordinary edit, deletion, crash/reopen and independent checker; repeat with a separately selected real project and two-estate isolation. Audit and test the exact Connectome pin using only generated/public client operations. Until an actual desktop is launched and compared with engine readback, UI acceptance remains open.

The first executable target is package 3's real source/span result joined to packages 1–2, then package 4's retrieval and package 6's installed proof. Preparation for the latter starts with its failing driver, not after a collection of isolated green unit tests. This dependency order does not require eleven runtime operations or prevent independent read-only exploration. The repository's existing D-05 sequence and gate prerequisites still govern acceptance.

## Tests, measurements and stop conditions

Existing characterization commands, not claims about this new implementation:

```text
cargo test --locked -p rrd-contract --test attunement_contract
cargo test --locked -p rrd-engine --lib inventory_checkpoint_and_source_records_commit_reopen_and_replan_together
cargo test --locked -p rrd-engine --lib git_selected_plan_ignores_irrelevant_edits_replans_tracked_edits_and_reopens
cargo test --locked -p rrflow-cli --test installed_lifecycle -- --nocapture
```

On #241, `cargo test --locked -p rrd-attunement` and its actual `parser_worker` process test characterize the draft; they do not prove native supervision or engine publication. Confirm every filter selects a nonzero test count. Add a named installed-flow driver and run the smallest red case before each package's code, then its owning suites, generated contract/SDK checks, candidate CI and protected merge-group CI. Preserve existing crash and reopen cases rather than replacing them with mocks.

The independent expected ledger is derived from the fixture files and authorized plan, not from the engine's claimed totals. Compare source bytes/digests, span bounds, current graph/lexical results, actual file effects, job receipts, and restored state. Required cases include excluded/secret paths, symlink and mount escape, permissions revoked mid-run, file replacement, malformed syntax, unsupported language, changed and unrelated files, deletes, size/node/time limits, slow or killed worker, stale lease, repeated idempotency key with same and different content, cancellation, storage-full, process death before and after publication, and two estates. Distinguish acknowledged success, denied, retryable, incomplete and uncertain outcomes.

Measure initial discovery/read/parse/index time, bytes read/written, records, peak RSS and storage, queue time, restart work, per-query latency, candidate counts, truncation and source freshness; record cache state, workload and hardware. Compare warm/cold and edit-one-file against full rebuild, and monitor foreground/idle resource budgets. A watcher is an optimization, not the source of truth: overflow, restart and missed events require rescan/reconciliation. Do not promise zero content reads based solely on metadata. For retrieval, freeze corpus, exclusions, judgments, exact filtered oracle, baselines, seeds, metrics, thresholds and confidence procedure before comparing. Report recall/precision or nDCG, freshness and citation validity with latency/resource distributions; include vector-disabled and stale/denied cases. Freeze 100,000-record and native capacity/soak profiles from the owning gates rather than inventing new passing thresholds.

Stop implementation or keep the affected capability unavailable if source containment or authorization cannot be rechecked, native memory/deadline/child cleanup cannot be enforced, output/transaction limits are exceeded, durable progress cannot be replayed without duplication, an old snapshot can appear current after an edit, or the independent oracle disagrees. Record failures and missing native platforms; do not increase budgets, fake storage errors, weaken assertions, or call this C-07 qualification. Exact-candidate C-07 native capacity, recovery and soak remain separately required.

## Research applied and open decisions

- [Tree-sitter's parser API](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Parser.html) supports cancellation-related options and requires an old tree to be edited to match changed text before incremental reuse. The draft pins 0.24.7; confirm behavior against that exact version, not only latest documentation. A callback is not OS memory isolation.
- Microsoft's [process creation flags](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags) define suspended startup; the chosen job/handle association and parent-death behavior still require real native tests. Linux [`getrlimit`](https://man7.org/linux/man-pages/man2/getrlimit.2.html) documents inherited limits and `RLIMIT_AS` virtual address-space behavior; it does not establish aggregate process-tree isolation.
- [Qdrant's hybrid-search tuning](https://qdrant.tech/documentation/search-tuning/how-to-tune-hybrid-search/) describes rank-fusion tradeoffs and calls for evaluation on one's own labels. Its [quantization documentation](https://qdrant.tech/documentation/quantization/) discusses accuracy, oversampling and rescoring tradeoffs. These motivate configurable experiments, not an RRFlow performance claim or automatic choice of Qdrant as authority.
- [Apache Parquet](https://parquet.apache.org/) defines a column-oriented analytical file format. Its presence does not establish transactional WAL, native indexing, full streaming or import. [OpenTelemetry sampling](https://opentelemetry.io/docs/concepts/sampling/) can omit spans; complete durable effect receipts require their own retained record.
- The owner withdrew the prior DuckDB adapter and worker direction. [Native analytical database research](../../reference/data/duckdb.md) maps DuckDB vector execution and ClickHouse sparse indexing to RRFlow query/storage boundaries and records upstream licenses. Both have substantial C++ implementations; adapting behavior or porting eligible code to Rust is a real engineering and testing task, not a rename or automatic transfer of performance.

Before the relevant package is ready, decide and document: (a) canonical versioned capability/plan compatibility for already installed estates; (b) exact bounded snapshot paging/proof and safe source invalidation with concurrent edits; (c) native child creation/parent-death containment per supported platform; (d) source/chunk schema and lexical/vector publication dependencies; (e) the actual public configuration mutation and question UX where the existing owner does not yet authorize one; (f) the held-out dataset and pass/fail thresholds. These are engineering decisions with named owners and tests, not reasons to block unrelated implementation or manufacture defaults.
