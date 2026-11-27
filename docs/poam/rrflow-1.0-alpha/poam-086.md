# POAM-086 — the Raft failover test sometimes elects no leader in time

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-086`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-18, full-matrix run 44398147981 failed `optional feature (rrd-cluster)` (job 135133823563) on a branch that changes no cluster code. In `rrflow/cx/tests/openraft_cluster.rs`, `real_consensus_elects_fails_over_installs_snapshot_and_changes_membership` partitions node 1 from nodes 2 and 3, sleeps 400 ms, triggers an election on node 2 and waits 15 seconds for node 2 to lead. It timed out at line 307: node 2 was still a candidate in term 2 with an uncommitted vote and no leader.

The same job passed in the six full runs before it (44320993635, 44123191670, 44104264596, 43886905368, 43878604248, 43715666883). The cause is not known. It is either a test that assumes an election finishes within its wait on a loaded runner, or a real case where the majority side does not elect after a partition.

## Impact

Failover is the property the cluster tier exists for. Until the cause is known, a passing run does not show that the majority side always elects, and a failing run blocks the full matrix for an unrelated change.

## Owning gates

C records the optional, post-alpha consensus implementation in [its evidence record](../../roadmap/rrflow-1.0/gate-c-evidence.md).

Roadmap owner: [Gate C](../../roadmap/rrflow-1.0/gate-c.md).

## Closure evidence

Closed when the cause is identified from the node metrics and logs of a failing run, fixed in the test or in the election path, and the test passes in repeated full runs.
