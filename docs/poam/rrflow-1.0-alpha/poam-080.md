# POAM-080 — vector scans pay fixed per-candidate work and the planner prices codecs like exact scans

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-080`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

A code read and a release-mode measurement at `99095df7` on 2026-11-11, recorded in [the baseline](../../evidence/e04-vector-cost-calibration-before-20000-windows.json), found three related faults:

- **Every vector scan pays fixed work per candidate.** At 20,000 cosine vectors an exact scan costs 1,237 to 2,675 ns per candidate at 128 to 768 dimensions, at least ten times the arithmetic of one score. TurboQuant at 4, 2 and 1 bits costs the same 0.61 to 0.67 of exact, although its codes differ in size by four times.
  - The TurboQuant scan built a `BTreeMap` keyed by reference over every stored version for each query, and cloned two references for every scored code even when its bounded heap discarded it (`crates/compute/rrd-vector/src/turbo_segment.rs`).
  - The exact path materializes visible versions, clones every hit's references and sorts every hit to keep the top k (`exact.rs:85-105`).
- **The planner prices a TurboQuant or quantized generation at its candidate count**, the unit of an exact scan (`runtime.rs:450-455`), then adds any overlay, and breaks ties by path id (`plan.rs:175-179`). A request that allows approximation therefore never chooses TurboQuant by cost: at 2,048 vectors of 64 dimensions with a 4-bit generation and four newer writes, exact scan is selected at cost 2,048.
- **TurboQuant's remaining scan cost does not fall with bit width**, which points at the scoring kernel rather than the codes.

## Impact

The engine cannot choose its compressed paths when they would serve a query faster, so the dynamic selection OBJ-13 calls for does not happen, and any comparison of RRFlow's vector speed measures overhead rather than its algorithms.

## Owning gates

E-04 owns TurboQuant persistent serving and its planner selection.

Roadmap owner: [Gate E](../../roadmap/rrflow-1.0/gate-e.md).

## Remediation in progress

- The TurboQuant scan resolves each identity's latest version in one linear pass over versions that `decode` now requires in reference and cursor order, and clones only hits the heap admits. Measured in [the after record](../../evidence/e04-vector-cost-calibration-after-turboquant-scan-20000-windows.json), 4-bit costs fall from 812 to 267, 1,144 to 591 and 1,642 to 1,072 ns per candidate at 128, 384 and 768 dimensions.
- The planner prices TurboQuant and quantized generations from measured per-candidate costs (`rrd_vector::VectorCostModel`), so a request that allows approximation takes TurboQuant at 2,048 vectors of 64 dimensions (cost 507 against 2,048) and keeps the exact scan at 40 (45 against 40). HNSW's navigation multiplier is still unmeasured, and the engine does not yet take the model from installed configuration (POAM-077).

## Closure evidence

Closed when all of the following hold:
- The planner prices every path from measured per-candidate costs that the calibration example reproduces, and the probe `allow_approximate_prefers_turboquant_when_scanning_codes_is_cheaper` passes.
- The exact and quantized scans no longer pay per-candidate work beyond scoring, as measured by the same example.
- TurboQuant's scan cost falls with bit width, or a measurement shows why it cannot.
