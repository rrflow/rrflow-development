# rrflowQL transactions and joins

**Status:** active implementation reference; final transaction port and native access-path convergence remain incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/query/transactions-and-joins`
**Owner:** implemented same-stamp equi-join and bounded mutation-program behavior

This record describes current behavior; it does not create another transaction
coordinator or claim transport and SDK conformance. `RrdEngine` remains the
[single authority](../../decisions/0001-single-engine-authority.md), roadmap
[Gate C](../../roadmap/rrflow-1.0.md#gate-c--establish-the-sole-hybrid-persistent-rrflowkv-substrate)
owns the final transaction and semantic-commit substrate, and
[Gate F](../../roadmap/rrflow-1.0.md#gate-f--connect-rrflowkv-to-arrowdatafusion-correctly)
owns streamed analytical execution.

## Same-stamp equi-joins

The implemented read form is:

```text
FROM <source> JOIN <source> ON <left-field> = <right-field>
AT VALID <time> KNOWN <cursor|HEAD>
[WHERE <predicate>] PROJECT <fields> [LIMIT <rows>]
```

Binding resolves both sources against one captured catalogue and requires at
least one compatible value type for the join fields. Output fields are
qualified as `left.<field>` and `right.<field>`. Null values never join.

Current execution reads both source snapshots at the same authenticated
valid-time and known-at coordinate, evaluates a deterministic equi-join, and
feeds the result through the existing DataFusion filter, projection, and
semantic-limit path. Output is identity-ordered and emitted in deterministic
bounded batches. If intermediate cardinality exceeds `max_rows`, execution
fails instead of returning a partial join.

This implementation proves one-stamp semantics but still reconstructs broad
source state from the runtime log and materializes Arrow input. It does not
implement cost-based join order, page-level pushdown, or streaming native
sources. Outer and cross joins, aliases, aggregates, and subqueries are not
accepted syntax.

## Bounded mutation programs

The implemented program form is:

```text
BEGIN;
MUTATE $first;
MUTATE $second;
COMMIT;
```

`CANCEL` may replace `COMMIT`. A program contains one to 256 unique mutation
bindings and at most 64 KiB of text. It canonicalizes to one stable form and
rejects malformed, empty, duplicate, or unbound statements. Bindings reuse the
provider-neutral `TransactionMutation` contract for schema, claim, record,
relation, event, vector, series, geospatial, object, and retirement mutations;
the request bounds their combined encoded payload.

`RrdEngine::execute_query_transaction` parses, validates, and binds the entire
program before opening a transaction. It then uses the existing authenticated
begin, commit, or abort operations, including authorization, leases,
idempotency, read-stamp compare-and-swap, runtime hash-chain, and audit
behavior. `COMMIT` submits one mutation list; `CANCEL`, a validation failure,
or a rejected mutation publishes no data mutation. The begin record is bound
to the canonical program and ordered-mutation digest. Exact retry returns the
same transaction and receipt, while a changed program under the same key
conflicts.

The program deliberately uses typed mutation bindings instead of introducing
a second data-mutation grammar. Direct textual create/update/delete forms and
interactive read-your-writes statements are not implemented. Any future HTTP,
WebSocket, SDK, MCP, GraphQL, or Connectome projection must lower to this same
engine operation rather than create adapter-owned transaction state.

## Verified implementation boundary

The focused corpus proves parser canonicalization and rejection, same-stamp
equi-join equality against its reference result on rrflowMX and rrflowKV,
bounded intermediate cardinality, one-row output batching, and rrflowKV
close/reopen. Engine tests prove commit, cancel, rejected mutation, missing
binding, exact retry, authorization denial, audit coordinates, and persistent
reopen through the existing transaction authority.

These tests do not prove Gate C's final point/range transaction port, one
atomic record/graph/index physical batch, or Gate F's streamed rrflowKV Arrow
provider. Those remain required before this behavior can be treated as the
finished persistent analytical path.
