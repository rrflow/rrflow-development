# Generated knowledge, API, and SDK surfaces

**Status:** active generation-boundary design
**Coordinate:** `rrflow://rrflow-instance/data/execution-design/generated-surfaces`
**Owner:** single-source generation and maintenance of RRFlow navigation and outward contract projections

RRFlow should make links and clients cheap to regenerate, not expensive to
remember. Authors maintain the smallest semantic owner; deterministic tooling
derives navigation, transport, SDK, and conformance views. The
[adaptive-reasoning decision](../../decisions/0002-adaptive-reasoning-governed-effects.md)
prevents this compiler-like boundary from becoming a schema for reasoning.

## Source and projection model

| Owning source | Generated projections | Authority boundary |
|---|---|---|
| Executable operation and capability catalogues plus public wire types | Router/dispatch metadata, OpenAPI, internal client bindings, language SDK operations and models, reference operation tables, and shared conformance cases | Code and validation own representable operations; a generated client or document cannot invent availability or engine semantics. |
| Coordinated Markdown records and their ordinary links | Parent indexes, warp discovery, incoming/outgoing relationship graph, retrieval neighborhoods, and broken-link checks | Narrative owners retain meaning; the graph is derived navigation and may be rebuilt. |
| Git tree, package metadata, and active change record | Current file/gate inventory, digest coverage, affected-package views, and final path reconciliation | Git and the accepted change boundary own exact repository effects; inventory is not edit authorization. |
| Test, benchmark, fault, and package journals | Evidence indexes and gate-linked proof discovery | Immutable artifacts report observations; only the roadmap owner can accept them for completion. |

## API and SDK compiler pipeline

One versioned product operation model must drive every supported surface:

```text
wire types + validation + operation/capability catalogue
    -> engine dispatch contract
    -> HTTP/WebSocket/GraphQL/CLI/MCP presentation metadata
    -> OpenAPI and protocol fixtures
    -> internal Rust client bindings
    -> TypeScript/Python/Go/Java/.NET bindings and models
    -> shared semantic and fault conformance corpus
    -> generated reference and coverage views
```

Language SDKs own only language-specific runtime concerns: idiomatic API
shape, transport implementation, cancellation, resource ownership, packaging,
toolchain/platform support, and consumer qualification. They do not maintain
a second operation list, request model, retry meaning, capability catalogue,
or completion ledger.

Generation is necessary but not sufficient. A generated binding becomes
supported only after it passes the same installed real-process semantic,
authorization, error, uncertainty, cancellation, resource, restart, and
credential-redaction corpus. Missing generator or harness configuration fails
closed or reports an explicit skip; it never reports conformance.

The current checkout already derives OpenAPI and operation identifiers from
`rrd-contract`, but generation remains fragmented and most complete language
models/conformance behavior are open under H-04. H-04 converges those tools
onto one compiler entry point and rejects any handwritten duplicate registry.

## Knowledge and navigation maintenance

A knowledge author writes a normal Markdown link to the owning record. The
navigation maintainer discovers the target's title, status, coordinate, and
owner, renders marked index regions, and can project links as graph edges for
import and retrieval. Adding a coordinated child and regenerating is enough;
authors do not copy it into multiple hand-maintained tables.

Generation does not force every exploratory note into the durable record
schema. A Markdown child enters a generated index only after its author gives
it a stable coordinate; an uncoordinated draft remains ordinary authored
material. Once coordinated, malformed metadata or a duplicate coordinate fails
closed because the record is now claiming a durable identity.

Only marked regions are generated. Narrative outside a marker stays
human/AI-authored. The maintainer must be deterministic, preserve unmarked
content, reject malformed or duplicate children, and provide a non-mutating
`--check` mode for CI.

A generated edge says that one record links to another; it does not claim that
the target is true, current, accepted, or authorized. Those meanings remain in
the owning record and its lifecycle/evidence policy.

## Evolution without over-schema

New reasoning or project concepts may first exist as narrative, typed
extensions, or experimental capabilities. Promote a concept into the closed
operation model only when stable cross-surface behavior and validation are
needed. Physical access paths, providers, and indexes remain engine choices;
semantic intent stays expressive and versioned rather than frozen into a
universal field or workflow taxonomy.

The system rejects unknown durable effects, not unfamiliar ideas.
