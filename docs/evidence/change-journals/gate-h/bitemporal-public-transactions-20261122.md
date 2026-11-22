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

Root PR: https://github.com/rrflow/rrflow-development/pull/494.
Dependent desktop PR: https://github.com/rrflow/connectome-development/pull/117.
The public binding delegates to the existing engine transaction authority;
no transaction implementation was replaced. New files are the HTTP oracle,
`scripts/dev/bitemporal_proof.py` and `docs/reference/data/database-semantics.md`.
The catalogue drives the Rust operation, OpenAPI and five language projections.
The generated fixture endpoint count changes from 87 to 88. No files are
deleted or moved. The launcher reuses identical private executable bytes.

Commands and results on Windows, with `CARGO_TARGET_DIR` set to
`D:/.usr/Temp/rrflow-root-target` and four Cargo jobs:

- Focused `public_query_transaction_preserves_corrections_cancel_and_replay_across_restart`:
  failed first with HTTP 404; passed after the public binding.
- `cargo test -p rrd-contract -p rrd-client -p rrd-server --all-targets --locked`:
  final run passed, 281 tests across targets. The first broad run exposed the
  stale 87-operation fixture, corrected to the actual 88 operations.
- `cargo clippy -p rrd-contract -p rrd-client -p rrd-server --all-targets --locked -- -D warnings`:
  passed. `cargo build --locked -p rrflow-cli --bin rrflow`: passed.
- `python scripts/dev/test_connectome_dev.py`: 11 passed.
- `pnpm --dir sdks/typescript check`: passed, including seven tests.
- `python scripts/ci/check_generated_surfaces.py --check`: passed, 88 operations,
  seven signal projections, OpenAPI digest
  `1408587885ad396797b05260bd0a7b5b855d95c2d37920ddc87e7ea21cafe232`.
- `python scripts/ci/check_documentation.py` and navigation render: passed
  during authoring; final journal/index reconciliation remains below.

The installed public SDK driver passed:

```powershell
$env:UV_PROJECT_ENVIRONMENT='D:/.usr/Temp/rrflow-python-proof-env'
uv run --locked --project sdks/python python scripts/dev/bitemporal_proof.py --engine D:/.usr/Temp/rrflow-root-target/debug/rrflow.exe --output D:/.usr/Temp/rrflow-installed-bitemporal-20261122
```

The retained database is the output directory's `project` subdirectory;
instance `instance-52ff55ae0e362f21637b617f`. Executable SHA-256 is
`7c2c933ed6d1c879d3141d76dbdbbb467f19857ecdffb87dbddbbd61a4f7dd1f`.
The driver installs through the public CLI, extends the existing schema through
the public Python SDK and commits two documents plus a relation. At valid 100,
known cursor 6 returns Original and cursor 7 returns Corrected. Valid 99 returns
no row. CANCEL publishes no replacement. After forcibly terminating its own
server child and reopening, both historical answers and the relation remain;
retrying the correction returns its retained receipt and cursor 7.
`evidence.json` SHA-256:
`f79bda9146ed4f056ce8b30035ce33529337879aa0098880e0ac58363615a295`.
Requests, installation plan, process announcements, stderr and complete query
plans/read manifests are retained alongside it; credentials are not copied into
the evidence. Build/test logs are in
`D:/.usr/Temp/rrflow-bitemporal-validation`. This is a debug build, not a signed
release or power-loss qualification.

Surfaced intermediate failures: the new operation's reference row was initially
absent and then out of binary-search order; the final catalogue is sorted and
the full suite passes. The existing Python environment refused access to its
lib64 entry; a separate locked UV environment ran the proof. The desktop's
existing Vite process briefly occupied port 1420 during restart; after its
normal shutdown the development launcher started successfully.

Actual desktop preview showed native storage location, server behavior and all
13 managed paths on demand; cancel left its isolated folder uninstalled. The
reopened proof database reached a new local server but readiness correctly
refused the desktop's old contract digest. PR117 must bind both its Rust and
TypeScript SDKs to this engine contract before this joined journey is complete.
Its initial CI run 44986579524 passed native jobs but failed the smoke assertion
for the removed heading; updating that assertion made the exact smoke journey
pass locally. Neither failure is hidden by accepting mismatched contracts.

No full workspace, power-loss, throughput, Fjall/SurrealDB/Qdrant parity, hosted
sync, controller or cold-tier qualification was run for this bounded package.
POAM-005/011/090/093 gain source-grounded findings, with no closure or roadmap
completion. The detailed reference names the remaining owners and distinguishes
reviewed files/ranges from the unreviewed remainder. Final paired desktop proof,
candidate and merge-queue CI are still required.

## Paired desktop verification

Engine source `dd1f7a182f6978b577915117f2090055df160749` is pinned by both
Connectome SDKs at child candidate `17f00fd485a01c8a6e5d6eb1be64fa02fc947fdc`.
No contract check was relaxed. Adding the retained project path recognized its
existing locator, skipped installation, started RRD and authenticated readiness.
The desktop showed the actual instance identity, three records and one relation.

Query lab visibly returned Original at valid 100/known 6 and Corrected at valid
100/known 7. A relation query returned alpha -> beta with valid_from 100. The
UI showed selected/rejected plans, exactness, source cursor, key scans/point
reads and decoded bytes. The engine diagnostics view showed DB/KV/QL/cold
publication cursors at 7. Those watermarks do not establish merged hot/cold QL.
Screenshots in the validation directory above: `connectome-live-database.jpg`,
`connectome-known-6-original.jpg`, `connectome-known-7-corrected.jpg`,
`connectome-persisted-relation.jpg`, `connectome-live-engine-diagnostics.jpg`.

The installed journey also reproduced a false Tables refusal: a schemaless
document table was treated as unreadable without a strict record schema. The
child repaired that condition; the same database now displays Corrected and
Evidence in Tables at cursor 7/schema 2, with 47 storage keys examined.
`connectome-schemaless-table.jpg` retains the result. The child journal records
the changed files and actual checks; no engine schema was altered for this fix.

The child regenerates and verifies value/query/offer golden fixtures by three
real process tests against the pinned source. Its final frontend check/build
and five project-manager smoke tests pass; native tests pass 88 with 34 ignored
process tests separately owned by CI (three were run locally as above).
The root's final documentation policy, navigation and execution-inventory
checks passed; generated surface parity passed. Source/diff review included
the new complete driver and HTTP oracle, full client data operations, HTTP data
handler/router, operation catalogue, launcher/tests, detailed reference and
changed documentation; generated projection changes were reviewed against the
canonical export. This is package review, not an exhaustive repository audit.

The root pin identifies the tested child candidate. Root candidate CI can run
against that exact published pin; root queue admission waits for the child's
green merge. No roadmap gate is marked complete. GitHub's PR/check/merge records
remain the authoritative landing evidence; candidate/main success is not
claimed before it is observed.
