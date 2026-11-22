# Public bitemporal database transactions and desktop proof

**Status:** active implementation and verification
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/bitemporal-public-transactions-20261122`
**Owner:** OBJ-01/OBJ-02 prerequisites, Gates C/E/H-04/H-06; database semantics and the installed public transaction journey

## Before-edit brief

Baseline `b7def0fc79d1ed4e4178d37d6cc15e0f14b0f70e`, Connectome
`36d388e919c27de7b72fbccb3d7e7af7bbb71030`. Driver: this owner-requested
database audit and installed Connectome session. No prerequisite PR is open.
Keep one complete root candidate draft while implementing and verifying.

Current failure: `RrdEngine::execute_query_transaction` implements bounded,
typed BEGIN/MUTATE/COMMIT or CANCEL programs through the existing transaction
authority, but no public operation descriptor, HTTP dispatch or generated SDK
operation exposes it. Clients cannot execute that program through RRD. The
architecture also incorrectly says that Vortex and cold-history readback do
not exist, while production cold publication still bypasses the columnar port.
The development launcher replaces a running staged Windows executable even
when its bytes are unchanged; this session reproduced WinError 5.

Target: expose the existing program through one catalogue-owned authenticated,
idempotent operation, retain begin/commit/abort authorization and temporal
receipts, and prove a corrected document plus relationship through HTTP,
historical queries and restart. Document DB/QL/KV/LSM/CLD/MX/TurboQuant and the
proposed rrflowTX responsibility precisely against source and primary SurrealDB
references. Reuse an identical private staged executable without replacing it.
The installed driver is Connectome add-project -> real RRD -> public SDK
transaction -> rrflowQL old/new knowledge coordinates -> restart/readback.

Scope: catalogue/data.rs, HTTP router/handler, Rust client operation/data.rs,
generated contract/SDK/reference projections, HTTP process transaction oracle,
scripts/dev launcher and tests, a reproducible installed proof driver if needed,
database semantics reference and owning architecture/client links, POA&M and
this journal/navigation. No second transaction authority, schema migration,
storage format replacement, provider activation or user-project reset. Any
Connectome source fix belongs in its own dependent PR before updating the pin.

Reviewed in full for this change: engine/query_transaction.rs; contract
catalogue/data.rs and catalogue/operation.rs; scripts/dev/connectome_dev.py and
test_connectome_dev.py; Connectome installation-facts.ts and
installation-summary.tsx. Reviewed ranges: HTTP handlers/data.rs query/export,
router operation map, client operation/data.rs schema/query/export, existing
HTTP data_transaction.rs first 150 lines; kernel runtime model/stamps and
storage semantic commit/keyspace/query files recorded in the detailed audit.
These reads are not a whole-codebase review.

Acceptance: public operation absent before; real HTTP program commits/replays,
cancel publishes nothing, malformed/unauthorized requests fail; queries at two
known cursors preserve the pre-correction answer across restart. Run focused
HTTP oracle then owning contract/server/client suites and generated projection
checks. Run development launcher tests and actual desktop journey. Source-based
documentation receives link/navigation/policy checks. No full ACID, Fjall,
SurrealDB/Qdrant parity or performance qualification is claimed.

Research: primary SurrealDB documentation for multi-model transactions, record
links/references/relations and protocols; no upstream source copied. Preserve
existing request/operation/read-stamp receipts and bounded diagnostics; do not
log credentials. Stop on unknown durable effects, divergent authority, lost
historical answers, required red checks or scope beyond this joined boundary.
No roadmap item is completed by documentation or a partial workflow.

## Initial installed observation

The debug engine at the baseline has SHA-256
`ae83b62a6e346a361cc6a80c3b141f652a4c4aaaa72808fea68a612f413d89d8`.
Both owner-supplied slash variants of the checkout path produced plan digest
`2eea776cfa5c885f0a47f46fe4a80a2246d110b0d75cfafa8ef5ba77448be797`
with the same identity; no installation was applied to that checkout.
Connectome installed the isolated `D:/temp/rrflow-bitemporal-proof-20261122`
project and authenticated its real engine at loopback port 11244. Existing
saved projects were preserved; one old locator is refused for `estate_id`.
The UI truthfully reports hosted sync and automatic discovery/idle as absent.
This establishes startup, not bitemporal transaction correctness.

## Validation and landing

Pending implementation, focused checks, complete diff review, candidate and
merge-queue CI. No completion or parity claim.
