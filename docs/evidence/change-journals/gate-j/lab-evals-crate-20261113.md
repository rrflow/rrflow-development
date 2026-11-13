# RRFlow Lab's reasoning evaluations get their own crate

**Status:** active immutable change-journal evidence; J-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-evals-crate-20261113`
**Owner:** evidence for the [tunable reasoning and Lab plan](../../../roadmap/tunable-reasoning-and-lab-engineering-plan.md), package T4

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: J-04's comparison lab and its reasoning tests; T4, the fourth lab crate
starting revision/tree and pull request: agent/lab-benchmarks-20261113 at 85ffcdd7; branch agent/lab-evals-20261113
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the planning tasks with known answers, the study configuration and capture audit, and the coding-agent provider lived in rrflow-eval's lab module -> crates/evaluation/lab/rrflow-lab-evals holds tasks.rs, study.rs and provider.rs with their tests. Unchanged: every schema, serialization and test. Stop if a schema golden or test changed
files and symbols read in full: the imports of tasks.rs, study.rs and provider.rs and of their users, and the paths the study tests read
implementation/evidence paths created, changed, moved, or deleted: moved the three files and tests/lab_tasks.rs and tests/lab_study.rs into the new crate; created its Cargo.toml and lib.rs; changed imports; registered the crate in Cargo.toml, Cargo.lock, workspace_architecture.rs, the product-operations engine suite and the execution inventory
research decision and primary-source/adaptation record: not applicable
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable
first-failure or characterization oracle and result: the moved tests named rrflow_eval::storage::Digest inline and decoded rrd_contract mutations; the first now names the core's Digest and the second gains rrd-contract as a dev-dependency; the study tests read eval/lab one directory further up from the deeper crate
acceptance commands and exact results: cargo test across rrflow-eval, rrflow-evaluation-core and the four lab crates with --no-fail-fast: 266 passed, 0 failed, every test name unchanged; cargo test -p rrd-engine --test workspace_architecture: 34 passed; python scripts/ci/check_workflow.py: CI policy OK; clippy -D warnings, cargo fmt --check and cargo metadata --locked clean
surfaced failures and corrections: the two above
checks not run and reason: lab_runner is Unix-only and runs in candidate CI's Linux lane
remaining known errors and owner: the definitions and tool crates are the next T4 steps
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
