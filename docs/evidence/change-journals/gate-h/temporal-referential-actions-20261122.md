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

## Implementation and surfaced failures

The pre-change storage oracle actually ran on MX and KV (rather than the
planned HTTP baseline): retiring B at valid 150 after the edge moved at 200
and retired at 300 was accepted at cursor 8. Both cases failed their no-effects
assertions. `temporal-integrity-before.log` retains that result under
`D:/.usr/Temp/rrflow-bitemporal-validation`. The kernel now sweeps complete
endpoint/edge intervals at the captured knowledge cursor. Store selection uses
node-wide incident prefixes, complete edge histories and endpoint histories;
engine preparation expands CASCADE before function receipts and commit intent.

A second real defect appeared in the new cascade readback: the ordinary
relation retirement effect wrote its version only if a current edge existed.
An already-retired edge therefore acquired a log retirement without its version
index entry. `semantic_commit/effects.rs` now always writes that version and
uses the authenticated valid-time retirement target for temporal adjacency.
Current adjacency cleanup still uses the actual current edge. Retirement
validation moved intact into `semantic_commit/retirements.rs`, with ordered
same-batch target handling and histories grouped by identity to avoid rebuilding
a whole-scope snapshot for every retirement.

Definite preparation failure closes the lease before any domain commit attempt.
A typed non-publication error plus absent outcome also closes it. Unknown I/O
outcomes retain intent. Terminal records use the existing control transaction
and session journal; no new status store. Explicit abort acknowledges a refused
lease. Existing write-admission tests now inspect the retained terminal record
and assert Aborted, not merely an absent intent.

The initial engine suite exposed query-budget coupling and invalid fixtures.
Write relation integrity now uses the storage's finite authenticated-integrity
bound, including proposed identities; query-read limits remain query policy.
The context fixture now gives edges their endpoints' valid intervals and retires
the matching edge atomically. The inventory repair fixture similarly retires
its incident edge when deleting the record. The context visibility and inventory
repair assertions remain. Focused context, inventory-reuse, query-budget,
HNSW-unrelated-growth and large-inventory/reopen checks passed after those fixes.

The first broad engine run was deliberately stopped after these failures were
recorded and its compiled candidate was superseded; it is not a passing suite.
A simultaneous attempt to relink that still-running Windows test executable
failed LNK1104. The exact owned process was stopped before rerunning; no user
engine, desktop or unrelated process was terminated. The final broad run is
pending. Python generation initially selected a pre-existing Linux `.venv` and
failed its `lib64` cleanup. That environment was preserved; generation uses the
already-owned Windows `D:/.usr/Temp/rrflow-python-proof-env` instead.

## Scope review and verification so far

Reviewed changes: kernel schema declarations, canonical encoding, exports,
interval sweep and first-gap selection; store versioned source/range/key
validation, relation selection and commit resolution, retirement validation and
physical effects; engine lower/raise, internal schema literals, function-bound
preparation/replay, commit refusal outcomes and tests; public schema, protocol
identity, process oracle and installed driver. These are the changed symbols
and their directly supporting ranges, not a claim to read all listed crates.
The new code lives within the existing kernel/store/engine boundaries. The
public schema is projected by the existing generators into all five SDKs.

- Kernel focused temporal tests: five passed.
- Store unit suite after the historical-retirement repair: 114 passed, three
  pre-existing ignored tests. Later bound/refactoring changes are awaiting the
  final owning suite.
- Real HTTP process tests: both the previous correction/replay test and the new
  REJECT/CASCADE/restart test passed. The latter uses a one-transaction quota
  and three distinct refused requests before the successful cascade.
- All five SDK projections generated. OpenAPI becomes
  `59fbc172ad1c75997b82984a12e851f015076562eb69fcbc1cf90b3347e30f27`;
  the catalogue still has 88 HTTP operations. The protocol drift test initially
  failed until its owning digest constant was intentionally updated. No gate
  was relaxed.
- Installed proof now includes three refusals, schema-authorized cascade,
  old/new knowledge readback at valid 149/150/175, and exact replay after process
  termination. Its new candidate run, desktop readback, strict lint and final
  candidate/merge-group CI are pending.

Paired Connectome PR119 owns both SDK pins and a read-only Tables panel that
explains source/target rules from the authenticated schema, with its revision
and cursor. Parent gitlink advances only after that paired candidate passes.
Field references, UNSET, vector/geo cleanup, fine-grained concurrent semantic
commits, cold-tier query convergence and parity remain unfinished. POAM-072/073
are not closed on the evidence available at this commit.
