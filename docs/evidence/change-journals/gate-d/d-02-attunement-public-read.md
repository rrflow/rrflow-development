# D-02: expose the stored attunement plan and status

**Status:** active implementation candidate; dependency and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-attunement-public-read`
**Owner:** OBJ-07 / D-02a / POAM-006 / PR146

## Brief, scope and authority

The starting public development main was `04fae27665c3081ef6bc193d54d8c1c9c8d3f2e0`,
tree `0effb6f859ddd0a99e24efd6016f80e06405cd40`. Empty opening commit
`380fe7aff00b16bf775bdafcbcffcfcf04e3c742` opened
[PR146](https://github.com/rrflow/rrflow-development/pull/146) with its change
brief before code changes. [PR145](https://github.com/rrflow/rrflow-development/pull/145)
records the related public-operation decision and is a landing dependency.

Previously the installed `RrdEngine` could read one job internally, but the
public catalogue did not expose that read or its stored plan. This package adds
one authenticated, non-mutating `attunement-read` operation using EstateRead.
The same engine loads and validates its stored plan and status; the Rust client
checks the requested job and target instance identities. `not_checked` states
that this read does not check the current source. The earlier installation
precondition digest is not a current project snapshot. This does not add job
discovery or control, source scanning, replan, a provider effect, an additional
lifecycle owner, a storage format, or a product version change. Existing
authorization audit remains in place. Stop on a second authority, source
activation, unverifiable identity, or generated contract drift.

Implementation and tests changed in `crates/authority/rrd-engine/src/engine/`
(`attunement.rs` and `tests/attunement.rs`); `crates/transport/rrd-contract/`
(`src/attunement.rs`, `src/lib.rs`, and the attunement and public contract
tests); `crates/transport/rrd-server/` (router, handler module, new
`handlers/attunement.rs`, and `tests/http_process.rs`); and
`crates/transport/rrd-client/` (`src/operation.rs`, `tests/rrflowdb_local.rs`).
The shared SDK corpus endpoint count and five SDK endpoint-count tests changed.
The language endpoint/schema and signal projections and current protocol/SDK
digest citations were regenerated or reconciled from the same contract. This
record and generated navigation/inventory complete the repository receipt.
No implementation was moved, removed, or replaced.

Reviewed the README, D-02 and POAM-006 owners, PR145's pending decision and
source-binding gap, the change-authoring and generated-surface procedures,
attunement contract validation, engine load/read and transition tests,
installation inventory, authenticated router/handler/client patterns, HTTP
security fixture, SDK generation scripts, and the diff. No external research,
upstream adaptation, new tracing, or resource benchmark is needed for this
read-only projection of existing engine state. Existing authorization audit is
preserved; this is not proof of a memory-creation audit.

## Oracles and results

The first catalogue test failed on the starting implementation because
`attunement-read` was absent, then passed after the implementation. The full
contract suite initially caught its previous first-endpoint assertion after the
new sorted entry was added; after correcting it,
`cargo test --offline --locked -p rrd-contract --quiet` passed all tests.
The new contract tests reject unknown verification claims, extra fields,
unsupported versions and a plan/job digest mismatch.

On Windows `cargo test --offline --locked -p rrd-engine --lib
engine::tests::attunement::` passed nine tests, with one subprocess fixture
ignored as a direct test and exercised by its parent. This includes actual
failed inventory state and unchecked source, bad token, cross-project job
isolation, worker interruption, recovery and replay of the existing job
control; it does not claim those controls are public. `cargo test --offline
--locked -p rrd-client --test rrflowdb_local` passed all three tests: the new
installed HTTP read returns the original plan and pending job, continues to
report unchecked source after a source edit, rejects an unknown job and reads
the same durable job after the server reopens. The server's real-process
security test passed with 401 for no session and 403 without EstateRead, and
read back the denied EstateRead event from the existing audit journal.

`cargo clippy --offline --locked -p rrd-contract -p rrd-engine -p rrd-server
-p rrd-client --all-targets -- -D warnings` passed. The complete local
`cargo test --offline --locked -p rrd-client --quiet` suite passed. TypeScript
client tests passed six tests; `go test ./...` passed.
`python scripts/ci/check_generated_surfaces.py --check` passed for 56 operations
and seven signal projections, with OpenAPI SHA-256
`d9d95f65139470584c31e08cfc16495d33d689785e42b45b25637d25d0068bca`.

The first `cargo test --offline --locked -p rrd-server --quiet` run failed
because `rrd-estate-controller.exe` had not been built for the local process
fixture. After the documented CI build, `cargo build --offline --locked -p
rrflow-cli --bin rrd-estate-controller`, the same server suite passed all
30 tests. The added audit assertion was subsequently rerun and passed.
The initial Python SDK generation lacked `ruff` on PATH; the declared pinned
`ruff==0.17.1` was installed in an isolated tooling environment, and the
generator succeeded. Offline `uv sync` could not find the required Python 3.15
interpreter; local Python SDK tests remain unrun. The local pnpm wrapper also
failed, so the TypeScript generator and tests used the already installed pinned
toolchain via Node. A PowerShell command with brace expansion failed during
inspection; a subsequent explicit path search succeeded. The first PR-body
update attempt failed on PowerShell backtick expansion and was corrected.
`cargo fmt --all -- --check`, `git diff --check`, navigation and execution
inventory checks, documentation policy and CI workflow policy passed; the
generated inventory includes 1769 current, generated and planned paths. A
subsequent inventory check detected the journal's updated digest; regenerating
the inventory after the final journal edit resolves that expected drift.

## Remaining evidence and checklist

Checklist: baseline and owner identified; PR brief published first; smallest
failure observed; one engine authority used; exact plan/status and authorization
oracles added; source freshness explicitly unverified; catalogue projections
regenerated; diff and owning tests reviewed; linked journal and navigation
required. PR145, the exact candidate pipeline, and integration into current
main remain pending. Python, Java and .NET SDK suites and the full shared SDK
conformance corpus have not been run locally. This package does not change
roadmap or POA&M completion status. Per-prompt plans, minimal questions,
source-bound effect execution, public job discovery/control, memory effect
receipts, SWAG adaptation, and their end-to-end proof remain open.
