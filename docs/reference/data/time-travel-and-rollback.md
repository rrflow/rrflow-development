# RRFlow temporal reads and forward compensation

**Status:** active implementation reference; temporal reads are tested, forward compensation is an unqualified planner
**Coordinate:** `rrflow://rrflow-instance/data/reference/data/time-travel-and-rollback`
**Owner:** bitemporal read coordinates and forward-only historical compensation semantics

RRFlow never rewrites accepted history to simulate rollback. Historical reads
select an immutable transaction prefix and a modeled valid time. A correction
or rollback is a later governed transaction whose new versions compensate for
the selected earlier structural state.

This record separates the tested read behavior from the currently incomplete
forward-compensation surface. It does not declare multi-model rollback,
physical snapshot rollback, projection rollback, or public rollback delivery
complete.

## Two independent time coordinates

rrflowQL requires both coordinates:

- `AT VALID <millis>` selects modeled valid time; and
- `KNOWN <cursor|HEAD>` selects the accepted transaction-log prefix visible to
  the query.

A literal known cursor bounds the source watermark and authoritative replay.
Records, relations, events, claims, series, and geospatial values are reduced
only from changes at or before that cursor and valid at the requested time.
Same-valid-time claim corrections resolve within that transaction prefix.

Query-index eligibility uses the same source cursor. A materialized index
newer than the requested `KNOWN` cursor is ineligible, so the current planner
selects its authoritative replay path instead of allowing future index state
to leak into a historical result. The
[query index catalogue](../query/index-catalogue.md) owns those selection
rules; the [multi-model query record](../query/multi-model.md) owns source and
predicate behavior.

The current historical path is logically consistent but physically eager. It
replays retained changes and materializes rows rather than reading versioned
rrflowKV keys or stamped Arrow-compatible pages. Gate C-04 must replace that
normal execution path while preserving these results exactly, and Gate F must
carry the same coordinates through every DataFusion access path.

## Implemented forward-compensation planner

`RrdEngine::plan_forward_rollback` accepts:

- a target valid time and older known cursor;
- a later effective time and bounded reason;
- the caller-supplied original read cursor; and
- an idempotency correlation identifier.

The planner validates that the target cursor is older than the supplied read
cursor and that the effective time follows the target valid time. It reads the
current authenticated log prefix, retains record, relation, and retirement
mutations, reconstructs current and target `RuntimeGraphSnapshot` values, and
calculates a deterministic differential.

It returns ordered transaction mutations that:

- publish new record/relation versions at the effective time when the target
  value is absent from or differs from the current state;
- close current record/relation versions at the effective time when they are
  absent from the target state; and
- append a `historical-rollback` evidence claim binding the original cursor,
  target coordinates, target-state digest, reason digest, compensation counts,
  and idempotency identity.

The plan and its ordered mutation list are content-addressed. Planning is
side-effect free. A caller must still submit the plan through the ordinary
authorized transaction operation for read-stamp validation, conflict
detection, audit, projection work, and durable commit semantics.

## Unqualified boundary

The repository currently provides strict request, plan, count, and receipt
types plus the engine plan builder. It does **not** yet provide one integrated
public operation that captures the original `ReadStamp`, plans, authorizes,
commits, and returns a validated `ForwardRollbackReceipt`. No focused test
executes a compensation plan and proves old-stamp stability, head-state
compensation, conflict behavior, idempotent retry, and rrflowKV reopen.

The planner deliberately covers only record and relation state. It excludes
claims other than its evidence claim, schemas, events and event-derived graph
rows, vectors, series, geo values, objects, index generations, reasoning-tree
state, feedback policies, security audit history, and deployment snapshots.
Each family needs explicit restoration or retirement semantics; none may be
silently inferred from the structural differential.

Therefore the existing capability is a useful pre-release draft, not a
supported historical-rollback endpoint. The product, HTTP, WebSocket, MCP,
SDK, CLI, and Connectome surfaces must not advertise it as executable until a
roadmap gate owns the full operation and its failure/reopen evidence.

## Required convergence

The following sequence preserves a single authority:

1. Gate C-02 supplies common rrflowMX/rrflowKV transaction rollback and
   conflict primitives; this is storage-transaction rollback, not historical
   compensation.
2. Gates C-03 and C-04 make every canonical family and synchronous index
   mutation atomic and readable at one retained `ReadStamp` without whole-log
   reconstruction.
3. Gates E and F prove graph, BM25, vector, scalar, and DataFusion execution
   cannot cross the selected known cursor.
4. Gate H-02 gives reasoning-feedback policy its own versioned,
   replay-stable compensation behavior.
5. A separately assigned delivery package may then integrate forward
   compensation through `RrdEngine` and qualify every outward adapter against
   the same contract.

The shared commit and object semantics are defined by the
[multi-model and immutable-object contract](multi-model-object-contract.md).
The [RRFlow 1.0 roadmap](../../roadmap/rrflow-1.0.md) remains the delivery
authority.

## Current executable evidence

`crates/compute/rrd-query/tests/query.rs` proves record and claim corrections
return different exact values at `KNOWN 10` and `KNOWN HEAD`, with identical
results on rrflowMX, rrflowKV, and rrflowKV after reopen.
`crates/compute/rrd-query/tests/index_catalogue.rs` proves an index newer than
a historical cursor is rejected in favor of the authoritative scan.
`crates/transport/rrd-contract/tests/public_contract.rs` proves strict rollback
request decoding and checked compensation counts.

Those tests do not execute `plan_forward_rollback`; the missing plan and commit
scenario is deliberately recorded above rather than converted into evidence by
documentation.
