# RRFlow data reference

**Status:** active stable-reference index
**Coordinate:** `rrflow://rrflow-instance/data/reference-index/data`
**Owner:** logical model, schema, temporal value, and multi-model contract discovery; linked from `docs/reference/README.md`

These records define canonical logical data semantics shared by rrflowMX and
rrflowKV. They do not claim that current physical storage or query access is
native, incremental, or complete. The
[engine data-flow architecture](../../architecture/engine-data-flow.md) owns
the target substrate, and roadmap Gates C and E own its delivery evidence.

| Subject | Durable warp | Checkout record | State |
|---|---|---|---|
| Logical schema catalogue | [`rrflow://rrflow-instance/data/reference/data/schema-catalogue`](rrflow://rrflow-instance/data/reference/data/schema-catalogue) | [`schema-catalogue.md`](schema-catalogue.md) | implemented logical validation and persistence; pre-1.0 derivation and replay paths remain to be removed |
