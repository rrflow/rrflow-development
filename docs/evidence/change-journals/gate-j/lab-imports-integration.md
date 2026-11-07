# Comparison-lab imports reconciled with the current candidate

**Status:** active draft implementation evidence; not a gate acceptance
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-imports-integration`
**Owner:** OBJ-10, OBJ-11, J-04a/J-04d and PR #260

## Package, scope and starting identity

The driver is the existing comparison lab and its real-project planning-study
successor, not an additional engine. PR #260 starts at
`8bec67c89eea61715dc7c8ae8fc551d2cc71ae9a` and depends on comparison-lab
PR #253 head `400633f113d1309db33a22e4eb30983561c1d619`, itself dependent on
#258. The PR description records the integration and imported-failure repair
before those edits. The ordinary merge was performed in an isolated worktree.
No source from the owner's local projects is included in this package.

The merge had five conflicts. The CLI retains the importer and the typed
progress callback; manifest validation retains both explicit RRFlow HNSW build
parameters and the custom IVF/RQ rules. The lab README retains Lance setup and
the current public HNSW status. The launcher retains both SurrealDB image
choices and uses the created Docker ID for cleanup. The small manifest keeps
RRFlow HNSW configured and retains the separately unavailable routed arm and
Lance arms. The old POAM-053 rationale for unavailable public HNSW is removed.
No implementation was moved or deleted. A manifest changed by reconciliation
has a new identity and cannot be combined with earlier runs.

## Code and owner review

Read the RRFlow README, OBJ-10/11 and J-04 owner sections, the change-authoring
procedure and PR #260/#253 descriptions. Read `eval/lab/import_corpus.py` and
`crates/evaluation/rrflow-eval/tests/lab_import.rs` in full. Examined the
conflicting sections of `src/bin/rrflow_lab.rs`, `src/lab/manifest.rs`,
`eval/lab/README.md`, `eval/lab/run_system.sh`, and `vector-small.json`; the
imported-result conversion and status handling in `src/lab/import.rs`; and the
existing source catalogue validation in `src/lab/sources.rs`. This is not an
all-lines review of the whole original #260 package. Its existing corpus,
Lance, external-harness and source-checking implementations still need complete
review and real-system evidence before it is ready.

## Defect, decisions and observed checks

The imported-result test originally expected `completed` when at least one
VectorDBBench case failed. After changing the existing test to require
`partial`, it failed with `left: Completed, right: Partial`. The importer now
marks mixed success/failure `partial` with the failure count, retains successful
observations and failed-case count, and keeps all-failed imports `failed`.
This is truthful report status, not a claim that any imported result was
independently remeasured.

The Unix launcher regression now includes both SurrealDB variants as well as
Qdrant and Milvus. The existing ARM Linux mock-Docker test passed against the
reconciled launcher and test file; it checks failed creation, malformed output
and cleanup by the created ID. `bash -n` passed in Ubuntu on Windows. On
Windows, `cargo test --offline --locked -p rrflow-eval` passed after the status
repair, including all five imported-result tests and the evaluation package
suite. The Unix-only launcher test is skipped on Windows. `cargo clippy
--offline --locked -p rrflow-eval --all-targets -- -D warnings` and `cargo fmt
--all -- --check` passed. The baseline full package suite also passed before
the new regression exposed the mixed-status issue.

Research decision: no outside code was adapted in this integration; the
existing imported formats are preserved and require their own source and license
review. Trace/resource decision: preserve existing progress and raw results;
no engine trace or model invocation is added. The test uses mocks for Docker
ownership and does not establish real Lance, COCO, SurrealDB 4, or VectorDBBench
behavior on this combined revision. No model, training or inference call was
made. Candidate and merge-group CI have not run for this new integration.

## Remaining work and checklist

- [x] Bind the dependency and public change brief before changing files.
- [x] Preserve both sides of each conflict, and characterize the imported
  mixed-failure regression and owned container cleanup.
- [x] Run the relevant package suite and formatting/lint checks; record limits.
- [ ] Complete review of the original #260 diff and real external adapters,
  and record any failures.
- [ ] Land #258 and #253 through their required gates, refresh this draft
  against their actual merge commits, then run candidate and merge-queue CI.

The owner retains all gate and POA&M statuses. This draft does not establish
an installed project journey, supervised-learning admission, answer quality,
C-07, J-04 acceptance, or alpha completion.

Documentation and navigation checks after resolving the combined candidate:
`python -X utf8 scripts/knowledge/render_navigation.py` passed and updated one
index; `python -X utf8 scripts/ci/check_documentation.py` passed (980 document
statuses); `python -X utf8 scripts/knowledge/test_navigation.py` passed six
tests; `python -X utf8 scripts/ci/build_execution_inventory.py` followed by
`--check` passed with 2,450 records. The generated schema consistency tests
passed as part of the evaluation suite.
