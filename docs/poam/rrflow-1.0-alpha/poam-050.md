# POAM-050 — clients cannot build RRFlow's HNSW or payload indexes

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-050`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Found while building the [comparison lab](../../../eval/lab/README.md), and
confirmed in source at `3a72a6b4adc43c842f67a59f4fc91d725eac7580`.

`rrd-contract` defines `EnsureVectorIndex` and `EnsureVectorPayloadIndex`, and
`RrdEngine` implements `ensure_vector_index`. Neither type appears in the
executable operation catalogue, and `rrd-client` and `rrd-server` have no route or
method for them. The catalogue's vector operations are limited to collection
ensure and list, point retrieve and scroll, the quantization lifecycle, and search.

So a client of an installed engine can use only an exact scan or a quantized
projection. It cannot build the HNSW generation that the
[search contract](../../reference/vector/search.md) describes. It also cannot
declare a payload index, so a quantized generation cannot cover a filter. A
filtered search that requires an approximate path was refused on a real installed
engine: `no vector access path satisfies the request contract`.

The vector capability text in `rrd-server/src/http/capabilities.rs` advertises
typed payload indexes and persisted HNSW artifacts. Both are reachable only by
embedding the engine.

## Impact

The lab cannot measure RRFlow's HNSW against Qdrant, SurrealDB or Milvus through
any public path, so its RRFlow HNSW arm is declared `unimplemented`. An application
using the public API has no approximate filtered search, and filtered vector
search falls to an exact scan. The [package journal](../../evidence/change-journals/gate-j/comparison-lab.md)
holds the measured effect.

## Owning gates

E-04 (immutable HNSW generations with an exact overlay) and H-04 (public
operations generated from the one catalogue).

Roadmap owners: [Gate E](../../roadmap/rrflow-1.0/gate-e.md) and
[Gate H](../../roadmap/rrflow-1.0/gate-h.md).
