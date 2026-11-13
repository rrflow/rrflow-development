# The rrflow-lab tool gets its own crate, and rrflow-eval no longer holds the lab

**Status:** active immutable change-journal evidence; J-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-tool-crate-20261113`
**Owner:** evidence for the [tunable reasoning and Lab plan](../../../roadmap/tunable-reasoning-and-lab-engineering-plan.md), package T4

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: J-04's comparison lab; T4, the sixth lab crate, which completes moving the lab out of rrflow-eval
starting revision/tree and pull request: agent/lab-definitions-20261113 at fd676b77; branch agent/lab-tool-20261113
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the rrflow-lab binary, the lab's settings and run plan, and the five tests that span every lab domain lived in rrflow-eval -> crates/evaluation/lab/rrflow-lab holds main.rs (the rrflow-lab command), settings.rs and tests contract, automation, definitions, references and runner; rrflow-eval keeps storage acceptance and the paired agent-trial runner, and drops the lab module, the rrflow-lab binary and the dependencies only the lab used (ed25519-dalek, getrandom, rrd-client, tokio, ureq and the lab crates); schema(), the draft 2020-12 JSON Schema projection, moves into rrflow-evaluation-core and storage re-exports it; the lab README and pod_run.sh build -p rrflow-lab. Unchanged: every command, schema, serialization and test. Stop if a schema golden or test changed
files and symbols read in full: settings.rs imports; sections read: the rrflow-lab binary's imports, storage/validation.rs (schema), every lab test's path roots, the lab README and pod_run.sh build lines
implementation/evidence paths created, changed, moved, or deleted: moved src/bin/rrflow_lab.rs to the new crate's src/main.rs, src/lab/settings.rs to its src/settings.rs, and tests/lab_{contract,automation,definitions,references,runner}.rs to its tests; deleted src/lab/mod.rs; moved schema() into rrflow-evaluation-core; changed rrflow-eval's lib.rs and Cargo.toml, the lab README and pod_run.sh; registered the crate in Cargo.toml, Cargo.lock, workspace_architecture.rs, the product-operations engine suite and the execution inventory
research decision and primary-source/adaptation record: not applicable
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable
first-failure or characterization oracle and result: workspace_manifest_descriptions_use_canonical_product_spelling refused a description containing the lowercase command name; it was reworded. The moved tests read eval/lab and the repository root one directory further up from the deeper crate
acceptance commands and exact results: cargo test across rrflow-eval, rrflow-evaluation-core and the six lab crates with --no-fail-fast: 266 passed, 0 failed, every test name unchanged; cargo test -p rrd-engine --test workspace_architecture: 34 passed; python scripts/ci/check_workflow.py: CI policy OK; clippy -D warnings, cargo fmt --check and cargo metadata --locked clean
surfaced failures and corrections: the description above
checks not run and reason: tests/runner.rs is Unix-only and runs in candidate CI's Linux lane; the lab pod (pod.sh, pod_run.sh) was not run on this Windows host
remaining known errors and owner: rrflow-eval still holds storage acceptance and the paired agent-trial runner; splitting them into qualification and experiments crates is the next step
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
