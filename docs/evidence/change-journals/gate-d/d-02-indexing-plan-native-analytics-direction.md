# Project-indexing plan and native analytical direction

**Status:** active design and owner-direction update; implementation and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-indexing-plan-native-analytics-direction`
**Owner:** Gate D project workflow, C-07 and [PR #243](https://github.com/rrflow/rrflow-development/pull/243)

Starting design candidate `2c7ed68140cc7ad103381415e436c19b5b24432b`; public main `3a72a6b4adc43c842f67a59f4fc91d725eac7580`; separately reviewed parser draft #241 `63ace16fa5155fc0d91ca57e594368549779922b`. PR #243's brief was extended before the plan and before the owner-direction updates. Neither candidate is merged or acceptance evidence.

Reviewed README, alpha objective, Gate D, executable dependency spine, POAM-006, ADR-0002, the existing adaptive developer design and its journal; current `rrd-engine` installation/inventory, attunement dispatcher, inventory publication/reuse/records, context and query export; draft #241 parser, its selected source reader and joined inventory/rrflowQL test; `rrd-query` manifest and Parquet tests; the former DuckDB reference, POAM-036, and current C-07/D-06 roadmap text. Also reviewed Tree-sitter, Microsoft process startup, Linux resource limits, Qdrant hybrid and quantization, Apache Parquet and OpenTelemetry sampling references linked in the plan. Inspected the pinned DuckDB data-chunk initialization and ClickHouse read-range refiner excerpts and upstream root licenses linked in the updated research reference; this is not a complete source or component license audit.

The new [indexing plan](../../../roadmap/rrflow-1.0/project-indexing-execution-plan.md) maps the actual current break, adaptive effect admission, native parser isolation, bounded source publication, configurable retrieval, job/memory evidence, independent installed proof, failure oracles, performance measures and explicit unresolved decisions. The existing design links to it. The owner also withdrew the proposed DuckDB runtime/adapter direction in favor of a Rust-native rrflowDB informed by DuckDB and ClickHouse. README, Gate D, C-07, dependency spine, connector reference, earlier DuckDB reference and POAM-036 now record that direction; POAM-036 is withdrawn because its requested scope changed, not closed as implemented. Historic proposal text and journals remain identifiable as history. Native Parquet, indexing, C-07 and alpha requirements remain open. A specific external project named RustDB is not assumed.

No code was copied, deleted, moved or implemented. The source reviews map candidate behavior and failure semantics to existing RRFlow ownership; any later port must separately review exact source, dependency licenses, semantic differences and tests. No runtime trace, performance measurement, installed flow, candidate CI, upstream parity or roadmap completion is claimed by this documentation update.

The first documentation check rejected the amended POA&M ledger row: its added
prose fell outside the required row grammar, and `Withdrawn` was not in the
governance vocabulary. The row was corrected and the vocabulary now explicitly
distinguishes owner-withdrawn scope from evidence-based completion. The first
attempt to edit README also stopped on a text match before writing any file;
the exact line was subsequently selected and the intended change verified. A
second documentation check found that the ledger title did not match its detail
heading; the title was corrected before rerunning the check.

On the corrected Windows tree, `python scripts/knowledge/render_navigation.py`,
`python scripts/ci/check_documentation.py`, `python
scripts/knowledge/render_navigation.py --check`, and `python
scripts/ci/check_workflow.py` passed. No runtime or native platform suite was
run for this documentation-only package; the exact-head candidate and merge-group
CI remain required.

Checklist: current and candidate revisions recorded; PR scope updated before changes; source and research boundaries identified; retained historical direction distinguished from current owner records; trace/resource and failure oracles stated; journal linked and navigation regeneration and documentation checks required before commit. Any remaining test failure and unrun candidate checks must be reported on the PR.
