# Comparison lab: public HNSW and filtered TurboQuant paths

**Status:** active draft evidence; no gate completed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journals/gate-j/comparison-lab-public-vector-paths`
**Owner:** OBJ-10, OBJ-11, J-04 comparison lab, PR #253

## Package and source

- Starting draft `6a303239e7d7f5c8f7efc9f5b34c8539b036bfcc`. Public main `1fd7f163de3448c694e09c916b328540303eab1c` was merged without conflicts into local commit `a44dcc6bc5cec6eb9140d366cc0ee3d95ddd4996`, tree `29f9f34913b184af6c27095f187b704ba2c5931a`. The PR description records this integration and the adapter's behavior, ownership, scope, oracle, decisions and stop conditions before implementation. It remains draft and still depends on #258 and required candidate CI.
- The old lab rejected RRFlow HNSW because public index operations did not exist on its original base. Main's [H-04 index operations](../gate-h/h-04-vector-index-operations.md) now expose HNSW and payload-index ensure via the generated public client. The lab candidate configures RRFlow HNSW explicitly, declares a category payload index for a filtered workload, builds a generation using configured `m`, construction width, maximum level, seed and build policy, then requests approximate search. For TurboQuant it also includes the configured filter property in the build request after declaring the payload index. The engine remains the authority, and errors and actual access paths are retained. No upstream code is copied.
- Changed `crates/evaluation/rrflow-eval/src/lab/{manifest,rrflow}.rs`, `tests/{lab_adapters,lab_contract}.rs`, `eval/lab/vector-{small,100k,medium}.json`, the generated manifest schema, `eval/lab/README.md`, and POAM-053's current-state note, this journal and generated navigation. No existing implementation is removed or moved. Default comparison manifests now contain explicit RRFlow HNSW settings; changed manifest and corpus identities must not be combined with prior runs. The generic custom and other vendor adapters are preserved.
- Reviewed the relevant current-main public contracts, native client, real-server H-04 test and HNSW reference, lab adapter, manifest, comparator manifests, lab tests and PR #253 brief. The entire repository and all historical evidence were not reviewed. No roadmap or POA&M lifecycle status is advanced; POAM-053 remains open pending its closure evidence and normal review.

## Test and observable engine evidence

- Baseline `cargo test --offline --locked -p rrflow-eval --test lab_contract rrflow_hnsw_requires_explicit_native_build_parameters -- --nocapture` failed because the old draft rejected the new RRFlow HNSW configuration. On the candidate, native request and explicit-parameter tests passed; tests verify that the HNSW and TurboQuant build requests contain the category filter when requested and omit it otherwise.
- On Windows, `cargo run --offline --locked -p rrflow-eval --bin rrflow-lab -- schemas --out eval/schemas`, `cargo test --offline --locked -p rrflow-eval`, and `cargo clippy --offline --locked -p rrflow-eval --all-targets -- -D warnings` passed. The full suite is a component/package check, not a competitor performance qualification.
- On `clyffy-01`, Linux ARM64, the local merged Git tree was exported as archive SHA-256 `6b3e513151fda13430c7b3c29d0fe227dc62b722776d568a545d6767b7b5aae5`, then the working source patch SHA-256 `8537104b6bf6ad68f7be0ff6596f863471e5401d2e63c7f266836fe46ff71674` was applied. The final `rrflow.rs` was copied before rebuilding, SHA-256 `e460cccd39f667da274da7b00f9a3fa71657a12f648a66d5565475d3d5ae0961`. Manifest and driver source hashes matched the local files (`359099cf3f0f088aa231c1b1750a963bce8230f6405b207d132fac125022d036`, `564479d0e1fe50e2259b43cc1c18e37b32b5b45994914eee8a2a7f3672c17393`). `cargo build --offline --locked -p rrflow-eval --bin rrflow-lab -p rrflow-cli --bin rrflow` succeeded in an isolated target. RRFlow executable SHA-256 `5af07983961a5e67b8e6ccb500036b403c0570fb4f24ff52fab80419cd4b8f0c`; lab executable `3ca7b4a18e45e6dffef577064bd8a2dccad9af1425b54b686e55eb290a37006c`. These are debug builds, not release benchmarks.
- The existing `eval/lab/run_system.sh` installed a fresh test estate for each RRFlow arm and ran the same generated 200-point, 32-dimensional workload with 20 queries each in filtered and unfiltered cases; SurrealDB 3.3.5 ran in its pinned container. Independent oracle recall was 1.0 and errors were zero in all six cases. The engine reported `hnsw` and `turboquant` respectively. The prior Windows smoke used an older engine and did not include the filter in the quantization build; it does not establish a current-engine impossibility. This new result demonstrates this small installed-path fixture, not general filtered coverage or scale.
- The combined report SHA-256 is `49854ea166250603851a8b8441ab2c60a77369daa6ef9e64a4bf34f26c3841dd`; server outputs and raw samples are retained on `clyffy-01` under `/tmp/rrflow-lab-hnsw-z7B3e5R6/runs/` and copied locally to `target/workbench-probe-20261106/arm-hnsw-smoke/`. Manifest digest `c22ead1406190399171cbaf7318004df13b617d08031f51875cf9252aca4c9a3`. Servers and client were unpinned, shared host interference was not excluded, and there was one debug run per arm. The output does not establish latency superiority.

## Remaining and integration

The merged-base navigation check passed, but its inventory projection needed regeneration and documentation check failed on the known missing POAM-050 through 052 supplied by draft #258. A read-only merge-tree trial with #258 showed conflicts only in the POA&M ledger and its generated index. Do not merge that unreviewed draft or close POAM-053 from this smoke. Review and integrate #258 normally, resolve both conflicts by preserving all ledger entries and regenerating indexes, then run candidate and merge-group CI on the actual resulting head.

Connectome does not yet consume these reports or offer an authorized launch workflow. The lab README describes the intended reader and launch boundaries, their failure tests, and the required installed-path evidence. The lab is an optional evaluation tool, not default engine readiness or an independent engine authority. Current local testing does not establish Windows/macOS native qualification, J-04, C-07, a tuned large-data comparison, or the alpha journey.

## Dependency integration trial

In a separate detached worktree from `a44dcc6b`, an ordinary no-commit trial merge of #258 (`5f1c6cbd`) produced the two predicted POA&M conflicts. The ledger was reconciled in numerical order, preserving 050 through 055; the generated index was regenerated. The first resolution script failed its own zero-padding assertion and its check failed with conflict markers still present; a corrected script resolved the ledger and `python -X utf8 scripts/ci/check_documentation.py` then passed. This trial contains neither the later HNSW working diff nor a merged #258, and is not candidate CI or permission to land it.

## Operator visibility and safe container ownership

The live 20,000-point diagnostic revealed that the existing CLI said only
`running` throughout a multi-minute operation. The package now adds typed
`LabProgress` observations in `driver.rs`, emitted by the CLI as JSON on
stderr for collection preparation, started and completed ingest batches, index
build and readiness, and started and completed cases. Callbacks occur outside
measured search cases. A batch completion means only that its adapter call
returned successfully. The observations are transient, not an audit or a
completed run; a slow stderr consumer can affect ingest or index timing. The
old entry point remains available to callers that do not observe progress. The
CLI still emits failure reasons, which may include untrusted server text.

The existing `eval/lab/run_system.sh` assigned a container name before Docker
created it. A mock-Docker baseline test on Linux failed as expected: when
creation of `lab-qdrant` returned an error, the trap called `docker rm -f -v
lab-qdrant`. The revised launcher assigns ownership only after a successful
creation returns a 64-digit hexadecimal container ID, and cleans up that ID.
`bash -n` and the Unix `lab_runner` integration test passed on ARM Linux across
Qdrant, SurrealDB and Milvus, for failed creation, invalid returned ID and
successful ID cleanup. A real Docker name-collision exercise also returned
125 and `docker inspect` confirmed the pre-existing test container remained;
the test container was stopped rather than running. The first shell cleanup
attempt did not remove that test container, and an initial manual removal had
a mistyped ID; the exact test-created ID was then removed and its absence
checked. A lost or invalid Docker response after a real creation can leave an
unowned container for operator inspection rather than justify deleting by name.

The typed-progress test passed on ARM Linux and in the Windows package suite;
it verifies batch counts, completed cases and that refused preparation emits no
completed stage. A real `custom-reference` process on ARM, using the debug lab
and a synthetic 200-point/32-dimensional input, emitted the expected JSON
batch, index and case events and completed 20 filtered and 20 unfiltered
queries. Its stderr SHA-256 was
`a2d38648eb09e0857c8590a94f6848cf4194e9a5130e9fecd0a29bb71a73c49a`.
It ran while the separate release diagnostic used other cores; this is an
integration smoke, not a performance measurement.

## Retained 20,000-point ARM diagnostic

The release executables were built offline on `clyffy-01` from the source
archive, patch, and final adapter file identified above. `cargo build --offline
--locked --release -p rrflow-eval --bin rrflow-lab -p rrflow-cli --bin rrflow`
passed; executable SHA-256 values were
`52ca1afbb866e82089cd36a556cc3db59f9ce69949789f18595d8a8827203b80`
for RRFlow and
`35d36ddd2357b7e026c21dba4a0791d06621df2465c0efe39c313da4e58f3df1`
for the lab. This diagnostic used that lab executable before the subsequent
progress implementation, and the original launcher for the already-started
HNSW arm; subsequent arms used the corrected launcher. The data and reports
are retained under `eval/lab/evidence/arm-20k-20261106/`. The executed raw manifest
file SHA-256 is `573d69d7843ab5c021e0e269e7473b7222baea8606d833cb74fbe04b5f5a1fa5`.
The retained manifest is the identical JSON with CRLF normalized to LF, SHA-256
`554e8ed64eee906862e56502d8b8a0530e2534baf20ca13cd60026a6cf8e9b09`;
this correction followed a `git diff --check` trailing-CR failure. The canonical
manifest digest in every run is
`7859639965b045c230d6ba7c9b6ef3e661396a32d11a974b7590a149e4f1da3f`.
The combined report SHA-256 is
`1132b667a0d6016dd61529fb6a114f04af00b37646631a3b578eb46a6e2a8b74`.
All eight report-referenced run and sample artifacts matched their recorded
sizes and digests; all four run identity maps matched. A bounded scan for
common credential patterns found none, but does not prove no secret is present.
Server logs are not included in the retained artifacts.

Each arm ran sequentially on disk-backed ext4 with server CPU set
`5-9,15-19` and client CPU set `0-4,10-14`, 20,000 points, 384 dimensions,
100 queries per case, one client, `k=10`, with and without a category filter.
HNSW used `ef=32`; TurboQuant used 4-bit quantization and rerank 40.
All four arms reported 100 successes and zero errors in each case. RRFlow
HNSW reported mean recall 0.982/1.0 and median client latency
1,765.8/1,743.8 ms (unfiltered/filtered); TurboQuant reported 0.985/1.0 and
60.0/31.4 ms. SurrealDB reported 0.979/1.0 and 15.5/22.8 ms. Qdrant reported
1.0/1.0 and 1.81/1.47 ms but its own evidence was `0 of 20000 points indexed`,
so it was not a matched HNSW comparison. These are one-run diagnostic numbers,
not a competitive or release qualification. The host was shared, a build and a
short custom-system smoke ran during HNSW, and CPU placement alone does not
prove isolation. The original combined report's server-resource verdict must
not be promoted into a claim of equivalence.

Source inspection in `engine/vector/search.rs` establishes that HNSW's selected
route currently reads the vector-version source before graph traversal when the
specialized TurboQuant path does not serve the request. This is a plausible
optimization lead, not a measured causal profile. New [POAM-056](../../../poam/rrflow-1.0-alpha/poam-056.md)
records it and its correctness and attribution requirements. No gate is closed.
The first attempt to start the larger run had a shell-script syntax error and
did not run a system; the corrected launch produced the retained evidence.

## Current integration state

PR #258 was refreshed separately against public main and its local navigation,
documentation, and six navigation tests passed. It was marked ready, starting
required candidate CI; it is not yet merged. PR #253 still depends on its
POA&M records and must reconcile its own ledger/index after #258 lands, then
pass candidate CI on that exact merged state. Connectome report reading and
authorized launching remain unimplemented. These records and small synthetic
runs do not establish alpha, J-04, C-07, native platform qualification, or
performance superiority.

## Package checks after the visibility and ownership changes

`cargo test --offline --locked -p rrflow-eval` passed on Windows (including
27 lab contract tests; the Unix-only runner test is skipped there), and
`cargo clippy --offline --locked -p rrflow-eval --all-targets -- -D warnings`
passed. `cargo fmt --all -- --check` and `git diff --check` passed. The first
full ARM Linux package suite failed `generated_schemas_match_the_checked_in_projections`:
the isolated server tree still contained an older `lab-manifest-v1.json`.
The local generated schema hash was
`fc293eaa140741ed137b8b6baa7dbf02df42d7be02201b7d5366fbc0da53bdf3`,
while the copied server tree's schema hash was
`333ab76184e0ad2a8c63b6b638574a10b32de6d86d4724515a7b13f8795494e2`.
After copying the candidate schema, the full ARM `cargo test --offline --locked
-p rrflow-eval` passed, including the Unix launcher test. The discrepancy was
in the test fixture copy; it does not change the earlier benchmark binaries.
No macOS or complete candidate CI has been run for this revised package.

`python -X utf8 scripts/ci/build_execution_inventory.py` regenerated the local
mechanical inventory (2,428 records) after staging the package paths.
