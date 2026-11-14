# Code-shape splits, batch 1

**Status:** active change evidence

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/code-shape-splits-batch1-20261114`

**Owner:** rules R1, R2, R5 and R9 of the code structure and conventions standard (kernel package K13)

**Starting revision:** `development/main` at `501a16bc`. `development/main` was merged in after #376 landed.

**Owner direction, 2026-11-13:**
- "holy shit you have to get serious patterns folders if necessary colocated and consistent naming and get these properly fully broken down this is a nightmare".
- "get to work".
- "please resume so we can get this code truly rewritten".

**Method.** Every split moves items unchanged, by exact `syn` spans, into a folder named after the parent module:
- Child modules are named for one responsibility.
- Each moved file starts with a one-line `//!` description.
- An item that was private becomes `pub(super)`, and only the fields and methods the compiler reports as needed are raised. The parent re-exports each child at the visibility of its most public item.
- Inline test modules move to `<stem>/tests.rs` (R9).

Integration-test roots that gained children moved to `tests/<name>/main.rs`, so their children resolve without `#[path]`:
- `public_contract`;
- `attunement_contract`;
- `real_server`;
- `http_process`;
- `installed_lifecycle`;
- `workspace_architecture`.

Each `include_*!` path in moved code gained one `../`.

**Earlier per-split journals on this branch:**
- [Kernel runtime split](core-runtime-split-20261113.md)
- [Kernel trace split](core-trace-split-20261113.md)
- [rrflowLSM segment split](lsm-segment-split-20261113.md)
- [rrd-store backup split](store-backup-split-20261113.md)
- [rrd-vector HNSW split](vector-hnsw-split-20261113.md)

**Further splits in this batch.** Every module was over its limit before.

Kernel:
- `rrd-core`: `schema`, `authenticated_log`.

Storage:
- `rrd-lsm`: `segment/reader`, `segment/format`, `physical_policy_lab`, `manifest`.
- `rrd-store`: `access/versioned_read`, `object`, `s3`, `key_codec` (tests only).

Compute:
- `rrd-vector`: `turbo_segment`, `runtime`, `quantized_segment`, `turboquant`, `compact`, `collection`, `catalog` (tests only).
- `rrd-query`: `syntax`, `graphql`.

Engine (`rrd-engine`):
- `runtime/vector_catalog`, `runtime/query`, `runtime/data_plane`;
- `engine/flight`, `estate_control`, `cold_history`, `diagnostic`, `retrieval_query`, `endpoint_worker`;
- `engine/vector/collection`, `vector/search`, `vector/quantization`;
- `engine/reasoning_budget`, `restore_publication`;
- `transaction` and `token_key/private_file/windows` (tests only).

Contract and server:
- `rrd-contract`: `attunement`, `function`, `diagnostic`, `websocket`, `router`, `knowledge_tree` (tests only).
- `rrd-server`: `http/websocket`.
- `rrd-client`: `operation`.

Estate and cluster:
- `rrd-estate`: `local_process`, `recovery`, `backup_job`.
- `rrd-cluster`: `openraft_adapter`, `transport`, `artifact_transfer`, `contract`, `node_runtime`, `authority`.

Operations:
- `rrd-operator-knowledge`: `pgvector_postgres`.

Evaluation:
- `rrflow-qualification-storage`: `sustained`, `progress` (tests only).
- `rrflow-lab-evals`: `tasks`, `study`.
- `rrflow-lab-benchmarks`: `driver`.
- `rrflow-lab-systems`: `adapters/rrflow` (tests only).

**Entry files thinned below the 200-line entry limit (R2):**
- `lib.rs` of `rrd-estate`, `rrd-security`, `rrd-inference`, `rrd-operator-knowledge`, `rrd-kubernetes` and `rrd-observation`.
- `main.rs` of `rrflow-lab`, `rrd-server` and `rrflow-lab-experiments`. Each binary's tests sit under the module they exercise.

**Harness files split below 1,500 lines:**
- Engine unit tests:
  - `vector_index`, `function`, `security`, `attunement`, `installation`;
  - the reasoning-budget `source_bound` worker and its `assessment` tests.
- CLI support: `sustained`, `sustained/workload`, `capacity`.
- Integration tests: the six roots listed under Method.

**Patterns removed:**
- `rrd-lsm` retires its last `mod.rs`.
- The engine's private-file platform modules now use plain `unix` and `windows` modules with a `platform` alias instead of two `#[path]` attributes.

**Paths that followed the moves:**
- The reducer allowlist in `direct_read_source_closure.rs` now names `runtime/data_plane/attributes.rs`, `backup/retention.rs`, `cold_history/tests.rs`, `diagnostic/graph.rs` and `artifact_transfer/session.rs`.
- The attunement crash-child test name is now `engine::tests::attunement::crash::attunement_crash_child`.
- `build_execution_inventory.py` maps the moved integration-test roots to their `main.rs` paths.
- `workspace_architecture` reads every split module through `module_source`, which includes its child folder.

**Result.** `scripts/ci/code_shape_baseline.json` is tightened. These files remain over their limits:
- `crates/transport/rrd-contract/src/lib.rs` (329 lines). It is module declarations plus explicit re-export lists, and shrinking it would widen what the crate exports.
- `rrd-maintenance/src/lib.rs` (1,556 lines). It is removed under R11, not split, in its own package with an implementation-traceability map.

**Commands, on Windows 11 (one shared laptop):**
- `cargo clippy --workspace --all-features --all-targets --locked -- -D warnings`: clean.
- `cargo fmt --all`: clean.
- Package suites passed: `rrd-core`, `rrd-lsm`, `rrd-store`, `rrd-vector`, `rrd-query`, `rrd-contract`, `rrd-client`, `rrd-estate`, `rrd-cluster`, `rrd-operator-knowledge`, `rrd-kubernetes`, `rrd-observation`, the lab and qualification crates, `rrd-engine` (including `workspace_architecture` and `direct_read_source_closure`), `rrd-server` and `rrflow-cli`. Full counts are in the pull request.
- `rrflow-cli --test installed_lifecycle` runs without `--all-features`, as CI does. With `--all-features` the diagnostic fault-injection build of `serve` waits for a control message on stdin and exits with `invalid diagnostic control`. That comes from local flags, not from this change.
- `python3 scripts/ci/check_code_shape.py`: OK after `--tighten`.
- `python3 scripts/ci/build_execution_inventory.py --check`: OK.

**Not done here:**
- macOS and Linux runs, which only candidate CI provides.
- Connectome restructuring, which happens in its own repository.
