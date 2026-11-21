# Project policy candidate fixture repair

**Status:** active bounded repair evidence; local tests passed; hosted gate pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/project-policy-candidate-repair-20261121`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), D-01/D-06 and [POAM-077](../../../poam/rrflow-1.0-alpha/poam-077.md)

## Change brief and checklist

PR [458](https://github.com/rrflow/rrflow-development/pull/458), starting revision
c4079b314b902c800034fbbf45de37e48f9f4705. The public policy operations changed
the catalogue identity but its synthetic storage fixture was not regenerated.
Candidate run [44924826660](https://github.com/rrflow/rrflow-development/actions/runs/44924826660)
failed catalogue validation in storage qualification and installed reopen tests.
The repair brief was added to the PR before editing.

Use the existing refresh_storage_contract example to regenerate the manifest,
result and report under rrflow-qualification-storage/tests/fixtures/storage.
Only four digest values change. Catalogue validation, failure provenance tests,
runtime code and qualification requirements remain intact. Stop on unexpected
schema or workload changes. No external research, runtime trace or resource
measurement is required for these deterministic synthetic projections.

Read in full: the generator. Read relevant regions: storage_contract.rs golden
writer and fixture builder, sustained/point_run.rs failure-provenance test,
validation.rs catalogue binding. Reviewed the complete three-file generated diff.
This record follows the [implementation journal](project-policy-controls-20261121.md).
No implementation is deleted or replaced; no upstream code is adapted.

## Commands and results

Windows; CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target; CARGO_BUILD_JOBS=4.

- cargo run --locked -p rrflow-qualification-storage --example refresh_storage_contract: passed.
- cargo test --locked -p rrflow-qualification-storage: passed (43 library and 11 integration tests; no failures).
- cargo test --locked -p rrflow-cli --test installed_reopen failed_campaign_preserves_closed_cause_and_readback_provenance: passed, one selected test.

Logs are in D:/.usr/Temp/rrflow-project-policy-20261121/storage-gate-repair.log
and reopen-gate-repair.log. The hosted failure log is ci-44924826660-failed.log.
No native performance or storage qualification run was performed. Candidate and
merge queue must still pass on the repaired revision. No roadmap or POAM status
changes; this is a generated-fixture repair, not branch-absorption proof.
