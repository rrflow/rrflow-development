# Discover installed jobs through the public engine

**Status:** active candidate evidence; dependency landing and full CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-installed-job-discovery`
**Owner:** D-02a/D-02b / OBJ-14 / POAM-040; PR231; active SWAG engineering session

## Brief and baseline

[PR231](https://github.com/rrflow/rrflow-development/pull/231) carries the
pre-edit change brief. Starting from actual main `5b4039fe`, the isolated
worktree composes recovery candidate `98996a107b28da1f4317cd501135805ae2660d4a`,
tree `b77d43cd51ff8304778af69e2319d467f914c2cd`.
PR222, PR229 and PR230 are landing prerequisites. This advances the missing
public job discovery required for Connectome ordinary-edit recovery; it does
not complete that UI or the project-to-learning journey.

Previously a client needed an already-known job ID for public reads and replan
preview. The installation record and existing successor links are the canonical
lineage. The new authenticated `attunement-job-list` operation follows that
lineage under `EstateRead`, returning bounded summaries and continuation IDs.
Clients do not derive identities, inspect private storage or own another job
registry. No persisted format, phase execution, provider policy, task authority,
project mutation or version changes are introduced.

## Implementation and change checklist

The contract owns request/page validation, a maximum of 64 summaries and a
256 KiB encoded response ceiling. The engine validates each encountered plan,
job, successor proposal, generation and installed-estate binding. It reads only
the requested page and one successor lookahead, plus the optional cursor job;
it does not scan the complete history. The control sequence must be unchanged
across collection. A conflict yields no partial result or automatic retry.
Each subsequent page is a new read, not a retained cross-page snapshot.
`source_verification: not_checked` explicitly prevents treating stored source
digests as a fresh project scan. Existing replan preview/accept still own edits.

The catalogue and operation reference own HTTP/OpenAPI/native/generated SDK
metadata. The server delegates to the existing engine; the Rust client checks
page identity, bounds and lineage. Existing installed HTTP tests now discover
their job ID through this operation before exercising control and replan.

Reviewed owners: AGENTS, README, alpha objective, roadmap, Gate D,
POAM-040's explicit job-list decision, change-authoring and CI procedures.
Reviewed implementations: installed-record identity, `load_attunement`, replan
generation/acceptance/receipt validation, control get/sequence, public read
handler/router, native generated carriage, catalogue/schema export, and
installed engine/client tests. New discovery implementation and test files are
read in full; large existing owner files are inspected at affected symbols and
through their complete diffs. Final review covers the authored change and the
structural generated delta; no unrelated implementation changes were found.

Authored changes stay in the contract discovery module, engine discovery module
and tests, existing HTTP/native carriage and contract catalogue/reference.
Generated SDK/signal projections, the current digest/corpus metadata, public
reference, POAM-040 link and journal navigation follow their existing owners.
No implementation is moved, deleted, duplicated or replaced.

Research/source adaptation: no new external algorithm or dependency. Trace and
debugging use existing authorization audit and request/operation identities;
summaries contain no credentials or raw project source. Resource decisions are
bounded traversal and response size, not a throughput qualification. Stop
conditions prohibit unbounded traversal, client-derived authority, ambiguous
lineage, unsafe concurrent reads or materially broader behavior.

## Verification and surfaced failures

The first `cargo test --locked -p rrd-contract --test attunement_discovery`
fails because the catalogue contains no job-list operation (one failed,
exit 101). The retained `first-oracle.log` records that absence. After the
implementation, the same oracle passes. The expanded five-test contract group
passes request bounds, identity/lineage/time/cursor rejection, wire strictness,
maximum-page encoding and schema-projected limits.

`cargo check --locked -p rrd-engine -p rrd-server -p rrd-client` passes.
`cargo test --locked -p rrd-engine --lib engine::tests::attunement::discovery`
passes all five focused tests on actual temporary installed estates: initial
discovery, source-edit/read preservation, restart, successor pagination,
foreign/unknown cursor and token rejection, corrupt-link rejection and
deterministic rejection when an actual control mutation occurs during a read.
These protocol fixtures are not training data or model-quality evidence.

The existing authenticated HTTP/native test
`cargo test --locked -p rrd-client --test rrflowdb_local installed_typed_attunement_controls_replan_and_reopen`
passes discovery before control, ordinary-edit preview/accept, bounded successor
discovery and exact discovery readback after server reopen. It uses the real
installed engine and HTTP service, not a replacement transport. The service is
in-process, so this is not shipped-primary-binary or desktop proof.

`cargo test --locked -p rrd-engine --all-features --all-targets` passes 369 tests,
including 32 architecture checks, with four existing ignored process fixtures.
Its log SHA-256 is `18d14f6ec00644a5a9569aef6167cfc21fc512f18fdb17e7fb623d74a64be917`.
All five SDKs regenerate from the one export; generated parity verifies 74
operations and seven signal projections. The reviewed OpenAPI digest is
`f3ab16d619759961c28fa5e4494c4431acd4213380db6b3ee5ffc093c251e7af`.
The bootstrap corpus changes only `expected.endpoint_count`, derived from the
catalogue; a parsed comparison verifies every other field remains identical.
TypeScript generation parity, lint, type checking and all six tests pass.
Structural comparison against the prior hash-bound OpenAPI export finds only
the new `/v1/attunement/jobs/list` path and endpoint-count change; every existing
endpoint definition is unchanged.

After building the CI-required `rrd-estate-controller` process helper,
`cargo test --locked -p rrd-contract -p rrd-client -p rrd-server --all-features --all-targets`
passes all 227 tests with no failures or ignored tests. Its log SHA-256 is
`88448be4e7a46b2682441de18644c00f101aeedfd6ab6799ea2e903582d82374`.

The first Python generator command used system Python without its required
`ruff` executable and failed with `FileNotFoundError`. Re-running through
`uv --directory sdks/python run --frozen python scripts/generate.py` uses the
repository's locked environment and passes; no generator or dependency was
changed. Documentation checking first rejected this journal's missing `active`
status marker, then its now-stale generated index. Correcting the marker and
regenerating navigation resolves both failures; documentation checking passes.

Strict Clippy first rejects the boxed closure collection in the new contract
test as an overly complex type. Replacing it with an array of plain function
pointers preserves all 18 negative cases without suppressing the lint. Final
`cargo clippy --locked -p rrd-contract -p rrd-client -p rrd-server -p rrd-engine --all-features --all-targets -- -D warnings`
passes (log SHA-256 `41e96e01f294511053adf8e5ecc04a69b19be011f86c2eaaac23362af5d180f8`),
and all five contract tests pass again. Final review also corrects the conflict
message to say control state changed: authorization auditing can advance the
sequence without any source edit. A source-hash comparison proves that diagnostic
string and the test-helper representation are the only non-documentation changes
after the broad suites above. All five affected engine checks pass again with
all features enabled, and `cargo fmt --all -- --check` passes. Schema/export and
runtime decisions are otherwise unchanged.

Local command logs and their digests are retained under
`.swag/evidence/20261031/attunement-job-discovery/` in the SWAG workspace.
Navigation verifies 874 nodes and 3,150 edges; the execution inventory covers
2,387 paths; documentation checking verifies 875 statuses, 873 coordinates and
local links. Whitespace checks pass. Both repositories' existing AGENTS files
remain unchanged. These local checks and the authored/generated review do not
establish installed primary-binary or Connectome workflow acceptance.

## Remaining work and status

Keep the published candidate draft, reconcile dependencies after their actual
protected landing, then run this candidate's full CI. The next UI package must
pin the published engine SDK and use returned job/plan identities to preview
and approve ordinary-edit successors, with an actual matching-binary trial.
Public UI discovery/replanning, project effects, RROFLOW correlation,
independently reviewed learning and native/container release qualification
remain open. No objective, roadmap gate or POAM status is changed or closed.
