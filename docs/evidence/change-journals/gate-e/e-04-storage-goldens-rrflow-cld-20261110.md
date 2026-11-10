# Storage goldens regenerated for the RRFlowCLD diagnostic fields

**Status:** active repair of a red main; no gate acceptance claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-storage-goldens-rrflow-cld-20261110`

**Owner:** E-04 vector memory tiers; `eval/schemas` goldens owned by `crates/evaluation/rrflow-eval/tests/storage_contract.rs`

**Starting revision:** main `10cd30e3` (after #317, #318, #319).

**Current behavior.** #317 added `rrflow_cld` to `DiagnosticRrflowKvSnapshot` and two fields to `VectorSearchResourceEvidence`. Its verification ran the contract, engine and query suites but not `rrflow-eval`, whose `generated_schemas_and_synthetic_receipts_match_goldens` pins the generated storage schemas. Main's `synthetic storage contract preflight` and `engine suite (product-operations)` jobs failed on the first candidate after the merge (#321, run 42905595015), and the failure reproduces on a clean checkout of main: `eval/schemas/storage-manifest-v1.json` differs by the new fields; every other `rrflow-eval` target passes (261 passed, 1 failed).

**Target behavior.** The goldens match the contract again; nothing else changes.

**Changed paths.** The regenerated files under `eval/schemas` (listed in the commit), this record.

**Acceptance.** On Windows at this branch off main `10cd30e3`, isolated target directory: `RRFLOW_UPDATE_STORAGE_GOLDENS=1 cargo test --locked -p rrflow-eval --test storage_contract` regenerated five files under `eval/schemas` (285 added lines, all the new diagnostic and evidence fields); `cargo test --locked -p rrflow-eval --test storage_contract`: 11 passed, 0 failed; before the regeneration `cargo test --locked -p rrflow-eval --all-targets --no-fail-fast` on main: 261 passed, 1 failed (this golden); `render_navigation.py` and `check_documentation.py` OK.

**Research decision:** none. **Checks not run:** Linux/macOS, candidate CI. **Roadmap/POA&M change:** none. **Lesson recorded:** a contract change must run `cargo test -p rrflow-eval --test storage_contract` before it lands; the #317 journal's acceptance list omitted it.
