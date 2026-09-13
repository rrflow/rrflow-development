# POAM-022 — deployment facts and profile semantics

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-022`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Public `DeploymentMode` places `rrflow_mx`, `embedded`, `local_daemon`, `edge`, `remote`,
and `distributed` in one scalar even though they represent storage, composition, process
locality, an offline artifact, a client-relative transport view, and an unavailable cluster
claim. `RrdEngine` infers the value from persistent-root presence, the server infers it from
TLS, generated SDKs repeat it, and the shared “deployment conformance” fixture contains only
two documents plus one exact-ID query; the edge test never enters `RrdEngine`.

## Impact

The same installed instance can report a different supposed mode when TLS changes or a
different client observes it; a server over rrflowMX cannot be described truthfully;
unsupported cluster/artifact values are advertised as peers of real engine profiles; and
passing seed lookups can falsely qualify rrflowMX/rrflowKV, DataFusion, graph/index/recall,
reasoning, transport, or deployment equivalence.

## Owning gates

A-07, B-04, C-02 through C-04, D-01, E-01 through E-05, F-01 through F-05, G-04, G-05, H-01
through H-05, H-07, J-01 through J-05

## Closure evidence

Replace the scalar with independently installed deployment-form, storage-profile,
endpoint-presentation, and security facts; remove root/TLS/location inference and
speculative active values with no alias; separate storage semantics, rrflowKV durability,
deployment-form, endpoint, cluster, and derived-artifact corpora; then prove identical
authorized transactions, temporal graph/index/vector/RRF behavior, stamped streamed
Arrow/DataFusion results, persisted reasoning/context/evidence, delivery, limits, and traces
across valid profiles, with rrflowKV-only crash/reopen and clean installed real-process
evidence.
