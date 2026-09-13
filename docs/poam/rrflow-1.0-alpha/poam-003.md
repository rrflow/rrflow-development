# POAM-003 — accepted single-node physical owner and readers

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-003`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

C-05a removes the pre-1.0 batch, manifest, and segment readers plus missing-accumulator
cursor-zero reconstruction; C-05c proves the Fjall selector, migration runtime, and
alternate stores are absent and that the active alpha reaches `rrd-lsm` through one required
production owner, `rrd-store`; C-05e removes vector artifact catalogue v1 and its alternate
identity digest; C-05f requires one explicit nonempty schema table map and removes model
inference; C-05g requires one collection plus named-vector address through mutation,
inference, persistence, commit identity, and artifacts; C-05h restricts generic artifacts to
exact/compact/HNSW, rejects generic quantization, removes the TurboQuant ensure adapter and
replay suppression, and restores only active explicit-lifecycle quantization into the shared
planner. The optional post-alpha OpenRaft implementation remains POAM-023.

## Impact

The single-node alpha now has one required physical owner and one accepted shape for each
audited schema/vector input; C-06 separately accepts the row format, while the POAM-023
distributed gap remains explicit.

## Owning gates

C-05, J-01

## Closure evidence

Negative pre-1.0/omission/generic-quantization tests, fresh rrflowKV reopen, lifecycle
search/reopen/retire, exact source/dependency guards, all 28 architecture guards, and the
complete default workspace test/check/strict-Clippy matrices pass. Retain those guards while
J-01 performs final release-source closure; keep post-alpha cluster code excluded under
POAM-023.
