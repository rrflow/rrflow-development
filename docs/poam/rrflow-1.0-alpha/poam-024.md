# POAM-024 — Kubernetes-controller authority

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-024`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

`rrd-kubernetes` is an outward deployment controller implemented as a parallel desired-state
and lifecycle authority. Its CRD directly owns instance/image/storage/bootstrap inputs and
caller time; renderer code hardcodes commands, paths, resources, and two independent
initialization paths; the controller watches all namespaces, owns only StatefulSet events,
unconditionally force-applies fields, declares application readiness from one ready
replica/TCP sockets, labels desired JSON as applied state, and removes its finalizer after
background delete requests without engine-prepared retention, backup, fence, absence, or
receipt proof. Checked tests are local render/schema/string assertions; no controller or
real API-server test exists, and the example image is not deployable.

## Impact

Kubernetes edits or controller restarts can create installation, policy, deployment,
completion, and deletion truth outside `RrdEngine`; foreign fields can be stolen; an
uninitialized or wrong RRD can appear ready; data protection can be released before effects
complete; cluster-wide permissions and namespace-label reachability can exceed the intended
project boundary; and passing manifests can falsely qualify the persistent
graph/index/vector/Arrow/DataFusion reasoning engine.

## Owning gates

A-07, B-04, C-02, C-03, D-01, D-02, E-01 through E-05, F-01 through F-05, G-04, G-05, H-01
through H-05, H-07, J-01 through J-05

## Closure evidence

Move useful mechanics into outward `rrflow-kubernetes` with no forwarding crate; accept only
the sealed D-01 cold-start handoff or an engine-prepared fenced effect plan; return bounded
observations/receipts for engine acceptance; replace phase/TCP status with standard
conditions and authenticated identity/digest readiness; use namespace-scoped least privilege
and explicit SSA conflict handling; gate finalization on retention/backup/absence/receipt
proof; remove both old bootstrap paths and hardcoded authority; then pass closed-contract,
real API-server, watch/relist/restart, effect-gap, field-conflict, RBAC/Secret/network,
retained-delete/recovery, clean offline install, rrflowKV crash/reopen, resource, and
complete document/graph/BM25/vector/RRF/reasoning/Arrow/DataFusion public-endpoint corpora.
