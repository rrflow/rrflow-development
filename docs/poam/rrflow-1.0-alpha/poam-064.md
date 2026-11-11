# POAM-064 — one record's properties are capped at 64 KiB with no engine chunking

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-064`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. 63 files, 22% of this repository's text, had to be split into chunk records by the client. The engine owns no text-chunking policy.

## Owning gates

C-02 record limits; D-02 plan P3 (AST chunks) and P6.

## Closure evidence

Closed when large text enters through the engine's own chunking with provenance, and a client never splits content to fit a limit.
