# Refresh storage projections after the resume catalogue change

**Status:** active candidate correction; validation and protected landing pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-budget-resume-storage-projections`
**Owner:** PR230; G-01b/G-02a / POAM-009; storage contract projection consumer

## Failure and bounded correction

The [current installed trial](task-budget-resume-current-installed.md) passed
on its recorded source and executable. The subsequent full candidate run
[42323376494](https://github.com/rrflow/rrflow-development/actions/runs/42323376494)
at `c43e020e92eee4c4733852b7dce4e8c0e45255bf` failed product-operations:
33 evaluator library tests rejected the synthetic fixture's stale public
catalogue identity, and the storage-contract golden also failed.
The same run's service-protocol job later failed
`failed_campaign_preserves_closed_cause_and_readback_provenance`: its observed
error was likewise the stale public catalogue identity. Both job logs are retained.

The pre-edit PR230 correction brief binds the existing fixture generator and
its consumers. The local focused golden test reproduced the failure with exit
101 and automatic golden updates disabled. This is a missing generated
dependency in the resume reconciliation; the earlier component checks did not
cover the evaluator. No failed CI result is replaced by the live model trial.

`catalogue_digest()` derives a domain-separated digest of the typed endpoint
catalogue, separately from OpenAPI. The fixture retained
`52b0d8a701ec4c9078e069c3ce2aab49565f428b07d56600bf1ec5c2cc364f04`;
the combined catalogue requires
`545a4847892c31393fcb5e3e5b45e6150a1835d25ebeb90b91d88bb0b82f1a17`.

The existing `refresh_storage_contract` example refreshes three synthetic
receipts and 13 schema projections. Structural comparison finds exactly four
changed digest fields: the manifest's catalogue digest, result's manifest
digest, and report's manifest/result digests. All 13 schemas remain byte-identical.
Evidence classes, ledger contents, expected outcomes and every other field are
unchanged. A second generator run reproduces all 16 output hashes exactly.

These are explicitly synthetic contract fixtures. Generation executes no
storage workload, creates no training data and supplies no quality or capacity
measurement. Validators, assertions, runtime implementation, real observations
and historical receipts remain unchanged.

## Validation and review

Commands run serially with one Cargo job:

- `cargo run --locked -j 1 -p rrflow-eval --example refresh_storage_contract`:
  both runs pass; complete output closure and deterministic bytes reviewed.
- `cargo test --locked -j 1 -p rrflow-eval --all-targets --no-fail-fast`:
  57 pass, none failed or ignored, including the prior failures.
- The `rrflow-cli` `installed_reopen` provenance negative test passes: one test.
- The explicitly selected ignored `sustained::installed_capacity_workload_smoke`
  passes: one real installed check. It reports `capacity_qualification=false`.
- The remaining `rrd-kubernetes`, `rrd-observation` and `rrflow-edge` packages
  pass with `--all-targets --no-fail-fast`: 14 tests, none failed or ignored.
  The unchanged evaluator suite was not duplicated in that command.

Navigation and execution-inventory generation/checks, documentation policy and
whitespace checks pass. Final staged bytes must match the reviewed projections.
Full candidate and protected CI are required after the corrected source is
published; this local repair cannot stand in for either pipeline.

Review covered the generator, catalogue digest/validation owner, synthetic
manifest/result consumers, golden test and complete four-field diff. The
generator writes existing projections; this journal and its generated nearest
index supply the correction record. No algorithm adaptation or new research is
needed. Retained CI/local logs and source hashes supply diagnostics without a
new runtime ledger or telemetry path. Raw commands and failures remain private.

## Integration limits

PR230 returned to draft for this repair. Connectome PR29's proposed native pin
and binary must be refreshed to the corrected published source; its reconciliation
has not started. Existing native/desktop receipts retain their original identities.
No roadmap completion or version change is made. Stop on non-digest semantic
changes, schema drift, changed source, failed checks or unknown effects. UI
recovery/replanning, governed project changes, reviewed learning and the complete
binary/score/choice assessment-cycle workflow remain unfinished.
