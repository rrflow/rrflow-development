# RRFlow Rust SDK

**Status:** active implementation reference; alpha SDK qualification is incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/sdk/rust`
**Owner:** Rust client construction, typed operation binding, transport, retry, cancellation, subscription, error, secret, and conformance behavior

`rrd-client` is the supported asynchronous Rust client foundation for public
RRD operations. Its only normal RRFlow dependency is `rrd-contract`; engine,
storage, query, vector, security, estate, and server crates are confined to
development fixtures. That dependency direction is correct and must remain.

This record does not make the client another engine. Authentication,
authorization, read-stamp selection, physical planning, rrflowMX/rrflowKV
access, rrflowQL, Arrow/DataFusion execution, graph/BM25/vector work, reasoning
mutation, installation, and attunement remain behind `RrdEngine`. The client
constructs and carries typed public operations and verifies their public
results. The [public contract](../protocol/public-contract.md) owns the wire
vocabulary, the [server reference](../protocol/server.md) owns the daemon
adapter, the [engine data flow](../../architecture/engine-data-flow.md) owns the
underlying execution, and the [roadmap](../../roadmap/rrflow-1.0.md) owns
acceptance.

## Required end-to-end boundary

```text
Rust application
  -> rrd-client typed operation + explicit endpoint/security profile
  -> HTTP or multiplexed WebSocket RRD transport
  -> RrdEngine authentication, authorization, stamp, planning, and transaction
  -> rrflowMX or rrflowKV + native graph/BM25/vector + rrflowQL/DataFusion
  -> one validated result/denial/receipt with the same IDs, stamp, and trace
