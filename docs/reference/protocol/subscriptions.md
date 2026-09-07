# RRD durable subscriptions

**Status:** active implementation reference; multiplexing and commit-impact delivery remain incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/protocol/subscriptions`
**Owner:** durable changefeed and live-query subscription state, replay, acknowledgement, and current WebSocket delivery

RRD exposes changefeeds and semantic rrflowQL deltas through one durable
subscription authority composed by `RrdEngine`. A subscription does not own a
second event log, query executor, transaction coordinator, or client
lifecycle. Its delivery coordinates refer to the authoritative runtime commit
cursor.

The [server reference](server.md) owns HTTP/WebSocket process and transport
profiles. The [live-query reference](../query/live-query.md) owns current
semantic-delta computation. Roadmap
[B-04](../../roadmap/rrflow-1.0.md#gate-b--freeze-public-and-model-neutral-contracts)
owns the target multiplexed WebSocket protocol, and
[H-03](../../roadmap/rrflow-1.0.md#gate-h--prove-context-flow-feedback-live-delivery-and-connectome)
owns commit-impact live-query evaluation. This record describes implemented
subscription behavior without claiming either gate complete.

## Public operations and stream definition

The generated endpoint catalogue defines three current operations:

- `POST /v1/subscriptions/open` creates one immutable `changefeed` or
  `live_query` stream definition plus its delivery policy.
- `GET /v1/subscriptions/{subscription}/stream` upgrades one authenticated
  connection to a subscription-specific WebSocket.
- `POST /v1/subscriptions/close` performs an idempotent administrative close.

An open request binds the subscription identity, stream and scope, starting
cursor, batch size, maximum in-flight deliveries, retention-cursor window,
lease duration, and heartbeat interval. A live-query stream additionally binds
the rrflowQL string, typed parameters, query budget, and maximum semantic-delta
rows. Contract validation rejects unknown fields and values outside the
declared bounds; `max_in_flight` is currently limited to 1 through 64.

The stream digest is stable for the accepted definition. Reusing the open
idempotency key with the same request returns the existing subscription;
changing the request conflicts. There is no update operation that silently
changes what an existing subscription means.

## Durable state, ordering, and replay

The runtime commit cursor is the only data-delivery order. Changefeed pages
retain the underlying change identity, commit identity and ordinal, previous
change digest, scope, actor, and event time. Live-query frames cover the range
from the durable acknowledged cursor through one captured runtime head.

Current subscription state includes:

- the owning session and immutable stream definition;
- the durable acknowledged cursor and configured retention window;
- the active connection generation and lease expiry;
- the next delivery sequence; and
- the bounded outstanding-delivery intervals for the active generation.

The engine currently encodes this pre-release state as a versioned JSON control
record and replaces it with compare-and-swap while appending a digest-chained
control-journal entry. That physical representation is implementation
inventory, not a public persistence format. A delivery interval is committed
before its frame is returned for transport.

An ACK must identify an exact outstanding delivery by connection generation,
delivery sequence, and through-cursor. It advances the durable cursor to that
delivery and cumulatively removes earlier outstanding deliveries. A dropped
connection does not acknowledge data. Reconnect increments the fencing
generation, clears the abandoned in-flight window, and resumes from the last
durable ACK; a frame from the replaced generation fails closed. The tested
rrflowKV path preserves this behavior across engine restart.

The configured retention floor is `head - retention_cursor_window`. Open,
connect, and delivery reject a resume cursor below that floor or beyond the
head. This is a logical replay bound, not evidence that runtime history has
been physically pruned. ACK and client heartbeat frames renew the durable
lease. An expired or closed subscription cannot reconnect or deliver.

## Backpressure and WebSocket frames

The outstanding window cannot exceed `max_in_flight`. Once full,
`RrdEngine` returns subscription backpressure; the server stops requesting new
delivery while the same socket remains able to receive ACK, heartbeat, or
close frames. Changefeed `batch_size` uses the bounded changefeed page
contract. Live queries use their separate execution and delta-row budgets.

Clients send `ack`, `heartbeat`, and `close`. The server sends `opened`,
`changefeed`, `live_query`, `heartbeat`, `acknowledged`, `error`, and `closed`.
The current server accepts JSON text frames, limits WebSocket messages and
frames to 64 KiB, responds to ping, rejects malformed JSON with a structured
error, and closes on unsupported binary application data.

Delivery is currently checked on the configured heartbeat interval rather
than awakened directly by each commit. Each socket carries exactly one
subscription. `changes/follow` and live-query polling also remain callable as
separate bounded HTTP surfaces. These are current pre-release paths, not a
compatibility promise; B-04 and H-03 own their direct convergence into the one
completed delivery model.

## Authentication and authorization

Opening, connecting, acknowledging, and closing use distinct deny-by-default
security actions. Open, connect, delivery, ACK, and stream close also enforce
the underlying `changefeed_follow` or `query_live_poll` permission. The stream
scope must equal the configured instance, and current ownership is bound to
the exact session rather than merely its principal.

The WebSocket upgrade uses the same session and bearer headers as ordinary RRD
requests. Loopback uses `ws://`. The real-process Rust client test proves that
the configured remote profile requires a client certificate, rejects a wrong
server name, and carries subscription delivery over `wss://` when the mutual
TLS identities and trust roots are valid. Certificate rotation, revocation,
external identity integration, and a complete untrusted-client-chain matrix
remain unqualified.

