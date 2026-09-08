# RRFlow deployment reference

**Status:** active stable-reference index
**Coordinate:** `rrflow://rrflow-instance/data/reference-index/deployment`
**Owner:** deployment-profile classification, executable packaging, and deployment contracts; linked from `docs/reference/README.md`

Deployment records describe how the same RRFlow authority is composed,
presented, and packaged. A profile or artifact cannot introduce another
engine, database, query planner, or configuration authority.

| Subject | Durable warp | Checkout record | State |
|---|---|---|---|
| Deployment profiles | [`rrflow://rrflow-instance/data/reference/deployment/modes`](rrflow://rrflow-instance/data/reference/deployment/modes) | [`modes.md`](modes.md) | accepted independent deployment-form, storage-profile, and endpoint-presentation classification; implementation convergence remains open |
| Offline edge artifact CLI | [`rrflow://rrflow-instance/data/reference/deployment/edge`](rrflow://rrflow-instance/data/reference/deployment/edge) | [`edge.md`](edge.md) | narrow executable artifact proof; not a persistent project estate or release installer |
| Local RRD process adapter | [`rrflow://rrflow-instance/data/reference/deployment/local-process-driver`](rrflow://rrflow-instance/data/reference/deployment/local-process-driver) | [`local-process-driver.md`](local-process-driver.md) | target launch/readiness/shutdown contract; current implementation requires direct convergence |
