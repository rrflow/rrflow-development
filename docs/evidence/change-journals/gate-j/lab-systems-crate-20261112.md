# RRFlow Lab's systems under test get their own crate

**Status:** active immutable change-journal evidence; J-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-systems-crate-20261112`
**Owner:** evidence for the [tunable reasoning and Lab plan](../../../roadmap/tunable-reasoning-and-lab-engineering-plan.md), package T4

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: J-04's comparison lab; T4 of the tunable reasoning and Lab plan, the first lab crate of the evaluation layout the owner approved on 2026-11-12
starting revision/tree and pull request: development/main 3e885a41 (after #369 and #370); branch agent/lab-systems-20261112
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the system descriptions, the adapter interface, the five adapters and one connect function naming every system lived in rrflow-eval's lab module, mixed with the vector test definition, and the RRFlow adapter carried the planning tasks' chunk importer -> crates/evaluation/lab/rrflow-lab-systems holds binding.rs (the system descriptions, validated against a k instead of the whole workload), adapter.rs (the interface, the HTTP helper and ConnectionLimits), adapters/ (one file per system) and registry.rs (one entry per system declaring its capabilities, replacing connect's match); rrflow-eval keeps the vector test definition in manifest.rs, whose WorkloadSpec supplies the adapters' connection limits; the RRFlow adapter's import_chunks becomes import_records(table, migration, records), and the planning tasks pass their chunk mutations to it. Unchanged: every wire format, schema, serialization and test. Stop if a test was lost or any adapter's requests changed
files and symbols read in full: rrflow-eval src/lab/manifest.rs and adapter.rs; sections read: the five adapters' constructors and imports, rrflow.rs (connect, connect_to, import_chunks), the rrflow-lab binary's connect and import call sites, tests/lab_adapters.rs (its workload helper)
implementation/evidence paths created, changed, moved, or deleted: moved src/lab/{adapter,rrflow,qdrant,surrealdb,milvus,custom}.rs and tests/lab_adapters.rs into the new crate; split src/lab/manifest.rs into the crate's binding.rs and rrflow-eval's manifest.rs; created the crate's lib.rs, adapters/mod.rs and registry.rs; changed imports in rrflow-eval's lab modules, binary and tests; registered the crate in Cargo.toml, Cargo.lock, workspace_architecture.rs, the product-operations engine suite and the execution inventory
research decision and primary-source/adaptation record: the registry follows the pattern of VectorDBBench, whose clients each implement one interface and are listed once (https://github.com/zilliztech/VectorDBBench)
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable; no runtime path changes
first-failure or characterization oracle and result: every test name on main is present on the branch and one is added (every_system_kind_has_exactly_one_entry); the moved adapter tests replay captured Qdrant, SurrealDB and Milvus responses and pass unchanged in assertion
acceptance commands and exact results: cargo test -p rrflow-eval -p rrflow-lab-systems: 264 passed, 0 failed; cargo test -p rrd-engine --test workspace_architecture: 34 passed, the evaluation layer rules included; python scripts/ci/check_workflow.py: CI policy OK; cargo clippy -p rrflow-eval -p rrflow-lab-systems --all-targets -- -D warnings and cargo fmt --all -- --check clean; cargo metadata --locked resolves; build_execution_inventory.py --check OK; test_select_ci_lanes.py OK
surfaced failures and corrections: a first count of test attributes matched main exactly although a test was added, because git grep skips untracked files; the per-name comparison is the evidence
checks not run and reason: lab_runner is Unix-only and runs in candidate CI's Linux lane
remaining known errors and owner: the adapter interface keeps its name, SystemAdapter, until the reasoning-harness crate adds a second capability; the benchmarks, datasets, evals, definitions and tool crates are the next T4 steps
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
