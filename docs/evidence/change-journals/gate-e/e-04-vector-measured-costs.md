# E-04: price codec paths from measured per-candidate costs

**Status:** active immutable change-journal evidence; E-04, OBJ-13 and J-04
remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-vector-measured-costs`
**Owner:** evidence for the E-04 TurboQuant persistent serving plan

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: E-04 (OBJ-13), the second remediation of POAM-080: dynamic selection of TurboQuant by cost
starting revision/tree and pull request: development/main after #362; branch agent/vector-measured-costs-20261111; the probe was written first, on 2026-11-11, as commit 112060f8, and failed with "selected ExactScan at cost 2048; rejected paths: []"
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): VectorRuntime::prepare_search_at priced a TurboQuant or quantized generation at candidate_versions, the unit of an exact scan, so AllowApproximate never chose one by cost and ties fell to path id -> rrd_vector::VectorCostModel holds each codec's per-candidate cost in thousandths of an exact score as a curve over dimensions, defaulting to the worst run median of the #362 calibration; a codec's cost is ceil(n x c / 1000) + exact_rerank + overlay; VectorRuntime::set_cost_model validates and replaces it; SearchMode::exact_rerank names the rerank. Unchanged: HNSW and exact-segment estimates, Exact and RequireApproximate semantics, served results. Stop if a codec were chosen where it serves different results
files and symbols read in full: crates/compute/rrd-vector/src/runtime.rs (VectorRuntime, prepare_search_at), plan.rs (VectorPlanner::plan_at); sections read: catalog.rs (candidate_path), quantized_segment.rs (QuantizedDescriptor, QuantizationMethod), turbo_segment.rs (TurboQuantDescriptor), contract.rs (SearchMode), the engine's catalogue_reads.rs test
implementation/evidence paths created, changed, moved, or deleted: created crates/compute/rrd-vector/src/cost.rs and this journal; changed rrd-vector lib.rs (exports), contract.rs (SearchMode::exact_rerank), runtime.rs (the cost model, the codec pricing, the probe and its opposite case); crates/authority/rrd-engine/src/engine/tests/vector_index/catalogue_reads.rs; docs/reference/vector/search.md; POAM-080's remediation
research decision and primary-source/adaptation record: not applicable; the costs are RRFlow's own measurements
trace/resource/debug decision (add/preserve/not applicable, with reason): preserved; the plan already records each path's estimated cost
first-failure or characterization oracle and result: allow_approximate_prefers_turboquant_when_scanning_codes_is_cheaper selects TurboQuant at cost 471 + 32 + 4 over 2,048 exact; allow_approximate_keeps_exact_when_reranking_costs_as_much keeps exact at 40 against 45. retained_quantization_state_survives_activation_retirement_and_reopen asserted that AllowApproximate on its four-vector fixture took the scalar path, which only the id tie-break produced; it now checks that AllowApproximate is served exactly there, that RequireApproximate serves the scalar generation at the active stamp before and after reopen with the same plan, and, newly, that RequireApproximate is refused at the ready and retired stamps
acceptance commands and exact results: cargo test -p rrd-vector --locked: library 86 passed, 1 ignored, all integration suites pass; cargo test -p rrd-engine --lib --locked: 473 passed, 13 ignored; cargo test -p rrd-engine --tests --locked passes; cargo test -p rrflow-cli --locked and cargo test -p rrd-server --locked pass after cargo build --locked --workspace --bins; cargo test -p rrd-engine --locked --test workspace_architecture: 33 passed; cargo fmt --all -- --check and cargo clippy --locked -p rrd-vector -p rrd-engine --all-targets -- -D warnings clean
surfaced failures and corrections: the catalogue test above
checks not run and reason: other platforms; candidate CI runs them
remaining known errors and owner: the engine builds its runtime with the default model and no installed setting replaces it (POAM-077); HNSW's navigation multiplier of 4 is not measured; the exact and quantized scans keep their per-candidate work (POAM-080)
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none; POAM-080 stays open
pull request, CI run, and merge evidence: recorded on the pull request
```

## Links

- [POAM-080](../../../poam/rrflow-1.0-alpha/poam-080.md)
- [The scan change](e-04-turboquant-linear-scan.md)
