# POAM-075 — commit-time BM25 changes are written but never applied

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-075`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

A code read at `de435238` on 2026-11-11 found the following:

- Every commit on a schema-bound lexical index writes an `IndexSourceDelta` (`crates/persistence/rrd-store/src/access/index.rs:538-560`).
- Its only reader is `index_source_deltas_since` (`crates/persistence/rrd-store/src/repository/runtime.rs:482`). Nothing outside `crates/persistence/rrd-store/tests/native_index_commit.rs` calls it.
- No path applies the deltas to persisted BM25 postings.
- An rrflowQL `MATCH` against a stale index rebuilds BM25 on every query (`crates/compute/rrd-query/src/execute.rs:1338-1360`).
- Context assembly builds its own BM25 index on every request (`crates/authority/rrd-engine/src/engine/context.rs:216`).

## Impact

Lexical search is never incremental. Its cost grows with the corpus on every query. A tuned BM25 index cannot change what context assembly returns. This is concrete evidence for [POAM-005](poam-005.md).

## Owning gates

E-03 owns incremental BM25 postings at a declared source cursor.

Roadmap owner: [Gate E](../../roadmap/rrflow-1.0/gate-e.md).

## Closure evidence

Closed when committed deltas reach persisted postings at their source cursor, `MATCH` and context read those postings, and results equal a full rebuild across update, retire and reopen.
