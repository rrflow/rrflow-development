# E-04c integration with object publication repair

**Status:** active immutable integration evidence; E-04, C-07 and the affected POA&M findings remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04c-object-publication-integration`
**Owner:** PR #190 integration of the candidate storage repair in PR #219.

## Evidence record

```text
gate/package and alpha outcome: E-04c / OBJ-13 and C-07 storage prerequisite; no gate closure.
starting revisions and pull requests: #190 b2f209712cb7c62f019e0c44d7fac3dcdad4d9e6 and #219 e93e9870ca8885d6dfc42f2667f013972566fc55, integrated by an ordinary merge without conflict; public main 6fe8b7590d913beda9b009551e5d2d2644e406d9.
change brief: integrate the existing candidate no-replace publication and cross-process serialization repair with E-04c and test its previously failing real-engine race. The brief was added to #190 before the merge. Stop on a failed race, lost implementation, or unaccounted merge conflict; this integration does not authorize merging without required candidate and merge-group CI.
files and boundaries reviewed: README and roadmap ownership, prior E-04c integration journal, PR #219 change brief and journals, rrd-store object publication and tests, rrd-lsm anchored rename boundary, incremental_generations engine test, documentation and CI policy scripts. The complete combined merge diff is staged for review.
implementation/evidence paths: rrd-lsm filesystem rename handling, rrd-store object publication and process/trace tests, object-contract reference, POAM-047 and the two Gate C evidence records originate in #219; this record and its generated navigation document the integration. E-04c implementation remains in its existing canonical engine and vector boundaries. No implementation was removed or moved.
failure oracle: the full E-04d engine test had failed on Windows with object-store Access denied; independent synchronized identical-object publication reproduced 29 failures in each of two 200-race runs before the candidate repair. The precise failing native suboperation was not isolated. The integrated exact racing-build test passed ten consecutive runs, then a visible exact invocation selected one test and passed.
acceptance commands: cargo test --offline --locked -p rrd-engine --lib engine::tests::vector_index::incremental_generations::racing_builds_publish_only_generations_equal_to_a_full_build -- --exact: 1 passed; ten preceding invocations passed. cargo test --offline --locked -p rrd-engine --lib --quiet: 253 passed, 3 ignored. cargo test --offline --locked -p rrd-store --quiet: passed; ignored child and platform tests remain ignored. python scripts/ci/check_workflow.py and python scripts/ci/build_execution_inventory.py followed by --check: passed. Navigation and documentation checks rerun after this record.
research/trace/resource decision: publication repair preserves content authentication and serializes cooperating writers with operating-system file locks; wait-start and elapsed wait tracing omit object data. PR #219 records the native API research and focused Windows and Linux ARM tests. An active stalled lock holder can block another writer without a configurable deadline; native power-loss durability, contention performance and corruption combined with reclamation remain unqualified.
surfaced failures: #190 candidate run 41088508920 failed macOS installed domain crash with command output did not close; exact cause is not established. A separate macOS diagnostic run 41094245511 hit the metadata fill's 25-second bound. An initial attempt to invoke scripts/ci/check_workflow_policy.py failed because that path does not exist; rerun used scripts/ci/check_workflow.py and passed. The two existing drafts #219 and #217 independently use POAM-047 for different findings; reconciliation must preserve both findings before their histories converge.
checks not run: candidate CI and protected merge-group CI for this new combined revision; native macOS capacity, Windows crash recovery, 100,000-record and soak qualification. Linux ARM focused object tests apply to the exact #219 parent, not the integrated merge tree. No C-07 or alpha completion claim.
remaining errors and owner: the macOS native CI failures remain under investigation; #219 requires review of blocking lock behavior and native durability. #190 auto-merge remains disabled until acceptance conditions are met.
review and checklist: package brief recorded before integration, parent identities and staged merge checked, exact failure test and package suites executed, evidence linked, generated inventory regenerated; source-bound candidate and protected merge-group evidence remain required.
roadmap/POAM status change: none beyond carrying #219's new finding in the candidate tree.
pull request and CI: https://github.com/rrflow/rrflow-development/pull/190 and https://github.com/rrflow/rrflow-development/pull/219; https://github.com/rrflow/rrflow-development/actions/runs/41088508920 applies only to the earlier #190 head.
```

## Links

- [Previous E-04c integration](e-04c-current-main-integration.md)
- [Object publication guard](../gate-c/c-07-object-publication-guard.md)
