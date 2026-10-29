# POAM-044 — rrflowQL refusals are unstructured, and queries cannot be checked without running

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-044`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Observed at engine revision `86adada6` by reading the engine and through
Connectome's real-engine process tests (`rrd::client::process::language` and
`live_subscriptions_are_bounded_and_refusals_end_them`):

- **No query is checked without running it.** No public operation parses or
  binds a query without executing it. `EXPLAIN CONTRACT` is carried into the
  bound query and its canonical text (`rrd-query/src/syntax.rs`,
  `plan.rs`), but nothing reads it afterwards, so the query executes in full
  and returns rows.
- **A refusal's location is only in debug text.** `ServiceError`'s display is
  its `Debug` form (`rrd-engine/src/engine/error.rs`), and `api_error`
  (`rrd-server/src/http/response.rs`) puts that display in `ErrorBody.message`
  with empty `details`. A parse refusal therefore reaches every client as
  `Query("query parse error at byte N: …")`, with the parser's message
  escaped. Its byte offset can only be recovered by parsing that text.
- **Some offsets are not where the problem is.** `Query::validate` reports its
  refusals at byte 0, so `LIMIT 0` is refused at the start of the query. The
  parser reports "end of query" one byte past the start of the last token,
  which can fall inside a multi-byte character.
- **A live query is checked only when a change arrives.** A durable live-query
  subscription opens without parsing its query (`open_subscription` in
  `rrd-engine/src/engine/subscription.rs`); `poll_live_query_page` parses it on
  each poll. An invalid query is refused at the first change it would have
  seen, not when it is opened.

The cited code is unchanged at `development/main` `618269b4`.

## Impact

- An editor cannot tell an operator that a query is invalid as they type
  without running the query.
- Each client has to parse the engine's debug text to place a refusal, and
  rebuild that parsing whenever the engine's internal error type changes.
- A live subscription can be accepted and then end, possibly much later, for a
  reason known when it was opened.

## Owning gates

E-05, H-03, H-04

E-05 owns explain plans; an explanation is produced without executing the
query. H-03 owns live-query evaluation, including validation when a live query
is opened. H-04 owns one catalogue-derived operation and error contract across
surfaces, including structured refusal details.

Roadmap owners: [Gate E](../../roadmap/rrflow-1.0/gate-e.md) and
[Gate H](../../roadmap/rrflow-1.0/gate-h.md).

## Closure evidence

- A public operation parses and binds a query against a read stamp and returns
  its plan, or its refusal, without executing it. Physical counters show no
  rows read.
- A query refusal carries structured details (the refusing stage, the offset
  and the reason) in the error contract, golden-tested across HTTP, WebSocket
  and the SDKs. The message no longer depends on a `Debug` rendering.
- A validation refusal reports the clause it refuses. "End of query" reports
  the end of the input.
- Opening a live query with an invalid query is refused at open.
