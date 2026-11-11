# POAM-066 — rrflowQL has no GROUP BY, ORDER BY or SUM and MATCH takes one word

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-066`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. Grouping, sorting and sums finished in the client; a MATCH of several words means any of them, so the three-word text question took 3,845 ms with a client-side filter.

## Owning gates

F-03 mixed analytical operators and E-03 lexical access; D-02 plan P5 for MATCH, F-03 for aggregates.

## Closure evidence

Closed when multi-word MATCH with explicit AND and OR is native, and GROUP BY, ORDER BY and SUM execute in the engine with read evidence.
