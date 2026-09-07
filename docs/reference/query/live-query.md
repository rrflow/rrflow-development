# rrflowQL live queries

**Status:** active implementation reference; commit-impact evaluation remains incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/query/live-query`
**Owner:** deterministic resumable semantic-delta polling

This record describes the current query-layer behavior. It does not declare
the live reasoning and recall loop complete. Roadmap
[H-03](../../roadmap/rrflow-1.0.md#gate-h--complete-context-feedback-delivery-tracing-and-connectome)
owns the replacement of broad snapshot polling with commit-impact evaluation,
while the [subscription contract](../../rrd-live-subscriptions-v1.md) describes
the separate durable delivery authority.

## Poll contract

A live query is an ordinary typed rrflowQL read with explicit `AT VALID` and
`KNOWN HEAD`. The caller separately supplies the last consumed runtime cursor.
The query executor captures one catalogue and immutable head, evaluates the
same bound query at the resume and head cursors, and compares rows by stable
identity.

The result contains deterministically ordered `added`,
`updated { before, after }`, and `removed` rows together with `from_cursor`,
`through_cursor`, and `head_cursor`. The caller advances only to
`through_cursor`; repeating that cursor returns an empty idempotent result
until authoritative runtime state advances.

The query digest binds the contract version, canonical query, and typed
parameters. A resume cursor beyond the captured head, a fixed `KNOWN` clause,
truncated snapshot execution, duplicate row identities, a zero delta budget,
or a delta exceeding `max_delta_rows` fails closed.

## Current execution boundary

`poll_live_query` currently executes the query twice: once at the resume
cursor and once at the captured head. Each execution follows the ordinary
bind, plan, and execute path, collects all returned rows into an identity map,
and then computes the semantic difference. rrflowMX and rrflowKV return the
same tested delta for additions, updates, removals, membership changes,
idempotent replay, and budget rejection.

This implementation is deterministic, but it is not the target live-query engine. It
reconstructs and materializes both snapshots, does not use changed-key or
index-impact analysis, and does not incrementally maintain the result set.
Persistent delivery, ACK state, fencing, retention, and WebSocket replay are
transport and engine concerns; they do not change these query-computation
costs.

H-03 must evaluate committed keys against graph, scalar, BM25, and vector
access paths created under Gate E, publish only affected semantic deltas from
the authoritative commit cursor, preserve one read stamp, and prove bounded
resource use and restart-safe delivery. Until that evidence exists, current
live queries are implemented but not the completed persistent reasoning and
recall feedback path.
