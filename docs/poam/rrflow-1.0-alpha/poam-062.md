# POAM-062 — a property named `kind` is silently unreadable

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-062`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. The schema accepts a user property named `kind`, but a query reads the record's built-in kind instead of the property. Reserved names must be refused at schema time.

## Impact

A schema that accepts a name the reader ignores returns wrong answers silently; the harness found it only through the independent oracle.

## Owning gates

C-05 schema catalogue; D-02 plan P6.

Roadmap owner: [Gate C](../../roadmap/rrflow-1.0/gate-c.md), [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when `PutSchema` refuses reserved property names with a named error and a regression test covers `kind`, `id` and `scope`.
