# POAM-061 — default query limits refuse a scan of this repository

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-061`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. The default `QueryBudget` (100,000 storage keys, 10,000 rows) refuses a scan of 15,300 items, so two of the thirteen dogfood questions cannot run on a default install and their index cannot be built either.

## Owning gates

C-02 query budgets; D-02 plan P6.

## Closure evidence

Closed when a default install answers every dogfood question or reports a named truncation, and the limits a project needs are derived at install from its inventory.
