# Worker composition storage catalogue repair

**Status:** active local repair evidence; candidate and protected merge checks pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-storage-catalogue`
**Owner:** G-01b/G-02a worker composition, PR 229; POAM-009 remains open

## Change brief and observed failure

Starting revision: `2949e8f8e08533f924c743c96552cde44f1345f3`.
Candidate run `41651622663` failed the service-protocol and product-operations
suites and installed capacity smoke on all three native platforms. The exact
logs reported `public catalogue identity differs from validator`. The worker
and observation API composition changed the canonical catalogue while the
existing synthetic storage contract fixture still named its predecessor.
The repair brief was added to PR 229 before regeneration.

Read root instructions, README and change-authoring/CI owners; inspected
`storage/validation.rs`, the sustained fixture constructor and CLI provenance
assertion, all three fixture diffs, and the complete existing
`refresh_storage_contract.rs` generator. That generator explicitly refuses
native evidence. The unchanged strict validator remains the failure oracle.

## Implementation and validation

Ran `cargo run --locked -j 1 -p rrflow-eval --example refresh_storage_contract`.
Only `manifest.json`, `result.json` and `report.json` under
`crates/evaluation/rrflow-eval/tests/fixtures/storage/` changed: four derived
digest lines. No production validator, recorded observation, model dataset,
historical native receipt or schema changed.

- `cargo test --locked -j 1 -p rrflow-eval`: 57 passed, zero failed.
- `cargo test --locked -j 1 -p rrflow-cli --test installed_reopen
  failed_campaign_preserves_closed_cause_and_readback_provenance`: one passed.
- `cargo test --locked -j 1 -p rrflow-cli --test installed_reopen --
  sustained::installed_capacity_workload_smoke --exact --ignored --nocapture`:
  one passed in 23.70 seconds on native Windows, debug profile.

The installed smoke exercises real storage effects and readback. It is not the
100-cycle release-profile suite or capacity qualification. Cross-platform
candidate CI must run on the corrected revision. No check was weakened and no
roadmap item closes. External research and new runtime instrumentation are
unnecessary for mechanical regeneration; retained CI/local logs identify the
failure and correction. Generate/check navigation and inspect whitespace before
commit. Continue the installed developer integration after candidate acceptance.
