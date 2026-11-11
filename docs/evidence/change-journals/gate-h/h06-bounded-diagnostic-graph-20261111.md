# H-06: a bounded diagnostic graph a real project's operator client can read

**Status:** active implementation evidence

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h06-bounded-diagnostic-graph-20261111`

**Owner:** the [Gate H owner](../../../roadmap/rrflow-1.0/gate-h.md) retains H-06; this package completes nothing on its own

**Starting revision:** `main` at `b28c57195df9950b0d35c64bb393c57b7dc27c42`.

**Current behavior.** `diagnostics/read` returns every graph record and relation, and a graph difference that, compared against cursor 0, repeats all of them. Driving the real Connectome desktop app through an install and attunement of this repository (2,779 tracked files, engine cursor 6,227) left its Graph and Schema tabs unable to load. The engine's Rust client refused the response: "RRD response exceeded four MiB". Measured on that estate with the same request Connectome sends, the response was 13,068,661 bytes: 6,447,891 for the graph (3,114 records, 3,111 relations), 6,447,948 for the difference, 161,552 for product capabilities, and under 4 KB for everything else, including the one-entry change and audit pages.

**Target behavior.** A request may set `graph_max_records`. The engine returns at most that many records, the most linked first with ties broken by reference, and only relations between returned records. The graph reports `records_total` and `relations_total`, and the difference keeps only entries about returned records and reports `omitted_entries`. Without the field nothing changes except that the totals are reported.

**Changed paths.** `crates/transport/rrd-contract/src/diagnostic.rs` (the request field, its validation, `MAX_DIAGNOSTIC_GRAPH_RECORDS`, and the reply's totals), `src/lib.rs` (export), `crates/authority/rrd-engine/src/engine/diagnostic.rs` (`bounded_records` and filtering), the request literals in four test files, the generated TypeScript OpenAPI types, and this record.

**Failure oracles.** `bounded_graph_tests::a_bounded_read_returns_the_most_linked_records_and_counts_the_rest` and `an_unbounded_or_roomy_read_leaves_nothing_out` in the engine; `diagnostic_read_contract_is_bounded_strict_and_separately_authorized` now refuses a bound of 0 and one above the maximum. They were written with the change, so their red was a compile failure.

**Acceptance.** Listed in the pull request with the exact commands and results.

**Not established.** Selection by link count is a default, not a measure of importance. Product capabilities, at 161,552 bytes, remain in every read. The real desktop drive with a bounded request is recorded by the Connectome repin.

**Research decision:** none. **Roadmap/POA&M change:** none.
