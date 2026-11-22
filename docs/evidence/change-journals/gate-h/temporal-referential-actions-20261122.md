# Temporal relation integrity and transaction outcomes

**Status:** active implementation package; acceptance evidence pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/temporal-referential-actions-20261122`
**Owner:** H-08 native data semantics; OBJ-01/04/07; POAM-072/073

## Change brief before implementation

Baseline: `b08542f7ec32f942321b27877473b3bc9bf01edf`. Driver: this engineering
session, following the installed `scripts/dev/bitemporal_proof.py` journey
landed by PR494 and paired Connectome PR117. No prerequisite branch remains.
GitHub owns candidate, queue and merge state. The broader database request is
not satisfied by this package alone.

Current behavior: semantic retirement removes a record head and appends its
retirement, without retiring or restricting incident relationships. Creation
checks current endpoint presence, not its full valid-time interval. A definite
commit refusal clears prepared intent but leaves the transaction lease open.

Target: schema-defined REJECT/CASCADE rules for each relation endpoint;
endpoint validity checked at one knowledge coordinate; generated cascade
retirements published as ordinary mutations in the existing atomic commit,
including authenticated history, adjacency, audit and projection effects.
Refused commits with proven non-publication must release the transaction;
unknown I/O outcomes must retain exact-retry reconciliation. Historical reads
before a correction or retirement must remain unchanged across restart.

Canonical scope: kernel schema/value encoding and temporal relation policy;
store versioned selection and semantic-plan validation; engine transaction
preparation/replay and terminal outcomes; public relation-schema projection and
its existing SDK generators; public HTTP/process oracle and installed proof
driver. Connectome consumes the changed schema through its existing SDK binding
and must demonstrate the same project after reopen. Its required pin/fixture
change uses its own linked PR. Detailed semantics and POAM owners receive
findings; no global capability or roadmap completion is implied.

Read boundaries: full `rrd-core/src/schema/types.rs`,
`rrd-store/src/access/semantic_commit.rs`, `constraints.rs`, `effects.rs`,
`rrd-engine/src/engine/query_transaction.rs`, and the linked POAM-072/073,
README, AGENTS and change-authoring procedure. Reviewed ranges additionally:
kernel schema registry object validation; graph snapshot reduction; canonical
schema encoding; store semantic-plan validation; engine commit/abort paths,
transaction records, lowering and function-binding preparation. These are
targeted reads, not a whole-repository review. Further exact paths and reviewed
objects will be recorded with implementation evidence.

Traceability: retain the existing `RuntimeCommit`/`SemanticCommitPlan` write
authority and versioned read authority. Replace current-only relation reference
validation there; do not invent a second relation store or bypass commit
receipts. Reuse the existing relation-retirement effects and temporal adjacency
encoding. Field links/backlinks/UNSET, vector/geo subject cleanup, SQL syntax,
wire protocols and cold-tier convergence remain separate unfinished behavior.

First-failure oracle: through the existing public transaction operation, retire
a referenced endpoint and observe the live dangling relation on this baseline.
Acceptance must show REJECT publishes neither endpoint nor relationship change;
CASCADE removes the endpoint and incident edge together; both retain old-known
answers and pre-retirement valid-time answers after forced restart. Include
backdated/future versions, self-loops, both endpoints, batch overrides, bounded
fan-out, malformed/auth-denied requests, exact retries and failed-commit cleanup.

Validation sequence: focused kernel/store/engine oracles, owning suites and
strict lint; generated SDK parity; HTTP process and installed-driver readback;
actual Connectome inspection; candidate and merge-group CI on final revisions.
Keep drafts incomplete until the joined proof passes. Commands/results are
pending and must not be inferred from this brief.

Research: inspected SurrealDB's current primary record-reference documentation
(https://surrealdb.com/docs/reference/query-language/language-primitives/record-references).
Its tracked field references differ from edge records; reference tracking alone
does not require target existence, and UNSET removes field references. RRFlow's
relation endpoint integrity is an explicit native contract, not SurrealQL
compatibility or a claim to implement field-reference behavior. No upstream
code is copied. Preserve existing engine diagnostics; retain exact requests,
read stamps, outcomes and readback. Resource evidence must expose bounded
selection/fan-out; no throughput, power-loss or hardware-lifetime claim.

Stop conditions: unresolved effect identity or replay, unbounded dependency
selection, lost historical answers, a second transaction authority, unrelated
user-state mutation, or a refused operation reported as success. Resolve these
within the canonical boundary before dependent work proceeds. No engine
capability, POAM closure or gate completion is claimed yet.
