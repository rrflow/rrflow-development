# RRFlow Lab's benchmarks get their own crate

**Status:** active immutable change-journal evidence; J-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-benchmarks-crate-20261113`
**Owner:** evidence for the [tunable reasoning and Lab plan](../../../roadmap/tunable-reasoning-and-lab-engineering-plan.md), package T4

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: J-04's comparison lab; T4, the third lab crate
starting revision/tree and pull request: agent/lab-datasets-20261113 at 96c29798; branch agent/lab-benchmarks-20261113
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the vector test definition, the runner, the catalog, the VectorDBBench import and published references lived in rrflow-eval's lab module, and the list of outside systems and their release pins sat beside them -> crates/evaluation/lab/rrflow-lab-benchmarks holds manifest.rs, driver.rs, catalog.rs, import.rs and references.rs with their catalog and import tests and fixture; sources.rs joins rrflow-lab-systems, because it describes systems. Unchanged: every schema, serialization and test. Stop if a schema golden or test changed
files and symbols read in full: the imports of manifest.rs, driver.rs, catalog.rs, import.rs, references.rs and sources.rs, and of their users
implementation/evidence paths created, changed, moved, or deleted: moved those five files, tests/lab_catalog.rs, tests/lab_import.rs and tests/fixtures/vdbbench-qdrant-result.json into the new crate and sources.rs into rrflow-lab-systems; created the crate's Cargo.toml and lib.rs; changed imports; registered the crate in Cargo.toml, Cargo.lock, workspace_architecture.rs, the product-operations engine suite and the execution inventory
research decision and primary-source/adaptation record: not applicable
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable
first-failure or characterization oracle and result: the import test could not find its fixture until the fixture moved with it; the catalog test reads eval/lab one directory further up from the deeper crate
acceptance commands and exact results: cargo test -p rrflow-eval -p rrflow-lab-systems -p rrflow-lab-datasets -p rrflow-lab-benchmarks -p rrflow-evaluation-core --no-fail-fast: 266 passed, 0 failed, every test name unchanged; cargo test -p rrd-engine --test workspace_architecture: 34 passed; python scripts/ci/check_workflow.py: CI policy OK; clippy -D warnings, cargo fmt --check and cargo metadata --locked clean
surfaced failures and corrections: the two above
checks not run and reason: lab_runner is Unix-only and runs in candidate CI's Linux lane
remaining known errors and owner: the evals, definitions and tool crates are the next T4 steps
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
