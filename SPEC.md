# RRFlow kernel invariants

Status: supporting semantic contract; the root `README.md` owns product
identity, architecture invariants, current status, and knowledge routing.

This is a supporting semantic contract. If it conflicts with the root README
or the linked RRFlow 1.0 roadmap owner, the applicable owning record wins.

## Canonical authority

RRFlow has one authoritative runtime change log. A committed change belongs to
one scope, receives one monotonic runtime cursor, and contains a canonical
mutation: schema, claim, record, relation, event, vector, series sample,
geospatial value, object reference, retirement, or projection work.

Derived indexes, caches, projections, vector artifacts, and UI state are never
authoritative. They must bind to the exact source cursor and catalogue/schema
revision from which they were produced and fail closed when that binding is
invalid.

## Time

RRFlow distinguishes:

- valid time: when knowledge is true in the modeled domain;
- transaction time: when RRFlow recorded it;
- runtime cursor: the ordered committed-log coordinate known to a read.

A read resolves values at an explicit valid time from changes no later than its
captured runtime cursor. Later writes must not leak into that snapshot. Closing
and reopening the engine must not change a result at the same coordinate.

The kernel does not read a clock. Outward adapters supply time explicitly so
tests and replay remain deterministic.

## Context assembly

`RrdEngine::assemble_context` is the only context/recall composition boundary.
It must:

1. validate caller-provided intent, anchors, and resource budgets;
2. authorize `MemoryContextRead` before reading knowledge;
3. capture one read stamp and resolve one temporal runtime snapshot;
4. resolve active claims and records from that snapshot;
5. discover eligible lexical, vector, and graph sources without caller storage
   wiring;
6. rank and fuse sources deterministically;
7. enforce scan, item, byte, graph-depth, and graph-work limits;
8. attach source/plan/evidence digests to every selected item;
9. return the exact read stamp, truncation state, and packet digest.

No adapter may assemble a competing context packet, maintain a second memory
database, or require provider-specific state to obtain the same result.

## Claims

A claim is a bi-temporal subject-predicate-object value with producer
provenance. Claims with the same subject and predicate form a version history.
Resolution returns the newest transaction-visible version whose valid interval
contains the requested valid time. Supersession closes earlier overlapping
valid intervals without erasing history.

Claims and typed records share the canonical runtime log and context snapshot.
A claim-only recall path is an internal compatibility primitive, not a public
context engine.

## Transactions and durability

A successful transaction publishes all canonical mutations or none. Its
receipt identifies the committed cursor range and operation digest. Repeating
an idempotency-bound mutation must return the durable prior result rather than
inventing a second commit.

Persisted integrity metadata must be authenticated and verified before use.
Recovery must prefer an explicit error over silently interpreting incompatible
or corrupt bytes.

The storage file named `MANIFEST.LOCK`, where present, is a database writer
exclusion primitive. It is not an editor hook, lifecycle system, or source of
reasoning state.

## Determinism and bounds

All public requests and responses are versioned, strictly validated, and
bounded. Ordering must be explicit, ties must have stable tie-breakers, and
content digests must cover every field that affects semantics.

When a request reaches a work or result limit, the response must either report
truncation or fail with a typed resource error. It must never silently claim a
partial answer is complete.

## Security and evidence

Public reads and mutations pass through `RrdEngine` authorization. Audit and
diagnostic evidence may describe operations, resource use, stamps, and outcomes
but must not persist raw secrets, credentials, prompt bodies, query parameter
values, or embedding source bytes.

## Verification rule

Compilation proves only type compatibility. A context change is acceptable
only when semantic tests exercise the real engine behavior at minimum:
temporal visibility, source discovery, ranking/evidence, resource truncation,
authorization, persistence/reopen determinism, and the affected transport or
adapter boundary.
