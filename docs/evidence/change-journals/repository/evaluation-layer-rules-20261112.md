# Architecture rules for the evaluation layer, before its crates move

**Status:** active immutable change-journal evidence; no gate advanced
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/evaluation-layer-rules-20261112`
**Owner:** the workspace architecture tests in `crates/authority/rrd-engine/tests/workspace_architecture.rs`

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: step 2 of the evaluation layout the owner approved on 2026-11-12 (crates/evaluation with rrflow-evaluation-core, lab/ and qualification/)
starting revision/tree and pull request: development/main 181b0e67; branch agent/evaluation-rules-20261112; the pull request carries the change brief
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): nothing states how evaluation crates may depend on each other or on the product -> evaluation_layer_is_measured_from_outside_and_keeps_its_parts_apart places every crate by its path (shared, crates/evaluation/lab, crates/evaluation/qualification, or the product) and requires that a product crate use an evaluation crate only as a dev-dependency, that lab and qualification never depend on each other, that rrflow-evaluation-core depend on no evaluation crate, that a lab crate be named rrflow-lab or rrflow-lab-<domain>, be listed in LAB_TIERS and depend only on lower tiers, and that a qualification crate be named rrflow-qualification-<gate>. Unchanged: every crate and dependency. Stop if any existing dependency broke a rule
files and symbols read in full: workspace_architecture.rs (WorkspaceMetadata, workspace_metadata, the layout test); sections read: rrflow-cli's manifest (rrflow-eval is a dev-dependency)
implementation/evidence paths created, changed, moved, or deleted: changed workspace_architecture.rs (LAB_TIERS, EvaluationArea, evaluation_area, the new test); created this journal
research decision and primary-source/adaptation record: the downward-only rule follows Feature-Sliced Design's layer rule, that a slice imports only from lower layers (https://feature-sliced.design/docs/reference/layers); domain folders follow Databend's nested layout
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable
first-failure or characterization oracle and result: moving rrflow-cli's rrflow-eval dependency from dev-dependencies to dependencies fails with "rrflow-cli may use evaluation crate rrflow-eval only in its tests"; adding a member crates/evaluation/lab/rrflow-lab-bogus fails with "lab crate rrflow-lab-bogus must be named rrflow-lab or rrflow-lab-<domain> and placed in LAB_TIERS"; both restored
acceptance commands and exact results: cargo test -p rrd-engine --test workspace_architecture: 34 passed; cargo clippy -p rrd-engine --test workspace_architecture -- -D warnings and cargo fmt --all -- --check clean
surfaced failures and corrections: none
checks not run and reason: the lab, qualification and tier rules have no crates to judge until the next packages add them; the two forced failures above show they fire
remaining known errors and owner: none
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```
