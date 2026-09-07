# RRFlow security authority

**Status:** active implementation reference; single-engine alpha security convergence remains incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/security/authority`
**Owner:** identity, credential, session, policy, authorization, and audit semantics enforced through `RrdEngine`

RRFlow Security is a cross-cutting capability of the one RRFlow engine. It is
not an independently authoritative database or service. `RrdEngine` is the
only boundary allowed to authenticate an operation, compile its authorization,
capture its read or transaction stamp, authorize its semantic effects, and
commit state. The `rrd-security` package supplies useful identity, policy, and
audit vocabulary today, but its current public storage repository is
implementation inventory rather than the accepted final authority boundary.

The [system overview](../../architecture/system-overview.md#security-boundary)
owns the architectural security boundary, [ADR-0001](../../decisions/0001-single-engine-authority.md)
owns the single-engine decision, the [public contract](../protocol/public-contract.md)
owns transport-neutral wire shapes, and the [roadmap](../../roadmap/rrflow-1.0.md)
owns completion. This reference records the exact useful behavior to preserve,
the current deviations to remove, and the evidence required before security is
alpha-qualified.

## Authority invariant

Every non-public operation follows one engine-owned sequence:

```text
typed ingress + explicit instance/resource + credential
  -> RrdEngine authenticates identity and session
  -> capture exact policy revision/digest and authorized data constraints
  -> authorize operation and requested semantic effects
  -> capture one read or transaction stamp
  -> plan only over authorized fields, rows, graph paths, and index candidates
  -> execute bounded native or rrflowQL/Arrow/DataFusion work
  -> for writes, validate and encode one atomic semantic batch
       canonical state + indexes/projections + outbox + audit + cursor
  -> commit through the selected rrflowMX or rrflowKV transaction profile
  -> return result, denial, or failure with correlated evidence
