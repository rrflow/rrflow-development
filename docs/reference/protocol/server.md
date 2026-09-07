# RRD HTTP and WebSocket server

**Status:** active implementation reference; alpha protocol convergence remains incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/protocol/server`
**Owner:** RRD HTTP/WebSocket process boundary, transport profiles, route discovery, and current conformance

RRD is RRFlow's embedded and daemon runtime. The `rrd-server` package exposes
public HTTP and WebSocket operations over the same `RrdEngine` used by
embedded callers. It is not a second engine, an HTTP wrapper around the CLI,
or an extension of the MCP protocol. HTTP, WebSocket, SDK, MCP, CLI, and
Connectome are clients of the engine's public contracts and own no persistence
or lifecycle semantics.

The [system overview](../../architecture/system-overview.md) owns component and
authority boundaries. The
[engine data-flow record](../../architecture/engine-data-flow.md) owns target
transactional, query, Arrow/DataFusion, context, and trace flow. The
[release roadmap](../../roadmap/rrflow-1.0.md) owns implementation order and
completion evidence. This record describes the server that exists now and
names the gaps without promoting them to completed behavior.

## Executable protocol authority

`rrd-contract::endpoint_catalogue()` is the authoritative public operation
catalogue. Every descriptor binds an operation name, HTTP method and path,
authentication mode, mutation classification, security action, and public
request and response types. The server router resolves operations from that
catalogue, and a router test requires the catalogue and executable dispatch
sets to match exactly.

Two unauthenticated discovery operations project the same authority:

- `GET /v1/schema/endpoints` returns the sorted runtime catalogue.
- `GET /v1/schema/openapi` returns deterministic OpenAPI 3.1 generated from
  the catalogue and public Rust wire types.

This record deliberately does not copy the complete endpoint list. Adding or
changing an operation starts in `rrd-contract`, updates generated surfaces,
and must pass catalogue/router/OpenAPI/SDK parity. Documentation cannot make an
unimplemented route public.

Every JSON operation returns `ResponseEnvelope`; failures use the closed
`ErrorCode` vocabulary. Request envelopes carry request and operation IDs, an
optional deadline, an operation-specific resource path, and an idempotency key
for mutation. Public payloads are contract types rather than private kernel or
storage types.

## Client bootstrap

A client discovers one configured instance in this order:

```text
GET /v1/health/live
GET /v1/health/ready
GET /v1/capabilities
GET /v1/schema/endpoints   # when operation discovery is required
GET /v1/schema/openapi     # when a generated HTTP description is required
```

Liveness proves only that the process answers. Readiness opens the configured
instance and checks its current format and operational state. Capabilities
characterize the active transport and supported operations; they are not
release evidence. Connectome and SDKs must reject an unexpected protocol
version or instance resource rather than inferring compatibility.

The current local process is initialized and started with:

```text
cargo run -p rrd-server -- initialize --root PROJECT --instance INSTANCE
cargo run -p rrd-server -- --root PROJECT --bind 127.0.0.1:9477
```

`initialize` is a narrow instance bootstrap command. It is not the planned
project installation and attunement workflow owned by roadmap Gate D.

## Transport profiles

| Profile | Current enforced boundary | Still open |
|---|---|---|
| Local daemon | Clear HTTP may bind only to an explicit loopback address. Library and binary startup reject non-loopback cleartext before opening a listener. | Released installation, service supervision, and client conformance remain Gate D/J work. |
| Remote | A non-loopback listener requires TLS 1.3, a server certificate and key, a client CA, mandatory client-certificate validation, and initialized RRFlow security state. | Certificate reload, revocation handling, external identity-provider/JWK adapters, and deployment-secret integration are not complete. |
| WebSocket | The authenticated subscription stream limits messages and frames to 64 KiB and uses durable subscription state, cumulative acknowledgements, generation fencing, leases, and bounded in-flight delivery. | One multiplexed connection carrying query, mutation, subscription, cancellation, and trace streams is Gate B-04 work. |

The current explicit remote invocation supplies all TLS inputs together:

```text
rrd-server --root PROJECT --bind 0.0.0.0:9477 \
  --tls-cert SERVER_CHAIN.pem --tls-key SERVER_KEY.pem \
  --tls-client-ca CLIENT_CA.pem \
  --jwt-key-file JWT_SIGNING_KEY
