# Gate B — freeze public and model-neutral contracts

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/gate-b`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [x] | B-01 | Define install plan, installation result, attunement plan, job, phase checkpoint, status, resume, cancel, and verification envelopes. | `rrd-contract` | Golden JSON and generated schema tests cover every state transition and reject skipped phases or mismatched digests. |
| [x] | B-02 | Define `RouterBackendDescriptor`, `RouteStepRequest`, and the `select_recipe`, `advance_branch`, and `request_context` decision variants. | `rrd-contract` | Golden vectors prove model/provider neutrality, strict fields, bounded inputs, and stable digests. |
| [x] | B-03 | Define the LFG model-manifest handshake: model/tokenizer digests, routing schema digest, capabilities, limits, runtime, and quantization. | `rrd-contract`, `rrd-inference` | Mismatched contract, model, tokenizer, or resource declarations fail before inference. |
| [x] | B-04 | Define one multiplexed WebSocket frame protocol for authenticated request/response, cancellation, subscription, ACK, and backpressure. | `rrd-contract` | Codec golden tests prove correlation, ordering, limits, unknown-frame rejection, and reconnect resume coordinates. |
| [x] | B-05 | Define GraphQL as a schema-derived ingress adapter that lowers into the same bound RRFlow query representation. | `rrd-contract`, `rrd-query` | Equivalence fixtures show GraphQL and rrflowQL produce the same bound logical request; the shared engine authorization path is unchanged and no second executor exists. |


## [Accepted evidence](gate-b-evidence.md)

## Exit condition

Gate B exits only when other languages and LFG can implement the contracts from
golden vectors without importing Rust internals.
