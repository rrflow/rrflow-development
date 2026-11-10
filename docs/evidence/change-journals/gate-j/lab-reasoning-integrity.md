# Planning-study result integrity and dependency reconciliation

**Status:** active draft implementation evidence; no agent quality claim
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-reasoning-integrity`
**Owner:** OBJ-10, OBJ-11, J-04 and PR #262

## Package and source

PR #262 head `2a7a0e6592bff99c8345f21f1f0e3949439b101f` was merged in an
isolated worktree with PR #260 head `813fe2f8854ced4bf5284185888747e9e39b7db7`.
The PR description records the dependency reconciliation and result-integrity
changes before implementation. Three conflicts were resolved: CLI imports,
RRFlow client's text and vector imports, and the generated journal index.
The first build then failed because the automatically merged text-only
constructor referenced `binding` and `workload` outside their scope. The
vector constructor now applies its manifest HNSW and filter settings after the
shared connection; text-only import has no vector setup. Compilation alone is
not proof of an installed vector or text journey.

The original planning metric considered a plan successful solely when its
file list matched the historical change. It labelled that `solved` and used
`Tasks solved` in the report. A modified existing test failed before the repair:
two discordant pairs were reported `NotMet` instead of an inconclusive
result. The draft now calls this a historical file-threshold match, preserves
other valid plans as a known limitation, and never calls that metric task or
answer correctness. A new explicit `inconclusive` report verdict distinguishes
completed inconclusive comparisons from `not_run`. With one repetition, a
paired significance verdict requires every scheduled pair and usable replies;
missing pairs and provider failures cannot become a win. Multiple repetitions
of one task are not independent, so no significance claim is made until a
preregistered clustered analysis exists. The floor describes only its
observed file-match rate, not proof of no leakage. Unknown task/model/repetition
coordinates are rejected. Incomplete or provider-failed arms stay partial or
failed; failed provider attempts are excluded from the file-match denominator.

The changed manifest and report projections have new field/outcome meanings;
clients must reject or explicitly handle unsupported outcomes. The dependent
study draft #267 still needs reconciliation. This work does not supply an
executable patch oracle or establish that a plan would work.

## Review and research

Read the RRFlow README, OBJ-10/11 and J-04 sections, PRs #262/#260, the
repository change-authoring procedure, `eval/lab/extract_package_tasks.py`,
`tests/lab_tasks.rs` relevant fixtures and assertions, and the planning score,
run and report sections of `src/lab/tasks.rs`. Examined the affected report,
RRFlow adapter, CLI, generated schema and README sections. This is not an
exhaustive review of the original planning-study package. No upstream code was
copied. SWE-bench's official methodology uses real repository changes and
fail-to-pass tests; it informs a future executable oracle, not an assertion that
this file-list heuristic is SWE-bench or an answer-quality result:
https://www.swebench.com/original.html

## Local real-project characterization

The owner selected use of existing project code. In isolated local snapshots,
one Rust LFG historical test failed on its base revision and passed on its fix;
one Go Planfail test could not compile on the base because the tested API did
not exist and passed on its fix. Exact local source/test identities and logs
are retained outside the public branch under the local target directory. The
initial procedure accidentally ran zero tests and reused a Rust build target;
those results were discarded. The corrected procedure verifies the selected
test, copies only the historical test into each local base snapshot, and uses
separate Rust build targets. This is a characterization of two historical
oracles, not an agent evaluation, held-out dataset, training label, or proof of
RRFlow integration. No local source, test patch, prompt or private log is
committed or sent to a provider.

## Checks and outstanding limits

`cargo run --offline --locked -p rrflow-eval --bin rrflow-lab -- schemas --out
eval/schemas` regenerated the draft projections. `cargo test --offline --locked
-p rrflow-eval` passed on Windows, including 23 planning tests and the import
and vector tests. `cargo clippy --offline --locked -p rrflow-eval --all-targets
-- -D warnings` passed. The Unix-only launcher test remains separately covered
by the prerequisite journal, not by this Windows suite.

The production study runner still lacks a verified immutable checkout binding,
a real provider capture proof, a checked execution/test oracle, authorized
model/cost configuration, and supervised-feedback admission. The existing
#267 draft describes missing-cost and MCP-capture hazards; do not call a model
or mark this stack ready until those are resolved and the real invocation is
explicitly configured. No real model or LFG/SWAG/RRFlow end-to-end comparison
was run. No gate or POA&M status changes. #258, #253 and #260 remain the ordered
predecessors; candidate and merge-queue CI are required for every landed head.

## Checklist

- [x] Describe owners, dependency and failure oracle in the PR before changes.
- [x] Preserve both sides of dependency conflicts and record compile failure.
- [x] Exercise misleading/incomplete verdict cases and package tests.
- [ ] Review the remaining original draft and reconcile #267's capture and
  budget fixes; prove source identity and the real project-to-answer path.
- [ ] Land prerequisites and pass candidate and merge-queue CI at final heads.

## Local observation and documentation checks

The existing `scripts/inference/capture_observations.py --once` was run
against a selected LFG repository and an explicitly authored local context.
It captured one record labelled `unreviewed`, with `trainingAdmitted: false`
and `engineReceipt: null`; its local log remains outside the public branch.
This uses the current repository snapshot, not the historical test base, and
is neither training nor an authenticated engine receipt.

`python -X utf8 scripts/knowledge/render_navigation.py` regenerated the
journal index. `python -X utf8 scripts/ci/check_documentation.py` passed
(982 document statuses); `scripts/knowledge/test_navigation.py` passed six
tests; and `scripts/ci/build_execution_inventory.py` followed by `--check`
passed with 2,461 records. The generated schema consistency checks passed in
the evaluation suite. Candidate CI has not run on this draft.
