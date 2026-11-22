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

## Configuration and diagnostic follow-through, 2026-11-22

The owner's six-layer configurability direction prompted a code-grounded
review before any new engine-tier implementation. Root starting revision:
`01efc47dfbbd2b13a5bcc143e9741452969c5753`; Connectome starting revision:
`dd54eb1d90b4ec1036ebb33910b206e741e6b545`. The
[six-layer review](six-layer-boundary-review-20261122.md) remains the record
of the failed production-backend feature probes and the QL/DB/CLD replacement
boundary. No additional QL, TX, MX, DB, LSM or CLD runtime implementation was
changed in this follow-through. The paired UI projection was repaired and
POAM-026 records the newly verified diagnostic deficiency without closing it.

Complete root files read for this review:

- `Cargo.toml`;
- `docs/reference/deployment/scoped-configuration.md`;
- `docs/evidence/change-journals/gate-h/observability-diagnostic-build-and-latency-evidence-planning-journal.md`;
- `crates/transport/rrd-contract/src/diagnostic.rs`;
- `crates/authority/rrd-engine/src/engine/diagnostic.rs`;
- `crates/authority/rrd-engine/src/engine/diagnostic/storage.rs`;
- `crates/operations/rrd-observation/src/lib.rs`;
- `crates/persistence/rrd-lsm/src/write_diagnostics.rs`;
- `crates/authority/rrd-engine/assets/install/default-engram-configuration-v3.toml`;
- `docs/poam/rrflow-1.0-alpha/poam-026.md` and `poam-029.md`.

The runtime-mode, metric and latency sections of `engine-data-flow.md` were
read as sections, not the entire file. The diagnostic Cargo profile and
bounded write recorder exist; full release/diagnostic parity, a complete
operation trace and six-layer live configuration do not follow from those
definitions. The UI [supporting pattern](https://github.com/rrflow/connectome-development/blob/feat/temporal-relation-policy/docs/engine-evidence.md)
maps the requested controls to their engine owners and the existing native,
model and view files. The same engine semantics must serve internal and
customer builds; optional diagnostic collection cannot add another authority.

PR119 now preserves independently measured cursors, physical sequences,
unavailable/reset counters and stale-read failures. A first model oracle
failed on the prior inferred KV agreement; it passes after the correction.
Final `corepack pnpm check`, `corepack pnpm build`, and
`corepack pnpm test:smoke --workers=2` pass (38 browser cases). The child
journal records the formatting, wrapper-type and terminology failures that
were fixed. Browser mocks qualify presentation only.

Actual desktop readback on the retained installed project now shows source
REJECT / target CASCADE at schema 3, knowledge 10; zero current `proof_link`
rows; one historical row at valid 175 / known 7; zero at valid 175 / known 10;
and one at valid 149 / known 10. Query work and unavailable policy-admission
evidence are displayed as returned. Engine overview shows semantic/cold
cursor 10 separately from the physical durable sequence. Screenshots/logs
remain under `D:/.usr/Temp/rrflow-bitemporal-validation`; the actual project
is `D:/.usr/Temp/rrflow-temporal-relations-20261122/project`.

Earlier runs now checked against retained logs: engine library 531 passed,
12 ignored; native Connectome 88 passed, 34 ignored; real-process goldens
three passed; latest focused engine tests four passed and real HTTP
`query_transaction` tests two passed. Those are earlier package runs, not
reruns in this documentation/UI review. The installed driver's retained
`evidence.json` identifies its unqualified Windows debug binary by SHA-256
`1aada6feeadeaaedba6d0dd6c8596338ed07e12f1e1500716708abf067cff03e`.
No benchmark or release/diagnostic parity ran; exact planned backend test and
benchmark commands remain in the linked six-layer review and are still not
runnable production targets. Candidate/queue qualification and the parent
gitlink advance remain pending. No schema, tier or parity gate is closed.

### Verified UI candidate and source pin

Connectome revision `a54f2f92d28145dda6f94a2445f1197d22517cbb` passed
[candidate run 45042570704](https://github.com/rrflow/connectome-development/actions/runs/45042570704):
frontend check/build/render smoke, native format/check/test/clippy on Linux,
macOS and Windows, real RRD process integration, and `ci-gate`. The final
local build and all 38 browser cases were also rerun successfully after the
last graph-total correction. PR119 is ready and queued through the protected
merge queue; the superseded candidate was cancelled and is not passing
evidence. The parent now pins that verified UI source. Queue/main results
remain GitHub-owned and are not inferred from candidate success. The engine
PR remains a draft under the six-layer review disposition; no additional
engine capability or roadmap closure is claimed by advancing this gitlink.

### Queue qualification and transaction refusal repair

Connectome PR119 merged at `54d178b8ba0ea6cea2463a9955f6998947bb5623`
through [merge-group CI](https://github.com/rrflow/connectome-development/actions/runs/45049877869).
The engine candidate at `e478b10f3421dcf7f3d06ce42abb92f541b43bec`
failed its [repository invariant check](https://github.com/rrflow/rrflow-development/actions/runs/45049859374/job/137292591981).
Its new refusal helper increased the engine crate's allowed oversized argument
lists from 108 to 109. The same failure reproduced locally with
`python scripts/ci/check_code_shape.py`.

Both refusal paths now pass the existing `EngineControlContext` to
`finish_rejected_transaction`. The session identity, action, time and trace
identifiers are the same inputs to the same session compare-and-swap.
The edit stays within transaction coordination. It changes no physical storage
format, query semantics or public contract. The focused review covered both
callers in `engine/transaction.rs`, the complete `transaction/outcomes.rs` and
`engine/control.rs` files, and the session storage and authentication methods.
`python scripts/ci/check_code_shape.py`, `cargo fmt --all -- --check` and
`git diff --check` passed locally after the repair. The separate worktree
`D:/.usr/Temp/rrflow-queue-496-shape` owns its Cargo target directory.
`cargo test --locked --jobs 3 -p rrd-server --test http_process query_transaction`
exercises the public transaction paths, including refusal, CASCADE and reopen.
Final candidate and queue outcomes are recorded by GitHub.
