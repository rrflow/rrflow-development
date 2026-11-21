# Archived development tooling integration

**Status:** active integration evidence; qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-development-tooling-integration-20261121`
**Owner:** repository development gate and Gate C tooling

## Before-edit brief

Join the original archived development-container, CI diagnostics and lab-tooling sources into their current canonical owners.

Driver: finish original source integration after PR485. Baseline green main db97d68c1ca9c432a186c4de159a5b8f634ec8b6; join PR485 before readiness. Owner: repository development gate / Gate C qualification tooling, not new engine capability. Sources: dd74d0bd6dd5375786cfc3ce19e6268822031bc2, cd7e208001dbd6e9e17435432f143fc37a10e066 (devcontainer parent e4103a72137f03c028456d76b5bd824d2dfac76c), 0cbd08ec73892036200f8caa04fc43d35e84d16b, 2458039e22aacac80684c96b1e33ba06d68887e7, 5a53fd096f20580790c425db592df15fb43bea15 and original integration merge 417e4eb04c47a3dfb36b28bf2c4ae99111d4b874. The latter has no unjoined non-merge ancestors; its integration object is not claimed as new implementation.

Current boundaries: .devcontainer files; scripts/ci/{check_workflow,test_candidate_admission,run_sustained_capacity,sustained_progress,test_sustained_launch,inspect_candidate_runs,test_inspect_candidate_runs}; scripts/lab; eval/lab import/run/source manifests; crates/evaluation/lab/rrflow-lab/tests/contract.rs. Keep current conditional CI lanes, merge-queue cache restrictions, engram naming, split qualification crates, pinned container inputs and lab Git identity. Original old rrflow-eval test modules map to split owners, not a second crate. Preserve source evidence with current status. No comparison or capacity workload, container execution, new provider, runtime instrumentation or installation change.

Read: original dd74 workflow/cache/preflight patch; current progress reader and its tests, runner change, candidate assertions; current devcontainer JSON/compose/seed and Containerfile first 130 lines; current pod scripts; full corpus importer and source manifest assertion; original inspection patch and current summarizer/test; original D03 journal and current inventory_delta ignored-test annotation. The existing interrupted-refresh oracle remains ignored with POAM-006 explicitly open; no repair is claimed. Inspect remaining conflict hunks before resolution; broad truncated diff is not a full review.

Acceptance: candidate-admission, sustained-launch and inspection Python suites; workflow policy; rrflow-lab contract tests (metadata only); architecture; docs/navigation and whitespace. Dedicated Cargo target D:/.usr/Temp/rrflow-root-target, four jobs. No new research or performance experiment is needed to join existing code. Stop on missing assertions or broken boundaries, repair before readiness, never weaken the gate. Journal records each source merge and changed-tree result; green CI/queue then automatic remote branch deletion, local deletion after main advances.
