# D-02 attunement cancellation candidate CI correction

**Status:** active correction evidence; fresh candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-public-attunement-cancel-ci-correction`
**Owner:** OBJ-14, D-02a/b and POAM-040; observed optional-cluster failure under POAM-023

## Candidate failure and correction

[PR 172](https://github.com/rrflow/rrflow-development/pull/172) candidate `73e7b72a98e0484fd30975cfd6676a118945cd45` ran [CI 40637793176](https://github.com/rrflow/rrflow-development/actions/runs/40637793176). Its [product-operations job](https://github.com/rrflow/rrflow-development/actions/runs/40637793176/job/122648671618) failed: `rrflow-eval` reported 32 library failures because the public catalogue identity in the synthetic manifest differed from the validator, and the storage golden test failed. The local focused reproduction `cargo test -p rrflow-eval --test storage_contract generated_schemas_and_synthetic_receipts_match_goldens --locked` failed on that same fixture. The exact failed job log was retained locally as `target/attunement-cancel-ci-product-operations.log`, SHA-256 `e13628229a380fb0b04350195f976fa688c898974be07b28785c8df4685c5ae1`.

After recording the correction in the PR brief, `cargo run -p rrflow-eval --example refresh_storage_contract --locked` regenerated only `crates/evaluation/rrflow-eval/tests/fixtures/storage/{manifest,result,report}.json`: the catalogue digest and dependent manifest/result hashes changed. The generator explicitly accepts synthetic evidence only. The focused golden test then passed; `cargo test -p rrflow-eval --locked` passed 56 tests; `cargo test --locked --all-targets -p rrd-kubernetes -p rrflow-edge -p rrflow-eval` passed 65 tests across 14 targets. No historical native result, acceptance threshold or capacity evidence changed.

## Independent optional-cluster failure

The same candidate's [optional rrd-cluster job](https://github.com/rrflow/rrflow-development/actions/runs/40637793176/job/122648671762) failed `independent_processes_recover_fail_over_snapshot_and_reject_corruption` because survivors did not agree on an eligible leader before the existing 10-second deadline. The retained local log is `target/attunement-cancel-ci-cluster.log`, SHA-256 `e56ec9fefe3a7195a8ad62787cdb6aa59540a5ae7e57fa351630b56b417a3b88`. This observation is added to existing POAM-023. Its cause and reproducibility are unknown; neither an unrelated passing test nor fixture regeneration fixes it. No cluster implementation was changed.

## Checklist and remaining proof

Starting correction revision: `73e7b72a98e0484fd30975cfd6676a118945cd45`. Read the original [package journal](d-02-public-attunement-cancel.md), relevant failure logs, `rrflow-eval/examples/refresh_storage_contract.rs`, fixture validation and test, `rrd-cluster/tests/openraft_process.rs` election wait, and POAM-023. No upstream adaptation, new runtime trace or performance claim. Reviewed the entire synthetic fixture diff; it changes only identities derived from the new catalogue. Created this journal and updated its generated index and execution inventory. The original implementation journal remains immutable.

- [x] Failure, source identity, exact CI jobs and bounded correction recorded before regeneration.
- [x] Focused and owning product-operations tests passed locally after correction.
- [x] Navigation, documentation, regenerated execution inventory and diff checks passed before the correction commit.
- [ ] Fresh exact-candidate and protected merge-group CI.
- [ ] Diagnose and qualify the optional-cluster failure if it recurs or otherwise blocks CI; source inventory, replan, worker cancellation acknowledgement, memory effects and C-07 native capacity/soak remain open in their owners.

No roadmap or POA&M completion status changes; a fresh green candidate is not yet established.
