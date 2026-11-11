# POAM-067 — a budget truncation does not name its budget

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-067`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. A result truncated "by its budget" leaves the client unable to tell which of the seven `QueryBudget` limits applied.

## Impact

A client cannot raise the right limit or report what was cut; truncation evidence is not actionable.

## Owning gates

C-02 query evidence; D-02 plan P6.

Roadmap owner: [Gate C](../../roadmap/rrflow-1.0/gate-c.md), [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when every truncation names the limit, its value and the amount consumed.
