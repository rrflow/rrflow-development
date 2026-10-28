# POAM-043 and POAM-044 engine gaps found by Connectome

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/poam-043-044-engine-gaps-from-connectome`
**Owner:** evidence for filing two verified engine gaps that Connectome's real-engine tests exposed

This record reports one documentation package. It adds POAM-043 and POAM-044
to the alpha POA&M. It implements no behavior, closes no gate, and changes no
checklist or existing POA&M status.

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: POA&M ledger only — record two verified engine gaps found while Connectome built its rrflowQL editor and engine-refusal tests: transaction previews (owning gates C-04, H-04) and rrflowQL refusal structure and query checking (owning gates E-05, H-03, H-04); no roadmap item advances
starting revision/tree and pull request: development/main 618269b4 (Merge pull request #156); branch agent/poam-043-044-engine-gaps; pull request recorded below
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the gaps were known only from Connectome's tests and its sprint journal -> POAM-043 records that `preview_transaction` builds the whole scope's prospective snapshot and validates the registry but not the written objects, so a preview is neither bounded by the change nor a dry run of the commit; POAM-044 records that no public operation parses or binds a query without executing it, that a query refusal's offset reaches clients only inside `ServiceError`'s Debug text, that validation refusals report byte 0 and "end of query" lands one byte into the last token, and that a live subscription is not parsed when opened; the ledger gains both rows (High, Open). Unchanged: every checkbox, gate exit, existing POA&M status, code, API, CI and the version. Stop if a claim cannot be shown in the engine source at the starting revision.
files and symbols read in full: docs/poam/rrflow-1.0-alpha.md (ledger); sections read: crates/persistence/rrd-store/src/repository/runtime.rs (preview_data_snapshot), crates/persistence/rrd-store/src/access/semantic_commit.rs (validate_schema_and_objects and its only production caller), crates/authority/rrd-engine/src/engine/transaction.rs (preview_transaction), crates/authority/rrd-engine/src/engine/error.rs (Display for ServiceError, whose fallthrough arm writes the Debug form, and the conversions into ServiceError::Query), crates/transport/rrd-server/src/http/response.rs (api_error), crates/compute/rrd-query/src/syntax.rs and plan.rs (explain_contract carried into the bound plan and canonical text; no reader in rrd-engine), crates/authority/rrd-engine/src/engine/query.rs (execute_query_scoped, poll_live_query_page, query_index_definition), crates/authority/rrd-engine/src/engine/subscription.rs (open_subscription validates, authorizes and digests the stream without parsing a live query).
implementation/evidence paths created, changed, moved, or deleted: created docs/poam/rrflow-1.0-alpha/poam-043.md, docs/poam/rrflow-1.0-alpha/poam-044.md and this journal; changed docs/poam/rrflow-1.0-alpha.md (two ledger rows); regenerated navigation indexes and docs/roadmap/rrflow-1.0-file-plan.jsonl.
research decision and primary-source/adaptation record: none needed — both gaps are RRFlow's own behavior, read in its source.
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable — documentation only. POAM-043's closure evidence asks for physical read counters on preview; POAM-044's asks for counters showing an explanation reads no rows.
first-failure or characterization oracle and result: Connectome's real-engine process tests observed both gaps against engine revision 86adada6 — `the_engine_and_the_native_layer_refuse_what_they_must` (src-tauri/src/rrd/client/process/effects.rs, on connectome-development main 5ba5a9d5) previews a strict-kind record that lacks a required property successfully and then has its commit refused; `live_subscriptions_are_bounded_and_refusals_end_them` (process/live.rs, same revision) opens an invalid live query (`FROM record:document PROJECT`) with its own cursor, which the engine accepts and ends only at the next change; `the_rrflowql_golden_matches_the_engine` (process/language.rs, on the unmerged F3 branch agent/f3-rrflowql-language 2670b09b) records every parse refusal as `Query("query parse error at byte N: …")` with empty details, validation refusals at byte 0 and "end of query" one byte into the last token. `git diff --stat 86adada6 618269b4` over the cited engine files is empty, and the code reading above confirms each claim at the starting revision.
acceptance commands and exact results: `py -3 scripts/knowledge/render_navigation.py` — "navigation: OK: 18 generated indexes, 695 nodes, 2704 edges, 2 index files updated"; `py -3 scripts/ci/build_execution_inventory.py` — "execution-inventory: wrote 1831 current, generated, and planned path records"; `py -3 scripts/ci/check_documentation.py` — "documentation-policy: OK: knowledge ownership, 696 document statuses, 694 classified coordinates, parent indexes, and local links"; `py -3 scripts/knowledge/render_navigation.py --check`, `py -3 scripts/ci/build_execution_inventory.py --check`, `py -3 scripts/ci/check_workflow.py` and `py -3 scripts/check_version.py` pass on the final tree (results on the pull request).
surfaced failures and corrections: none in the checks. While re-verifying, the Connectome live test was found to show the gap only for a live query opened with its own cursor (without one, Connectome runs the query once first and refuses it at open); the oracle above states that. For landing, the branch was brought up to date by merging the hook alias stubs package (#173 at feb65b95, which carries #175 and main at 9dea4eae); the generated file plan conflicted and was regenerated, not hand-merged, and the navigation, inventory, documentation, workflow and version checks pass on the merged tree.
checks not run and reason: no Rust, SDK or CI-workflow checks, because no code, schema or workflow changed; candidate CI on the pull request is authoritative.
remaining known errors and owner: POAM-043 is open under C-04 and H-04; POAM-044 is open under E-05, H-03 and H-04.
full-file reread and diff review: complete — every changed file's diff and this record were reread.
change checklist: complete — each claim tied to a function at the starting revision; observing tests named with their repository and revision; owning gates match the roadmap items that own the fix; no existing status changed; journal and indexes updated
roadmap/POA&M status change, or explicit none: the POA&M gains POAM-043 and POAM-044 (High, Open); no checkbox, gate exit or existing POA&M status changed
pull request, CI run, and merge evidence: recorded on the pull request for branch agent/poam-043-044-engine-gaps
```

## Links

- [POAM-043](../../../poam/rrflow-1.0-alpha/poam-043.md)
- [POAM-044](../../../poam/rrflow-1.0-alpha/poam-044.md)
- [RRFlow 1.0 alpha POA&M](../../../poam/rrflow-1.0-alpha.md)
- [Connectome client contract](../../../reference/client/connectome.md)
