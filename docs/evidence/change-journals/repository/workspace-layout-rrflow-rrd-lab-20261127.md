# Every crate sits at `rrflow/`, `rrd/` or `lab/`, named for its folder

**Status:** active landing evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/workspace-layout-rrflow-rrd-lab-20261127`
**Owner:** [Code structure and conventions](../../../architecture/code-structure-and-conventions.md), section 6.2; working plan [`.ai/plan-engine-restructure.md`](../../../../.ai/plan-engine-restructure.md), phases 0 to 2; kernel package K13.

## Change brief

Baseline: `main` at `1b50eaf58` (pull request 519). Current behavior: crates sit under `crates/<tier>/<package>`, eleven product crates carry the daemon's `rrd-` prefix, the storage port returns the six semantic repositories, and the engine re-exports `rrflow-core` as `kernel`. Target: one flat level of folders under `rrflow/`, `rrd/` and `lab/`, a package named `<prefix><folder>`, one pass with no alias left behind. The owner ordered this on 2026-11-26: pre-release, no compatibility, no staged renames, no `foundation` or `kernel`, self-describing folder names, and SurrealDB's repository as the model for the shape.

Failure oracle: the workspace does not compile, a test that passed on the baseline fails, or `check_architecture.py` reports a crate outside its group. Stop condition: a stored-format constant or a digest-bound record would have to change.

Research decision: SurrealDB's workspace was read for its shape only (a product-named top folder, short crate folders, `<product>-<folder>` packages); no code was taken. Trace and debugging decision: none; no runtime path changes.

This also repairs [the red nightly of 2026-11-26](https://github.com/rrflow/rrflow-development/issues/520): see "The red nightly" below.

## Traceability

Every folder moved with `git mv`. No file was merged.

| Was | Is |
|---|---|
| `crates/core` (`rrflow-core`) | `rrflow/core` |
| `crates/storage/rrflow-lsm`, `rrflow-cld` | `rrflow/lsm`, `rrflow/cld` |
| `crates/storage/rrd-store` (`rrd-store`, `rrd_store`) | `rrflow/db` (`rrflow-db`, `rrflow_db`) |
| `crates/compute/rrflow-arrow` | `rrflow/arrow` |
| `crates/compute/rrd-query`, `rrd-vector`, `rrd-inference`, `rrd-attunement` | `rrflow/ql`, `rrflow/vector`, `rrflow/inference`, `rrflow/attunement` (`rrflow-ql`, …) |
| `crates/operations/rrd-cluster`, `rrd-kubernetes` | `rrflow/cx`, `rrflow/cx-operator` (`rrflow-cx`, `rrflow-cx-operator`) |
| `crates/operations/rrd-observation`, `rrd-operator-knowledge` | `rrflow/observability`, `rrflow/knowledge` |
| `crates/authority/rrd-engram`, `rrd-security` | `rrflow/engram`, `rrflow/security` |
| `crates/adapters/rrflow-cli`, `rrflow-mcp`, `rrflow-edge` | `rrflow/cli`, `rrflow/mcp`, `rrflow/edge` |
| `crates/authority/rrd-engine` | `rrd/engine` |
| `crates/transport/rrd-server`, `rrd-contract`, `rrd-client` | `rrd/server`, `rrd/contract`, `rrd/client` |
| `crates/evaluation/rrflow-evaluation-core` | `lab/core` (`rrflow-lab-core`) |
| `crates/evaluation/lab/rrflow-lab-<x>` | `lab/<x>` |
| `crates/evaluation/lab/rrflow-lab` | `lab/runner` (`rrflow-lab-runner`) |
| `crates/evaluation/qualification/rrflow-qualification-storage` | `lab/qualification-storage` (`rrflow-lab-qualification-storage`) |
| `crates/architecture.toml` | `architecture.toml` |

Deleted: `crates/operations/rrd-maintenance` (1,574 lines). The manifest had marked it `retire`, no crate used it, and its one entry in the direct-read allowlist said "the crate has no in-tree caller". That allowlist is now empty.

## What changed

- **Names.** Package and library names were replaced in code, manifests, workflows, scripts and current documents. Binary names did not change: `rrd-cluster-node`, `rrd-kubernetes`, `rrd-kubernetes-crd`, `rrd-engram-admin`, `rrd-engram-controller`, `rrd-deployment-catalog`, `rrflow-lab` and `rrflow-qualification-storage` are programs that run, and deploy manifests name them. `rrflow/cx-operator` now declares its `rrd-kubernetes` binary explicitly, because the default would have followed the package name.
- **Storage port.** `StorageEngine` is the physical port only: `begin_transaction` and `physical_store_evidence`. The six repositories (`claims`, `control`, `function_catalogue`, `projections`, `runtime`, `invocations`) come from the `Repositories` extension trait over it. The engine imports that trait once in `engine/mod.rs`; 41 test modules that do not inherit it import it themselves.
- **No kernel.** The engine's `pub use rrflow_core as kernel` is gone; 170 uses name `rrflow_core`. `rrflow_db::Error::Kernel` is `Error::Core`, and its text prefix is `core: `. The CI suite `kernel-storage` is `core-storage`.
- **Architecture manifest, version 2.** Tiers are replaced by three groups and one ordered list. Each crate records the exact workspace crates it uses; a used crate must be listed above its user. `pending` marks the six dependencies the owner's component table removes. `rrflow-core` has a fixed module list and a line count that may only fall. `kernel`, `foundation`, `shared` and `base` join the banned names, for crate folders as well as modules. `check_architecture.py` and its 17 tests were rewritten for this.
- **Paths in code.** 76 relative path literals that climb out of a crate were recomputed from each file's old location. Three places counted folder depth or looked for a `crates` folder to find the repository root; they now look for `Cargo.lock`.
- **Scripts.** `workspace_packages.py` owns the list of crate groups. `check_code_shape.py`, `check_workflow.py`, `check_version.py`, `selected_tests.py`, `select_ci_lanes.py` and `build_execution_inventory.py` use the groups in place of `crates/`.
- **`rrflow-arrow` stays a crate.** The plan had `rrflow-cld` absorb it. The existing test "QL must access storage through its port" forbids QL from depending on `rrflow-cld`, and QL uses the Arrow row types, so the merge would have pulled the cold codecs into QL.
- **Left as they were:** the text of evidence, history and research records and the files under `eval/inputs` and `eval/results`. Only links that pointed at a moved file were repointed (three records). Digest domain strings such as `rrd-vector-search-request-v1` are stored identities and did not change.

## The red nightly

Run 45848681116 failed `the_checked_in_pilot_chooses_no_model_so_nothing_can_spend`: the study `rrflow-planning-pilot.json` pins the SHA-256 of the task catalog `eval/lab/tasks/rrflow-package-planning.json`, and pull request 519 had rewritten folder paths inside that catalog. Each task there names a repository revision, and its `expected_files` are paths at that revision, so they are history and must not follow a move. The catalog is restored to its content before pull request 519. The study pin is unchanged.

## Verification (Windows, local)

- `cargo check --workspace --all-targets --all-features` and `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean. `cargo fmt --all --check`: clean.
- `cargo test -p rrd-engine --test workspace_architecture --test direct_read_source_closure`: 37 passed. Three of them (`one_engine_authority_owns_every_product_storage_opening`, `storage_semantics_use_repositories_over_one_transaction_port`, `alpha_storage_closure_has_one_required_physical_dependency_and_current_reader`) failed first and were fixed: two named folders by their old names, and one still required the repository accessors on the physical port.
- `cargo test` for `rrflow-core`, `rrflow-arrow`, `rrflow-lsm`, `rrflow-cld`, `rrflow-db`, `rrflow-ql`, `rrflow-vector`, `rrflow-cx`, `rrflow-cx-operator`, `rrflow-inference`, `rrflow-attunement`, `rrflow-engram`, `rrflow-security`, `rrflow-observability`, `rrflow-knowledge` and `rrd-contract`: the first run failed 15 tests in four targets. Fourteen read recorded endpoint fixtures whose bytes the rename had changed (their file digests are recorded beside them); the fixtures were restored byte for byte. One read a fixture through a path built with `format!`, which the path rewrite had missed. After the fixes `rrd-contract`, `rrflow-ql` and `rrflow-inference` gave 313 passed, 0 failed; the other thirteen crates had no failure in the first run.
- `check_architecture.py`, `check_code_shape.py`, `check_selected_tests.py`, `check_workflow.py`, `check_documentation.py`, `check_generated_surfaces.py`, `check_version.py`, `build_execution_inventory.py --check`, and every `test_*.py` under `scripts/`: OK, except `test_windows_harness_image.py`, which fails six tests on a missing image path on this machine and was not compared with `main`.
- **Not run locally:** the full suites of `rrd-engine`, `rrd-server`, `rrd-client`, `rrflow-cli`, `rrflow-mcp`, `rrflow-edge` and the nine lab crates. The local run was stopped because the machine ran low on memory. `the_checked_in_pilot_chooses_no_model_so_nothing_can_spend`, the test behind the red nightly, is in that set and has not been seen to pass. A build of every test binary with `--all-features` also fails to link two `rrd-client` examples on Windows (`LNK1140`, the debug-database size limit); that was not compared with `main`. The candidate gate runs these on Linux.
- Not run: Linux, macOS.

## Observable differences

- Trace targets follow the library name: `rrd_store::diagnostic` and `rrd_store::transaction` are now `rrflow_db::…`, and the same holds for every renamed library. A log filter written against an old target no longer matches.
- A storage error that came from `rrflow-core` prints `core: …` where it printed `kernel: …`.
- `cargo` commands name the new packages: `-p rrflow-db`, `-p rrflow-lab-runner --bin rrflow-lab`.

## Not done here

- `rrflow-mx`, `rrflow-tx` and `rrflow-kv` are not crates yet; their code is inside `rrflow-db`. QL, CX, vector, engram and security still depend on `rrflow-db`, recorded as `pending`.
- `rrflow-core` is not reorganized; its 12,321 lines and 13 modules are only frozen.
- The engine's source folders are unchanged.
- The word "kernel" remains in comments and in test and fixture names that describe `rrflow-core`.

No objective, roadmap or POA&M status changes.