```

The current mutual-TLS loop serves HTTP/1.1 and handles one request per
connection. This is an implementation limit, not the target connection model.
A mesh may resolve and carry the endpoint, but mesh reachability does not
authenticate the RRD instance or grant an engine capability.

## Request boundary

An ordinary HTTP request follows this implemented sequence:

```text
bounded body and route
        |
RequestEnvelope validation
  content type, IDs, instance resource, deadline, idempotency
        |
credential or session extraction
        |
RrdEngine::begin_invocation
  authentication + exact action/resource authorization + audit reservation
        |
typed RrdEngine operation
        |
RrdEngine::complete_invocation
  outcome + response digest
        |
ResponseEnvelope
```

The HTTP boundary reads at most one MiB, rejects an elapsed deadline before
the operation, and rejects a resource that does not target the configured
instance. Authenticated routes require a matching session ID and bearer lease.
On a secured instance, session creation accepts a principal/API key or a
configured RRD-issued JWT; each later operation rechecks the stored principal,
credential revision, exact action, and resource policy through `RrdEngine`.

With no initialized security authority, the server advertises an anonymous
loopback development mode. Remote TLS and JWT configuration require initialized
security. That local mode is current characterization, not proof of the alpha
security or installation outcome.

The database-local token derivation secret is generated from operating-system
entropy and is permission-checked on Unix. The optional token-key and JWT-key
files are read under bounded file rules. Persisted session and audit records
store digests and credential revisions, not raw API keys, JWTs, signing keys,
or bearer lease tokens.

## Public operation families

The generated catalogue currently covers these capability families:

| Family | Server responsibility | Semantic authority |
|---|---|---|
| Inspection | Liveness, readiness, capabilities, endpoint catalogue, OpenAPI, and diagnostics. | `RrdEngine` and public contract projections. |
| Identity and control | Session create, renew, close, estate read, audit read, and capability administration. | RRFlow security and `RrdEngine`. |
| Transactions | Begin, preview, commit, and abort typed mutation work. | `RrdEngine`; rrflowKV owns durable physical commit. |
| Query and context | rrflowQL reads, live-query poll, index administration, and context assembly. | `RrdEngine`, rrflowQL, and the selected storage profile. |
| Recall | Vector collection, point, and search operations. | RRFlow's vector subsystem under `RrdEngine`; never a separate database authority. |
| Delivery | Changefeed read/follow and durable WebSocket subscriptions. | Committed engine state and durable subscription coordinates. |
| Recovery | Logical backup catalogue, backup creation, and restore into a generated inactive root. | Engine-authorized logical recovery; no active-root switch. |

Existence in this surface proves only that a typed path is callable. Native
graph, scalar, BM25, vector, Arrow/DataFusion, and reasoning-tree execution are
qualified by their owning Gate E through H evidence, not by the route.

## Idempotency and lifecycle durability

Mutation envelopes require an idempotency key. The engine durably binds the
instance, session, key, operation digest, and accepted result identity. An
exact replay returns the accepted result; reuse with a different digest
conflicts. Runtime content identity is an independent guard.

Session and transaction control state is durable rather than process-local.
Lifecycle transitions use compare-and-swap materialization plus a sequenced,
digest-chained journal. Renewal, expiry, close, prepare, commit intent,
terminal commit, and abort have explicit states. A disconnect is not a commit
or abort decision, and cleanup does not delete canonical data.

The current commit path is intentionally characterized without overstating
atomicity:

```text
durable transaction prepare
        -> authoritative runtime commit and receipt
        -> durable terminal transaction state
