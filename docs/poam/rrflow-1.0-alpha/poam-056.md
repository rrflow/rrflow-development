# POAM-056 — storage maintenance stalls every search, and searches cause it

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-056`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Found by the [comparison lab](../../../eval/lab/README.md) on `clyffy-01`, on an
engine built from `d0e6c7f26e4d`, a tree holding unlanded search packages, on
default limits with an HNSW index.

Searches that take under 12 ms are stalled for 400 to 960 ms, several times in ten
seconds, on a run that only searches. On COCO at ef-64 with one client, the median
is 7.87 ms, which implies about 127 searches per second, and the run achieved 92.5.
The maximum was 711 ms where p99 was 11.36 ms, so the lab's reported percentiles do
not show it.

`RrflowKvStore` holds its database lock exclusively across a storage flush and a
compaction step, in `try_maintenance_step` and in a soft write-triggered flush, so
no read proceeds during either.

Searches cause the work. Each search commits two audit records, and the engine's
`write_bytes` grew by about 21 KB per search on a run that only searches. That
figure counts whole pages, so it overstates the logical bytes.

Moving the flush moved the stall. On `vector-small`, raising `wal_flush_target_bytes`
from 64 MiB to 4 GiB took plain ef-64 with one client from 115.8 to 164.1 searches
per second and its maximum from 623.6 ms to 9.9 ms. The stall then arrived later, at
ef-128, with a matching burst of writes. That is one run of each.

## Impact

Every caller of an engine under read load waits up to about a second, several times
a minute, with nothing else running. A workload that writes would stall its readers
the same way. It also makes single runs hard to compare: a stall that lands in one
run's case and not another's reads as a throughput difference, which was once
misread here as the cost of a smaller page cache.

## Owning gates

E-04 and OBJ-13 (vector search within bounded resources) and OBJ-10 (reproducible
competitive evidence).

Roadmap owner: [Gate E](../../roadmap/rrflow-1.0/gate-e.md).

## Closure evidence

A test shows that a read proceeds while a flush and a compaction step run, and that
a crash at each new boundary recovers as today's flush does. On `clyffy-01`, a run
of the same test shows its maximum within a stated multiple of its p99 across a
flush, and the lab reports a case's maximum beside its percentiles.
