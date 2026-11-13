# rrflow-eval dissolves into release qualification and lab experiments

**Status:** active immutable change-journal evidence; J-02 and J-04 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/evaluation-split-20261113`
**Owner:** evidence for the [tunable reasoning and Lab plan](../../../roadmap/tunable-reasoning-and-lab-engineering-plan.md), package T4, and the evaluation layout the owner approved on 2026-11-12

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: the evaluation layout's last step: crates/evaluation holds rrflow-evaluation-core, lab/ and qualification/, and rrflow-eval no longer exists
starting revision/tree and pull request: agent/lab-tool-20261113 at 5f1c357b; branch agent/qualification-split-20261113
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): after the lab moved out, rrflow-eval still held storage acceptance (the storage release gate) and the paired agent-trial runner in one binary -> crates/evaluation/qualification/rrflow-qualification-storage holds the storage library, its command (rrflow-qualification-storage schema|validate|verify|campaign|acceptance ...), its contract test, fixtures and fixture refresher; crates/evaluation/lab/rrflow-lab-experiments holds the paired-trial runner (rrflow-lab-experiments run|summarize|verify) and its provider compatibility test; rrflow-eval is deleted. CI, check_workflow, the two storage scripts, the execution inventory, rrflow-cli's installed tests and the operational docs follow. Unchanged: every storage command's behavior and output, every schema, fixture and test. Stop if a schema golden, fixture or test changed
files and symbols read in full: rrflow-eval Cargo.toml, lib.rs, main.rs (dispatch), storage/cli.rs (run); sections read: storage_contract.rs and provider_compatibility.rs path and binary uses, collect_storage_acceptance.py, run_sustained_capacity.py, the CI steps and check_workflow strings that name the package, rrflow-cli's tests that import storage, eval/README.md, docs/operations/ci.md
implementation/evidence paths created, changed, moved, or deleted: moved src/storage/* (mod.rs as lib.rs), tests/storage_contract.rs, tests/fixtures/storage and examples/refresh_storage_contract.rs into rrflow-qualification-storage with a new src/main.rs; moved src/main.rs and tests/provider_compatibility.rs into rrflow-lab-experiments; deleted rrflow-eval's Cargo.toml and lib.rs; changed the workspace members, workspace_architecture.rs, ci-reusable.yml, check_workflow.py, build_execution_inventory.py, collect_storage_acceptance.py, run_sustained_capacity.py, run_offline_lifecycle.py, rrflow-cli's Cargo.toml and tests, eval/README.md, docs/operations/ci.md, docs/architecture/one-engine-execution-plan.md and docs/reference/automation/project-command-capabilities.md
research decision and primary-source/adaptation record: not applicable
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable
first-failure or characterization oracle and result: two include_str! fixture paths in sustained.rs pointed one directory too far once the storage files moved up from src/storage; the experiments tests needed tempfile as a dev-dependency
acceptance commands and exact results: cargo build --workspace --all-targets: clean; cargo test across rrflow-evaluation-core, the seven lab crates and rrflow-qualification-storage with --no-fail-fast: 266 passed, 0 failed, every test name unchanged; cargo test -p rrd-engine --test workspace_architecture: 34 passed; python scripts/ci/check_workflow.py: CI policy OK; python -m unittest discover -s scripts/ci: 240 passed, 14 skipped, and test_windows_harness_image's 6 setUp errors, which occur on main too because that file needs a built harness passed on its command line; build_execution_inventory.py --check OK; cargo fmt --check and cargo metadata --locked clean
surfaced failures and corrections: the two above
checks not run and reason: rrflow-cli's installed and sustained suites were compiled but not run here; candidate CI runs them
remaining known errors and owner: none for the layout; the lab's data regroup, Connectome's Lab and the Lab service are later packages
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
