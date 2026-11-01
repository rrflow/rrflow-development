# C-04e catalogue status correction: integration with main for landing

**Status:** active integration evidence; C-04, E-04 and POAM-028 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-04e-catalogue-status-main-landing`
**Owner:** OBJ-13 / C-04 / E-04 / POAM-028 / [PR155](https://github.com/rrflow/rrflow-development/pull/155)

## Evidence record

```text
gate/package and alpha outcome: OBJ-13 / C-04 documentation correction; no gate closure and no status change.
starting revisions and pull request: PR #155 head 42d6bb54, merging development/main 2cadd3a9045e3c1a2d9f873fe4215343680ab18b, after #190, #191, #211 and #217 landed. The repository owner approved landing this PR on 2026-11-01, after #217.
change brief: recorded on #155 before the push. Redo the correction on the current text. Stop if a statement cannot be verified against the code, or if a correction would imply that POAM-028 or C-04 has closed.
source check: runtime/vector_catalog.rs obtains all three catalogue readers through read_vector_catalogue_versions (lines 387, 779 and 1183), a budgeted direct read at a read stamp. No catalogue path pages the change log. POAM-028's repair evidence records the same.
conflict analysis and traceability: docs/roadmap/rrflow-1.0-file-plan.jsonl was retired on main and takes main's descriptor. docs/reference/vector/quantization-lifecycle.md was rewritten on main, so it takes main's text and the correction is reapplied to it. Main had more stale present-tense statements than the PR first covered, and these are also corrected: docs/reference/vector/hnsw-projection.md, memory-tiers.md and search.md, docs/reference/context/retrieval.md and docs/reference/data/multi-model-object-contract.md. OBJ-13 no longer says there is no inner-product variant, because E-04d landed through #191; J-04 still owns its qualification. Every correction keeps C-04 and POAM-028 open.
deliberately unchanged: POAM-028's own account of the 2026-10-17 reopening and gate-c-evidence.md, which are dated history, and the dated research inventories docs/research/qdrant-capability-inventory.md and surrealdb-capability-inventory.md, which still repeat the old statement and own nothing. POAM-023 and the Connectome client reference mention cursor zero about other subjects.
research/trace/resource decision: none; no runtime source or test changed and no upstream source adapted.
first oracle: cargo test --offline --locked -p rrd-engine --test direct_read_source_closure: 1 passed.
acceptance commands: navigation, execution inventory, documentation, workflow and version checks, run after this record.
checks not run: every Rust suite other than the source-closure test, because only documents changed; candidate CI and protected merge-group CI for this head.
remaining errors and owner: POAM-028 and C-04 remain open: the rrd-maintenance log replay, large-estate resource qualification and current-head generation lookup. E-04 remains open.
roadmap/POAM status change: none.
pull request and CI: https://github.com/rrflow/rrflow-development/pull/155; candidate CI is pending for the integrated head.
```

## Links

- [C-04e: correct quantization catalogue status](c-04e-vector-catalogue-status-correction.md)
- [C-04e stamped vector catalogues](c-04e-stamped-vector-catalogues.md)
- [POAM-028](../../../poam/rrflow-1.0-alpha/poam-028.md)