```

The SDK cannot select a storage profile, raw key, physical index, DataFusion
plan, model backend, or project database. It may express semantic intent and
budgets exposed by `rrd-contract`. Cross-surface proof must compare the result,
denial, stamp, digest, receipt, and causal identity produced by the engine—not
merely compare JSON shapes.

## Audited current surface

The executable `endpoint_catalogue()` contains 33 HTTP operations and one
subscription WebSocket descriptor. `RrdClient` currently supplies typed HTTP
methods for 28 of those 33 operations and implements the dedicated
subscription socket. The current boundary is:

| Family | Catalogued HTTP operations | Rust methods now | Missing now |
|---|---:|---:|---|
| Service health | 2 | 0 | `health-live`, `health-ready` |
| Capability and schema discovery | 3 | 3 | none |
| Sessions | 3 | 3 | none |
| Transactions | 4 | 4 | none |
| rrflowQL query | 1 | 1 | none |
| Query indexes and live polling | 3 | 0 | `query-index-ensure`, `query-index-list`, `query-live-poll` |
| Vector collections, points, and search | 5 | 5 | none |
| Changefeeds | 2 | 2 | none |
| Durable-subscription administration | 2 | 2 | none |
| Backup and restore | 3 | 3 | none |
| Estate | 1 | 1 | none |
| Audit | 2 | 2 | none |
| Diagnostics | 1 | 1 | none |
| Context | 1 | 1 | none |
| **Total** | **33** | **28** | **5** |

The WebSocket method connects, receives an opening snapshot, reads server
frames, sends cumulative ACKs and heartbeats, and requests closure. This is
real current behavior, but it is the single-subscription protocol documented
in [durable subscriptions](../protocol/subscriptions.md), not the B-04
multiplexed target.

## Behavior worth retaining

The complete source and real-server fixtures establish useful foundations:

- local cleartext construction is restricted to an explicit loopback socket;
- network construction requires an HTTPS origin and an explicit Rustls client
  configuration;
- capability discovery checks protocol and expected RRD instance identity;
- common request, resource, correlation, deadline, and mutation-idempotency
  coordinates are typed;
- HTTP response accumulation is capped at four MiB;
- API error bodies preserve their current closed error code, message,
  retryability value, and HTTP status;
- loopback tests exercise a real secured server and durable subscription
  reconnect/ACK behavior rather than a mocked client; and
- a separate HTTPS/WSS fixture proves a configured mutual-TLS success path,
  missing-client-certificate denial, and wrong-server-name denial.

Those facts characterize the present client. They do not prove complete
operation coverage, full request/response validation, stable retry semantics,
server-side cancellation, bounded WebSocket allocation, endpoint rotation,
installed deployment, or cross-language equivalence.

## Catalogue-driven operation binding

The operation catalogue is the sole executable registry. A Rust method must
bind its operation identifier to the catalogue descriptor and derive or verify
the exact method, route template, authentication kind, mutation flag, security
action, request type, response type, response media type, and successful HTTP
status. The current descriptor must be extended to own the latter two facts;
no separately maintained client route table or documentation list may silently
drift.

Every catalogued operation must have exactly one Rust binding. The coverage
test must fail on a missing or extra binding. Dynamic path values are encoded
only through the descriptor's typed path parameters; string formatting cannot
create an uncatalogued endpoint.

Before network I/O, the binding must validate both the common envelope and the
operation payload. After I/O, it must enforce the declared media type and byte
limit, decode once, validate the complete `ResponseEnvelope` and operation
payload, match the exact expected status/outcome, and match protocol, instance,
request, operation, resource, stamp, and receipt coordinates applicable to the
operation. A generic `2xx` check or deserialization without the type's semantic
validation is insufficient.

The current client validates the common request envelope but does not invoke
each operation payload's validator. It decodes response envelopes without
calling their complete validator, validates only selected success payloads,
does not check response `Content-Type`, discards the observed successful status
before the typed outcome helper, and supplies `200 OK` there. Consequently an
unexpected successful `2xx`, invalid bounded error body, or semantically
invalid typed success can pass farther than the public contract intends.

## Bootstrap and endpoint resolution

A client connection begins with transport reachability, not database
initialization:

1. resolve an explicitly configured loopback or network endpoint candidate and
   its expected RRD instance/security identity;
2. establish the configured transport and authenticate the TLS server where
   applicable;
3. call liveness, then readiness;
4. negotiate protocol and exact instance identity through capabilities;
5. load the operation catalogue, and optionally its OpenAPI projection, at the
   advertised digest; and
6. create an authenticated application session before protected work.

The client never creates an instance because a path or endpoint is absent and
never treats a TCP connection, mesh membership, liveness, or TLS alone as RRD
readiness. The H-07 resolver accepts loopback or an installed mesh adapter as
address discovery only. It returns bounded candidate addresses plus expected
transport and RRD identities; endpoint rotation repeats authenticated
negotiation. Zuul Zero/shippin.ai reachability never grants RRFlow permission
and never owns RRFlow state.

## Retry, deadline, and outcome certainty

Retry is an operation-semantic decision, not a boolean transport convenience.
The canonical policy is explicit, bounded by an overall deadline and attempt
count, and records attempt/decision evidence without secrets. Backoff and
jitter are bounded configuration, not hidden global policy.

| Operation condition | Automatic replay rule |
|---|---|
| Public safe discovery request | May retry a classified transient transport failure within the declared budget. |
| Session-bound read pinned to one accepted read coordinate or durable result binding | May retry only while the server can preserve the same semantic observation. |
| Unpinned read whose first response was lost | Must not silently return a later observation as though it were the first; return a typed uncertain observation or require an explicit caller policy. |
| Mutation with an engine-durable exact request/operation/idempotency binding | May retry the identical bytes and coordinates; the server's recorded receipt is authoritative. |
| Mutation without that durable binding, changed bytes, expired binding, or unknown commit disposition | Must not replay automatically; return a typed uncertain outcome requiring receipt/status reconciliation. |
| Typed API denial or invalid protocol/result | Never becomes a transport retry merely because an error says it is retryable. Any future server retry hint must be authenticated, catalogued, and bounded. |

The current implementation does not satisfy that matrix. Public discovery and
idempotency-bound mutations can retry up to the configured 1–8 attempts, with
no backoff or jitter. Ordinary session reads execute once, contrary to the
removed flat record's statement that reads retry. `request_timeout` applies to
each attempt while an optional absolute deadline bounds the remaining time.

Dropping a Tokio future or reaching the local timeout ends local waiting; it
does not prove that the server stopped compute or a mutation did not commit.
B-04 must add a correlated cancellation request and terminal outcome for
multiplexed work. Mutation certainty remains governed by the durable engine
receipt even when the caller disconnects.

## Authentication, secrets, and transport identity

The client authenticates three distinct facts and must not collapse them:

- endpoint resolution identifies where to connect;
- TLS validates the configured server name/trust and, for mutual TLS, presents
  the configured client identity; and
- RRD capabilities plus application authentication validate the exact instance
  and authorize the principal/session operation.

The normal constructor consumes an installed or explicitly supplied
authenticated transport profile. Any test-only verifier override remains
visibly unsafe and cannot be accepted by the default constructor or qualify a
deployment.

API keys, bearer tokens, private keys, session identifiers, and other
credential material must not appear in `Debug`, `Display`, errors, traces,
metrics, URLs, process arguments, or response excerpts. The current public
`Session` derives `Debug` and contains a public `SessionLease`; that lease
contains the bearer token, so formatting the session can disclose it. Direct
convergence replaces this with an opaque session handle, redacted custom debug
output, and a secret-bearing token wrapper with deliberately limited exposure
and copying. Transport errors retain typed sources and classifications without
including authorization headers or uncontrolled server bodies.

## WebSocket bounds and frame integrity

The current client gives `tokio-tungstenite` no `WebSocketConfig`, so its
locked dependency defaults permit substantially larger messages and buffers
than the RRD server's 64-KiB contract. That is not a client resource bound.
B-04 must configure limits before reading frames and prove them with malicious
peer tests.

The multiplexed client must:

- cap frame/message/read/write buffering at the negotiated RRD limits before
  allocation and reject binary or unknown frame shapes;
- validate every frame's protocol, request or subscription identity,
  connection generation, sequence/cursor monotonicity, and bounded error;
- expose a receive deadline and correlated cancellation;
- enforce bounded in-flight requests and subscription deliveries with explicit
  backpressure;
- resume only from a durably acknowledged cursor and reject stale generations;
  and
- route request/response, cancellation, subscription, ACK, heartbeat, and
  terminal frames over the one B-04 connection without creating client-owned
  lifecycle state.

## Errors and causal evidence

The final error surface is closed and phase-aware: local contract, endpoint
resolution, TLS identity, transport, timeout/cancel, protocol, API denial,
response validation, resource exhaustion, and uncertain observation or
mutation outcome remain distinguishable. It retains safe underlying causes
for diagnostics, exposes retry classification derived from operation semantics,
and never turns strings into authorization or completion truth.

Every request carries the canonical request and operation IDs plus W3C
`traceparent`/`tracestate` when valid. Client attempts are child spans of the
same operation and transport reconnects are causal links, not new semantic
operations. Tracing never contains credentials or hidden model reasoning and
never substitutes for the authoritative response, job, transaction, or
subscription record.

## Conformance that actually counts

`fixtures/rrd-sdk-conformance-v1.json` is a shared input, not proof by its
existence or by a list of domain labels. Each scenario must declare its exact
preconditions, operation(s), assertions, faults, and expected engine evidence.
The current corpus overstates several rows:

| Current label | Scenario actually executed | Required closure |
|---|---|---|
| `crud` | one document create and read | create, read, update, retire/delete, conflict, and reopen semantics |
| `backup` | create and list | restore, interrupted restore, identity, and reopen |
| `vectors` | ensure and search | list, scroll, retrieve, update/delete, filters, exact comparison, and reopen |
| `live_feeds` | HTTP changefeed read/follow | multiplexed WebSocket delta, ACK, reconnect, backpressure, and cancellation |
| `typed_errors` | one unauthenticated denial | the closed error/denial matrix with bounded details and redaction |
| `retries` | one connection dropped before a capability request | before-send, after-send, after-commit, lost-response, bounded replay, and uncertain-outcome cases |
| `cancellation` | caller wraps one follow future in a local timeout | correlated server cancellation and terminal evidence, including disconnect races |
| `versions` | one incompatible capability response | protocol, catalogue/schema digest, package, and supported-release matrix |

The harness currently seeds schema, estate, and security by directly opening
`RrflowKvStore` and repositories and creates its own `InstanceManifest`. That
is a useful fixture shortcut but cannot qualify installation or the SDK. The
release corpus starts a D-01-installed instance using only public operations,
runs the same semantic cases against rrflowMX and rrflowKV where applicable,
adds rrflowKV crash/reopen cases, and compares loopback, authenticated network,
HTTP, multiplexed WebSocket, Rust, generated languages, CLI, MCP, GraphQL, and
Connectome at one expected engine result.

The Rust conformance test must never report a successful scenario when
`RRD_SDK_CONFORMANCE_MANIFEST` is absent. It must either be an explicitly
ignored integration target or fail closed. The runner verifies scenario
coverage structurally rather than comparing a set of self-declared domain
names.

## Evidence observed in this review

| Command or inspection | Result | Honest boundary |
|---|---|---|
| `cargo test -p rrd-client --test real_server --locked` | 3 passed | Real loopback, narrow deployment corpus, and mutual-TLS/WSS behavior; not full operation, storage-profile, or release conformance. |
| Rust conformance test with no manifest | Cargo reported 1 passed in 0.00 s | The test returned before executing a scenario; this is not conformance evidence. |
| Rust conformance test against the live example harness with the absolute shared-corpus path | 1 passed; runner reported the expected corpus SHA-256 | Real execution of the present limited rrflowKV corpus; not installation, rrflowMX parity, full labelled behavior, or cross-language proof. |
| Generated-surface parity check | 33 HTTP operations matched OpenAPI | Checks generated TypeScript/Python/Go/Java/.NET endpoint maps; it does not inspect Rust method coverage. |
| Normal dependency inspection | no RRFlow implementation crate below `rrd-client` | Correct client dependency direction; dev-only server/engine/store dependencies remain test fixtures. |

No row proves the persistent multi-model reasoning/recall engine. That proof is
owned by C through H and then exercised through the SDK in H-04/J.

## Direct-convergence file plan

A-07 mechanically splits the current 1,235-line monolith along already
accepted responsibilities without retaining forwarding modules:

```text
crates/transport/rrd-client/src/
├── lib.rs             # narrow public exports
├── client.rs          # construction and public typed methods
├── endpoint.rs        # expected identity and resolved endpoint candidates
├── error.rs           # closed redacted client errors
├── operation.rs       # catalogue binding and request/response validation
├── retry.rs           # semantic retry/deadline/outcome-certainty policy
├── session.rs         # opaque credential-bearing session handle
├── subscription.rs    # bounded multiplexed stream state
└── transport.rs       # loopback and authenticated network carriage

