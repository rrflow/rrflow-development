# C-07 stale transaction recovery: integration with main for landing

**Status:** active integration evidence; C-07 is not qualified and POAM-035 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-write-conflict-main-landing`
**Owner:** OBJ-02 / C-07 / POAM-035 / [PR214](https://github.com/rrflow/rrflow-development/pull/214)

## Evidence record

```text
gate/package and alpha outcome: C-07 stale transaction recovery and conflict diagnostics / OBJ-02; no gate closure and no qualification claim.
starting revisions and pull request: PR #214 head a56eb62a6d37fa73562b7a6fa1308e06f073d08a, merging development/main 15f36a8758e32b33c930a20da836def3e5cc3207, in a new detached worktree. The earlier worktree target/c07-write-conflict-diagnosis holds another session's uncommitted merge of ffe3af9f, which #224 has since landed; it was neither used nor changed.
owner decision: on 2026-11-01 the repository owner approved landing #214 and pushing its update. This supersedes the session-written note that the PR stays draft pending #224's landing and a full changed-file review: #224 has merged and the review is below. The corrected-cadence 4,000-record characterization remains non-qualifying (qualification_passed=false, four rejected mixed writes, two latency breaches); landing records that evidence and claims no C-07 pass.
change brief: recorded on #214 before the push. Merge main and keep both independent error mappings. Stop on a changed API identity or authentication, uncertain-outcome handling, hidden retries, lost selected-plan or source-change details, a nontrivial conflict, or a need to weaken qualification.
conflict analysis and traceability: one conflicting path, crates/transport/rrd-server/src/http/response.rs, where api_error gained this PR's TransactionSnapshotStale mapping and main's AttunementSourceChanged mapping at the same place. Each is a separate early return and both are kept unchanged, with their codes, retryability and detail keys. Every other path merged without conflict. The diff against main is this PR's own 28 files, 1,807 insertions and 28 deletions, plus this journal and its generated index.
changed-file review: read the engine, server and contract-text diffs. classify_rejected_commit runs only after exact-outcome reconciliation found no commit, and only a strictly newer cursor yields TransactionSnapshotStale; an equal or older cursor stays the retryable StorageConflict. The new error is Conflict and non-retryable, HTTP and WebSocket return the same body, and the engine does not rebase or retry the commit. public-contract.md states this. No exported contract change: the generated surface check reports OpenAPI 8bc8cbdba9d90ab1239ea91624e639886971e58977c0f23a31007d1b2878a9c5, unchanged. The rrflow-cli sustained-workload changes are test support and were compiled by clippy, not executed.
research/trace/resource decision: none beyond the PR's recorded PostgreSQL and SQLite review; no upstream source adapted.
first oracle: py -3 scripts/ci/test_analyze_sustained_conflicts.py: 8 passed. py -3 scripts/ci/test_native_sample_policy.py: 7 passed. cargo test --offline --locked -p rrd-engine --lib -- stale: 11 passed.
acceptance commands: cargo build --offline --locked --workspace --bins: passed. cargo test --offline --locked -p rrd-engine --lib: 306 passed, 4 ignored. cargo test --offline --locked -p rrd-server: 34 passed. cargo clippy --offline --locked -p rrd-engine -p rrd-server -p rrflow-cli --all-targets -- -D warnings: passed. cargo fmt --all -- --check: passed. py -3 scripts/ci/check_generated_surfaces.py --check: passed. Navigation, execution inventory, documentation, workflow and version checks are run after this record.
checks not run: the installed sustained workload and native C-07 runs; candidate CI and protected merge-group CI for this head.
remaining errors and owner: POAM-035 and C-07 remain open. The four observed conflicts still do not identify which internal branch produced them or which action advanced the cursor.
roadmap/POAM status change: none beyond the PR's own POAM-035 evidence notes.
pull request and CI: https://github.com/rrflow/rrflow-development/pull/214; candidate CI is pending for the integrated head.
```

## Links

- [C-07 typed stale transaction recovery](c-07-stale-transaction-recovery.md)
- [Corrected-cadence 4,000-record characterization](c-07-corrected-cadence-4k-outcome.md)
- [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)
