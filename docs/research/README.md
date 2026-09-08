# Platform industry research index

**Status:** active platform-industry research index; linked source records remain partially unclassified
**Coordinate:** `rrflow://rrflow-instance/data/research-index/platform-industry`
**Owner:** primary-source external-system research discovery; not RRFlow terminology or delivery planning
**Reviewed:** 2026-09-08
**Scope:** `docs/research/`

This directory is the single entry point for industry research used to shape
the RRFlow platform. This index lists source systems and repository research;
it is not a second terminology list or an implementation plan. Canonical RRFlow
terms are defined only in the repository [`README.md`](../../README.md).

## File naming

Use lowercase kebab-case and one of these forms:

- `<subject>-capability-inventory.md` for a source-system inventory;
- `<subject>-architecture-research.md` for bounded technical research; and
- `<subject>-comparison.md` for source-backed comparative analysis that makes
  no benchmark or acceptance claim.

Accepted decisions belong in `docs/decisions/`, measured artifacts and their
exact reproduction metadata belong in `docs/evidence/`, and delivery plans
belong in `docs/roadmap/`.

This file is the active research index. New platform-industry research must be
linked here. Existing flat documents remain at their current paths until each
receives a separate full review and link-preserving classification.

## External reference systems

| Subject | Terms to inventory | Primary sources | Repository note |
|---|---|---|---|
| Qdrant | collection, point, vector, payload, segment, HNSW, quantization, shard, replica, tenant, strict mode | [`v1.19.1` release](https://github.com/qdrant/qdrant/releases/tag/v1.19.1), [pinned source](https://github.com/qdrant/qdrant/tree/6ab21cac18ebb6f4ae29102c7f8f5cc11affd5de), [collections](https://qdrant.tech/documentation/manage-data/collections/), [hybrid queries](https://qdrant.tech/documentation/search/hybrid-queries/) | [`qdrant-capability-inventory.md`](qdrant-capability-inventory.md) |
| HelixDB | labeled property graph, node, edge, label, property, index, query, workspace, project, cluster | [introduction](https://docs.helix-db.com/database/helix-db/start-here/introduction), [data model](https://docs.helix-db.com/database/helix-db/core-concepts/data-model), [run modes](https://docs.helix-db.com/database/helix-db/start-here/run-modes), [repository](https://github.com/HelixDB/helix-db) | No peer capability inventory exists yet. |
| SurrealDB | query/document transaction pipeline, ordered keys, temporal graph, indexes, live/changefeed, permissions, RPC/MCP, installation | [`v3.2.4` release](https://surrealdb.com/releases/3.2), [pinned source](https://github.com/surrealdb/surrealdb/tree/93ab219d69f09d8f999851b0359c80ebe6726102), [architecture](https://surrealdb.com/docs/learn/data-models/architecture) | [`surrealdb-capability-inventory.md`](surrealdb-capability-inventory.md); unclassified [`rrflow-surrealdb-differential.md`](../rrflow-surrealdb-differential.md) |

The pinned SurrealDB reference explicitly rejects its namespace → database →
table/record hierarchy as RRFlow authority. `estate` is an RRFlow-owned term;
the accepted project ↔ estate ↔ instance topology remains defined only by the
[RRFlow topology owner](../architecture/instance-topology.md).

## RRFlow vocabulary reference

The research above informed the one standardized list in
the repository [`README.md`](../../README.md). Research notes may compare foreign
terms, but they must link to that list instead of restating or redefining RRD
terminology.

## Existing RRFlow research and supporting notes

The following reviewed history, supporting references, and unclassified flat
records participate in this research trail; listing them here does not transfer
their authority into this index:

- [`rrflow-system-convergence-architecture-research.md`](rrflow-system-convergence-architecture-research.md)
  (active primary-source basis for the 1.0 execution map)
- [`rrd-data-services-architecture-research.md`](../history/rrd-data-services-architecture-research.md) (historical)
- [`estate-control.md`](../reference/operations/estate-control.md)
  (active implementation reference)
- [`local-estate-authorization.md`](../reference/security/local-estate-authorization.md)
  (active supporting security reference)
- [`instance-topology.md`](../architecture/instance-topology.md)
  (active accepted RRFlow architecture; research records cannot redefine it)

## Historical research follow-ups

These captured follow-ups are not the current roadmap; the executable gate
order lives in [`docs/roadmap/rrflow-1.0.md`](../roadmap/rrflow-1.0.md).

- Add a HelixDB capability inventory at
  `docs/research/helixdb-capability-inventory.md`.
- Complete the code audit against the canonical platform list across contracts,
  engine, storage, services, SDKs, tests, and documentation.
- Classify and migrate each conflicting use without retaining aliases or a
  second glossary.
- Recheck time-sensitive implementation guidance when its owning execution
  gate begins; the current researched implementation map is linked from the
  [RRFlow 1.0 roadmap](../roadmap/rrflow-1.0.md).
