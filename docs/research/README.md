# Platform industry research index

**Status:** active platform-industry research index; linked source records remain partially unclassified
**Coordinate:** `rrflow://rrflow-instance/data/research-index/platform-industry`
**Owner:** primary-source external-system research discovery; not RRFlow terminology or delivery planning
**Reviewed:** 2026-08-27
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
| Qdrant | collection, point, vector, payload, segment, shard, replica, alias, tenant, strict mode | [overview](https://qdrant.tech/documentation/overview/), [collections](https://qdrant.tech/documentation/manage-data/collections/), [distributed deployment](https://qdrant.tech/documentation/scaling/distributed_deployment/), [multitenancy](https://qdrant.tech/documentation/manage-data/multitenancy/) | [`qdrant-capability-inventory.md`](../qdrant-capability-inventory.md) |
| HelixDB | labeled property graph, node, edge, label, property, index, query, workspace, project, cluster | [introduction](https://docs.helix-db.com/database/helix-db/start-here/introduction), [data model](https://docs.helix-db.com/database/helix-db/core-concepts/data-model), [run modes](https://docs.helix-db.com/database/helix-db/start-here/run-modes), [repository](https://github.com/HelixDB/helix-db) | No peer capability inventory exists yet. |
| SurrealDB | namespace, database, table, record, relation, organization, project, instance, cluster | [architecture](https://surrealdb.com/docs/architecture), [namespace/database hierarchy](https://surrealdb.com/docs/learn/schema-management/multi-tenancy/namespace-and-database-architecture), [repository](https://github.com/surrealdb/surrealdb) | [`surrealdb-capability-inventory.md`](../surrealdb-capability-inventory.md); [`rrflow-surrealdb-differential.md`](../rrflow-surrealdb-differential.md) |

The reviewed SurrealDB material does not establish **estate** as its canonical
data-isolation term. Its documented logical hierarchy is namespace → database
→ table/record; its managed and deployment vocabulary is separate. Any RRD use
of `estate` therefore needs an RRD-owned definition rather than attribution to
SurrealDB.

## RRFlow vocabulary reference

The research above informed the one standardized list in
the repository [`README.md`](../../README.md). Research notes may compare foreign
terms, but they must link to that list instead of restating or redefining RRD
terminology.

## Existing RRD research and architecture notes

The following reviewed history and unclassified flat records participate in
this research trail; listing them here does not make them current authority:

- [`rrd-data-services-architecture-research.md`](../history/rrd-data-services-architecture-research.md) (historical)
- [`estate-control-v1.md`](../estate-control-v1.md) (unclassified)
- [`local-estate-authorization-v1.md`](../local-estate-authorization-v1.md) (unclassified)
- [`instance-topology.md`](../instance-topology.md) (unclassified)

## Historical research follow-ups

These captured follow-ups are not the current roadmap; the executable gate
order lives in [`docs/roadmap/rrflow-1.0.md`](../roadmap/rrflow-1.0.md).

- Add a HelixDB capability inventory at
  `docs/research/helixdb-capability-inventory.md`.
- Complete the code audit against the canonical platform list across contracts,
  engine, storage, services, SDKs, tests, and documentation.
- Classify and migrate each conflicting use without retaining aliases or a
  second glossary.
- Author the full researched implementation plan only after the terminology
  and migration gaps have been incorporated into the enforced RRFlow board.
