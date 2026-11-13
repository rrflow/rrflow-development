# RRFlow Lab's datasets get their own crate

**Status:** active immutable change-journal evidence; J-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-datasets-crate-20261113`
**Owner:** evidence for the [tunable reasoning and Lab plan](../../../roadmap/tunable-reasoning-and-lab-engineering-plan.md), package T4

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: J-04's comparison lab; T4, the second lab crate
starting revision/tree and pull request: agent/lab-systems-20261112 (#374) at 677179ea; branch agent/lab-datasets-20261113
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the vector corpora with their exact answers (corpus.rs) and the repository text chunker (knowledge.rs) lived in rrflow-eval's lab module, and CorpusSource and CorpusSpec sat in the vector test definition -> crates/evaluation/lab/rrflow-lab-datasets holds corpus.rs, with CorpusSource and CorpusSpec, and knowledge.rs; LabMetric moves into rrflow-evaluation-core, because both systems and datasets need it and tier-1 crates may not depend on each other. Unchanged: every schema, serialization and test. Stop if a schema golden or test changed
files and symbols read in full: rrflow-eval src/lab/corpus.rs and knowledge.rs (imports); sections read: manifest.rs (CorpusSource, CorpusSpec), the users of corpus and knowledge
implementation/evidence paths created, changed, moved, or deleted: moved src/lab/corpus.rs and knowledge.rs into the new crate; moved CorpusSource and CorpusSpec into corpus.rs; moved LabMetric from rrflow-lab-systems' binding.rs into rrflow-evaluation-core; created the crate's Cargo.toml and lib.rs; changed imports; registered the crate in Cargo.toml, Cargo.lock, workspace_architecture.rs, the product-operations engine suite and the execution inventory
research decision and primary-source/adaptation record: not applicable
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable
first-failure or characterization oracle and result: the evaluation layer rule refused "lab crate rrflow-lab-datasets depends on rrflow-lab-systems, which is not in a lower tier", which exposed that the dependency existed only for LabMetric; moving the metric to the shared core removed it. generated_schemas_match_the_checked_in_projections refused a doc comment added to LabMetric, because schemars publishes doc comments; it was removed and every committed schema is unchanged
acceptance commands and exact results: cargo test -p rrflow-eval -p rrflow-lab-systems -p rrflow-lab-datasets -p rrflow-evaluation-core --no-fail-fast: 266 passed, 0 failed, every test name unchanged; cargo test -p rrd-engine --test workspace_architecture: 34 passed; python scripts/ci/check_workflow.py: CI policy OK; clippy -D warnings, cargo fmt --check and cargo metadata --locked clean
surfaced failures and corrections: the two refusals above
checks not run and reason: lab_runner is Unix-only and runs in candidate CI's Linux lane
remaining known errors and owner: the benchmarks, evals, definitions and tool crates are the next T4 steps
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
