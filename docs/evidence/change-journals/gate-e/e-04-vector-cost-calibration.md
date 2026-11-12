# E-04: measure what each vector codec's scan costs against an exact scan

**Status:** active immutable change-journal evidence; E-04, OBJ-13 and J-04
remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-vector-cost-calibration`
**Owner:** evidence for the E-04 TurboQuant persistent serving plan

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: E-04 (OBJ-13), the measurement that the TurboQuant scan and planner cost packages after it are judged by
starting revision/tree and pull request: development/main 99095df7; branch agent/vector-cost-calibration-20261111; the pull request carries the change brief
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the planner prices a TurboQuant or quantized generation at its candidate count, the unit of an exact scan (crates/compute/rrd-vector/src/runtime.rs:450-455), and no measurement says what a codec scan costs relative to an exact one (the only codec evidence, g04-w04-quantization-local-512x64.json, is a 512 x 64 debug build without an exact baseline) -> a release-mode example, crates/compute/rrd-vector/examples/vector_cost_calibration.rs, times an exact scan and every codec's scan on the same vectors and queries, interleaved round by round, and reports each codec's cost per candidate relative to exact; docs/evidence/e04-vector-cost-calibration-before-20000-windows.json records main's baseline. Unchanged: all library code. Stop if the host's noise made the ratios unrepeatable
files and symbols read in full: crates/compute/rrd-vector/src/turbo_segment.rs (search_candidates_at_with_kernel, CandidateHits, decode), tests/quantization_matrix.rs; sections read: runtime.rs:420-500 (path costing), plan.rs:120-200 (VectorPlanner::plan_at), hnsw.rs:15-30 and 510-550 (estimated_search_cost), exact.rs:85-215 (search_exact_ref, materialize_visible_refs)
implementation/evidence paths created, changed, moved, or deleted: created the example, the evidence file and this journal
research decision and primary-source/adaptation record: not applicable; a measurement of RRFlow's own code
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable; an example binary outside the library
first-failure or characterization oracle and result: a first method took each side's median of five rounds separately; on this shared host its three runs disagreed by up to a factor of two and showed codec scans slower than exact (relative costs up to 2.06), so it was replaced by interleaved rounds and a median of per-round ratios, whose three runs agree within 0.03 on every ratio
acceptance commands and exact results: cargo run --release --locked -p rrd-vector --example vector_cost_calibration -- 20000 128,384,768, three runs. At 20,000 cosine vectors the fastest exact scan costs 1,237, 1,856 and 2,675 ns per candidate at 128, 384 and 768 dimensions, at least ten times what the arithmetic of one cosine score needs. Relative to exact, TurboQuant 4-bit costs 0.66, 0.63 to 0.64 and 0.61 to 0.64; 2-bit and 1-bit cost the same as 4-bit at every width although their codes are half and a quarter the size; scalar costs 0.75, 0.58 and 0.55 to 0.58; binary 0.74 to 0.78, 0.60 to 0.62 and 0.49 to 0.52. Fixed per-candidate work, not code size, dominates every path. cargo fmt --all -- --check and cargo clippy --locked -p rrd-vector --all-targets -- -D warnings are clean
surfaced failures and corrections: the first measurement method, above
checks not run and reason: no other host or architecture; the GB10 hosts are not used for this. No exact-path or HNSW change is measured here
remaining known errors and owner: the per-candidate overhead in the TurboQuant scan (a per-query BTreeMap over every stored version and two reference clones per scored code), in the exact path (version materialization, clones and a full sort of every hit) and in the planner's costs; the packages after this one own them
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: none
pull request, CI run, and merge evidence: recorded on the pull request
```

## Links

- [E-04 TurboQuant persistent serving plan](../../../roadmap/e04-turboquant-persistent-serving-engineering-plan.md)
- [Baseline measurement](../../e04-vector-cost-calibration-before-20000-windows.json)
