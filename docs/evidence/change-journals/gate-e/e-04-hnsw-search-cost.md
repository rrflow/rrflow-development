# A cheap HNSW cost estimate

**Status:** active immutable change-journal evidence; E-04 and OBJ-13 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-hnsw-search-cost`
**Owner:** evidence that estimating an HNSW search's cost no longer scans the graph

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: OBJ-13 and E-04, and OBJ-10 through the lab's measurement. Owner direction of 2026-11-04: "how do we get this moving forward we are dog shit".
starting revision/tree and pull request: development/main at 12039148 merged with agent/hnsw-connected-graph, because its tests build graphs only a connected build can produce; pull request #282, based on #280.
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the description of pull request #282. estimated_search_cost visited every identity with a tree lookup on every query. It now reads a fixed sample of at most 512 evenly spaced identities, derived when an artifact is read and never stored, and counts exactly only when that sample finds nothing eligible in a larger corpus. The artifact format and what a search returns are unchanged. is_visible during traversal keeps its lookup.
files and symbols read in full: crates/compute/rrd-vector/src/hnsw.rs estimated_search_cost, is_visible, from_parts and its crossover test; crates/compute/rrd-vector/src/runtime.rs's use of the estimate. The profile on clyffy-01.
implementation/evidence paths created, changed, moved, or deleted: changed crates/compute/rrd-vector/src/hnsw.rs (COST_SAMPLE_IDENTITIES, HnswIndex.cost_sample, from_parts, estimated_search_cost, eligible_identities, latest_version, admits, is_visible, tests). Created this journal and generated the gate-e index.
research decision and primary-source/adaptation record: none needed. Nothing is adapted from another project.
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable to tracing. A test-only counter records how many identities one estimate visits. The sample costs at most 512 short version lists per loaded artifact, and a read still checks its digest.
first-failure or characterization oracle and result: a perf profile on clyffy-01, 20,000 points, ef-32, one client: 44.20% in prepare_search_at, 43.91% of that in estimated_search_cost, against 3.28% in score_dense. The first tests on a branch from main failed for the wrong reason, the connectivity error #280 fixes, so I discarded that result and stacked on #280. 80d6f99d then committed the tests, and an_estimate_does_not_visit_the_whole_corpus failed with "one estimate visited 4000 identities". Four others passed on the exact count and hold what the sample must keep.
acceptance commands and exact results: at fc390903 on Windows: cargo test --locked -p rrd-vector --no-fail-fast, 91 passed, 0 failed, 1 ignored. cargo test --locked -p rrd-engine --lib vector, 56 passed. cargo clippy --locked -p rrd-vector --all-targets with -D warnings, clean. On clyffy-01, vector-small, 20,000 points, comparing measurement tree 515b60b3 with cd171544, which adds only this package. rrflow-hnsw at ef-32, recall 0.9782: 49.4 to 71.9 queries per second with one client and p50 18.68 to 12.11 ms, and 166.9 to 190.5 with eight. Filtered at ef-16, recall 0.9998: 57.7 to 124.5 with one client and 168.2 to 192.7 with eight. These are one run each on one host, with the GPU idle in every sample, and no repeat was made.
surfaced failures and corrections: the wrong-reason failure above. An estimate from a sample can differ from the exact count, so a planner near a crossover could choose another path. On a 4,000-point corpus with one in ten matching, the estimate stayed within a factor of two, as a test holds. Results stay exact-reranked on either path. In the measurement tree this merged with #278's derived collections field in the same place, and both were kept.
checks not run and reason: candidate CI, because it skips drafts. The full rrd-engine suite on this commit, since only hnsw.rs changed and #280's run covers the rest; its vector tests ran. Linux for this crate's suite, beyond the real run. macOS.
remaining known errors and owner: a filter matching nothing in the sample in a large corpus still counts exactly, so the cost of a rare filter is unchanged, and that query is the case where a planner most needs it. The estimate is unchanged when a read's cursor equals the generation's. #278 and this package conflict textually in hnsw.rs, because both add a derived field in one place, and whichever lands second keeps both. This package is based on #280.
full-file reread and diff review: complete for the changed code, the tests and this record.
change checklist: complete. Brief before implementation; failing test committed first; no format change; no public change; no parallel path; no runtime fetch; no provider state.
roadmap/POA&M status change, or explicit none: none.
pull request, CI run, and merge evidence: recorded on pull request #282.
```

## Links

- [Pull request #282](https://github.com/rrflow/rrflow-development/pull/282)
- [Connected graphs](e-04-connected-hnsw-graphs.md)
- [Gate E](../../../roadmap/rrflow-1.0/gate-e.md)
