# POAM-059 — the product path cannot take in a project

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-059`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. An existing-project install on this repository at `ae01ce32` planned one inventory item and nothing to parse, index or link for 2,451 files; only Connect and Inventory execute. Every answer in the comparison needed a hand-written loader over the public transaction operations.

## Owning gates

OBJ-07; D-02 to D-05 and the D-02 evidence-driven attunement plan (P1 to P5).

## Closure evidence

Closed when a job on this repository reaches Succeeded with parsed spans, a lexical index and graph relations produced by the installed engine, and the dogfood questions run without a loader.
