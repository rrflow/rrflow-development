# RRFlowCLD trigger observations lifted from the editor's unsynced commits

**Status:** active build-and-test absorption; no gate acceptance claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-rrflow-cld-triggers-20261110`

**Owner:** E-04 and OBJ-13 for vector memory tiers; `docs/reference/vector/memory-tiers.md`

**Starting revision:** main `a8551dc9` (after PR #314). First opened as #315 on the integration branch; split here.

**Current behavior.** The repository owner's editor branch `agent/project-map` carried four commits (9b60e959, 94e685c5, 16c7318a, 2e4eb9d1, 2026-11-08/09) that cannot be pushed: they contain four 176 MB `rrflow-runner.exe` copies and a 54 MB JSON under `eval/results/`, above GitHub's file limit. Mixed into them is engine work absent from main: process-local RRFlowCLD trigger observations for cold-served vectors, and a streaming index-artifact decoder in the query executor. Also mixed in is a half-applied Rust rename of estate to engram with out-of-grammar identifiers at 82 sites, and the eval inputs already landed by #308.

**Target behavior.** The engine work lands on its own, on top of the integration branch, with the rename and the binaries left out. A vector search that serves an artifact from a cold source (`cold_mapped`, `cold_owned`, or a cache-bypass cold load) records a trigger; the public search response reports `resources.cold_source_hits` and `resources.rrflowcld_triggered`; storage diagnostics report `rrflow_kv.rrflow_cld` counters whose invariants the contract validates. These are lifecycle signals, not canonical vector authority.

**Changed paths.** `crates/authority/rrd-engine/src/runtime/{coldlight.rs (new), mod.rs, vector_residency.rs}`, `engine/{core.rs, diagnostic.rs, vector/search.rs}`; `crates/transport/rrd-contract/src/{diagnostic.rs, lib.rs}` and `tests/public_contract.rs`; `docs/reference/vector/memory-tiers.md`; regenerated contract projections. The streaming index-artifact decoder in `rrd-query` and the POAM-040 paragraph from #231 land in the sibling package `agent/index-decoder-20261110`, so each stays under the 400-line candidate limit.

**Conflicts with the integration branch.** In `vector/search.rs` both hunks met #304's `generation_serves` source-identity check; the check is kept and the cold trigger is recorded beside it. The contract import line kept main's `EstateSnapshot` name. `plan-vectors.json` carried the editor's plan digest; the fixture is regenerated from the merged tree, so the digest recorded here is the merged tree's.

**Acceptance.** On Windows at this package off main `a8551dc9`: `cargo test --locked -p rrd-contract` and `-p rrd-engine --lib` pass (engine library 448 passed, 13 ignored, including the `runtime::coldlight` unit test); `cargo clippy --locked -p rrd-contract -p rrd-engine --all-targets -- -D warnings` and `cargo fmt --check` clean; `check_generated_surfaces.py` verified (83 HTTP operations, 7 signal projections); `render_navigation.py` and `check_documentation.py` OK.

**Research decision:** none; the behaviour is the owner's own 2026-11-08 work.

**Checks not run:** Linux/macOS, candidate CI, real-server client tests, a search that actually serves from a cold tier under pressure (no test exercised the trigger with a cold acquisition; the counters are covered by the contract invariants and the diagnostic snapshot path).

**Roadmap/POA&M change:** none.
