# RRFlowCLD trigger observations lifted from the editor's unsynced commits

**Status:** active build-and-test absorption; no gate acceptance claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-rrflow-cld-triggers-20261110`

**Owner:** E-04 and OBJ-13 for vector memory tiers; `docs/reference/vector/memory-tiers.md`

**Starting revision:** `agent/integration-20261110` at `6b02d8c8` (PR #314).

**Current behavior.** The repository owner's editor branch `agent/project-map` carried four commits (9b60e959, 94e685c5, 16c7318a, 2e4eb9d1, 2026-11-08/09) that cannot be pushed: they contain four 176 MB `rrflow-runner.exe` copies and a 54 MB JSON under `eval/results/`, above GitHub's file limit. Mixed into them is engine work absent from main: process-local RRFlowCLD trigger observations for cold-served vectors, and a streaming index-artifact decoder in the query executor. Also mixed in is a half-applied Rust rename of estate to engram with out-of-grammar identifiers at 82 sites, and the eval inputs already landed by #308.

**Target behavior.** The engine work lands on its own, on top of the integration branch, with the rename and the binaries left out. A vector search that serves an artifact from a cold source (`cold_mapped`, `cold_owned`, or a cache-bypass cold load) records a trigger; the public search response reports `resources.cold_source_hits` and `resources.rrflowcld_triggered`; storage diagnostics report `rrflow_kv.rrflow_cld` counters whose invariants the contract validates. These are lifecycle signals, not canonical vector authority.

**Changed paths.** `crates/authority/rrd-engine/src/runtime/{coldlight.rs (new), mod.rs, vector_residency.rs}`, `engine/{core.rs, diagnostic.rs, vector/search.rs}`; `crates/transport/rrd-contract/src/{diagnostic.rs, lib.rs}` and `tests/public_contract.rs`; `crates/compute/rrd-query/{Cargo.toml, src/execute.rs, fixtures/plan-vectors.json}`; `docs/reference/vector/memory-tiers.md`, `docs/reference/query/index-catalogue.md`; regenerated contract projections; `docs/poam/rrflow-1.0-alpha/poam-040.md` gains the one paragraph #231 had that the integration branch lacked.

**Conflicts with the integration branch.** In `vector/search.rs` both hunks met #304's `generation_serves` source-identity check; the check is kept and the cold trigger is recorded beside it. The contract import line kept main's `EstateSnapshot` name. `plan-vectors.json` carried the editor's plan digest; the fixture is regenerated from the merged tree, so the digest recorded here is the merged tree's.

**Acceptance.** On Windows at this branch: `cargo test --locked -p rrd-contract`, `-p rrd-query` and `-p rrd-engine --lib`: 720 passed, 0 failed (engine library 448 passed, 13 ignored, including the new `runtime::coldlight` unit test that only cold sources trigger and every per-kind counter sums to the cold-hit count). `RRFLOW_UPDATE_GOLDENS=1` regenerated `plan-vectors.json` to the merged tree's digest `c84240d8…`, identical to the integration branch's, so the executor change does not alter any planned shape. `cargo clippy --locked -p rrd-contract -p rrd-query -p rrd-engine -p rrd-client -p rrd-server -p rrflow-cli --all-targets -- -D warnings` clean (one `map_or` simplification applied). `cargo fmt --check` clean. Projections regenerated: 83 HTTP operations, OpenAPI `fef69c50fd39…`, 7 signal projections verified; `render_navigation.py` and `check_documentation.py` OK.

**Research decision:** none; the behaviour is the owner's own 2026-11-08 work.

**Checks not run:** Linux/macOS, candidate CI, real-server client tests, a search that actually serves from a cold tier under pressure (no test exercised the trigger with a cold acquisition; the counters are covered by the contract invariants and the diagnostic snapshot path).

**Roadmap/POA&M change:** none.
