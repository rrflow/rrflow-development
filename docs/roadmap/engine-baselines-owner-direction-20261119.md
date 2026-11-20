# Engine baselines and the minimum prompt schema: owner direction of 2026-11-19

**Status:** active owner direction record; not an accepted decision, plan or status. The [kernel contract](kernel-contract.md) remains the one active engineering plan
**Coordinate:** `rrflow://rrflow-instance/data/work-package/engine-baselines-owner-direction`
**Owner:** records, verbatim, what the repository owner said on 2026-11-19 about the baselines of rrflowLSM and rrflowCLD and about prompt schemas, and maps each named part of the engine to what `main` holds; the [roadmap](rrflow-1.0.md) and [POA&M](../poam/rrflow-1.0-alpha.md) keep status authority
**Verified against:** `main` at `557c3e3d5` on 2026-11-19, by reading the named source files. Nothing was built or measured; every number below is quoted from an existing record

This record continues the [storage and DevAssist direction](storage-and-devassist-owner-direction-20261116.md).

## The owner's words

Typos are the owner's and are kept.

> I need this to be clearly mapped out this is never going anywhere if we do not get serious about my fucking engine truly being what I thought I already had and clearly do not.
> rrflowKV rrflowQL rrflowDB
> paraquet was only suppose to be used to test and compare later with the harness. vortex is our baseline for rrflowCLD rrflowLSM is baseline is vyrm which was suppose to properly be optimized fjall that we enhanced for AI but still needs to be truly proven. Turboquant, the capabilities from surrealDB, qdrant, the concept is that a user types a prompt. Default schemas ensure behavior and intent are properly fully make sure we we replace the costar and look our schemas need to be never rigid minimum requirements minimum paramaters

