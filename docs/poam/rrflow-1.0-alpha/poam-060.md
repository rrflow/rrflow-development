# POAM-060 — no index can be built over more than about a thousand small records

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-060`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. `ensure_query_index` builds an index by running an all-fields query bound by the per-query output budget, whose protocol ceiling is `MAX_QUERY_OUTPUT_BYTES` (768 KiB). About 791 file rows fill 512 KiB, so every path lookup and text search over this repository is a full scan (2,376 ms for 20 path lookups against 46 ms and 30 ms in the comparison systems).

## Owning gates

E-02 native index reader; D-02 plan P6.

## Closure evidence

Closed when the builder reads its source through bounded pages independent of the query output budget, and an index over this repository's 2,451 entries builds and serves a path lookup in under 50 ms warm.
