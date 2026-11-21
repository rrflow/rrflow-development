# Recovered workbench result integrity

**Status:** active repair; installed workload qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/recovered-workbench-results-20261121`
**Owner:** Gate J, POAM-012

## Before-edit brief

Baseline main 134421476a37505df56979eff0c403ef72f78f59. Driver: sequential recovered-source review. Original oldest excluded history 9b60e959fa3e251b6da7d8593f9cf5d57f4d97e5 adds query decoding and the workbench script; later source e305e80f7d2af3b8d63f703f5e6d3b0a9208c12b supplies the script currently retained byte-for-byte. Original history is not merged because it includes the oversized generated executable. PR484 owns the already-integrated decoder. No original executable or runtime capture is imported.

Concrete defect: the retained script sets `comparison.comparable` true when `require_comparable` is false, even if workload checks fail. Required comparison failure also reaches the final successful script return. Target: measured comparison remains false independently of enforcement policy; required failure writes the report then terminates with failure. Preserve optional-failure results and successful results without claiming broader retrieval or competitive qualification.

Scope: .github/workflows/ci-reusable.yml scripts-lane execution of the reporting regression; scripts/evaluation/run-real-project-workbench.ps1 comparison construction and terminal report publication; scripts/evaluation/test-workbench-report.ps1 regression driver; POAM-012 finding; this journal and generated navigation. Current single script remains the owner. The regression executes the actual comparison/report statements without running installation, providers or comparison workloads. It covers all combinations of three workload outcomes and the enforcement flag, including report readback and terminal failure. No test fixture is represented as measured engine evidence.

Read: full current workbench script; original query patch and fixture change; current query read_path.rs 285-560; current Cargo serde settings; POAM-012; original H06 journal introduction; README, change-authoring procedure and archived kernel/query journal. The old query fixture digest and current source match; the script names SurrealDB and ArangoDB explicitly. No claim to have reread unrelated original source bodies.

Acceptance: run the PowerShell reporting regression before and after repair; parse script; review complete diff; docs/navigation, workflow policy and whitespace checks; required candidate and merge-group gates. No new external research, performance measurement, daemon or UI claim. Stop if failure extends outside reporting, and preserve it before expanding scope. Land and delete this branch before moving to the next excluded source.

## Reproduction and repair

The test imports the actual parser-delimited comparison and final report-publication statements, plus the two reporting helper functions. It executes all sixteen outcome/policy combinations and reads the JSON files back. Installation, data collection and comparator execution do not run. Original source produced seven false-positive comparison results and seven missing required failures. Two test-authoring mistakes were corrected before that reproduction: a PowerShell variable-colon interpolation parse error, and a hashtable fixture that did not model the JSON-derived PSCustomObject acceptance value. Neither intermediate run is claimed as a product failure.

The production fix preserves `$overallComparable` as the measured report field for both policy settings. After the final report is written, required failure throws a specific error naming the retained report. Optional failures remain false results with successful execution; all-pass results remain true. The regression verifies the expected failure message so unrelated exceptions cannot satisfy the failure oracle. The scripts CI lane now executes this same PowerShell regression.

`powershell -NoProfile -File scripts/evaluation/test-workbench-report.ps1` failed against the original reporting code with the fourteen expected defects, then passed all sixteen cases after repair. `python scripts/ci/check_workflow.py` passed. `git diff --check` passed after normalizing edited files to LF; the initial check identified CRLF introduced on patched lines. No data/engine source, saved prompts, binary or original bundle changed. POAM-012 records the verified reporting deficiency without declaring its broader qualification closed.

The retained script's row-count-based retrieval checks, installation flow, runtime/provider integration and competitive validity remain unqualified. This package does not fix those by renaming metrics or asserting that the original code was correct. No comparison workload or UI was run. This reporting defect is a concrete limitation found during source review, not proof that every other historical change was faithfully retained.

Final local checks: the reporting regression passed after the error-message assertion; navigation passed (1337 nodes), documentation policy passed (1340 statuses/1336 coordinates), execution inventory regenerated and checked (3767 records), CI workflow policy passed, and whitespace checks passed. The full production script was read; final semantic diff is one corrected field plus the three-line terminal failure guard. Its historical CRLF file is normalized to LF, so the raw diff includes line-ending changes; the ignore-space-at-eol review confirms no other semantic edits. Final diff review also caught the new CI step initially inserted in the full lane; it was moved into the scripts lane before commit. No engine or competitive measurement is inferred from these checks.
