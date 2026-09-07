# RRFlow vector reference

**Status:** active stable-reference index
**Coordinate:** `rrflow://rrflow-instance/data/reference-index/vector`
**Owner:** vector collection, point, search, projection, quantization, and memory-tier contract discovery; linked from `docs/reference/README.md`

RRFlow vectors are one multimodal branch of the authoritative runtime, not a
sidecar database. These records separate tested logical and artifact behavior
from the native persistent access paths still required by roadmap Gates C, E,
and F.

| Subject | Durable warp | Checkout record | State |
|---|---|---|---|
| Collections and points | [`rrflow://rrflow-instance/data/reference/vector/collections`](rrflow://rrflow-instance/data/reference/vector/collections) | [`collections.md`](collections.md) | persistent administration and point semantics; native point/payload access remains open |
| Search semantics and execution | [`rrflow://rrflow-instance/data/reference/vector/search`](rrflow://rrflow-instance/data/reference/vector/search) | [`search.md`](search.md) | exact oracle and rebuildable HNSW execution exist; direct incremental persistence remains open |
| HNSW projection | [`rrflow://rrflow-instance/data/reference/vector/hnsw-projection`](rrflow://rrflow-instance/data/reference/vector/hnsw-projection) | [`hnsw-projection.md`](hnsw-projection.md) | deterministic immutable generations and exact overlay exist; compact native graph storage remains open |
