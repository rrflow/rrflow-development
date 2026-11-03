# Selected effects: public-main integration

**Status:** active draft integration; dependency and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-selected-effects-main-integration`
**Owner:** D-02/D-03 and [PR #241](https://github.com/rrflow/rrflow-development/pull/241)

The local selected-effects commit `335fb83a6c065d774dc78d889686eae58969a7e8`
was merged normally with accepted public main
`a2fc4a3c57960e1d4fd10cdafe208d5b908daa62`. Merge preview and merge both
identified one content conflict in `docs/poam/rrflow-1.0-alpha/poam-006.md`.
The parser candidate's Tree-sitter allocation/isolation deficiency and main's
Linux ARM64 native-installed-evidence gap are separate; the resolved file
retains both entire entries. The generated journal index is regenerated from
the combined records. No product implementation was removed by the resolution.

The [selected-effects journal](d-02-selected-effects-completion.md) records
the pre-merge installed driver, contract and engine results and the exact
limits of this work. Main includes merged schemaless-document PR #256; this
merge does not infer its checks for the combined tree. The local branch also
contains unmerged dependency #246. On the resolved Windows tree,
`cargo test --offline --locked -p rrflow-cli --test installed_lifecycle adaptive_installed::selected_inventory_completes_and_replans_across_primary_process_restarts -- --exact --nocapture`
passed one selected test. It uses three source-built primary-process lifetimes,
public checkpoint-pinned source readback and a completed successor after a
project edit. Navigation and documentation-policy checks passed on the
combined tree. The broader installed Parse driver remains red, #252 remains open,
and automatic default capability selection is not implemented. No C-07,
alpha, native matrix, candidate-CI or merge-group success is claimed.

## Change checklist

- Scope: integrate accepted main, preserve both non-overlapping gap records,
  regenerate derived navigation and retain all selected-effect source/tests.
- Evidence: pre-merge tests are linked above; the merged-tree installed-driver
  command and result are recorded above. Full candidate CI remains unrun.
- Trace and resources: existing plan/checkpoint/receipt evidence is retained;
  no changed storage format or performance claim.
- Stop: no push of the local unmerged #246 dependency as if it were accepted
  main; no claim that Parse, project indexing, or the #252 visibility defect
  is complete.
