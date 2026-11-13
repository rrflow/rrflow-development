# RRFlow Lab's definition packs get their own crate

**Status:** active immutable change-journal evidence; J-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-definitions-crate-20261113`
**Owner:** evidence for the [tunable reasoning and Lab plan](../../../roadmap/tunable-reasoning-and-lab-engineering-plan.md), package T4

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: J-04's comparison lab and its signed definition packs; T4, the fifth lab crate
starting revision/tree and pull request: agent/lab-evals-20261113 at fd197f11; branch agent/lab-definitions-20261113
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the product-neutral pack envelope and store, the pack home and RRFlow Lab's pack content with its trusted keys lived in rrflow-eval's lab/definitions module -> crates/evaluation/lab/rrflow-lab-definitions holds pack.rs, home.rs, content.rs and assets/lab-trusted-keys.json. A pack carries tests, systems and references, so content reads the benchmark catalog and references; LAB_TIERS gains a tier for definitions between the domain crates and the rrflow-lab tool. stem, systems_of and write_atomically become public because the lab's settings call them from another crate. Unchanged: the trusted key, every pack format, schema and test. Stop if a pack signature, schema golden or test changed
files and symbols read in full: definitions/{mod,pack,home,content}.rs imports and include paths; sections read: their users in settings.rs, the rrflow-lab binary and the lab tests, and the README link to the trusted keys
implementation/evidence paths created, changed, moved, or deleted: moved pack.rs, home.rs and content.rs and assets/lab-trusted-keys.json into the new crate and deleted definitions/mod.rs; created the crate's Cargo.toml and lib.rs; changed imports and the README link; registered the crate in Cargo.toml, Cargo.lock, workspace_architecture.rs (with the new tier), the product-operations engine suite and the execution inventory
research decision and primary-source/adaptation record: not applicable
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable
first-failure or characterization oracle and result: tests/lab_definitions.rs and tests/lab_references.rs run the rrflow-lab binary, which only its own package can name, so they stay with the binary until the tool crate moves; the three helpers above failed as private until made public
acceptance commands and exact results: cargo test across rrflow-eval, rrflow-evaluation-core and the five lab crates with --no-fail-fast: 266 passed, 0 failed, every test name unchanged; cargo test -p rrd-engine --test workspace_architecture: 34 passed; python scripts/ci/check_workflow.py: CI policy OK; clippy -D warnings, cargo fmt --check and cargo metadata --locked clean
surfaced failures and corrections: the two above
checks not run and reason: lab_runner is Unix-only and runs in candidate CI's Linux lane
remaining known errors and owner: the rrflow-lab tool crate is the next T4 step
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
