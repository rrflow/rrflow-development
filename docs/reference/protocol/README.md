# RRFlow protocol reference

**Status:** active protocol-reference index
**Coordinate:** `rrflow://rrflow-instance/data/reference-index/protocol`
**Owner:** public RRD transport and protocol-contract discovery; linked from `docs/reference/README.md`

These records describe the public transport boundary implemented by RRD. They
do not own engine semantics, release status, or client-specific behavior.

| Subject | Durable warp | Checkout record | State |
|---|---|---|---|
| HTTP and WebSocket server | [`rrflow://rrflow-instance/data/reference/protocol/server`](rrflow://rrflow-instance/data/reference/protocol/server) | [`server.md`](server.md) | implemented transport foundation; single-scope transactions, multiplexing, tracing, and release qualification remain open |
| Durable subscriptions | [`rrflow://rrflow-instance/data/reference/protocol/subscriptions`](rrflow://rrflow-instance/data/reference/protocol/subscriptions) | [`subscriptions.md`](subscriptions.md) | implemented durable ACK/replay foundation; multiplexing and commit-impact live evaluation remain open |
