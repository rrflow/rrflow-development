# POAM-055 — a vector search the engine will refuse first reads every vector

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-055`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Found by the [comparison lab](../../../eval/workbench/lab/README.md) on `clyffy-01`, at
100,000 points of 384 dimensions, on an engine built from
`2f84fdf0fbf7d254678d1cf65c1cbd92a1c1accd` with its read limits raised.

A filtered search with `require_approximate`, over a TurboQuant generation that
cannot cover the filter, is always refused:

```text
RRD API 400 Bad Request: InvalidArgument: Vector("invalid runtime object: no vector access path satisfies the request contract")
```

That refusal took far longer than the searches the same engine answers. One client's
20 refusals took 146.9 seconds, or 7.3 seconds each, where an answered TurboQuant
search takes about 313 ms. At 8 clients, 153 refusals took 307.1 seconds, or about 16
seconds each.

`search_vectors_at` in `rrd-engine/src/engine/vector/search.rs` already reads the
vector catalogue before it reads any vector. When `search_turboquant_generation`
declines, it continues to `read_versioned` over every stored vector. Only then
does `prepare_search_at` reach `VectorPlanner::plan_at` in
`rrd-vector/src/plan.rs`, which refuses because no candidate path covers the
filter's properties.

## Impact

A request that cannot succeed costs a read of the whole corpus, authenticated and
held in memory, and the cost grows with the corpus. Concurrent refusals slow each
other as searches do. In the lab, refused searches and their warmup took about 2,240
of a run's 3,742 seconds and returned nothing. They would take the same capacity
from other callers of an engine.

They also leave the engine holding more memory. The same engine held 6,044 MiB
resident at the end of a run that included refused searches, and 1,174 MiB at the end
of one that did not. Each figure is one sample at the end of a run, not a peak.

## Owning gates

E-04 and OBJ-13 (vector search that serves what the engine accepts, within bounded
resources).

Roadmap owner: [Gate E](../../roadmap/rrflow-1.0/gate-e.md).

## Closure evidence

A test through the engine shows that a search no active path can satisfy returns
the same refusal without reading any vector version, shown by its read evidence
and by a corpus size that does not change what the refusal reads. A search that
can be served, including through an overlay, returns the same result as before.
