# RRFlow query reference

**Status:** active stable-reference index
**Coordinate:** `rrflow://rrflow-instance/data/reference-index/query`
**Owner:** rrflowQL syntax, binding, planning, and execution-reference discovery; linked from `docs/reference/README.md`

These records describe implemented query behavior without declaring the target
storage or analytical architecture complete. The
[engine data-flow architecture](../../architecture/engine-data-flow.md) owns
the target fast and analytical paths, the
[release roadmap](../../roadmap/rrflow-1.0.md) owns delivery status, and the
[POA&M](../../poam/rrflow-1.0-alpha.md) owns verified gaps.

| Subject | Durable warp | Checkout record | State |
|---|---|---|---|
| Multi-model reads | [`rrflow://rrflow-instance/data/reference/query/multi-model`](rrflow://rrflow-instance/data/reference/query/multi-model) | [`multi-model.md`](multi-model.md) | implemented breadth over materialized snapshots; native access paths remain open |
