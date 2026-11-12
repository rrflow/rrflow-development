# Comparison lab scratch ownership: partial repair

**Status:** active draft evidence; stop condition remains; no gate completed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journals/gate-j/comparison-lab-scratch-ownership-2026-11-07`
**Owner:** OBJ-10, OBJ-11, J-04a and [PR #253](https://github.com/rrflow/rrflow-development/pull/253)

## Scope, baseline and review

Starting candidate `23c3f16e776f2cd5dae49130acdb26a100926ff3`, tree `9a4b31df25ffeddbcbc002014dee02871bc246d3`. The PR is draft because pre-create deletes, ambiguous provider ownership and untrusted error text were found after its previous CI. The PR description was amended before edits. The package driver is this working session. POAM-057 records the open candidate deficiency; it does not assert the candidate is on main. The checked-in laboratory evidence predating this repair is unchanged.

Read in full for the change: lab `adapter.rs`, `custom.rs`, `surrealdb.rs`, `milvus.rs`, `Cargo.toml`, and the repository change-authoring procedure. Examined relevant ranges and diffs of `driver.rs`, `qdrant.rs`, `rrflow.rs`, `rrflow_lab.rs`, `lab_adapters.rs`, `lab_contract.rs`, `manifest.rs`, Gate J, README and the lab README. A complete full-file reread and implementation traceability review is **still required** before readiness; this is not a whole-codebase review.

Current to target: deterministic names and preemptive deletes -> a new random attempt identity, intent recorded before CLI preparation, exclusive-create failures rejected where observed, and no unfenced Qdrant/SurrealDB teardown. Qdrant's create, version, index, ingestion and query operations remain in its adapter. SurrealDB's namespace/database setup, table, indexes, ingestion and query remain in its adapter, with the table definition separated so a failure cannot fall through to index creation. Existing custom, RRFlow and Milvus lifecycle implementations have not yet been made safe; the PR **must remain draft**. The brief requires ownership-safe behavior for every adapter, not just a random suffix.

Changed: `Cargo.lock`, `crates/evaluation/rrflow-eval/Cargo.toml`, `src/bin/rrflow_lab.rs`, `src/lab/{adapter,driver,milvus,qdrant,surrealdb}.rs`, `tests/{lab_adapters,lab_contract}.rs`, `eval/lab/README.md`, preserved the v1 run schema and generated `eval/schemas/lab-run-v2.json`, the POA&M and this journal/index. Created: POAM-057 and three sanitized evidence summaries under `eval/lab/evidence/ownership-20261107/`. No engine or Connectome code changed. Reconcile the POA&M ID with current main before readiness.

## Research and evidence

The [Qdrant create API](https://api.qdrant.tech/api-reference/collections/create-collection), [SurrealDB table definition](https://surrealdb.com/docs/reference/query-language/statements/define/table) and [Milvus collection-existence API](https://milvus.io/api-reference/restful/v2.5.x/v2/Collection%20%28v2%29/Has.md) inform the wire boundary, not a claim that a check-then-create or check-then-delete is atomic. No upstream code was copied. In the disposable local probes, Qdrant 1.20.0 returned HTTP 409 and SurrealDB 3.3.0 returned a statement error on duplicate creation, with sentinels intact. The pinned Milvus 3.0.5 returned code 0 on the duplicate and retained the sentinel. No conclusion extends to other versions or concurrent replacement.

The Windows debug `rrflow-lab.exe`, SHA-256 `b09aceedc48cc61b01380f4344810ba77cac42c4eafe0fecd9cb4fdbe03e12eb`, was built from the starting candidate plus the code-path diff SHA-256 `cbc1e318f139094694bde762dec3eb261b6d1bce22720e061c12ce9ce7dc1265` (`git diff --binary` over Cargo.lock and the changed lab production files). Subsequent POA&M/evidence documentation is not part of the binary. [Real adapter smoke](../../../../eval/workbench/lab/evidence/ownership-20261107/real-adapters.json) ran 100 generated 8-dimensional points against separate local, loopback-only disposable containers with no host data mounts: Qdrant 1.20.0 and SurrealDB 3.3.0 each completed two exact-search cases, and each final run identity matched its pre-prepare intent. Containers were stopped by their own returned IDs. The systems ran concurrently on a shared host; these are functional smoke tests, not comparative performance measurements.

[Interruption readback](../../../../eval/workbench/lab/evidence/ownership-20261107/interruption.json) used a controlled HTTP server: the Windows lab process was killed while a create request waited for a response. The intent existed before termination and was readable afterward; no completed report existed. This is not a test of remote recovery, process restart/replay, directory fsync, or sudden power loss. The [Milvus probe](../../../../eval/workbench/lab/evidence/ownership-20261107/milvus-duplicate.json) used the pinned 3.0.5 image, an isolated container and synthetic sentinel. Its full local scripts and raw run files remain in ignored `target/lab-ownership-*-20261107*`; the three checked-in summaries do not independently reproduce the probes. No credentials or user data were used.

## Commands and failures

- `cargo test --offline -p rrflow-eval --test lab_adapters`: initially 15 passed, 2 failed because old scripted response indices assumed the removed delete and combined SurrealDB preparation. Scripts were corrected to the new actual request sequence.
- `cargo run --offline --locked -p rrflow-eval --bin rrflow-lab -- schemas --out eval/schemas`: passed; generated run schema now includes the attempted scratch identity.
- `cargo test --offline --locked -p rrflow-eval`: passed before the later observer-result preservation change; 51 library, 18 adapter, 29 lab contract, 3 provider and 11 storage tests passed (platform-dependent runner suite: zero tests on Windows). After the later change, `cargo test --offline --locked -p rrflow-eval --test lab_contract` passed 30, `--test lab_adapters` passed 18, and `cargo clippy --offline --locked -p rrflow-eval --all-targets -- -D warnings` passed. The complete package suite must be rerun at the final candidate.
- The first redaction test pass exposed two tests expecting raw Milvus messages; they were updated to require numeric code or a fixed failure category without the raw body. HTTP status remains visible. Full RRFlow/custom/other response-field sensitive-data review is still outstanding.
- `python scripts/knowledge/render_navigation.py` passed. `python scripts/ci/check_documentation.py` initially failed on the new POA&M heading, missing owner link and section shape; after correcting those records it passed with 982 document statuses and 980 classified coordinates. `git diff --check` passed before this journal edit.
- Real smokes: `python target/lab-ownership-real-smoke-candidate-20261107.py`, `python target/lab-ownership-interruption-candidate-20261107.py`, and `python target/milvus-ownership-probe-20261107.py` passed as described above. The Milvus image is pinned by digest in the retained summary. Scripts are local diagnostic inputs, not a portable test harness.

## Checklist and stop

[done] Brief and failure oracle; [done] version-specific primary documentation and real isolated probes; [done] progress and interruption observations; [partial] component repair and focused validation; [open] provider-specific safe ownership, concurrency, lost-ack/replay, foreign replacement and fenced cleanup; [open] full sensitive-data handling; [done] generated navigation and documentation policy; [open] portable reproducible real-system tests, complete-file review, final package suite, remaining repository checks, candidate and merge-group CI. The scratch intent is not an engine audit receipt, and a random name or read-before-write is not exclusive ownership. No J-04, C-07, alpha, performance-superiority, or roadmap completion claim is made. Do not run this candidate against user-owned services or mark it ready.

## Milvus containment follow-up

The former Milvus `prepare` unconditionally dropped a collection and then created it, while the observed 3.0.5 duplicate-create response did not prove ownership. The PR brief was amended before changing this path. The draft now refuses direct preparation before it sets a collection name or sends a request; the old drop/create implementation is removed. A loopback listener test calls preparation, ingestion, finish, search and teardown and observes no connection. This is a temporary refusal, not a working Milvus comparator. The former real-wire and failed-index characterization tests remain explicitly ignored until an owned isolated service admits safe operation; restoring that behavior and rerunning both tests are required before readiness. The RRFlow/custom paths and Qdrant/SurrealDB replacement fencing remain unresolved, so the entire PR remains unsafe for shared services and draft.

`cargo test --offline --locked -p rrflow-eval --test lab_adapters milvus -- --nocapture` exited 0: 3 passed, 2 ignored, 14 filtered out. It used `CARGO_TARGET_DIR=C:/Users/jessa/AppData/Local/Temp/rrflow-c07-observer-7e60a4ac`, `CARGO_PROFILE_TEST_DEBUG=0`, and `CARGO_INCREMENTAL=0`. The log is `C:/Users/jessa/AppData/Local/Temp/rrflow-lab-milvus-containment-20261107.log`. This loopback test does not prove an isolated container, lost acknowledgements, replay, replacement fencing, or performance. No candidate CI, merge, J-04, or alpha completion is claimed.

The subsequent full `cargo test --offline --locked -p rrflow-eval` exited 0: 112 passed and 2 ignored across nine test/doc-test groups (log SHA-256 `e1853f1d1f1af1ab6b735c03237e5e2919723aa43dcb6d0918c3180ec6f56fa7`). `cargo clippy --offline --locked -p rrflow-eval --all-targets -- -D warnings` passed (log SHA-256 `53f69bc40e826f73aeb98fde0c39a0f4b32e6494da89b1320eb4c6f694eb3275`); formatting, navigation, documentation policy and `git diff --check` passed. The two ignored Milvus tests are a readiness blocker, not successful comparator coverage.

## SurrealDB 4.0 isolated functional reference

After amending PR #253's brief, the existing lab reference records the user's
bounded onboarding and real-task comparison requirements, current primary
benchmark references, and the distinction between a time budget and an
evidence-based estimate. This is documentation in the existing unmerged draft,
not an engine, UI or evaluation-harness implementation. The baseline remains
draft head `23c3f16e776f2cd5dae49130acdb26a100926ff3` with the prior uncommitted
ownership changes. This entry does not claim a complete-file review beyond the
files identified above and the complete lab README reviewed for this addition.

On Windows x86-64, Docker ran the official Linux x86-64 SurrealDB image
`surrealdb/surrealdb:v4.0.0`, digest
`sha256:c35893410644e65fa90aa1bef446ac9cb0a0c2733b9d405ce3eb56754b85c330`,
with `--rm -i --network none`, an in-memory store, no host mounts and no
project data. The SQL was:

```sql
DEFINE INDEX idx_pt ON pts FIELDS point HNSW DIMENSION 4;
INSERT INTO pts [{id:pts:a,point:[1,2,3,4],flag:true},{id:pts:b,point:[4,3,2,1],flag:false},{id:pts:c,point:[3,3,3,3],flag:true}];
SELECT id, flag, vector::distance::knn() AS distance FROM pts WHERE flag = true AND point <|2,40|> [2,3,4,5] ORDER BY distance;
SELECT id, flag, vector::distance::knn() AS distance FROM pts WHERE flag = true AND point <|2,40|> [2,3,4,5] ORDER BY distance EXPLAIN;
```

The query returned `pts:a` and `pts:c`, with distances `2.0` and
`2.449489742783178`; EXPLAIN reported `KnnScan`, index `idx_pt`, and
predicate `flag = true`. The container exited successfully and its in-memory
data did not persist. Three synthetic records do not establish latency,
ANN recall, fair cross-system equivalence, durable storage or the RRFlow
project journey. The experiment did not run the unsafe draft lab adapters.

After these additions, `python scripts/knowledge/render_navigation.py --check`
reported 19 indexes, 981 nodes, 3,498 edges and no updated indexes;
`python scripts/ci/check_documentation.py` reported success for 982 document
statuses and 980 classified coordinates. `git diff --check` passed before the
last wording adjustment. After the final README edits, `git diff --check` and
`python scripts/ci/check_documentation.py` passed again with the same status and
coordinate counts. No Rust or benchmark package tests were run for this
documentation and isolated SurrealDB reference update.

The comparison-lab and task-study arms still need isolated ownership,
source-bound real project data, executable independent correctness checks,
LFG/SWAG routing, UI consumption and qualification. The current task study's
historical-file-match metric is not a correctness verdict. No benchmark, gate,
CI or alpha status changes from this documentation and smoke.

## Current dirty repair review (2026-11-09)

The package PR brief was extended before this follow-up implementation after
review found that Custom Protocol v1 also has an unscoped destructive
`/teardown`. The driver calls teardown after a successful or partial run, so a
custom endpoint could remove unrelated state. The custom adapter now fails
closed for every operation and performs no HTTP request; the small-corpus
manifest declares the reference arm unavailable, and the runner refuses to
launch the old example. The protocol schema and example remain reference
material, not an enabled adapter. Re-enabling custom comparison requires a
versioned per-run resource identity across prepare, ingest, readiness, search,
and cleanup, plus collision, lost-acknowledgment, interruption and replay tests.

On the dirty PR #253 tree at base candidate `23c3f16e776f2cd5dae49130acdb26a100926ff3`,
`cargo test --offline --locked -p rrflow-eval --test lab_adapters
custom_v1_adapter_fails_closed_without_contacting_endpoint -- --exact
--nocapture` passed (1 test). The focused adapter suite passed (15 passed, 2
ignored; the two ignored tests exercise disabled Milvus mutation). The owning
package suite passed (110 passed, 2 ignored across its test groups) with
`cargo test --offline --locked -p rrflow-eval --quiet`, using
`CARGO_TARGET_DIR=C:/Users/jessa/AppData/Local/Temp/rrflow-c07-observer-7e60a4ac`,
`CARGO_PROFILE_TEST_DEBUG=0`, and `CARGO_INCREMENTAL=0`. No failures surfaced.

The actual branch still requires full candidate CI after reconciliation with
current main, a clean diff review, and evidence for concurrent replacement,
crash recovery and retained-resource management. Qdrant and SurrealDB resources
are deliberately retained; RRFlow needs an isolated instance; Milvus and
custom v1 are disabled. This does not make the comparison lab generally safe
for shared endpoints and does not close POAM-057 or J-04.

This follow-up additionally changes `crates/evaluation/rrflow-eval/src/lab/custom.rs`,
`crates/evaluation/rrflow-eval/tests/lab_adapters.rs`, `eval/lab/vector-small.json`,
`eval/lab/vector-medium.json`, `eval/lab/vector-100k.json`,
`eval/lab/run_system.sh`, and this lab README. The small manifest no longer
declares the custom reference or Milvus available; the documented local runner
does not attempt to start them. The schemas remain reference-only.

After all code and manifest edits, `cargo fmt -p rrflow-eval -- --check` and
`bash -n eval/lab/run_system.sh` passed. `rrflow-lab validate` passed for
`vector-small.json`, `vector-medium.json`, and `vector-100k.json`. A first
schema-generation invocation failed because its output directory did not
exist; after creating it, `rrflow-lab schemas` succeeded. The baseline
`lab-run-v1.json` is byte-identical to the PR-base file, and generated
`lab-run-v2.json` matches the checked-in file byte-for-byte. The targeted
v1/v2 compatibility test passed; the full package suite passed (111 passed, 2
ignored), and `cargo clippy --offline --locked -p rrflow-eval --all-targets
-- -D warnings` passed. The comparison-lab manifests refused unavailable
Milvus and custom systems before creating run directories; the launcher refused
them before directory creation or Docker startup. The first PowerShell schema
comparison command incorrectly compared a Git text pipeline after newline
conversion and reported a false mismatch; a byte-level Python comparison then
confirmed the checked-in v1 schema exactly matches the PR base (SHA-256
`95b2d28b0280ffcdfc17c1871b42490e7473afa37356917088b1231a10ffd746`) and the
v2 schema exactly matches fresh generator output (SHA-256
`59cf6f899ecbaf7eab741e4b0c94a5270e80c2f9c4757e753495d08985146408`). Final
`python scripts/knowledge/render_navigation.py --check`,
`python scripts/ci/check_documentation.py`, `bash -n eval/lab/run_system.sh`,
`git diff --check`, all three manifest validations, formatting, the 111-pass / 2
ignored package suite, and Clippy pass. No candidate CI or merge-group result
exists for this dirty revision.

## Integrated candidate on current main (2026-11-09)

Ordinarily merged public development `main` at `a9f1b0fb8d948e113dca347e5eeac10dd224330a` into this PR #253 candidate; merge commit `ebe4ffe61f05e09060cb5ec604b979de7301fa93`. The merge was conflict-free and retains the candidate work plus current main. This is a local integration; the remote PR head is still `23c3f16e776f2cd5dae49130acdb26a100926ff3` until the integrated candidate is pushed and reviewed by CI.

On the integrated tree, `cargo test --offline --locked -p rrflow-eval --quiet` passed: 111 passed, 2 ignored, 0 failed across the package's test groups. The ignored tests are Milvus wire behavior, whose mutation path is intentionally disabled. `cargo fmt -p rrflow-eval -- --check` and `cargo clippy --offline --locked -p rrflow-eval --all-targets -- -D warnings` passed. Navigation generated and checked 19 indexes, 1,043 nodes and 3,648 edges; documentation policy passed 1,044 document statuses and 1,042 classified coordinates. All three vector manifests validate, the generated v2 schema matches byte-for-byte, and the preserved v1 schema matches the original PR-base blob byte-for-byte. The post-merge `git diff --check a9f1b0f..HEAD` found trailing spaces on two metadata lines in POAM-057; these are being removed before push and the full diff check will be rerun. No current-head hosted candidate or merge-group CI exists; no merge readiness is claimed.
