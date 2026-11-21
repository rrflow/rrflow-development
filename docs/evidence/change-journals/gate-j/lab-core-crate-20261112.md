# The comparison lab's shared vocabulary becomes its own crate

**Status:** active immutable change-journal evidence; J-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-core-crate-20261112`
**Owner:** evidence for the [comparison lab](../../../../eval/lab/README.md), step 1 of splitting it into crates

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: J-04's comparison lab; step 1 of the owner's 2026-11-11 direction that the workbench and harness be modular crates driven from Connectome, comparing vector systems and reasoning engines
starting revision/tree and pull request: development/main 181b0e67; branch agent/lab-core-crate-20261112; the pull request carries the change brief and the traceability map
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the lab is one module of rrflow-eval and borrows Id and Digest from the storage acceptance harness; the report format and Availability sit beside vector-only code although the reasoning-task experiments use them -> crates/evaluation/workbench/lab/rrflow-lab-core owns Id, Digest, Availability, require, sha256_hex, canonical_json and the report; storage re-exports Id and Digest from it; every user imports the core directly. Unchanged: every type, schema, serialization and test. Stop if any schema golden or test count changed
files and symbols read in full: rrflow-eval src/lib.rs, src/lab/mod.rs, src/lab/adapter.rs, src/storage/model.rs (Id, Digest); sections read: every lab module's and lab test's imports, workspace_architecture.rs (package layout, tracked targets), build_execution_inventory.py (PACKAGE_GATES)
implementation/evidence paths created, changed, moved, or deleted: moved src/lab/report.rs to rrflow-lab-core/src/report.rs; moved Id and Digest from storage/model.rs and Availability from lab/manifest.rs into rrflow-lab-core/src/lib.rs; changed imports in lab modules, the rrflow-lab binary and the lab tests; registered the crate in Cargo.toml, Cargo.lock, workspace_architecture.rs and the execution inventory
research decision and primary-source/adaptation record: not applicable
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable; no runtime path changes
first-failure or characterization oracle and result: every_workspace_target_source_is_tracked failed until the new lib.rs was staged; the evaluation tree has 265 tests on main and 267 here, the two new core tests
acceptance commands and exact results: cargo test -p rrflow-eval -p rrflow-lab-core --offline: every suite passes, the storage schema goldens included; cargo test -p rrd-engine --test workspace_architecture: 33 passed; cargo build -p rrflow-cli --tests compiles its storage imports; cargo clippy -p rrflow-lab-core -p rrflow-eval --all-targets -- -D warnings and cargo fmt --all -- --check clean
surfaced failures and corrections: a first rewrite gave Availability a second public path through manifest; it was removed and each user imports the core
checks not run and reason: none beyond candidate CI's platforms
remaining known errors and owner: the vector workload, the system adapters, the reasoning-task harness, probes, white-box tuning and the engine operations Connectome calls are later steps
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
