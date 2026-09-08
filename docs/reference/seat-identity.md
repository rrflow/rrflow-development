# RRFlow seat identity and memory warps

**Status:** active implemented reference; broader installed specialization remains incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/seat-identity`
**Owner:** durable provider-neutral seat identity and `rrflow://` record-warp semantics

An RRFlow seat is the durable identity the engine resolves as self inside one
estate. A provider account may represent a seat, but never replaces or owns
it. Changing Claude, OpenAI, Gemini, Grok, LFG, or another provider therefore
does not change the RRFlow identity.

## Clyffy boundary

**Clyffy is this project's primary RRFlow seat specialization.** It is not a
second product, kernel, repository, daemon, model provider, storage engine,
query engine, or lifecycle authority. Generic RRFlow installation templates do
not hardcode Clyffy; this repository's planned versioned specialization must
select that seat identity explicitly. Another installed project may select a
different seat without changing RRFlow semantics.

The executable Clyffy path must remain first-party Rust inside this repository.
`RrdEngine` owns authentication, authorization, routing orchestration,
deterministic predicates, context intent, compare-and-swap, audit, and commit.
LFG or any other `RouterBackend` may return only one of the three bounded router
proposals. Go can be installed as an outward SDK, generator, harness, or other
attuned capability; it is not a Clyffy or RRFlow routing, persistence, or
orchestration authority.

## Canonical records and relations

The implemented contract uses three canonical data kinds:

| Kind | Meaning |
|---|---|
| `rrflow-seat` | Provider-independent identity, display name, and purpose. |
| `rrflow-provider-identity` | Stable local provider-identity record containing an opaque provider name and subject digest. |
| `rrflow-represents` | Temporal relation from a provider identity to the seat it represents. |

A seat resolves as self only when at least one visible provider identity has a
valid `rrflow-represents` relation to it at the requested read coordinate.
Provider credentials and provider runtime sessions are never fields of these
records. The bind input hashes the provider subject and persists only its
SHA-256 digest.

Representation establishes durable attribution, not permission. The target
request path authenticates the calling provider or local principal, resolves
that exact identity's visible representation edge at the same `ReadStamp`, and
then evaluates ordinary RRFlow policy for the requested operation. A caller
cannot become Clyffy by supplying an actor string, a seat identifier, or another
provider's subject digest, and a valid representation cannot bypass a denied
operation.

The provider-neutral schemas are implemented in
[`memory_estate.rs`](../../crates/transport/rrd-contract/src/memory_estate.rs),
and their authorized planning and resolution are composed by
[`RrdEngine`](../../crates/authority/rrd-engine/src/engine/memory_estate.rs).

## Stable record warps

Every addressable canonical data record uses this path-safe form:

```text
rrflow://<instance>/data/<record-kind>/<record-id>
```

The URI identifies canonical RRFlow data; it is not a checkout file path and
does not copy record state into Markdown. During bootstrap, a README warp pairs
the durable URI with one local documentation fallback. Once the knowledge
import and persistence gates pass, clients resolve the URI through the engine
while the checkout link remains a recovery path.

The bootstrap coordinate for this project's primary seat is
[`rrflow://rrflow-instance/data/rrflow-seat/clyffy`](rrflow://rrflow-instance/data/rrflow-seat/clyffy).
An installed estate substitutes its actual instance identity in that URI.

Warp resolution validates the URI, converts its record kind and ID into a
canonical anchor, and invokes the same bounded temporal
`RrdEngine::assemble_context` path used by ordinary context requests. It does
not open physical keys or create a second retrieval mechanism.

## CLI operations

Bind or update one seat and one provider representation explicitly:

```bash
rrflow identity bind \
  --seat clyffy \
  --provider openai \
  --provider-identity codex \
  --provider-subject '<provider-subject>' \
  --representation codex-represents-clyffy
```

Resolve self or follow a record warp:

```bash
rrflow identity resolve --seat clyffy
rrflow context \
  --warp rrflow://rrflow-instance/data/rrflow-seat/clyffy \
  --max-graph-depth 1
```

`identity bind` first produces strict seat, provider-identity, and relation
schema plus mutation plans, then commits them through the ordinary
authenticated data-transaction boundary. `identity resolve` and `context
--warp` read after reopen from the selected RRFlow storage profile. They never
bypass mutation authorization or create provider-owned identity state.

## Executable proof and remaining boundary

The CLI real-boundary test
[`identity_bind_resolve_and_readme_warp_share_the_persistent_engine`](../../crates/adapters/rrflow-cli/tests/operator_surface.rs)
binds an identity, proves plaintext provider subject is absent, reopens the
persistent engine, resolves the same seat URI, follows the warp through context
assembly, verifies the representation edge evidence, and checks invocation
records contain only the subject digest.

This is characterization of a useful persisted foundation, not proof of the
complete installed Clyffy path. The current `MemoryEstate*` type and module
names conflate seat identity with an estate; the CLI supplies Clyffy-specific
defaults instead of consuming an installed specialization; its caller commits
the returned mutation plan manually; resolution reconstructs a broad runtime
snapshot; the older claim path accepts an arbitrary producer actor string; and
the authenticated session, representation edge, authorization decision, route
packet, router proposal, and resulting mutation are not yet bound at one read
coordinate. No `RouterBackend` dispatch exists in the checkout.

These are active convergence gaps, not compatibility promises. D-01 owns the
installed specialization and initial seat binding; A-07/C-03 own canonical
naming and effect-complete identity attribution; G-01 through G-05 own stamped
routing and persisted tree execution; H-04/H-05 own cross-surface identity and
causal evidence. The versioned specialization manifest, project attunement,
provider conformance, and normal rrflowDB-backed documentation resolution
remain owned by the
[agent-bootstrap reference](agent-bootstrap.md),
[roadmap](../roadmap/rrflow-1.0.md), and
[POA&M](../poam/rrflow-1.0-alpha.md).
