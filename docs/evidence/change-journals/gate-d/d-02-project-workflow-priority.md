# D-02 real-project priority and qualification correction

**Status:** active correction evidence; no runtime capability or gate completion claimed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-project-workflow-priority`
**Owner:** OBJ-07, existing D-02 through D-05 workflow and C-07 qualification ordering

## Baseline and change brief

Public development main baseline: `dd225072bcc639c89b0eef968625cc73303ea998`,
tree `0ee0344e039b94ea23f124bb2a50209fc7263882`.
Opening commit: `738e1a1c635f03260feb2d50084a12f6781cad31`.
[PR 136](https://github.com/rrflow/rrflow-development/pull/136) published the
bounded brief before edits; scope additions were published before their edits.
Driver: the owner-directed Codex session. No source dependency PR is required
for this correction. One current pre-release implementation and version 1.0.0
remain; public operations, routes, schemas and wire contracts are unchanged.

Before: the dependency spine required all C-07 qualification before starting
D-02 ingestion, while C-07 expressly prohibited new UI/configuration packages
and Gate D deferred D-05 until every E/F gate passed. After: the existing owners
prioritize a real installed project flow, require its specific canonical index
and streaming prerequisites, and stage final growth/soak qualification after
integrated capability. Defects affecting a slice still stop that slice. Every
runtime package now requires a before/after oracle and must stop/rework if its
promised behavior does not advance. This package proves context alignment only.

The changed owners are README, the alpha objective, release roadmap, dependency
spine, Gate D, C-07, execution checklist and change-authoring procedure, plus
POAM-006/035. Supporting execution-map and D-01 evidence navigation point to the
same current order. The documentation validator's expected navigation heading
changes with its owner; no check is removed. This journal and generated index/
file inventory are the only new evidence/projections. No runtime, UI, test
fixture, workload, binary, historical receipt or completed checkbox is changed.

## Review and traceability

Complete focused records read: executable dependency spine, execution checklist,
change-authoring procedure and D-01 acceptance-reference-alignment journal.
Other reads were targeted sections, not a full codebase audit: README identity,
entry and status; objective OBJ-07; parent roadmap; Gate C-07 execution/admission;
Gate D requirements, installed evidence, knowledge-path/DuckDB dependencies;
POAM-006/035 current deficiencies and relevant retained failures; execution-map
navigation; CI candidate policy and relevant documentation/inventory script code.
All changed passages and the complete diff were reviewed; unchanged long
historical evidence bodies are not represented as newly audited in full.

Source inspection was bounded to these paths/symbols:

- `crates/authority/rrd-engine/src/engine/installation.rs`: pending job construction.
- `crates/authority/rrd-engine/src/engine/installation/inventory.rs`: production
  implementation of installation preconditions, explicitly not D-03 inventory.
- `crates/transport/rrd-contract/src/attunement.rs`: phase and job declarations.
- `crates/transport/rrd-contract/src/deployment.rs`: sealed estate configuration.
- `crates/authority/rrd-engine/src/engine/context.rs::assemble_context_at`:
  materialized snapshot and lexical construction; E/H work remains.
- `crates/authority/rrd-engine/src/engine/vector.rs::read_vector_versions` and
  `read_vector_search_versions`: broad vector sources before collection selection.
- Pinned Connectome `38142b77110a8f1db77e306e7d1ad375fd918267`, inspected in the
  original checkout: `docs/native-workbench.md`, `src/rrflow/attunement.ts` and
  `src/screens/connectome/pages/RuntimeConnection/workspace.tsx` relevant sections.
  Project controls/data lenses exist; phase descriptions remain contract-pending.
- `docs/reference/inference/lfg-swag.md`: effective preference/digest origins
  and hard-policy boundaries.

Structural traceability: obsolete scheduling paragraphs stay at the same
canonical owners and are replaced directly. Accepted safety oracles, gate
checkboxes and historical receipts retain their identities. No implementation
is deleted, moved or consolidated. A pending attunement record and seeded data
lens cannot serve as replacement evidence for an executor or source ingestion.

PR 133's obsolete planning package was returned to draft and dequeued; its
useful inventory/lexical/vector findings are retained in Gate D and POAM-035.
PR 135's unimplemented telemetry package was closed on the owner's instruction.
PR 130/134 source and frozen campaign evidence were preserved. This record does
not claim those PRs merged or their running campaigns passed. No new campaign
was launched for this correction.

## Research and decisions

Primary sources consulted online on 2026-10-25:

- [Google SRE: Data Processing Pipelines](https://sre.google/workbook/data-processing/)
  motivates checking user-facing pipeline correctness and freshness, rather than
  treating isolated storage operations as the full result.
- [Google SRE: Canarying Releases](https://sre.google/workbook/canarying-releases/)
  supports staged validation against a baseline before broad exposure.
- [Qdrant benchmark methodology](https://qdrant.tech/benchmarks/) separates upload,
  indexing, retrieval latency/throughput and precision. Use matched workload and
  quality evidence, not its vendor superiority claims.
- [PostgreSQL regression tests](https://www.postgresql.org/docs/18/regress.html)
  describe functional SQL checks. Functional acceptance and performance evidence
  answer different questions; neither implies the other.

The resulting RRFlow scheduling decision is local engineering judgment grounded
in the missing executor and existing owners, not a claim these sources prescribe
RRFlow's gates. No upstream code, algorithm, dependency or new runtime authority
is introduced. Trace/resource/debug decision: preserve current contracts and
frozen limits; require source/configuration/model identity, real query spans,
readback, edit/delete and isolation in the future workflow evidence. No new
telemetry is needed to prove this documentation correction.

## Verification and limitations

First characterization reproduced contradictory current instructions: C-07's
prohibition, D-05's blanket E/F dependency and the README active-item statement.
The edited owners now agree on D-02's executor followed by real project indexing
and retrieval. Explicitly retain fixed API, safety oracles, real embeddings,
settings origins and isolated projects; no acceptance checkbox is changed.

Initial documentation validation passed: 608 statuses, 606 classified coordinates,
17 navigation indexes, 607 nodes and 2413 edges before this journal was added.
Final commands/results are recorded below after regeneration.

Surfaced command errors: initial PR 133 dequeue used the wrong GraphQL input
field and failed; corrected `id` input succeeded. A PR lookup without explicit
repository resolved the wrong remote and failed; `--repo rrflow/rrflow-development`
resolved PR 136. One brief lookup used the wrong relative parent; subsequent
reads used `../project-workflow-roadmap-brief.md`. A nonexistent scripts knowledge
README and a Windows wildcard path produced search errors; searches were rerun
against existing directories. Oversized read/diff output was truncated and the
changed portions reread in smaller ranges. None is counted as passing evidence.

Runtime/Cargo, new native faults, performance campaigns and a 24-hour soak were
not run for this context/validator-label correction. Existing campaigns keep
their original identities. Missing executor, source inventory, parsing, full
native indexing, effective configuration apply and full C-07 qualification remain
open. No inference about PostgreSQL compatibility or alpha readiness follows.

Checklist: owner/request and baseline bound; brief published before mutations;
existing implementation sampled and uncertainty explicit; exact changes and
research recorded; canonical destinations preserved; before/after documentation
oracle defined; complete changed diff reviewed; retained failures/unrun checks
explicit; linked journal and projections regenerated before commit. Stop for API
changes, alternate stores, ungrounded completion or relaxed failure thresholds.
Roadmap status: priority corrected, no gate completed; POAM-006/035 remain open.
Candidate CI and merge evidence remain pending and are observable on PR 136.

Final local acceptance:

- `python -X utf8 scripts/ci/check_documentation.py`: passed, 609 statuses and
  607 classified coordinates, ownership/index/local-link checks retained.
- `python -X utf8 scripts/knowledge/render_navigation.py --check`: passed,
  17 indexes, 608 nodes, 2414 edges, no drift.
- `python -X utf8 scripts/ci/build_execution_inventory.py --check`: passed,
  1731 current/generated/planned records.
- `python -X utf8 scripts/ci/check_workflow.py`: passed, 3 workflows,
  9 substantive jobs, 5 engine suites, 20 default/6 optional packages, 19 pins.
- `git diff --check`: passed.
- Inline baseline comparison: all changed roadmap checkbox ID/status pairs
  identical; current owners contain no obsolete C-07 active/next claim; no
  runtime, schema, Cargo version/lock, Connectome gitlink or historical
  active-change bytes changed. Allowed scope is docs plus the exact documentation
  heading assertion. Generated inventory is refreshed after this journal edit.

These are documentation acceptance results, not a runtime feature demonstration.