```

Public liveness, readiness, catalogue, and capability inspection are explicit
catalogue dispositions, not an accidental result of missing security state.
They never receive mutation authority. A transport, mesh, model, DataFusion
operator, index, hook adapter, SDK, or Connectome cannot authorize itself or
write around this sequence.

## Component responsibilities

| Boundary | Owns | Must not own |
|---|---|---|
| `rrd-contract` | closed public actions, resource paths, request coordinates, audit envelopes, and error vocabulary | credential verification, policy decisions, storage, or a second operation registry |
| `rrd-security` target | bounded principal/role/grant/credential metadata, pure validation and policy compilation, audit value semantics, and cryptographic helpers | opening a physical store, committing control/runtime state, issuing an independent read stamp, or accepting an operation outside `RrdEngine` |
| `RrdEngine` | identity/session authentication, current-policy resolution, authorization, effect validation, stamps, transaction coordination, audit causality, and adapter-independent outcomes | transport parsing or provider-specific identity lifecycle |
| rrflowMX / rrflowKV | equivalent transactional point/range/CAS semantics; rrflowKV additionally owns durability and recovery | interpreting principals, granting actions, or selecting policy |
| rrflowQL, native graph/BM25/vector operators, and DataFusion | execute the already-authorized plan at its exact stamp and budget | widening rows/fields, bypassing filters, changing policy, or committing results |
| HTTP, WebSocket, SDK, MCP, CLI, mesh, and Connectome adapters | validate and carry typed credentials, identities, requests, responses, and TLS evidence | semantic authorization, hidden session state, or provider-owned lifecycle rules |
| External identity or secret provider adapter | verify the provider assertion or supply secret material under explicit configuration | becoming an RRFlow principal authority or persisting credentials into rrflowDB |

## Implemented identity and policy foundation

The current `SecurityState` has an exact format and monotonically replaced
revision. It contains bounded canonical maps for principals, roles,
third-party identity bindings, and JWT issuers. Complete-state replacement uses
compare-and-swap: the next revision must advance exactly once, exact replay is
a no-op, and stale or substituted state is rejected.

A principal is a user, service, or node with a credential-verifier digest,
credential revision, validity window, disabled flag, direct grants, and role
memberships. Roles form a bounded acyclic inheritance graph. A grant binds one
closed `SecurityAction`, one exact canonical resource prefix, and an optional
`DataPolicy`; wildcard strings and allow-on-missing-policy behavior do not
exist. The most-specific matching grants apply, and equally specific grants
with different data policies fail closed.

Current data policy can express one tenant equality predicate, bounded row
equality predicates, and an allowed-field set. Query execution applies those
predicates and projection before binding and physical planning, and its public
plan evidence carries the policy revision and authorization digest. An
operation that cannot enforce a constrained policy denies it rather than
silently widening access.

The policy evaluator itself fails closed when authority is uninitialized, a
principal is absent, disabled, or outside its validity window, a credential
does not match, or no action/resource-prefix grant applies. That behavior is
distinct from the server's current missing-policy loopback branch described
below, which bypasses normal principal authentication and is not an accepted
profile.

That implementation is a foundation, not complete authorization for the target
engine. The current transaction actions authorize a broad transaction
operation, not every table, record, relation, graph edge, field, vector,
projection, routine, or external effect inside the mutation plan. The accepted
write path must authorize those semantic effects after parsing and before one
atomic commit. Native access-path selection, DataFusion pushdown, spill files,
result shaping, explain output, errors, and caches must preserve the same
authorized row/field boundary; F-05 caches must include the authorization
digest and policy revision in identity and invalidation.

## Credentials, identity binding, and sessions

The repository currently implements these concrete mechanisms:

- API-key authentication compares the SHA-256 of supplied bytes against a
  persisted lowercase digest without an early byte-mismatch return; the engine
  helper generates a 256-bit random, owner-only printable key. This mechanism
  is for high-entropy API keys, not human passwords, and raw key material must
  never enter durable state, request logs, traces, errors, or audit.
- RRD-issued JWTs currently use bounded HS256 tokens. Header and claims bind
  issuer, audience, key identity, principal, token identity, time window,
  policy revision, and principal credential revision. Only the signing-key
  digest is policy state; the server receives key material separately.
- Verification reloads current policy, so principal disablement, issuer
  disablement, key mismatch, and credential-revision rotation reject an older
  token. Session state persists the principal credential revision and checks it
  again on later operations and after reopen.
- A third-party identity assertion is usable only after an external adapter has
  cryptographically verified it. RRFlow then resolves the exact persisted
  issuer/subject/audience tuple to a current principal and still applies
  `session_create`. Unsigned identity headers are never sufficient.

Transport authentication and application authorization remain distinct. A
valid client certificate proves the TLS peer accepted by the configured trust
roots; it does not itself select an RRFlow principal or grant. Likewise, mesh
reachability and provider identity cannot substitute for RRD server identity,
RRFlow credentials, or current policy.

The current closed action enum covers session, query, index, transaction,
changefeed/subscription, vector collection/point, backup/restore, estate,
audit, diagnostic, security-administration, and implemented governed-runtime
operations. This is implementation inventory, not permission for speculative
features; new install, attunement, routing, event, trigger, routine, skill, and
adapter actions are introduced only with their typed engine operation and
denial tests.

The final install path must create or bind all required key material through a
previewed, digest-bound, provider-neutral `rrflow install` plan. The existing
standalone `rrd-security-bootstrap` helper accepts a database path, manifest,
and mounted credential files and proves useful bounded/private-file semantics,
but it is not the D-01 installation surface and receives no compatibility
status. Secret-provider integration, rotation, revocation, and reload behavior
must be explicit configured adapters with failure and restart evidence.

## Policy and data stamp

Authorization must be reproducible at the same logical observation as the
data it protects. An accepted read binds at least:

- instance, principal, credential revision, operation, and resource;
- policy revision plus a digest of the compiled row/field/effect constraints;
- schema and catalogue revisions;
- the rrflowKV or rrflowMX `ReadStamp` and valid-time coordinate;
- logical and physical plan digests, resource budget, and result evidence.

The current implementation loads JSON security state from the control keyspace
and then obtains the runtime data stamp separately. Its query result records a
policy revision and digest, but there is no atomic policy/data snapshot that
prevents a concurrent policy replacement between authorization and the read.
This is not sufficient evidence for the target. C-02 through C-04 must provide
one accepted transaction/snapshot relationship for policy, semantic data,
indexes, and audit on both profiles, with rrflowKV reopen and conflict proof.

## Audit and causal evidence

The implemented `AuditRecord` is immutable and binds audit identity, time,
optional principal, closed action, resource, request/operation coordinates,
authorization or completion phase, allow/deny/fail decision, status,
request/response digests, previous audit digest, and its own digest. It excludes
bodies, credentials, bearer tokens, signing keys, filesystem paths, and
arbitrary headers.

Audit records form an independent genesis-to-head chain. Record and head are
published atomically in one bounded control batch; concurrent head conflicts
retry within a fixed bound. Bounded read and canonical JSON Lines export expose
the global scanned control sequence plus chain anchor/head so a consumer can
validate lineage and make progress across unrelated control activity. Existing
tests cover exact replay, collision, concurrent linearity, head substitution,
reopen, redaction, protected read, and export validation.

The current value validator permits an authorization record only as
`authorized/allowed` with status 100 and requires completion records to carry a
final status and allowed, denied, or failed decision. Audit read and canonical
JSON Lines export require their separate actions; export verification parses
every line and rejects record-count, content-digest, framing, or chain
substitution.

An invocation currently appends `authorized` before executing and appends
`completed` after the outcome is known. A missing completion is therefore
visible, but authorization, domain mutation, and completion audit are separate
commits. A domain write can succeed before completion audit fails, and audit
does not yet carry the complete policy/read/plan/commit coordinates required by
the final causal trace. C-03 must include the accepted write's audit and outbox
in the same semantic batch; H-05 must correlate ingress, authorization, stamp,
native/DataFusion work, commit, and delivery without turning trace state into
authority. A denial or pre-execution failure remains an audit-only engine
transaction.

Current security-state initialization/replacement does publish its state and
`security_admin` completion audit in one control batch. Estate administration,
reconciliation, backup reconciliation, retention pruning, and restore recovery
also emit `estate_admin` phase records. Session-backed embedded work attempted
outside an active invocation writes an authorization reservation or terminal
denial so it cannot disappear entirely. These useful causal safeguards must be
absorbed into the final semantic transaction; they do not make the ordinary
three-commit invocation path atomic.

## Transport boundary

The server currently rejects cleartext non-loopback binding. Its remote server
configuration fixes TLS 1.3, requires a server certificate and client trust
roots, and uses a client-certificate verifier. A real client/server test proves
missing-client-certificate rejection, server-name rejection, authenticated
HTTPS, and WSS delivery.

This does not prove production transport security. Certificate rotation,
revocation, trust-root reload, external identity verification, rate limiting,
deployment-specific cipher and key policy, and mesh endpoint rotation remain
unqualified. The server currently permits anonymous application sessions and
operations on loopback whenever no security state exists. That absence-driven
development mode contradicts the final always-explicit authority invariant and
must be replaced by an explicit configured profile/bootstrap outcome; only
catalogued public inspection may remain anonymous.

## Current implementation deviations

The complete review found these direct-convergence requirements:

1. `rrd-security` still documents itself as policy truth, refers to removed RRO
   provisioning, depends on `rrd-store`, and exports a `SecurityRepository`
   that can commit directly to any `StorageEngine`. A-07 must retain its useful
   values and pure decisions while making `RrdEngine` the only caller that can
   persist or replace authority and audit state.
2. Policy, sessions, and audit are JSON values under `server/state/*` control
   keys with a control-journal coordinate separate from runtime data. C-01
   through C-04 must give them the accepted binary key/transaction/snapshot
   relationship without adding another database or compatibility reader.
3. Query row/field policy is implemented, but mutation-effect authorization,
   graph/vector/index side-channel constraints, policy-aware caches, and one
   policy/data stamp are not.
4. The missing-policy loopback branch grants anonymous application operation;
   it is current characterization, not an accepted pre-release fallback.
5. Security bootstrap is a standalone database-path command rather than a
   previewed phase of the generic offline installation and attunement flow.
6. Authorization and completion audit are not atomic with domain mutation;
   audit values also lack the final stamp, plan, effect, and commit evidence.
7. Current actions cover existing endpoints but not all planned install,
   attunement, routing, engine-event, trigger, routine, skill, and adapter
   operations. Their vocabulary must expand only with the owning implemented
   public operation, never as speculative permission strings.

These are tracked by the [POA&M](../../poam/rrflow-1.0-alpha.md). They are not
silently repaired during this documentation-classification package and do not
create a legacy security mode.

## Evidence and acceptance map

| Evidence or gate | Proven now or required outcome |
|---|---|
| `rrd-security` tests | Current validation, exact-scope denial, role inheritance failure, API-key/JWT/identity binding, rotation/revocation, audit chain, concurrent append, and rrflowKV reopen behavior. The suite is not an engine-authority or MX/KV semantic differential. |
| `rrd-engine` security tests | Current engine invocation, embedded denial, credential-revision invalidation, query policy injection, context fail-closed behavior, and audit phases. They do not prove atomic policy/data stamps or effect-level write authorization. |
| Server/client real-process tests | Current API-key sessions, denials, redaction, audit read/export, TLS 1.3 mTLS identity, authenticated HTTP/WSS, and selected reopen behavior. They do not qualify every adapter, provider, rotation mode, or deployment. |
| A-07 | Remove the direct-store/public-repository authority, obsolete RRO language, ambiguous action vocabulary, and dependency-direction conflicts while mapping every retained invariant to its one owner. |
| C-01 through C-04 | Encode and transact security/audit state through the one accepted rrflowMX/rrflowKV substrate; bind policy and data observations; atomically include audit with accepted writes. |
| D-01/D-02/D-06 | Integrate secure initialization, secrets, external identity providers, and optional adapters into previewed, persisted, provider-neutral install/attunement without copying credentials. |
| F-01 through F-05 | Prove native and DataFusion plans see only authorized stamped batches, enforce budgets, and never return stale authorization-scoped cache entries. |
| H-04/H-05/H-07 | Prove cross-surface authorization equivalence, complete redacted causal evidence, RRD TLS identity, and mesh independence. |
| J-01 through J-05 | Remove superseded branches and pass denial, crash/reopen, resource, clean-install, secret-accounting, and deployment qualification before release. |

## Implementation anchors and focused verification

- Policy, identity, JWT, audit values, and current direct repository:
  `crates/authority/rrd-security/src/lib.rs`
- Sole accepted composition boundary and invocation policy:
  `crates/authority/rrd-engine/src/engine/{security,session,invocation}.rs`
- Current bootstrap helper:
  `crates/authority/rrd-engine/src/engine/security_bootstrap.rs` and
  `crates/adapters/rrflow-cli/src/bin/rrd-security-bootstrap.rs`
- HTTP credentials and invocation wrapping:
  `crates/transport/rrd-server/src/http/{auth,envelope,server}.rs`
- Public action and audit shapes:
  `crates/transport/rrd-contract/src/lib.rs`

Focused characterization starts with:

```text
cargo test -p rrd-security --all-targets --locked
cargo test -p rrd-engine --lib engine::tests::security --locked
cargo test -p rrd-server --test http_process initialized_security_authority_binds_sessions_and_denies_ungranted_routes --locked
cargo test -p rrd-client --test real_server remote_transport_requires_mutual_tls_and_exact_server_identity --locked
```

Those commands characterize present behavior only. Alpha acceptance additionally
requires the gate-specific semantic, differential, failure, resource,
cross-surface, and clean-deployment evidence above.
