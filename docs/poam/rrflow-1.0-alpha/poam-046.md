# POAM-046 — a vector index makes the public changefeed refuse its scope

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-046`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Observed while authoring H-04q2 (the public quantization lifecycle), on the
E-04 stack above development/main `81fce08e`:

- **The repro.** Ensure an HNSW vector index with
  `RrdEngine::ensure_vector_index`, then read the scope's changefeed
  (`read_changefeed_page`). The read fails with "canonical id must start with
  lowercase ASCII or a digit and contain only lowercase ASCII, digits, '-',
  '_', or '.'", and no page is returned.
- **Why.** The vector artifact catalogue writes its record as
  `{projection}@{generation}`
  (`rrd-vector/src/catalog.rs::VectorArtifactCatalogEntry::record_reference`)
  and its object as `{projection}@{generation}:bytes`
  (`rrd-engine/src/runtime/vector_catalog.rs::publish_traced_vector_artifact`).
  The kernel accepts both identities. The public changefeed projects every
  runtime change into public types whose identities must be canonical ids
  (`rrd-engine/src/engine/model/raise.rs::public_change_ref`). It refuses `@`
  and `:`, so a single such change fails the whole page.
- **Same fault, already fixed.** The quantization lifecycle wrote identities of
  the same shape. H-04q2 made them canonical (`{projection}.{generation}` and
  `{projection}.{generation}.quantized-bytes`), because its public operations
  reach them. Its oracle is
  `engine::tests::vector_index::quantization_replay::a_scope_with_quantization_artifacts_keeps_its_public_changefeed`.

## Impact

No public operation ensures a vector index today, so only embedded engine
callers reach this. When one does, every changefeed read and follow of that
scope fails, and a subscriber such as Connectome sees no further changes.

## Owning gates

C-05, H-04

C-05 owns the vector projection lifecycle and its catalogue identities. H-04
owns the public operations and projections that must accept every change in a
scope.

Roadmap owners: [Gate C](../../roadmap/rrflow-1.0/gate-c.md) and
[Gate H](../../roadmap/rrflow-1.0/gate-h.md).

## Closure evidence

- **Canonical identities.** Every identity the vector artifact catalogue
  writes, for its record and its object, is a public canonical id. The writer
  checks this before it stages bytes.
- **A changefeed oracle.** After an HNSW and an exact-segment ensure, the
  scope's changefeed projects every change.
- **Review of every engine-internal writer.** An inventory of the engine's
  internal record and object writers shows that none writes an identity the
  public projection refuses.
