# POAM-047 — a collection whose vectors all moved to another collection cannot be deleted

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-047`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Observed while authoring [POAM-045](poam-045.md) (pull request #211), on the
E-04 stack at E-04d `f34451ee`, with POAM-045's fix applied:

- **The repro.** Put seven identities in collection `documents`, then put each
  again in `archive` with the same valid time. Point retrieval of `documents`
  at valid times 10 and 60 finds none of them. Deleting `documents` at valid
  time 60 (`RrdEngine::delete_vector_collection`) is refused: "vector
  collection documents still contains live or future points; retire them
  through one transaction at or before the deletion time". This was a
  temporary engine test, run once and not committed.
- **Why.** `RrdEngine::require_empty_collection`
  (`rrd-engine/src/engine/vector/collection.rs`) keeps, for each identity, the
  latest version among those that name this collection. It refuses when any of
  them is live or future. A moved identity's latest version names another
  collection, so the version checked here is the one it replaced, and that
  version has no end.
- **No way out.** Retiring the identity retires its version in the collection
  it moved to. A caller cannot retire only the replaced version.
- **POAM-045's rule is not enough.** POAM-045 resolves membership at one valid
  time. Deletion asks whether the collection holds any identity at any valid
  time from the deletion onward, so a move that takes effect after the
  deletion time still keeps the collection occupied.

## Impact

A collection whose vectors have all moved elsewhere can never be deleted. Its
definition and payload-index declarations stay in the catalogue.

## Owning gates

C-04

C-04 owns current and temporal reads at one `ReadStamp`, including the read
that decides which identities a collection holds.

Roadmap owner: [Gate C](../../roadmap/rrflow-1.0/gate-c.md).

## Closure evidence

- **A deletion oracle.** Deletion succeeds once every identity has moved to
  another collection at or before the deletion time. It is refused while any
  identity is held there at or after the deletion time, including one held
  there because its move out takes effect later in valid time.
- **One rule.** Deletion decides membership with the same latest-version rule
  as search and point retrieval.
