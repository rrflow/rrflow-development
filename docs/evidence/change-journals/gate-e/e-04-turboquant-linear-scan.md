# E-04: resolve TurboQuant versions in one pass and clone only admitted hits

**Status:** active immutable change-journal evidence; E-04, OBJ-13 and J-04
remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-turboquant-linear-scan`
**Owner:** evidence for the E-04 TurboQuant persistent serving plan

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: E-04 (OBJ-13), the first remediation of POAM-080
starting revision/tree and pull request: development/main after #361 (d4357885 measured); branch agent/turboquant-linear-scan-20261111; the pull request carries the change brief
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): TurboQuantSegment::search_candidates_at_with_kernel built a BTreeMap keyed by reference over every stored version per query to find each identity's latest visible version, then cloned two references for every scored code before its bounded heap decided to keep it -> decode refuses versions that do not strictly ascend by (reference, source cursor), which build and extend already produced, so each identity's versions form one run and the last visible one in it is the latest; CandidateHits::admits decides admission with the heap's own ordering before cloning. Unchanged: served versions, scores, ordering, the artifact format and every other path. Stop if any served result changed
files and symbols read in full: crates/compute/rrd-vector/src/turbo_segment.rs; sections read: exact.rs (materialize_visible_refs, visible_versions, validate_candidate_versions), contract.rs (SearchHit::compare_best_first)
implementation/evidence paths created, changed, moved, or deleted: changed crates/compute/rrd-vector/src/turbo_segment.rs (CandidateHits::admits, the linear version pass, the order check in decode, two tests); created docs/evidence/e04-vector-cost-calibration-after-turboquant-scan-20000-windows.json, docs/poam/rrflow-1.0-alpha/poam-080.md and this journal; changed the POA&M ledger and the journal index
research decision and primary-source/adaptation record: not applicable; RRFlow's own code
trace/resource/debug decision (add/preserve/not applicable, with reason): preserved; no span or budget changes
first-failure or characterization oracle and result: turbo_version_resolution_matches_the_exact_oracle compares served (reference, cursor) pairs with the exact path's visible_versions over 24 identities with one to four versions, future valid_from, retired versions, three commit cursors, five valid times, with and without a filter, and asserts it exercised fallback to an older version and hidden identities. Picking the first visible version instead of the last fails it; restored. decode_refuses_versions_out_of_reference_and_cursor_order forges swapped and repeated versions with valid digests and both are refused
acceptance commands and exact results: cargo test -p rrd-vector --locked: library 82 passed, 1 ignored, and every integration suite passed, quantization_matrix included; cargo test -p rrd-engine --lib --locked vector_index: 71 passed; cargo test -p rrflow-cli --locked --test installed_quantization: 1 passed; cargo fmt --all -- --check and cargo clippy --locked -p rrd-vector --all-targets -- -D warnings clean. Calibration, three runs as in #361: TurboQuant 4-bit falls from 812 to 267, 1,144 to 591 and 1,642 to 1,072 ns per candidate at 128, 384 and 768 dimensions, or from 0.66 to 0.22, 0.64 to 0.33 and 0.61-0.64 to 0.41-0.50 of exact; inner-product 4-bit from 1,014 to 466, 1,642 to 1,089 and 2,548 to 2,012. Exact (1,236 against 1,237 at 128) and scalar and binary are unchanged controls
surfaced failures and corrections: none in the code; the measurement method's correction is recorded with #361
checks not run and reason: the rrd-server process suites and other platforms; candidate CI runs them
remaining known errors and owner: TurboQuant's remaining cost does not fall with bit width; the exact and quantized scans keep their per-candidate work; the planner still prices codecs like exact scans. POAM-080 owns all three
full-file reread and diff review: complete
change checklist: complete
roadmap/POA&M status change, or explicit none: POAM-080 opened, High
pull request, CI run, and merge evidence: recorded on the pull request
```

## Links

- [POAM-080](../../../poam/rrflow-1.0-alpha/poam-080.md)
- [Baseline](e-04-vector-cost-calibration.md)