The owner then gave four pillars, each with a minimal requirement. They are recorded as the default structure `prompt-minimum-v1` in [kernel contract S8](kernel-contract.md#38-s8-reasoning-structure).

## What changes in the records

| Subject | Before | Now |
|---|---|---|
| rrflowCLD format | "started from Parquet", "the Parquet generation first" (2026-11-11) | Vortex is the baseline rrflowCLD is measured against, not its format. Parquet is a comparison arm and nothing else |
| rrflowLSM | "started from Fjall" | Fjall is the baseline it must capture and beat; it is "still to be truly proven" |
| Prompt and handoff structure | The owner's working name was COSTAR; no record or code held it | The four pillars, as minimums that grow |
| Schemas | ADR-0002 and S8 already say no fixed schema of thought | Restated: minimum requirements and minimum parameters, never rigid |

## The engine against that direction

| Part | On `main` | Gap |
|---|---|---|
| **rrflowLSM** (`crates/persistence/rrd-lsm`) | A native LSM: write-ahead log, memtable, leveled compaction, bloom filters, page cache, LZ4, snapshots, io_uring on Linux. No Fjall code; Fjall is an optional dependency of `rrd-store` behind `fjall-baseline` | Not proven. The recorded Fjall numbers are of August 2026, from a harness removed on 2026-09-07 ([history](../history/rrd-lsm-promotion-benchmark.md)). The restored comparison ran two debug trials and is not in CI ([journal](../evidence/change-journals/repository/fjall-baseline-20261117.md)). One writer (`database/transactions.rs`), one process per directory (`manifest/open.rs`). No serializable multi-writer transactions, no tree per keyspace, no key-value separation. Compaction constants are fixed in code |
| **rrflowCLD** (`rrd-engine/src/engine/cold_history*`, `rrd-query/src/cold_history.rs`, `parquet.rs`) | Parquet, written by the product and on by default (`default-engram-configuration-v3.toml`, `[storage.cold_history]`). A derived copy that one endpoint reads back | Vortex is named nowhere. No reader boundary: the encode and decode functions call the Parquet crate directly and `.parquet` is in the paths. It is not in a storage crate. No query or context path reads it and it reclaims no hot space |
| **rrflowKV** (`crates/persistence/rrd-store`) | Ordered keyspaces over rrflowLSM; secondary, unique and BM25 indexes in the same transaction; both edge directions; bitemporal histories; changefeed | Every commit must match one engram-wide cursor, so writers on unrelated rows cannot both commit ([POAM-071](../poam/rrflow-1.0-alpha/poam-071.md)) |
| **rrflowQL** (`crates/compute/rrd-query`) | A native grammar: one source, AND filters, one equi-join, `MATCH`, a transaction program, and breadth-first traversal of one relation type from one start node to depth 32 | No OR, ORDER BY, GROUP BY, aggregates or subqueries ([POAM-066](../poam/rrflow-1.0-alpha/poam-066.md)). No all-paths, shortest path or pattern match. No vector operator. Traversal cannot feed a projection or a join |
| **rrflowDB** | No crate or module of that name; the engine composition and a client example | No PostgreSQL wire protocol or SQL dialect ([protocol record](../reference/protocol/postgresql.md)). Snapshot isolation only |
| **TurboQuant** (`crates/compute/rrd-vector`) | Implemented and persisted at 4, 2, 1.5 and 1 bit with both estimators | Not used by default: `VectorSearchMode` defaults to `Exact`. The engine never loads a measured cost model ([POAM-080](../poam/rrflow-1.0-alpha/poam-080.md)) |
| **Qdrant and SurrealDB capabilities** | Two inventories, 16 and 25 rows ([Qdrant](../research/qdrant-capability-inventory.md), [SurrealDB](../research/surrealdb-capability-inventory.md)) | Neither says which rows are met. Payload index definitions are never read by search. Sparse and multi-vector search are exact scans |
| **A user types a prompt** | `context-assemble` and `flight-launch` return a ranked context packet from BM25, records, relations, an optional exact vector scan and a graph walk. The MCP adapter has one tool; the CLI takes no prompt | No model is called: the one flight provider is `Observe`. No default schema is applied to a prompt |
| **The four pillars** | Role text exists as install-template prose | No question to the user, no per-turn limit, no discovery-to-execution state ([POAM-039](../poam/rrflow-1.0-alpha/poam-039.md)). The proposed handoff schema is referenced by no code |
| **Schemas as minimums** | The contract's request and record types refuse unknown fields and use closed enums; one type has an extension variant | The opposite of a minimum that grows, for the types that would carry prompts, plans and handoffs |

Recorded speed, 384 dimensions, 20,000 points, one client, recall target 0.95 ([Lab evidence](../../eval/lab/evidence/arm-20k-20261106/report.json), [HNSW arm](../evidence/change-journals/gate-j/lab-rrflow-hnsw-arm.md)): Qdrant 567 queries a second, SurrealDB 64, RRFlow TurboQuant 15.3, RRFlow HNSW 140 to 176 in later runs. No speed measurement exists against ArangoDB or pgvector.

## Order the owner approved

The owner returned this list on 2026-11-19. Each is one package and one pull request.

1. Correct the records (this package).
2. Prove rrflowLSM against Fjall: the restored comparison at benchmark scale on Linux, and in CI.
3. rrflowCLD behind one reader boundary in a storage crate, with Vortex as the product format and Parquet moved to the Lab. Corrected on 2026-11-20: Vortex is a reference module like Parquet and Fjall, not the product format; rrflowCLD is native ([direction](reference-modules-and-reasoning-fabric-owner-direction-20261120.md)).
4. The four pillars as the first default schema, as minimums with open extension, wired to `context-assemble` and `flight-launch`.
5. TurboQuant by default, with the measured cost model loaded.

## Open

1. Whether the Vortex crates meet this repository's pinned-dependency and all-platform rules has not been checked. Package 3 starts there.
2. Question 1 of the 2026-11-16 record stands: many writers in one process, several processes on one directory, or both.
