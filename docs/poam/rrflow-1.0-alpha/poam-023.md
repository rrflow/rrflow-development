# POAM-023 — distributed-cluster authority

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-023`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

`rrd-cluster` is a parallel pre-release authority: it defines loose
cluster/node/zone/region/tenant/table/scope identities, persists a self-installed 4 MiB JSON
topology catalogue, opens `rrd_lsm`, `RrflowKvStore`, and object paths directly, accepts raw
probe/`RuntimeCommit` commands through process and TLS control, derives artifact closure by
replaying semantic history from cursor zero, stores transfer sessions/receipts in private
JSON/filesystem markers, and emits `cluster.*` traces through direct-store helpers. Its
defaulted/pre-1.0 snapshot state is not the sole current format. At clean baseline
`03ba77c`, the all-feature package suite failed because real consensus apply returned
`unsupported rrflowKV application format None`; the other isolated consensus test did not
terminate within 120 seconds.

## Impact

Cluster code can bypass installed identity, effect-complete authorization, the semantic
transaction/index/audit boundary, current-format policy, job/effect truth, and causal
vocabulary while one-host tests appear to prove a distributed engine. It cannot safely
replicate the native graph/BM25/vector/Arrow/DataFusion reasoning system or be advertised as
a deployment profile.

## Owning gates

A-07, C-01 through C-07, D-01, D-06, E-01 through E-05, F-01 through F-05, H-04, H-05, H-07,
J-01 through J-05; a distributed availability gate is not yet scheduled

Roadmap owners: [Gate A](../../roadmap/rrflow-1.0/gate-a.md),
[Gate C](../../roadmap/rrflow-1.0/gate-c.md),
[Gate D](../../roadmap/rrflow-1.0/gate-d.md),
[Gate E](../../roadmap/rrflow-1.0/gate-e.md),
[Gate F](../../roadmap/rrflow-1.0/gate-f.md),
[Gate H](../../roadmap/rrflow-1.0/gate-h.md), and
[Gate J](../../roadmap/rrflow-1.0/gate-j.md).

## Closure evidence

Preserve every safety behavior enumerated in the distributed contract through pure contracts
plus an injected engine-compiled proposal/replica-apply port; use installed identities and
the sole rrflowKV format; remove the catalogue, direct openers, raw mutation ingress,
cursor-zero scan, private authority, old/defaulted shapes, and synthetic traces with no
wrapper; retain bounded staging only beneath engine-owned jobs/receipts; rerun the complete
single-node semantic/graph/BM25/vector/RRF/reasoning/Arrow/DataFusion corpus through
consensus, then pass deterministic faults, resource limits, reshard/recovery, security
rotation, every public surface, and sustained independent-host qualification. Before
`clustered_server` can be emitted, the roadmap owner must add that distributed gate and
accept its exact evidence.
