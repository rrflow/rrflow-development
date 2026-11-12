# The evaluation layer's shared vocabulary becomes rrflow-evaluation-core

**Status:** active immutable change-journal evidence; J-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/evaluation-core-crate-20261112`
**Owner:** evidence for the [comparison lab](../../../../eval/lab/README.md) and release qualification

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: J-04's comparison lab and release qualification; step 1 of the evaluation layout the owner approved on 2026-11-12: crates/evaluation holds rrflow-evaluation-core, lab/ (RRFlow Lab's domain crates) and qualification/ (release gates), and rrflow-eval dissolves into them
starting revision/tree and pull request: development/main 181b0e67; branch agent/evaluation-core-20261112, which reuses the first commit of the closed #364 and renames its crate; #364's CI failure (a crate missing from the engine suite list) is corrected here
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the lab is one module of rrflow-eval and borrows Id and Digest from the storage acceptance harness; the report format and Availability sit beside vector-only code although the reasoning-task experiments use them -> crates/evaluation/rrflow-evaluation-core owns Id, Digest, Availability, require, sha256_hex, canonical_json and the test-agnostic report; storage acceptance re-exports Id and Digest from it, the lab and its tests import it directly, and the product-operations engine suite runs it. Unchanged: every type, schema, serialization and test. Stop if a schema golden or test count changed
files and symbols read in full: rrflow-eval src/lib.rs, src/lab/mod.rs, src/lab/adapter.rs, src/storage/model.rs (Id, Digest); sections read: every lab module's and lab test's imports, workspace_architecture.rs (package layout, tracked targets), check_workflow.py (engine suite coverage), build_execution_inventory.py (PACKAGE_GATES)
implementation/evidence paths created, changed, moved, or deleted: moved src/lab/report.rs into rrflow-evaluation-core/src/report.rs; moved Id and Digest from storage/model.rs and Availability from lab/manifest.rs into rrflow-evaluation-core/src/lib.rs; changed imports in lab modules, the rrflow-lab binary and the lab tests; registered the crate in Cargo.toml, Cargo.lock, workspace_architecture.rs, the product-operations engine suite and the execution inventory
research decision and primary-source/adaptation record: the layout follows the evaluation design approved by the owner; Databend nests crates by domain the same way (src/query/storages/fuse)
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable; no runtime path changes
first-failure or characterization oracle and result: on #364, check_workflow.py reported "engine suite coverage mismatch: missing=['rrflow-lab-core']"; here it reports OK. The evaluation tree has 265 tests on main and 267 here, the two new core tests
acceptance commands and exact results: cargo test -p rrflow-eval -p rrflow-evaluation-core: every suite passes, the storage schema goldens included; cargo test -p rrd-engine --test workspace_architecture: 33 passed; python scripts/ci/check_workflow.py: CI policy OK; cargo metadata --locked resolves; cargo fmt --all -- --check clean
surfaced failures and corrections: #364's missing engine suite entry, above; a first rewrite there gave Availability a second public path through manifest, removed before commit
checks not run and reason: none beyond candidate CI's platforms
remaining known errors and owner: the lab's domain crates, release qualification's crates, and the dependency rules that keep them apart are the next steps
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