```

Restart reconciliation uses the frozen operation identity and runtime receipt
to close either process-failure gap without duplicating data. This is not one
cross-keyspace atomic transaction. Gate C-03 owns the target atomic write batch
for canonical model changes, temporal versions, graph adjacency, synchronous
indexes, runtime-log state, and durable projection deltas.

The public transaction contract also still accepts separate `claims` and
`data` scopes. That is conflicting pre-release inventory, not a compatibility
promise. Gates C-03, H-04, and J-01 must preserve claim semantics inside the
one canonical multi-model transaction and remove the claim-only successful
path, contract branch, and fixtures.

## Query, context, and delivery limits

Query and context operations pass through the engine's binder, planner,
authorization, and explicit public budgets. Their transport does not prove
that target physical access exists. Current reads may reconstruct state from
the runtime log and allocate materialized rows; current live-query polling
executes two stamped snapshots and computes a deterministic difference.
Roadmap Gates C, E, F, and H replace those costs with direct stamped reads,
native graph/lexical/vector indexes, streamed Arrow batches, bounded
DataFusion execution, and commit-impact delivery without changing transport
authority.

`changes/follow` is a bounded waiting read over the durable cursor contract.
It is neither the canonical multiplexed stream nor a promise to retain two
delivery designs. The WebSocket subscription route currently provides push,
ACK, backpressure, lease, retention-floor, and restart replay behavior for one
subscription. Gate B-04/H-03 decides the single completed delivery surface.

The server records a process-local request span today, but it does not yet
accept and propagate the canonical W3C trace context or emit the complete
low-cardinality `rrflow.<boundary>.<operation>` causal graph. A-07 and H-05 own
that direct trace convergence.

## Current evidence and remaining qualification

| Evidence boundary | What current tests establish | What they do not establish |
|---|---|---|
| Contract/router parity | Every catalogued HTTP operation has exactly one dispatch; generated OpenAPI is derived from the same catalogue. | Released SDK or GraphQL conformance. |
| Real socket process | Loopback enforcement, liveness/readiness, authentication and scope denial, bounded envelopes, session/transaction replay, concurrent idempotency, multi-model commit/reopen, query, vector, changefeed, backup/restore, and secret exclusion. | Native target storage/index execution, generalized cancellation, or full resource/trace export. |
| Mutual-TLS transport | The CLI requires the certificate, key, and client CA together; a real-process Rust client test rejects a missing client certificate and wrong server name, accepts the configured identities, and carries HTTPS plus WSS subscription traffic. | Untrusted-client-chain coverage, certificate rotation and revocation, external identity, and production deployment integration remain open. |
| Local estate driver | Exclusive root ownership, restart/reopen, identity-safe process control, bounded stop escalation, and controller kill-gap convergence. | `rrflow install`, project attunement, or a released service manager. |

The focused characterization commands are:

```text
cargo test -p rrd-server every_catalogued_http_operation_resolves_to_one_executable_dispatch --locked
cargo test -p rrd-server --all-targets --locked
python3 scripts/ci/check_generated_surfaces.py
```

Remaining product work is governed by the roadmap and POA&M. In particular,
this server reference cannot close the one-transaction storage work in Gate C,
the native access paths in Gate E, streamed Arrow/DataFusion execution in Gate
F, reasoning/context integration in Gates G/H, install and attunement in Gate
D, automation in Gate I, or release proof in Gate J.

## Implementation anchors

- Public catalogue and wire types: `crates/transport/rrd-contract`
- Server composition and transport: `crates/transport/rrd-server`
- Sole semantic authority: `crates/authority/rrd-engine`
- Target architecture: [engine data flow](../../architecture/engine-data-flow.md)
- Observed gaps: [RRFlow 1.0 alpha POA&M](../../poam/rrflow-1.0-alpha.md)
