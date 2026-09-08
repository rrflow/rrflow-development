# RRFlow SDK reference

**Status:** active SDK-reference index
**Coordinate:** `rrflow://rrflow-instance/data/reference-index/sdk`
**Owner:** supported language-client behavior and conformance discovery; linked from `docs/reference/README.md`

SDKs are outward clients of the public RRD protocol. They may provide typed
construction, transport, authentication, cancellation, retry, subscription,
and result APIs, but they do not own engine semantics or open rrflowMX,
rrflowKV, rrflowQL, DataFusion, graph, index, vector, reasoning, attunement, or
automation state directly. A generated schema or compiling package is not a
supported SDK until its real-process conformance rows pass.

| Language | Durable warp | Checkout record | State |
|---|---|---|---|
| Rust | [`rrflow://rrflow-instance/data/reference/sdk/rust`](rrflow://rrflow-instance/data/reference/sdk/rust) | [`rust.md`](rust.md) | implemented client foundation; operation, validation, retry, cancellation, subscription, resolver, and release conformance remain open |

Other language records enter this index only after their complete flat record,
generated implementation, shared corpus behavior, and toolchain execution have
been reviewed. The [public-contract reference](../protocol/public-contract.md)
owns the language-neutral operation and wire authority; the
[release roadmap](../../roadmap/rrflow-1.0.md) alone owns completion.
