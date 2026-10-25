# Engine-first alpha code review and execution plan

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-engine-alpha-review`
**Owner:** OBJ-02/OBJ-06/OBJ-07/OBJ-10; existing C/D/E/F/G/H/J execution owners

## Scope and evidence

[PR133](https://github.com/rrflow/rrflow-development/pull/133) published its
brief before edits. Baseline is public main
`dd225072bcc639c89b0eef968625cc73303ea998`, tree
`0ee0344e039b94ea23f124bb2a50209fc7263882`; opening revision
`105f73820440a1db2b27ca1770bc579eb3373e0d`. The owner explicitly requested
engine reliability, scalability, useful debug statistics and code-grounded
research/planning before cloud work. PR132 was returned to draft; its contract
work is paused. This review is bounded to the engine paths named below, not a
claim that every repository file has received an independent review.

The existing executable dependency spine now records the engine-first focus;
C-07's owner records its next diagnostic/growth package. They retain the
existing gate dependencies and acceptance requirements. This journal records
review evidence, not another global implementation inventory or status board.
No runtime, storage format, public schema, configured limit, product version or
running campaign was changed by this planning package.

## Findings grounded in implementation

| Priority / finding | Inspected implementation and consequence | Existing owner and acceptance consequence |
|---|---|---|
| High: ordinary context still performs estate-sized preparation | `rrd-engine/src/engine/context.rs::assemble_context_at` calls runtime `data_snapshot`, copies selected records into a map, extracts text from all visible records/claims and calls `Bm25Artifact::build` on each lexical request. The code explicitly reports `SnapshotBm25`. This is useful reference behavior, not proof of a scalable context path. | E-01/E-03 and H-01: use stamped native adjacency and persisted lexical access. Hold requested output/neighborhood fixed while increasing unrelated estate size; count examined keys/pages and retained bytes. Preserve `engine/tests/context.rs` evidence/reopen, claim, vector discovery, budget and graph-anchor semantics. |
| High: vector hydration precedes selective planning | `engine/vector.rs::read_vector_versions/read_vector_search_versions` requests `Vectors { kind: None }`; `vector/search.rs::search_vectors_at` creates canonical candidates before collection filtering and planner execution. `retrieval_query.rs::execute_retrieval_query` similarly loads scope vectors before evaluating its program. The request key bound limits work but does not make it selective. | C-04/E-04/E-05: select eligible collection/named-vector artifacts and exact delta ranges before hydration. Keep source/catalogue stamp validation, filters, deletes and exact oracle; a cheap ANN stage cannot hide an expensive preceding full read. |
| High: installed project is not an indexed project | `engine/installation/inventory.rs` explicitly computes bounded installation preconditions rather than a committed incremental snapshot. `installation.rs` creates an `AttunementJobState::Pending` record. Existence of that plan/job does not prove phase execution, parsing or source refresh. | D-02/D-03/D-04: durable bounded jobs, committed source inventory and incremental parsing/checkpoints; real edit/rename/delete/revoke/restart fixture with exact affected identities and no duplicate canonical effects. |
| High: native timing exists but installed debug attribution is incomplete | `rrd-lsm/src/write_diagnostics.rs` and database.rs enable/take methods collect bounded accepted-write phase samples; `rrd-store/src/rrflow_kv.rs` exposes native enable/take. `rrd-contract/src/diagnostic.rs::DiagnosticStorageSnapshot` and engine diagnostic projection expose pressure/cache/I/O counters but not those write samples. The collector retains first N samples then counts drops; it is not a rolling view of later growth. | C-07g/H-05: authorized bounded capture through existing engine/public owners, explicit sample selection/window/reset and dropped counts, redaction and measured overhead. No indefinite sampling or destructive interference between observers. |
| Medium: growth setup lacks useful timed progress | `rrflow-cli/tests/support/sustained/corpus.rs::seed_inner` records intents, acknowledgements and independent verification, with shared setup and per-operation limits, but no per-window timing/resource report during those loops. Existing resource_window sampling is used in later workload windows. | C-07g: extend the existing progress/evidence owner with bounded timed ingestion/readback windows and public diagnostics. Preserve original deadlines, independent expected data and full readback. Do not relabel ingestion as soak. |

These are source-level findings. The asymptotic preparation risks follow from
the inspected loops and hydration order; they are not measured speedup claims.
Existing graph traversal, vector artifacts and native index types are retained
implementation inputs, not reasons to skip the flow-level qualification.

## Research and decisions

Primary sources were checked on 2026-10-25:

- [RocksDB write stalls](https://github.com/facebook/rocksdb/wiki/Write-Stalls)
  distinguishes flush, L0 and compaction pressure. Adopt explicit pressure
  attribution and bounded admission at RRFlow's existing maintenance/store
  owner; do not import its backend or assume more worker threads solve the
  measured bottleneck.
- [Qdrant bulk ingestion](https://qdrant.tech/documentation/production-operations/bulk-data-import/)
  separates upload completion from indexing completion and discusses memory,
  batching and read/write contention. Apply explicit committed/indexed source
  cursors and bounded scheduling; preserve RRFlow's canonical state and exact
  delta correctness. Index readiness is not inferred from successful ingestion.
- [Qdrant's experimental deferred visibility](https://qdrant.tech/documentation/tutorials-operations/prevent-unoptimized-usage/)
  trades result completeness for latency. Do not silently adopt hidden or
  blinking records. A partial-index policy would need an explicit RRFlow
  completeness contract and its own qualification; it is not selected here.
- [OpenTelemetry metrics data model](https://opentelemetry.io/docs/specs/otel/metrics/data-model/)
  distinguishes gauges, counters/histograms and cumulative/delta temporality,
  including reset/gap interpretation. Reuse RRFlow's A-07 signal owner with
  explicit units, intervals and process/reset identity; avoid deriving rates
  across restart or treating missing observations as zero.
- [Anthropic contextual retrieval](https://www.anthropic.com/research/contextual-retrieval)
  evaluates contextual lexical/vector retrieval and reranking while describing
  added cost. Treat these as comparisons on RRFlow's held-out code tasks;
  preserve source provenance and measure preparation plus query cost. Do not
  copy their top-K settings or transfer their reported improvement to RRFlow.

No upstream implementation is copied. Three options were considered:
retaining snapshot rebuilding as the normal path (reject for large-project
scaling); introducing a separate database/index service (reject as a second
authority/default dependency); completing the accepted native stamped indexes,
exact delta overlay and bounded execution (selected existing direction).
The tradeoff is more incremental-maintenance and recovery work in exchange for
explicit freshness and work proportional to selected data. Small-data reference
oracles stay tests, not a parallel production lifecycle.

## Execution and review procedure

The six rows in the canonical spine order storage/diagnostics, project
inventory, indexes, query budgets, context/reasoning and joined qualification.
Each bounded runtime PR must name current/target behavior, exact symbols and
replacement evidence; establish its smallest failing oracle before changing
behavior; ship trace/resource attribution with the change; review the complete
diff; run focused then owning/cross-boundary checks; and retain failed as well
as passing results. Freeze exact source/binary/harness/configuration/corpus and
hardware for qualification. No timing-only pass can waive durability,
completeness, quality or authorization.

Current code reading included README and relevant objective/gate/spine owners;
engine-data-flow; the full small vector helper and corpus seeding module;
context assembly snapshot/lexical/graph stages and its named tests;
vector search and recursive retrieval entry paths; installation precondition
inventory/pending-job creation; diagnostic physical projection and public
storage structs; native write collector/phase structs and enable/take callers;
HTTP request trace and installed serve/tracing entry points. Complete changed
roadmap/POAM sections and the diff were reread. This is a package-specific
traceability map, not an exhaustive audit claim.

Changed paths are the existing executable spine, C-07 plan, POAM-035, this new
journal and mechanically regenerated navigation/file inventory. No source was
moved or deleted. The diagnostic gap is linked in existing POAM-035; indexing
and inventory findings remain with their existing D/E owners. No gate closes.

Validation passed: `python scripts/knowledge/render_navigation.py` (17 indexes,
608 nodes, 2415 edges); `python scripts/ci/check_documentation.py` (609 statuses,
607 coordinates); `python scripts/ci/check_workflow.py`;
`python scripts/ci/build_execution_inventory.py` and `--check` (1731 records);
and `git diff --check`. Runtime suites are not rerun for
this planning-only change. Native PR131 acceptance and PR130's frozen full-size
preflight continue under their own exact identities; they are not evidence of
this document implementing missing engine features. C-07, the full alpha and
actual 24-hour soak remain incomplete.

Final rendered-text review corrected six table separators and one gate range
that the shell had reduced to question marks; no scope or status changed.

Change checklist: pre-effect review brief published; owners and relevant code
inspected; concrete findings and replacement oracles named; primary research
mapped to existing boundaries with rejected alternatives; debug/resource and
scope stops recorded; changed documents reviewed; linked journal and generated
projections supplied; no implementation or acceptance claimed by planning.
