# Gate B accepted evidence

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/gate-b-evidence`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

## B-01 evidence (2026-09-04):

- `rrd-contract` defines strict provider-neutral installation plans/results and
  attunement plans, jobs, leases, phase checkpoints, status, resume, cancel,
  and verification payloads. They contain no provider hook, secret, local
  path, or client-owned lifecycle state.
- The canonical phase order is connect, inventory, parse, normalize,
  entity-link, lexical-index, embed, vector-index, graph, ground, and verify.
  Checkpoints must be an exact prefix, bind the plan configuration and runtime
  coordinates, and chain each input digest to the preceding output digest.
- `install-attunement-v1.json` freezes every payload shape, all seven job
  states, and all 12 permitted transitions. The transition matrix test accepts
  valid snapshots for those 12 transitions and rejects every other one of the
  49 possible state pairs; rejection tests cover phase skips, content,
  configuration, chain, revision, retry, and verification digest drift.
- All 45 `rrd-contract` tests, strict package Clippy, the implementation-free
  architecture test, and `cargo check --workspace --all-targets --locked`
  passed. No engine, persistence, endpoint, SDK, CLI, or Connectome behavior
  changed in B-01.

## B-02 evidence (2026-09-04):

- `RouterBackendDescriptor` declares a canonical identity, revision, supported
  decision kinds, hard dispatch limits, and content digest. B-03 now extends
  that current descriptor with an exact model-manifest identity, revision, and
  digest without changing B-02's three proposal semantics.
- `RouteStepRequest` reuses the A-02 reasoning cursor, recipe, edge, and
  condition types. It supplies bounded scalar signals, candidate-closed
  decisions, an exact deadline, and an optional context allowance whose scope
  must equal the cursor read scope.
- `select_recipe` can return bounded typed parameters only for an offered
  recipe revision; `advance_branch` can select only an offered outbound edge
  and cannot claim condition evaluation; `request_context` can only narrow
  offered seeds and resource budgets and cannot select a storage key, field,
  index, vector backend, authorization, physical plan, or mutation.
- `router-contract-v1.json` freezes the descriptor, stamped request, all three
  decisions, and their SHA-256 bindings. Six focused tests cover closed
  generated schemas, unknown-field rejection, provider/model neutrality,
  encoded-byte and nesting bounds, candidate/budget containment, request
  binding, and deadlines.
- All 51 `rrd-contract` tests, strict package Clippy, the implementation-free
  architecture test, and `cargo check --workspace --all-targets --locked`
  passed. No inference runtime, engine, endpoint, SDK, CLI, or Connectome
  behavior changed in B-02.

## B-03 evidence (2026-09-08):

- `RouterModelManifest` is a closed provider-neutral contract for immutable
  model, tokenizer, runtime, and constrained-decoding grammar artifacts. It
  binds canonical media/format revisions, exact byte lengths and SHA-256
  digests, the generated `RouteStepDecision` schema digest, supported decision
  kinds, backend and model resource limits, runtime ABI/device/configuration,
  quantization, and grammar source/revision. It contains no provider, path,
  URL, endpoint, credential, or secret authority.
- `RouterBackendDescriptor` now binds one manifest identity/revision/digest.
  `RouterModelHandshake` independently reports the runtime's observed
  manifest, backend, artifacts, schema, capabilities, limits, ABI/device,
  quantization, and grammar; its own domain-separated digest cannot repair a
  stale manifest or backend digest.
- `rrd-inference::load_router_model_after_handshake` invokes its loader closure
  only after the manifest and handshake validate and the supplied model,
  tokenizer, runtime, and grammar byte slices match both declared lengths and
  digests. The returned admission token is opaque and grants no route,
  authorization, storage, or mutation capability. No `RouterBackend`, LFG
  adapter, registry, dispatch, persistence, endpoint, or provider integration
  was created; those remain G-01 and later work.
- `model-manifest-v1.json` freezes the complete manifest/backend/handshake
  chain. Six focused model-contract tests independently reject contract,
  manifest, backend, model, tokenizer, schema, capability, backend-limit,
  model-limit, ABI, device, quantization, and grammar drift. Three inference
  admission cases prove a matching loader runs exactly once while every
  declaration or byte mismatch leaves its call count at zero. The full
  `rrd-contract` package passed 66 tests, `rrd-inference` passed 10 integration
  cases, both packages passed strict all-target Clippy, and all 23 workspace
  architecture checks passed.
- The required full-file inference review also corrected the malformed
  pre-release embedding-job digest domain directly from
  `rrd-inferenceding-job-v1` to `rrflow-embedding-job-v1`; a byte-level test
  freezes the corrected identity and no compatibility branch remains.

## B-04 evidence (2026-09-08):

- `rrd-contract` now owns one closed `WebSocketFrame` protocol with exact
  protocol/version, connection, direction-local sequence, request,
  cancellation, subscription, cumulative ACK, heartbeat, error, and
  backpressure coordinates. Validated negotiated limits bound message, frame,
  buffer, request, subscription, timeout, heartbeat, JSON, error, and retry
  inputs before application allocation; unknown members, wrong-direction
  frames, sequence gaps, identity substitution, and oversized inputs fail
  closed.
- The authenticated generic `GET /v1/ws` adapter and Rust `RrdWebSocket`
  reference carriage use that contract. One connection multiplexes durable
  changefeed and live-query subscriptions while `RrdEngine` alone owns resume
  cursor, connection generation, lease, outstanding delivery window, ACK, and
  close semantics. Reconnect clears only the interrupted delivery window and
  replays from the last durable ACK. Cursor-only progress is deliverable and
  ACK-able, so an unchanged live-query result cannot loop forever at an older
  read stamp.
- Generic request, response, and correlated cancellation shapes are frozen,
  but the server deliberately returns `failed_precondition` until H-04 binds
  them to the shared operation dispatcher. TypeScript, Python, Go, Java, and
  .NET document the same dependency and do not claim a WebSocket
  implementation. No transport connection state was added to `RrdEngine` and
  no second lifecycle authority was created.
- The golden protocol suite passed 8 cases; the focused engine subscription
  suite passed 6; the Rust real-server suite passed 3 loopback/mTLS/WSS cases;
  and 3 transport-fault tests covered six malicious-peer scenarios. The full
  four-package run passed 74 contract, 117 engine, 29 server, and 8 Rust-client
  cases. Strict all-target Clippy, locked workspace all-target compilation,
  all 23 architecture checks, documentation/inventory/generated-surface/
  knowledge/workflow/version policies, Cargo formatting, and diff integrity
  passed. This evidence does not qualify persistence, native indexes,
  Arrow/DataFusion execution, reasoning, installation, generated-SDK parity,
  Connectome, or release.

## B-05 evidence (2026-09-08):

- `rrd-query` now owns a bounded GraphQL query-ingress adapter. It parses with
  a standards-based GraphQL parser, selects exactly one query operation and
  one root source, derives permitted source/field definitions from the
  catalogue at the requested historical cursor, validates declared scalar
  variables and schema membership, and lowers into the existing `Query` plus
  `Parameters` representation before calling the ordinary `bind` path.
- The adapter supports the seven existing rrflowQL source families, temporal
  coordinates, filters, projection, bounded limits, explain modes, directed
  traversal, and one bounded equi-join. Mutations, subscriptions, fragments,
  directives, undeclared or unused variables, unknown arguments, unknown
  fields/sources, invalid GraphQL names, future cursors, oversized documents,
  and excess variables fail closed. It contains no resolver, storage trait,
  executor, transport, authorization decision, or mutation authority.
- `graphql-equivalence-v1.json` freezes five record/event/traversal/join/claim
  pairs and nine denial cases. Each accepted pair produces the same complete
  `BoundQuery` and bound digest from GraphQL and rrflowQL; the derived schema
  digest is also frozen. A historical-schema test and typed-default test prove
  the adapter cannot invent a second field or parameter model.
- `cargo test -p rrd-query --locked` passed 50 tests across all package
  targets, including all four GraphQL equivalence/denial cases. Strict
  all-target Clippy passed for `rrd-query` and `rrd-engine`; all 23 workspace
  architecture checks passed; and `cargo check --workspace --all-targets
  --locked` passed all 20 packages. This closes Gate B only. H-04 still owns
  the public GraphQL HTTP adapter and cross-surface authorization/semantic
  proof; C-02 now resumes the engine dependency spine.
