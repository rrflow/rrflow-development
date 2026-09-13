# POAM-002 — accepted segment-v6 physical layout

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-002`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Accepted C-06 replaces row segment v3 with one exclusive segment-v6 reader: ordered
key/version state, six aligned Arrow-layout buffers, authenticated row-group filters,
authenticated none/adaptive-LZ4 policy, bounded owned projected generations, and one
process-local exact-byte cache. C-06j makes a scope-aware family-neutral
probationary/protected LRU the default while retaining exact LRU as a selectable
oracle/operator policy; same-stream touches cannot promote scan pages.

## Impact

The physical-layout deficiency is closed. rrflowQL still materializes semantic values and
allocates new Arrow arrays, but that separate analytical gap remains POAM-004/F work;
installed recovery, maintenance, concurrency, storage-full, and cross-platform lifetime
remain C-07 work. No release, scale, latency, or competitor claim follows from C-06.

## Owning gates

C-06

## Closure evidence

Retain the v6/manifest/snapshot vectors; exact generated/projected none/adaptive reads;
malformed filter/codec/length/envelope denial; prior-format rejection;
ownership/filter/decompression/cache counters; authenticated writer budgets;
pinned-generation/GC proof; C-06h stable/stress/fuzz qualification; clean C-06i
filter/compression artifacts; and clean C-06j artifact SHA-256
`8a008ee33bb50ca197783227cfcfbb4d58945dd2f26dca8cdf12e1804106b9ad` from revision `5b1c31d`,
where exact LRU reloads 48 hot pages and scan-resistant LRU reloads zero with identical
durable/semantic digests and exact capacity. Value separation and family partitioning are
rejected for the accepted format rather than left implicit.
