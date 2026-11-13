# ArangoDB and SurrealDB planner probe, 2026-11-13

**Status:** active evidence; one machine, one corpus, captured plans; not a speed benchmark
**Coordinate:** `rrflow://rrflow-instance/data/evidence/planner-probe-20261113`
**Owner:** what ArangoDB 3.12.13 and SurrealDB 3.3.0 actually do with the same data and questions, as input to RRFlow's trigger conditions

## What was run

- **Corpus:** [`corpus.py`](corpus.py) read this repository at `297bd98f`, giving:
  - 909 files and 7,217 chunks of 60 lines;
  - 17,021 symbol definitions (regex, not tree-sitter);
  - 17,021 defines edges, 146,624 references edges and 72 crate-dependency edges;
  - a 64-dimensional feature-hash vector per chunk.

  The vectors exist to observe the planners, not to judge answer quality.
- **Containers:** both systems ran in Docker on the Windows 11 laptop with 4 CPUs and 4 GB each, beside the owner's other containers:
  - `arangodb:3.12.13`, started with `--vector-index true`;
  - `surrealdb/surrealdb:v3.3.0`, on RocksDB.
- **Same indexes in both:** symbol name, symbol path, chunk (crate, path), file bytes, file crate, BM25 over chunk text, and an approximate vector index (Arango IVF with 85 lists; Surreal HNSW).
- **Capture:**
  - [`arango_probe.py`](arango_probe.py) records each question's `/_api/explain` plan and a `profile: 2` execution: rules applied, nodes, estimated cost and items, and index versus full scans.
  - [`surreal_probe.py`](surreal_probe.py) records `EXPLAIN FULL` with per-operator metrics.
  - Raw output: [`arango_results.json`](arango_results.json), [`surreal_results.json`](surreal_results.json), and Arango's full optimizer rule list, [`arango_rules.json`](arango_rules.json) (64 rules).

## What each system did

| Question | ArangoDB plan, observed | SurrealDB plan, observed |
|---|---|---|
| Symbol `control_attunement` by name | rule `use-indexes` → `IndexNode[symbol_name]`; 1 index read, 0.39 ms server | `IndexScan symbol_name = 'control_attunement'`; 0.44 ms server |
| Filter on fields with no index (`kind`, `line`) | `EnumerateCollectionNode`; 17,021 scanned, 17,008 discarded | `TableScan` with `pre_decode_filter`; 2.6 ms |
| Natural-language question "where is the LSM compaction policy" | rule `arangosearch-constrained-sort`; BM25 over the inverted index scanned 3,487 chunks for the top 5. Ranks `rrd-engine/tests/workspace_architecture.rs` first and misses `rrd-lsm/src/database.rs`, the same miss as RRFlow | `FullTextScan` returns **0 rows**: Surreal's `@1@` requires every term, including "where", "is" and "the" |
| "how does the vector planner choose TurboQuant or the exact scan" | `rrd-vector/src/runtime.rs` first, then `hnsw.rs` and `cost.rs`, the same first file as RRFlow | 0 rows, for the same reason |
| BM25 "compaction policy" filtered to crate `rrd-lsm` | the filter is pushed into the search; 144 index reads; LSM tests first | `BitmapFullTextScan` intersected with `IndexScan chunk_crate_path`; **`rrd-lsm/src/database.rs` first** (score 8.54) |
| Who references `control_attunement` (1 hop) | `IndexNode` then `TraversalNode`; 17 index reads | `IndexScan` then an embedded `<-references` edge operator |
| 2 hops inbound | `TraversalNode` breadth-first with global uniqueness; 37 index reads, 10 cursors | `IndexScan` then nested edge operators; 0.69 ms |
| Shortest crate path `rrflow-cli` → `rrd-lsm` | `ShortestPathNode`; answer `rrflow-cli → rrd-store → rrd-lsm` | not run (syntax not probed) |
| Vector kNN top 5 | rule `use-vector-index` → `EnumerateNearVectorNode` | `KnnScan` (HNSW, ef 40); distances came back null, so this row is **inconclusive** |
| Vector kNN filtered to crate `rrd-lsm` | `EnumerateNearVectorNode` with the filter folded in | the predicate is pushed **into** `KnnScan`; 226 records fetched; `database.rs` at distance 0 |
| `bytes > X` for X from 500,000 down to 0 (0 to 909 rows) | the `file_bytes` index is used at **every** selectivity, including 100% | `IndexScan file_bytes` at every selectivity |
| Equality on crate, then sort by bytes, limit 1 or 50 | index, then `SortNode`, then `LimitNode` | index, then `SortTopKByKey` |

## What this shows

1. **Neither system decides what to search.** Both execute the query they are given. A natural-language question succeeds or fails depending on how it is phrased: Arango treats it as OR-ranked terms, Surreal as AND. Turning a question into the right query belongs to whoever writes the query, which in the Claude + Jev and Codex + Clef-flash stacks is the model. That is the slot RRFlow's planner, SWAG and LFG are meant to fill.
2. **Path choice is rule-first.** An index whose leading field matches the predicate is used. Arango names each rule it applied (`use-indexes`, `remove-filter-covered-by-index`, `use-vector-index`, `arangosearch-constrained-sort`, `optimize-traversals`) and attaches cost estimates. Neither system switched to a full scan when selectivity was low, at 909 rows.
3. **Filters are pushed into specialised indexes:**
   - into the text index: Arango's search condition, Surreal's bitmap intersection;
   - into the vector index: Surreal's filtered `KnnScan`.

   That pushdown is what put the right file first for "compaction policy" in rrd-lsm.
4. **Graph cost is edges touched:** 17 index reads for 1 hop, 37 for 2, starting from an index seek on the start vertex.

## Limits

- One laptop shared with other containers. Wall times include HTTP and are not a speed comparison.
- Regex symbols and feature-hash vectors. The planners were observed; answer quality was not measured.
- The workload did not exercise cost-based switch-over thresholds; 909 files is too small to trigger them.
- No LLM was involved. Nothing here measures how a model turns a question into a query.
