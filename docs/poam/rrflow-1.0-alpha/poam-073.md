# POAM-073 — a refused commit leaves its transaction open

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-073`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-11, the same live probe showed that after a refused commit the transaction stayed open until it was aborted or timed out.

## Impact

A client that treats a refused commit as the end of the transaction leaks it until its lease expires. Postgres clients expect a failed commit to end the transaction.

## Owning gates

H-08 owns the public transaction behavior.

Roadmap owner: [Gate H](../../roadmap/rrflow-1.0/gate-h.md).

## Closure evidence

Closed when the contract states what a refused commit does to its transaction and a process test holds it.
