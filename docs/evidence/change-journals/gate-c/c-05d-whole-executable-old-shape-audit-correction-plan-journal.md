# C-05d whole-executable old-shape audit correction plan journal

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-05d-whole-executable-old-shape-audit-correction-plan-journal`
**Owner:** the unchanged historical package receipt identified by its legacy heading and payload digest
**Legacy source:** `docs/roadmap/rrflow-1.0-execution-map.md@b7b061900b67d535ffec2710e6e4c30815d0f216#L2570`
**Legacy payload SHA-256:** `01dcfb7320977e21842d03dd6700fa2db7d8b47f6ba378f1e41eb3593bcdb720`

This record refracts one completed package receipt out of the former
monolithic execution map. Its fenced payload is preserved byte for byte.
It reports historical evidence and cannot change roadmap completion or
POA&M lifecycle status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
for acceptance and the [change-journal index](../) for discovery.

## Journal payload

```text
gate/package: C-05d / correct the incomplete C-05 closure and remove remaining successful pre-release readers; planned, no completion claim
starting revision: cd1bf495ece3c6698cde1cca413bfbe7be55588c; tree 185d1f7055f3ebb43e09ef09af2b043a808ede0f
implementation-requirements trace: the first C-06 owner read followed every active C-05 reference and disproved C-05c's whole-executable scope. Although its physical dependency/reader conclusion is valid, default compiled code still accepts vector artifact catalogue contract v1 beside v2, derives a table catalogue when RuntimeSchemaRegistry.tables is omitted, treats a missing RuntimeVector collection address as a successful old row, and suppresses a second TurboQuant catalogue authority during vector-runtime reconstruction. These are successful executable branches, not history. C-05 therefore cannot remain complete
files read in full before this correction: repository instructions; README; complete canonical roadmap, POA&M, engine-data-flow owner, current-format reference, vector-search reference, schema-catalogue reference, preceding C-05 plan/evidence chain, and complete current rrd-vector catalog.rs. Relevant source spans and every repository hit for LEGACY_VECTOR_ARTIFACT_CATALOG_VERSION, tables.is_empty, catalogue_tables, legacy_model, missing vector collection, suppress_legacy_turboquant, legacy/compatibility/default/alternate readers, and their callers/tests were classified. Complete affected implementation files are required before their individual removal packages edit them
baseline characterization: source inspection proves VectorArtifactCatalogEntry::validate accepts versions 1 or 2 and identity_bytes selects a separate v1 tuple; RuntimeSchemaRegistry serde permits omitted tables and catalogue identity, catalogue_tables installs specialized tables into that missing map, RuntimeDataSnapshot falls back to caller-selected models when tables is empty, RuntimeVector serde defaults collection to None, and reconstruct_vector_runtime deletes TurboQuant entries from the older VectorCatalog after replay. Existing exact tests intentionally pass the missing-table and missing-vector-collection shapes. The accepted physical v2/v2/v3 readers and sole required rrd-store-to-rrd-lsm edge remain unchanged
planned files changed/created/deleted/moved: first correct only the owning status/deficiency records so no active document claims C-05 complete; update README, canonical roadmap, POAM-003, engine-data-flow current boundary, vector-search and schema-catalogue references, this journal, and deterministic inventory. Then execute separately journaled direct-removal packages over fully read source/test/fixture consumers. No implementation is changed in this correction commit
planned contract and behavior: C-05 returns to unchecked and Gate C to 4/7. POAM-003 returns to Verifying and names the confirmed residues. C-05c remains accepted evidence only for the physical dependency/reader closure it actually proved. C-06 is blocked until all successful old-shape readers in the default alpha executable are removed or proven to be current optional semantics rather than compatibility behavior
planned proof: exact source searches and current positive old-shape tests establish the deficiency; documentation ownership, deterministic inventory, version, workflow, formatting, and diff policies must pass. Each later removal package adds negative omission/version tests, focused and owning suites, MX/KV/reopen evidence where persisted shapes are affected, architecture guards against revival, and the complete default workspace matrix before C-05 may close again
scope boundary: this is an evidence-integrity correction, not a deletion, format rewrite, or C-06 implementation. Operational states named Superseded, negative incompatible-peer tests, hyper-util's upstream legacy module name, S3-compatible protocol terminology, and semantically optional current fields are not mechanically deleted. The audit must distinguish useful current optionality from successful alternate pre-release representations
roadmap checkbox changed: pending correction; C-05 must be reopened before implementation continues
```