crates/transport/rrd-client/tests/
├── operation_coverage.rs
├── protocol_validation.rs
├── transport_faults.rs
├── real_server.rs
└── sdk_conformance.rs
```

The gates then close behavior in dependency order:

1. **A-07:** preserve characterized behavior while freezing the modules,
   public names, secret boundary, and exact mapping from each current symbol
   and test; no compatibility module or old export remains.
2. **B-04:** replace the dedicated socket with the bounded multiplexed protocol
   and correlated cancellation.
3. **H-04:** bind all 33 operations from the catalogue, validate every request
   and response, and compare Rust with every supported surface against one
   real `RrdEngine` corpus.
4. **H-07:** add endpoint resolution/rotation as an outward adapter while
   keeping TLS, RRD instance identity, and authorization independent.
5. **J-02/J-03/J-05:** pass the failure, resource, clean-install, offline
   distribution, package, and release matrix with retained raw evidence.

## Primary constraints

- [RFC 9110, HTTP method idempotency](https://www.rfc-editor.org/rfc/rfc9110.html#name-idempotent-methods)
  bounds automatic replay by operation semantics; RRFlow's body-level
  idempotency binding remains its own stronger application contract.
- [Tokio timeout](https://docs.rs/tokio/1.53.1/tokio/time/fn.timeout.html)
  defines local cancellation by dropping the future; it is not server
  completion evidence.
- [Rustls `ClientConfig`](https://docs.rs/rustls/0.23.43/rustls/client/struct.ClientConfig.html)
  owns TLS server verification and client-credential selection, distinct from
  RRD authorization.
- [W3C Trace Context](https://www.w3.org/TR/trace-context/) defines interoperable
  `traceparent` and `tracestate` propagation; RRFlow still owns its bounded
  causal attributes and redaction.
- [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html#data-to-exclude)
  identifies access tokens, authentication secrets, and session identifiers as
  data that should not be recorded directly.
- The locked `tungstenite` 0.29.0 `WebSocketConfig` source is the executable
  authority for current client defaults; release limits come from explicit RRD
  configuration and adversarial tests, never from an assumed library default.
