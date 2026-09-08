# RRFlow distributed reference

**Status:** active stable-reference index; distributed execution is not available
**Coordinate:** `rrflow://rrflow-instance/data/reference-index/distributed`
**Owner:** distributed contract discovery; linked from `docs/reference/README.md`

These records define how one installed RRD instance may eventually execute
across several nodes without becoming another database, transaction, graph,
query, reasoning, or installation authority. They do not advertise a clustered
deployment or change roadmap status.

| Subject | Durable warp | Checkout record | State |
|---|---|---|---|
| Cluster consistency, replication, recovery, and qualification | [`rrflow://rrflow-instance/data/reference/distributed/cluster-contract`](rrflow://rrflow-instance/data/reference/distributed/cluster-contract) | [`cluster-contract.md`](cluster-contract.md) | target contract; current package is non-conforming implementation inventory |
