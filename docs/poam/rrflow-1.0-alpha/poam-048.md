# POAM-048 — an approximate generation keeps proposing a vector that moved to another collection

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-048`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Observed while authoring [POAM-045](poam-045.md) (pull request #211), on the
E-04 stack at E-04d `f34451ee`, with POAM-045's fix applied:

- **The repro.** Use the estate of
  `engine::tests::vector_index::collection_moves`:
  - `a` moves from `documents` to `archive`, and `c` to `mirror`;
  - `b` moves to `archive` from valid time 50;
  - `d` moves out and back;
  - `e` moves in from `archive`.

  Build each accelerator after the moves. Search `documents` at valid time 60
  near `a`'s value with `top_k` 3 and
  `RequireApproximate { exact_rerank: 3, ef_search: 32 }`. Exact search
  returns `e`, `d` and `f`. TurboQuant, scalar quantization and HNSW each
  return only `e`. This was a temporary engine test, run once and not
  committed.
- **Why.** A generation is built from every version that names its collection
  and named vector, and it keeps them all. That covers `ensure_vector_index`,
  `build_vector_quantization_artifact` and the E-04c extend. Nothing in a
  generation records that an identity's later version names another
  collection, so it proposes the replaced versions of `a`, `b` and `c`.
  POAM-045's rerank resolves each proposal against the canonical candidates
  and discards those. The rerank width is spent, and fewer hits remain.
- **HNSW never recovers.** HNSW's incremental advance only inserts versions.
  A graph built before a move keeps proposing the moved identity until a full
  rebuild.
- **Why the build cannot simply record the move.**
  - A version in the old collection that marks the move and is never visible
    needs an empty validity window when the move takes effect at the replaced
    version's own valid time. That is the common case, and
    `RuntimeVector::validate` refuses `valid_to <= valid_from`.
  - Dropping, at build time, every version that a version in another
    collection supersedes at all valid times handles the common case. But
    HNSW maintenance would then have to remove nodes. A move that takes
    effect later in valid time would still be proposed after it takes effect.

## Impact

An approximate search returns fewer hits than it asked for whenever its
nearest proposals have moved away. It raises no error and gives no evidence
that recall was lost. The loss grows with moves and persists across
generations.

## Owning gates

E-04

E-04 owns the accelerator generations, their maintenance and their recall.

Roadmap owner: [Gate E](../../roadmap/rrflow-1.0/gate-e.md).

## Closure evidence

- **A recall oracle.** After moves in both directions and a move that takes
  effect later in valid time, an approximate search near a moved identity's
  replaced version fills its rerank width with identities the collection
  holds. This holds for HNSW (built before the moves, after them, and
  advanced), scalar quantization, and TurboQuant (full and extended).
- **Generation contents.** No generation proposes an identity's replaced
  version at a valid time where its later version names another collection.
