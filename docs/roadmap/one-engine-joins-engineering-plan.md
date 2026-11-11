# One engine: joining rrflowDB, rrflowQL, rrflowKV/LSM and rrflowCLD

**Status:** active proposed engineering plan; no package below is implemented
**Coordinate:** `rrflow://rrflow-instance/data/work-package/one-engine-joins`
**Owner:** supporting plan for P5 and P6 of the [one-engine execution plan](../architecture/one-engine-execution-plan.md#5-executable-packages-and-acceptance), under E-03/E-04 ([Gate E](rrflow-1.0/gate-e.md)), F-06 ([Gate F](rrflow-1.0/gate-f.md)) and J-04 ([Gate J](rrflow-1.0/gate-j.md))

This plan cannot change the order or status set by the [README](../../README.md), the [alpha objective](../objectives/rrflow-1.0-alpha.md), the [roadmap](rrflow-1.0.md) or the [POA&M](../poam/rrflow-1.0-alpha.md). It maps the owner's design onto the code as it stands and orders the work that makes the parts one engine. Every code claim was read at `development/main` `6afb77a1` on 2026-11-11.

## 1. The owner's design

RRFlow is "one reasoning-data engine" (README), and rrflowDB, rrflowKV, rrflowQL, graph, lexical and vector "remain parts of that one system". The owner's own statements about it:

- **One query, the way SurrealDB or ArangoDB works, with Qdrant-grade vectors.** "It works like Surreal or Arango does when typing reasoning, Qdrant with a system that triggers all of them properly, hitting hot first and if necessary to cold" (2026-11-11). Gate J records the same: capabilities are "triggered properly, so that different reasoning can be chosen dynamically".
- **The engine owns the access paths, chosen by the reasoning need, not by cost.** The objective says the engine "turns authorized source revisions into retrievable records, source spans, semantic relationships and lexical/vector access paths", and that "a prompt retrieves current permitted evidence". Cost is a budget. It is not the goal.
- **rrflowLSM is the hot tier: an AI-optimized engine evolved from Fjall** (owner, 2026-11-11). **rrflowCLD is the cold tier: a custom, AI-optimized format that breaks from Parquet**, and becomes canonical once compaction moves data into it, so the LSM can reclaim space (owner decisions, 2026-11-11; F-06).
- **Knowledge paths default to cold**, but the engine can still reach them. Everything is tunable and evolvable, and nothing is hardcoded.
- **TurboQuant in all four parts:** persistent search, the inner-product variant, the model KV cache, and sync transfer ([quantization lifecycle](../reference/vector/quantization-lifecycle.md), OBJ-13).
- **Measured and tuned as one system** against SurrealDB, ArangoDB and Qdrant running beside it ([comparison lab](../../eval/lab/README.md)), on real hosts that also run other projects.

## 2. What exists and what is not joined

The parts below exist and work. What is missing is the joins between them.

| Part | Exists | Not joined |
|---|---|---|
| rrflowQL | Parser, binder, rule-based path choice, DataFusion execution over one read stamp. Sources: record, relation, event, series, geo, traverse, claim (`rrd-query/src/syntax.rs:163-191`). `MATCH` (BM25), one equi-join, `AT VALID`/`KNOWN` cursors | No vector operator (`syntax.rs:288`). No `ORDER BY`, `GROUP BY`, aggregates or `OR` (`syntax.rs:926-945`). Path choice is rules, not a planner (`plan.rs:400-692`). `EXPLAIN CONTRACT` still executes (POAM-044) |
| Indexes | Every commit writes physical scalar index entries and BM25 source deltas (`rrd-store/src/access/index.rs:343, 570`) | rrflowQL never reads them. Its "scalar index" is a full JSON copy of every row, built only on a client request, and unusable after any write (`rrd-query/src/index.rs:594-621`, `plan.rs:466-469`). A per-file symbol lookup on this repository examined 99,936 keys |
| Vector | Exact, HNSW and TurboQuant paths, a planner (`rrd-vector/src/plan.rs:111-210`), filter-aware rejection, an exact rerank | Not in rrflowQL. Default mode `Exact` excludes every index (`rrd-contract/src/lib.rs:1723`). Every index, quantization and payload index is built only when a client asks (`engine/vector/index.rs:23`, `quantization.rs:85`, `collection.rs:109`). Selection is lowest estimated cost (`plan.rs:176`) |
| Hybrid retrieval | `search_hybrid` (`engine/retrieval.rs:17`) and the vector query algebra (`engine/retrieval_query.rs:40`) combine rrflowQL `MATCH` with the real vector runtime and fusion at one stamp | No public operation reaches either one; tests only |
| Context assembly | `context-assemble` (`engine/context.rs:46`), which flights, the task worker and MCP all use | Scans the whole scope snapshot (`:107`), rebuilds BM25 on every request (`:215`), scores vectors exactly (`:466`) and runs its own BFS (`:678`). It ignores every index and the vector runtime |
| rrflowKV/LSM | Memtable, WAL, segments, bloom filters, 64 MiB page cache, compaction (`rrd-lsm/src/database.rs:1272-1299, 2445-2482`) | Logical history is all hot and is never reclaimed. A read at any cursor scans every stored version (`rrd-store/src/access/versioned_read.rs:583-590`). Compaction trigger, target size, levels, row groups and I/O mode are hardcoded (`database.rs:29-32`, `segment/format.rs:19-27`, `io.rs:51`) |
| rrflowCLD | Two-phase publication of generations with a receipt chain (`engine/cold_history.rs:336-594`) | Write-only. No query, vector or context path reads it (`engine/cold_history.rs:61` is a separate operation). It is a Parquet copy, not the owner's custom cold format, and nothing is moved or reclaimed |
| Ingest to vectors | `execute_traced_embedding` (`runtime/data_plane.rs:38`), `embed_and_search_vectors` (`engine/inference.rs:109`) | Tests only. Parsed source chunks are never embedded, so typed text never reaches vector search |
| Tracing | 13 boundaries and 53 operations defined (`rrd-core/src/trace.rs:113-391`); read evidence returned per request | No production code emits the typed operations. The public query path is untraced (`engine/query.rs:26-140`). The traced executor is test-only (`runtime/query.rs:70`). A durable span costs 2 commits |
| Per-request cost | — | Every request appends two hash-chained audit records, each synced to the WAL, even with security off (`engine/invocation.rs:313-385`, `rrd-store/repository/control.rs:184`, `rrd-lsm/src/wal.rs:303-307`). Measured on 2026-11-11: RRFlow's cheapest request costs 5.96 ms against Qdrant's 0.22 ms, and RRFlow's search p50 is 5.92 ms |

Duplicates that converge into the joined paths, under the implementation-traceability procedure:

- `execute_traced_query` and `execute_operator_query` (`runtime/query.rs:70`, `operator.rs:178`);
- three embed-and-search paths (`edge.rs`, `inference.rs:109`, context);
- two reasoning-tree models (kernel and contract);
- the unwired `rrd-maintenance` crate.

## 3. Target

1. **One statement.** rrflowQL carries relational filters, graph traversal, `MATCH` and vector similarity in one statement, with fusion as a plan operator whose method and constant are recorded. The research and its sources are in [section 6](#6-research). The request states the reasoning need: what evidence, which freshness (`AT VALID`/`KNOWN`), what quality (exact, or a recall target), and the budget.
2. **The engine chooses and reports.** The planner picks access paths to meet the stated quality within the budget, using estimated cardinality, and pushes filters into vector search. It never post-filters silently or falls back silently. `EXPLAIN` and every result report the chosen path, its rejected alternatives with reasons, and estimated against actual rows.
3. **The engine builds its paths.** The engine maintains scalar, BM25, adjacency, HNSW and TurboQuant paths, by declared policy, from the schema and the attunement evidence. Filterable fields come from the declared schema, and nothing waits for a client to ask.
4. **Hot first, cold if necessary, at one stamp.** A read at a stamp is served from the memtable, then the page cache, then segments. Below the cold watermark it falls through to rrflowCLD, using the same stamp and the same authentication. Each result reports which tier served it.
5. **Measured and tunable.** Each request carries one correlation id from HTTP through the engine, the store and the LSM, down to CLD, with time per layer. Every threshold is configuration bound into the estate's sealed configuration, and a lab run records its configuration digest and before/after diagnostics.

## 4. Packages, in order

Each package is one PR of at most 400 lines, opened with its brief and a linked journal. Each starts from a failure oracle on the real engine and is accepted on the real installed path. Section 5 lists the decisions the owner must make first.

| # | Package | Smallest change | Failure oracle, then acceptance |
|---|---|---|---|
| J1 | Per-request layer timing | A non-durable request trace with one correlation id through HTTP, the engine, `rrd-store` and `rrd-lsm`. Producers move onto `TraceOperation`, read-side phases are timed (memtable, page cache, page load, decompression), and the public query and vector handlers are wired | Today one query's time cannot be split (`engine/query.rs:26-140`). Accept when a query on a real estate reports per-layer durations that sum to within 10% of wall time, with tracing off adding no commit |
| J2 | The audit cost on reads | Follow the owner's decision in section 5 on read-audit durability | Accept when the cheapest-request p50 is measured before and after in the lab pod. Security-on audit guarantees stay tested |
| J3 | rrflowQL reads the commit-time indexes | Bind a query on an indexed field to the scalar and BM25 entries `rrd-store` already writes, at the read stamp; retire the JSON-copy artifact | The per-file `source-symbol` lookup on this repository examined 99,936 keys. Accept when keys examined stay proportional to the result, the answer matches a full scan, and the result survives writes and a reopen |
| J4 | Engine-maintained access paths | A declared per-estate policy builds and refreshes scalar, BM25, adjacency, HNSW and TurboQuant paths, with filter properties taken from the schema. The existing ensure operations become overrides | Today no path exists without a client call. Accept when, after attunement, a fresh estate answers indexed queries and filtered approximate vector search with no client build call |
| J5 | Vector similarity in rrflowQL | A `NEAR` source or predicate, with a quality target and an optional `ef`/`rerank`, executed through `search_vectors_at` at the statement's stamp (`engine/vector/search.rs:29`), with data-policy enforcement equal to `execute_query`, identity tie-breaks, and `exact` reported honestly | One statement combining a filter, `NEAR` and a traversal is refused at parse today. Accept on the RRFlow repository: results match an exact oracle at matched recall, and the plan reports the path, rejected paths and the filter strategy |
| J6 | Fusion and one context path | Fusion (RRF or weighted, with the constant recorded) becomes a plan operator. `search_hybrid` and `execute_retrieval_query` move behind it. `context-assemble` uses the same planner and drops its own BM25 rebuild, exact scan and BFS | Today context rebuilds BM25 on every request (`context.rs:215`). Accept when context results equal the planner's results at one stamp, and keys examined fall on the repository workload |
| J7 | Source embeddings | Parse output (`source-chunk`) is embedded in the same commit path when an embedding capability is admitted, using the existing embedding execution (`runtime/data_plane.rs:38`) | Today no chunk is ever embedded. Accept when a typed question reaches vector search over this repository's chunks in one call, and an edited file re-embeds only its chunks |
| J8 | Hot then cold at one stamp | F-06's canonical move: a compaction moves versions below a watermark into rrflowCLD under an atomic manifest, the LSM reclaims them, and `scan_source` (`versioned_read.rs:567-600`) merges CLD rows authenticated by the same digest and accumulator proofs | Today no read touches CLD. Accept when hot, cold and retained-stamp reads agree across update, delete, interruption, restart and reclamation (F-06), and each result names its tier |
| J9 | Every threshold configurable and measured | The hardcoded LSM constants become sealed estate configuration. The lab gets a sealed per-run configuration, records diagnostics before and after, and records the configuration digest. Its section-blind settings rewrite (`eval/lab/rrflow_settings.py:15-21`) is fixed | Today the compaction trigger is fixed at 8 (`database.rs:29`). Accept when a one-setting change reruns and the two reports compare in the lab |
| J10 | One RRFlow in the comparison | The lab's `rrflow` system sends the default request and lets the engine choose; HNSW, TurboQuant and exact become diagnostics. ArangoDB and a graph-and-text workload built from a real repository join | Today `rrflow-routed` is `unimplemented`. Accept with one combined report of RRFlow, Qdrant, SurrealDB and ArangoDB from the lab pod |

J1 comes first because nothing else can be tuned without it. J3 and J4 come before J5, because a vector operator needs maintained paths. J8 depends on the owner's CLD format decision.

## 5. Decisions the owner must make

1. **Read-audit durability.** Every read now writes and fsyncs two audit records, even with security off. The choices:
   - keep both syncs on every read;
   - record reads in a batched, group-committed audit log;
   - record reads durably only when security is enforced.

   The answer sets J2.
2. **The rrflowCLD format for J8.** Either define the custom format before the canonical move, or move under the current Parquet generation first and replace the format afterwards. Either way the format is behind one reader.
3. **The default request.** The proposal is that a plain request lets the engine choose with recall 0.95 or exact, whichever its policy supports. Any other default is the owner's choice.

## 5b. Evidence to keep

- The lab pod run of 2026-11-11 on Windows: RRFlow HNSW p50 5.92 ms, against Qdrant 0.89 ms and SurrealDB 21.2 ms (recorded in PR #352).
- The diagnostic Parse run: 897 files, 53,063 symbols, and a per-file lookup of 99,936 keys.

## 6. Research

- SurrealDB vector forms `<|K,EF|>` and `<|K,METRIC|>`, filters pushed into KNN as shown by `EXPLAIN`, and the 4.0.1 change from silent fallback to error ([operators](https://surrealdb.com/docs/surrealql/operators), [similarity search](https://surrealdb.com/docs/learn/data-models/vector-search/similarity-search)).
- ArangoDB 3.12 vector index (IVF) with `APPROX_NEAR_*`, the FILTER restrictions, and cost-based plan selection ([vector indexes](https://docs.arango.ai/arangodb/3.12/indexes-and-search/indexing/working-with-indexes/vector-indexes/), [optimizer](https://docs.arango.ai/arangodb/3.12/aql/execution-and-performance/query-optimization/)).
- Qdrant prefetch with RRF, DBSF or formula fusion; filterable HNSW; a per-segment planner choosing full scan, payload index, graph or ACORN by cardinality ([hybrid queries](https://qdrant.tech/documentation/concepts/hybrid-queries/), [indexing](https://qdrant.tech/documentation/concepts/indexing/), [filterable HNSW](https://qdrant.tech/articles/filterable-hnsw/)).
- ACORN ([arXiv 2403.04871](https://arxiv.org/abs/2403.04871)), Filtered-DiskANN ([WWW 2023](https://dblp.uni-trier.de/rec/conf/www/GollapudiKSKBRL23.html)), and RRF ([SIGIR 2009](https://research.google/pubs/pub36196/)).
- Hot-then-cold reads at one stamp: Neon's GetPage@LSN ([pageserver](https://github.com/neondatabase/neon/blob/main/docs/pageserver-storage.md)), RocksDB's time-aware tiering ([blog](https://rocksdb.org/blog/2022/11/09/time-aware-tiered-storage.html)), and TiFlash's read-timestamp wait ([overview](https://docs.pingcap.com/tidb/stable/tiflash-overview/)).

Each package adapts behavior to RRFlow's own types, stamps, budgets and tests, as AGENTS.md requires. It copies no upstream code.
