# POAM-072 — retiring a referenced row leaves its relation live

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-072`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-11, the same live probe retired a row that a relation still referenced. The commit was accepted and the relation remained live with a retired endpoint. A relation to a missing row is refused when it is created, so the guarantee holds only at creation.

## Impact

The graph can hold relations whose endpoints no longer exist, and there are no restrict or cascade rules for a schema to choose.

## Owning gates

H-08 owns the data model's public behavior.

Roadmap owner: [Gate H](../../roadmap/rrflow-1.0/gate-h.md).

## Closure evidence

Closed when a schema can declare restrict and cascade behavior for a relation, retiring a referenced row follows it, and the outcome holds through time travel and restart.
