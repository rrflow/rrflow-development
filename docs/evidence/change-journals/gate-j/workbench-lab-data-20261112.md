# The lab's data, scripts and schemas move under eval/workbench/lab

**Status:** active immutable change-journal evidence; J-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/workbench-lab-data-20261112`
**Owner:** evidence for the [comparison lab](../../../../eval/workbench/lab/README.md)

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: J-04's comparison lab; the owner's direction of 2026-11-12 that the lab be grouped, by folder and file, under a lab folder nested under the workbench
starting revision/tree and pull request: development/main 181b0e67; branch agent/workbench-lab-data-20261112; pull request #364 nests the Rust crates the same way under crates/evaluation/workbench/lab
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the lab's systems, tests, references, studies, tasks, scripts and evidence lived in eval/lab and its seventeen schemas in eval/schemas beside the storage schemas -> all of them live in eval/workbench/lab, schemas in eval/workbench/lab/schemas; rrflow-lab finds the lab at eval/workbench/lab; the scripts find the repository three levels up; every live reference is rewritten. Unchanged: every file's content other than paths, the storage schemas in eval/schemas, the lab's recorded run evidence, the prompt corpora under eval/inputs, and the text of earlier journals, whose local links alone are retargeted so they resolve. Stop if a signed artifact covered a changed file (none is committed; packs are built from these files at release) or any test changed
files and symbols read in full: eval/lab/README.md, pod.sh, pod_run.sh, run_system.sh, run_all.sh; sections read: rrflow_lab.rs (lab discovery, write_schemas), catalog.rs (resolve_test, substitute), driver.rs (probe), definitions/pack.rs (pack paths are relative to the lab folder), every lab test's path helpers, lab_contract.rs (schema projections), storage_contract.rs
implementation/evidence paths created, changed, moved, or deleted: moved eval/lab to eval/workbench/lab and eval/schemas/lab-*.json to eval/workbench/lab/schemas; rewrote references in code, tests, scripts, test definitions, .devcontainer/Containerfile and current docs; retargeted local links in eight earlier journals
research decision and primary-source/adaptation record: not applicable
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable
first-failure or characterization oracle and result: the documentation policy refused three journal links into eval/lab until links were retargeted; the schema projection test reads the moved schemas
acceptance commands and exact results: cargo test -p rrflow-eval --offline: every suite passes; bash -n on every lab script and python -m py_compile on every lab script pass; render_navigation.py and check_documentation.py OK; cargo fmt --all -- --check clean
surfaced failures and corrections: the journal links above
checks not run and reason: lab_runner is Unix-only and runs in candidate CI's Linux lane
remaining known errors and owner: the lab's Rust modules still sit in rrflow-eval until the later lab-split steps move them into crates/evaluation/workbench/lab; the real-project workbench script, scripts/evaluation/run-real-project-workbench.ps1, is not part of the lab and is not moved
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
