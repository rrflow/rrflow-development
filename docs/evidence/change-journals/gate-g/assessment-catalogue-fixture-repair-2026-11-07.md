# Assessment catalogue: synthetic storage fixture repair

**Status:** active candidate repair; required candidate and merge-group CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/assessment-catalogue-fixture-repair-20261107`
**Owner:** OBJ-06/OBJ-14, G-01/G-02, POAM-009; [PR #292](https://github.com/rrflow/rrflow-development/pull/292)

## Brief and source boundaries

Starting candidate `900e57d522aa0f8e0ffe9673fd555e0da58075cd`, based on main `b9aaa9abce3c25108a53cc43c70262947dda0f62`. The PR description was amended before regeneration. The candidate adds public operations, changing the catalogue identity. CI run [42380307212](https://github.com/rrflow/rrflow-development/actions/runs/42380307212) failed product operations, service protocol and the Linux installed reopen job against the old synthetic storage fixture. In the reopen job, the smoke test stopped at `public catalogue identity differs from validator` before a 100-cycle workload could be established. These failures are retained; no capacity or soak success is implied.

Read the complete fixture generator `crates/evaluation/rrflow-eval/examples/refresh_storage_contract.rs`; the fixture setup and failed assertion in `crates/evaluation/rrflow-eval/tests/storage_contract.rs`; the catalogue validation in `crates/evaluation/rrflow-eval/src/storage/validation.rs`; the CI failure excerpts; the existing PR brief and package journal; and the applicable change-authoring procedure. The generator refuses a non-synthetic manifest, recalculates the candidate catalogue and configuration identities, validates the resulting fixture and regenerates derived reports and schemas. The native evidence and validators remain unchanged. No external algorithm or research is involved; the relevant diagnostic is the exact failed CI test and catalogue digest, not a new runtime trace.

## Changes and verification

`cargo run --locked -p rrflow-eval --example refresh_storage_contract` succeeded offline on Windows with dev/test debug symbols disabled. Only `crates/evaluation/rrflow-eval/tests/fixtures/storage/{manifest,result,report}.json` changed: the synthetic public catalogue digest changed from `545a4847892c31393fcb5e3e5b45e6150a1835d25ebeb90b91d88bb0b82f1a17` to `e6e28b6f14fc38659c5b9a0b53494f09922a3c619ae1ae4d46396f1e3722e55a`; dependent manifest and result digests were recomputed. Schema outputs and historical/native evidence did not change.

`cargo test --locked -p rrflow-eval` passed on Windows, including 43 library tests, 3 provider compatibility tests and 11 storage-contract tests. With `CARGO_NET_OFFLINE=true` and dev/test debug symbols disabled, `cargo test --locked -p rrflow-cli --test installed_reopen sustained::failed_campaign_preserves_closed_cause_and_readback_provenance -- --exact` passed one test. The previously stopped installed boundary was exercised with `cargo test --locked -p rrflow-cli --test installed_reopen sustained::installed_capacity_workload_smoke -- --exact --ignored --nocapture`: it passed on Windows in 19.24 seconds, with a 20-attempt held segment, reader retention/release and interrupted-engine reopen/readback. Its output explicitly states `capacity_qualification=false`; it is neither the 100-cycle matrix nor soak evidence.

`python scripts/knowledge/render_navigation.py` and `--check` passed; one index was updated. `python scripts/ci/check_documentation.py` passed. A pre-regeneration `python scripts/ci/build_execution_inventory.py --check` reported drift in its ignored Git-derived inventory, which needs regeneration against the committed candidate. Final candidate and protected CI remain pending. No objective, roadmap gate, POA&M status or release claim changes.
