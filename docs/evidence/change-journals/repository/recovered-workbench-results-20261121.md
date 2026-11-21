# Recovered workbench result integrity

**Status:** active repair; installed workload qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/recovered-workbench-results-20261121`
**Owner:** Gate J, POAM-012

## Before-edit brief

Baseline main 134421476a37505df56979eff0c403ef72f78f59. Driver: sequential recovered-source review. Original oldest excluded history 9b60e959fa3e251b6da7d8593f9cf5d57f4d97e5 adds query decoding and the workbench script; later source e305e80f7d2af3b8d63f703f5e6d3b0a9208c12b supplies the script currently retained byte-for-byte. Original history is not merged because it includes the oversized generated executable. PR484 owns the already-integrated decoder. No original executable or runtime capture is imported.

Concrete defect: the retained script sets `comparison.comparable` true when `require_comparable` is false, even if workload checks fail. Required comparison failure also reaches the final successful script return. Target: measured comparison remains false independently of enforcement policy; required failure writes the report then terminates with failure. Preserve optional-failure results and successful results without claiming broader retrieval or competitive qualification.

Scope: scripts/evaluation/run-real-project-workbench.ps1 comparison construction and terminal report publication; scripts/evaluation/test-workbench-report.ps1 regression driver; POAM-012 finding; this journal and generated navigation. Current single script remains the owner. The regression executes the actual comparison/report statements without running installation, providers or comparison workloads. It covers all combinations of three workload outcomes and the enforcement flag, including report readback and terminal failure. No test fixture is represented as measured engine evidence.

Read: full current workbench script; original query patch and fixture change; current query read_path.rs 285-560; current Cargo serde settings; POAM-012; original H06 journal introduction; README, change-authoring procedure and archived kernel/query journal. The old query fixture digest and current source match; the script names SurrealDB and ArangoDB explicitly. No claim to have reread unrelated original source bodies.

Acceptance: run the PowerShell reporting regression before and after repair; parse script; review complete diff; docs/navigation, workflow policy and whitespace checks; required candidate and merge-group gates. No new external research, performance measurement, daemon or UI claim. Stop if failure extends outside reporting, and preserve it before expanding scope. Land and delete this branch before moving to the next excluded source.
