# POAM-043 — transaction previews rebuild the whole scope and skip object checks

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-043`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

`RrdEngine::preview_transaction` in
`crates/authority/rrd-engine/src/engine/transaction.rs` builds its prospective
snapshot with `preview_data_snapshot` in
`crates/persistence/rrd-store/src/repository/runtime.rs`. That function:

- reads every versioned source of the scope (`RuntimeVersionedSource::all()`)
  through `read_versioned_access`, within the request's `max_storage_keys`;
- appends the transaction's mutations to everything it read; and
- returns the whole prospective `RuntimeDataSnapshot` of the scope.

It validates the schema registry, but not the objects the transaction writes.
`validate_schema_and_objects`, which the commit runs, is not called.

Observed through Connectome's real-engine process test
`the_engine_and_the_native_layer_refuse_what_they_must` at engine revision
`86adada6`: a record of a strict kind without a required property previews
successfully, and the commit of exactly that preview is refused. The cited
functions are unchanged at `development/main` `618269b4`.

## Impact

- A preview's cost and response size grow with the scope, not with the change.
  A scope with more versioned keys than the preview's key budget cannot preview
  any change at all, however small.
- A preview is not a dry run of the commit. An operator approves a change the
  engine then refuses for a reason the preview could have reported.

## Owning gates

C-04, H-04

C-04 owns reads served from direct versioned keys without whole-log
reconstruction. H-04 owns operation semantics that agree across surfaces: a
preview reports what the commit of the same mutations would refuse, apart from
conflicts that arise after the preview.

Roadmap owners: [Gate C](../../roadmap/rrflow-1.0/gate-c.md) and
[Gate H](../../roadmap/rrflow-1.0/gate-h.md).

## Closure evidence

- A preview reads only the versioned keys the transaction's mutations touch,
  and returns those objects' prospective state. Physical counters show a
  preview's read cost unchanged as unrelated records are added to the scope.
- A differential test shows that preview and commit refuse the same mutations
  for the same schema and object reasons, across strict, schemaless,
  constrained and claim-only transactions.