## Current semantic live-query cost

The delivery layer does not make live-query computation incremental.
`poll_live_query` currently captures a head, executes the same bound query at
the acknowledged and head cursors, materializes both result sets by stable row
identity, and computes deterministic added, updated, and removed rows. It
fails closed on fixed `KNOWN` queries, cursors beyond head, truncation,
duplicate identities, or delta-budget excess.

H-03 must replace this two-snapshot work with commit-impact evaluation over the
native graph, scalar, BM25, and vector access paths, while retaining cursor
ordering, exact predicate semantics, authorization, bounded resources, ACK
replay, and restart behavior.

## Characterization evidence and open qualification

| Boundary | Current evidence | Not established |
|---|---|---|
| Public contract | One focused test covers changefeed/live-query shapes, bounds, round trips, actions, and catalogue registration. | The B-04 multiplexed frame envelope or cross-language golden conformance. |
| Engine | Five focused tests cover durable open/reopen, idempotent open, bounded delivery, cumulative ACK, generation fencing, retention, ownership, dual authorization, one global cursor chain, semantic live delta, and close. | Commit-triggered wakeup, principal-level transfer, or H-03 impact evaluation. |
| Query | Two tests prove deterministic semantic deltas and failure behavior across rrflowMX and rrflowKV. | Direct stamped access paths, incremental result maintenance, or bounded cost independent of full snapshots. |
| Rust client/server | Real-process tests cover loopback push, ACK, disconnect/reconnect replay, close, audit actions, and mutual-TLS WSS delivery. | Generated SDK parity, multiplexed cancellation/request traffic, or release deployment qualification. |

The focused characterization commands are:

```text
cargo test -p rrd-contract durable_subscription_contract_bounds_retention_backpressure_and_stream_shape --locked
cargo test -p rrd-engine subscription --locked
cargo test -p rrd-query --test live_query --locked
cargo test -p rrd-client --test real_server --locked
```

## Implementation anchors

- Public stream and frame contract: `crates/transport/rrd-contract`
- Durable subscription authority: `crates/authority/rrd-engine/src/engine/subscription.rs`
- Current semantic delta computation: `crates/compute/rrd-query/src/live.rs`
- WebSocket adapter: `crates/transport/rrd-server/src/http/websocket.rs`
- Supported Rust client: `crates/transport/rrd-client`
