# Gate G — connect LFG without creating another engine

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/gate-g`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | G-01 | Add a provider-neutral `RouterBackend` capability separate from `EmbeddingBackend`; implement LFG as one adapter. | `rrd-inference`, `rrd-engine` | Fake/reference adapter and LFG adapter pass the same descriptor, bounds, timeout, invalid-output, and digest checks. |
| [ ] | G-02 | Build a bounded route packet from one stamped tree, eligible recipes, verified observations, and allowed query fields. Bind the authenticated principal, current provider-representation edge, resolved durable seat, policy revision, and authorization digest at that same read coordinate; a caller-supplied actor label is never identity evidence. | `rrd-engine` | Golden packet and denial corpus prove representation establishes attribution rather than permission, stale/revoked/foreign representations fail closed, and the model sees no raw KV keys, secrets, hidden reasoning, unauthorized fields, or unbounded workspace content. |
| [ ] | G-03 | Grammar-constrain LFG to the three routing decisions and validate again after decoding. | LFG adapter | Corpus includes valid, malformed, unknown-recipe, unauthorized-query, stale-cursor, and prompt-injection cases; invalid decisions produce no mutation. |
| [ ] | G-04 | Execute recipe selection and branch navigation on the fast path without rrflowQL/DataFusion; keep deterministic predicates and CAS mutation in `RrdEngine`, and commit the resolved seat attribution with the resulting tree state, audit, and causal trace. | `rrd-engine`, `rrd-store` | Trace and physical-plan evidence show bounded rrflowKV operations, no DataFusion plan, conflict denial, no actor-string impersonation, and correct attributed state after reopen. |
| [ ] | G-05 | Lower `request_context` into semantic rrflowQL/context intent while leaving physical access selection to the engine. | `rrd-engine`, `rrd-query` | LFG cannot select an index/backend; resulting plan is authorized, stamped, budgeted, and equivalent to a typed SDK request. |
| [ ] | G-06 | Publish model and storage latency separately with task-success, routing-accuracy, invalid-decision, and escalation metrics. | evaluation harness | Reproducible hardware/model manifest and raw samples support every reported latency or quality claim. |

## Exit condition

Gate G exits only when the trained LFG artifact passes the conformance corpus
and can steer a persisted tree without direct storage or planner authority.
