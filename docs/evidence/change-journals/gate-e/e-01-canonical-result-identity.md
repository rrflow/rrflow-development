# E-01/E-05 canonical query and context result identity

**Status:** active draft implementation evidence; no gate or release status changed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/e-01-canonical-result-identity`
**Owner:** [Gate E](../../../roadmap/rrflow-1.0/gate-e.md); [POAM-005](../../../poam/rrflow-1.0-alpha/poam-005.md)
**Candidate:** [PR 266](https://github.com/rrflow/rrflow-development/pull/266); starting main `a2fc4a3c57960e1d4fd10cdafe208d5b908daa62`, bootstrap `08344aef94c358c9ba8c629b5d2f3af38a50c9b0`.

## Change brief and traceability

The typed native reference `(kind, id)` accepted colons, while query and
context identities concatenated raw members. Context keyed candidates by the
result string, so `(a:b, c)` and `(a, b:c)` collided. Series rows omitted the
sample's reference kind. The versioned shared encoder now escapes `%` and `:`
within components, preserves the typed pair, and is used by query and context.
Traversal paths retain those component boundaries. Claims keep the existing
opaque context SHA-256 identity, shared with query; it is not reversible and
is not a mathematical collision-free guarantee.

The canonical source remains `RuntimeRef` in `rrd-core` and the native
rrflowKV semantic transaction. Previous query/context string construction in
`rrd-query/src/execute.rs`, `physical/graph.rs`, and `rrd-engine` context is
replaced by `rrd-core/src/result_identity.rs`; the physical store key codec is
unchanged. Query Arrow rows, live diff keys, BM25 documents, and persisted
index artifacts consume these result identities. Index catalogue entries and
artifacts carry the result identity version; the planner excludes entries
without the current version, and a public ensure rebuilds from authoritative
rows. The stored state remains visible and the public projection reports
`identity_rebuild_required`. Publication verifies the addressed artifact;
unique commit enforcement remains installed. The new tests cover prior-format
entry rejection/full rebuild, unique enforcement, source reopen and identity
separation. They do not prove SQL/PostgreSQL compatibility or the project
indexing journey.

Further cross-boundary review found that prompt-flight evidence parsed record
identities with a raw delimiter, while hybrid and keyword retrieval duplicated
that assumption and memory warp resolution rebuilt a raw string. Prompt-flight
and both retrieval paths now decode through the same canonical reference
function; memory warp and graph context evidence use the shared encoder.
Native references that the narrower public flight contract cannot represent
are explicitly refused before runtime publication; they are not silently
rewritten or attached to another record. This does not remove the public
identifier restriction in POAM-005.

Review identified a separate continuation hazard: clients and durable live
subscriptions could resume with old result keys after the engine changed
encoding. The in-progress repair carries the result-identity version in query
results and live deltas, requires it on nonzero live-poll continuation, and
binds it to durable live-subscription state. An older live subscription is
refused for connect/delivery but can be closed; changefeed subscriptions do not
depend on row identity. This added behavior remains subject to its current
tests and installed-driver evidence; no cursor or receipt is silently rewritten.

The parent change brief in PR 266 records exact scope, research, publication
guards and stop conditions. PR 265 also edits the HTTP process test; its ready
candidate must land first and be normally merged into this branch before
candidate CI. No platform, C-07, or alpha gate is marked complete.

## Research, diagnostic and resource decisions

PostgreSQL's [lexical rules](https://www.postgresql.org/docs/current/sql-syntax-lexical.html)
illustrate why punctuation and quoted identifiers require explicit parsing;
they are not evidence of RRFlow PostgreSQL compatibility. Apache Arrow's
[columnar format](https://arrow.apache.org/docs/format/Columnar.html) describes
variable-width string offsets, not a canonical identity scheme. No upstream
code was copied. Existing query budgets, read stamps, index receipt and control
journal are retained. Escaping can expand a component up to three times its
UTF-8 byte length; a native performance or peak-memory benchmark has not been
run. Public `DataReference` and source-kind grammar remain more restrictive
than native IDs, as recorded in POAM-005.

## Working evidence and outstanding checks

- Windows `cargo test --offline --locked -p rrd-query --test query --test index_catalogue --test graph_traversal`: 40 passed on the earlier local implementation, before the final graph-path and legacy-artifact test additions.
- Windows `cargo test --offline --locked -p rrd-engine --lib`: 334 passed, 4 ignored. It includes distinct native typed records through real context after rrflowKV reopen and public ensure of a legacy identity index while uniqueness remains active.
- The first combined core/query/contract run failed the intentional OpenAPI digest golden after adding the public snapshot fields. The generated schema was reviewed and the pinned digest updated. A subsequent `cargo test --offline --locked -p rrd-core -p rrd-query -p rrd-contract` passed, including the mixed-kind relationship path and source reopen. One additional missing-artifact publication assertion was added to the index test after that run began and still needs its final run.
- The TypeScript pnpm wrapper initially failed due to a broken global pnpm installation; an isolated Corepack home with the pinned pnpm version installed the lockfile offline from the local store. SDK projections were generated from the contract exporter. Runtime installation does not fetch these tools.
- TypeScript `pnpm lint`, `pnpm typecheck`, and `pnpm test` passed (six tests), using the local locked package store.
- Authenticated socket-level `rrd-server` data/relationship/reopen integration passed one selected test; its server is in-process, not the primary executable.
- The first primary-executable installed lifecycle run failed in its C-07c-style storage-history oracle: returned claims had the new opaque keys but the predeclared oracle expected old cleartext keys. The oracle was updated to calculate the documented key independently. The subsequent Windows `cargo test --offline --locked -p rrflow-cli --test installed_lifecycle installed_engine_plans_applies_serves_discovers_authenticates_and_reopens -- --exact` passed, including the primary executable, authenticated query/context identity agreement and restart readback. This is not the separate C-07 native capacity or soak qualification.
- After adding versioned live continuation, the grouped query `live_query`, `index_catalogue`, and `graph_traversal` tests passed 16 tests. The engine subscription subset passed six tests after the legacy ACK/renew guards were added. The public and WebSocket contract tests passed 54 tests before the subsequent context-packet schema addition.
- The full socket-level HTTP suite passed 21 tests, including denial of a live cursor without its identity version; this preceded the final plan and context-packet bindings.
- Plan golden vectors were intentionally refreshed: the diff adds `result_identity_version: 2` and updates only the affected plan digests.
- After the final plan and context-packet changes, Windows `cargo test --offline --locked -p rrd-query --test query --test live_query --test index_catalogue --test graph_traversal --test golden` passed 44 tests; `cargo test --offline --locked -p rrd-contract --test public_contract --test websocket_contract` passed 54; and `cargo test --offline --locked -p rrd-engine --lib engine::tests::context` passed six. The native collision fixture is seeded at the engine storage boundary because the public `DataReference` contract rejects these particular legal native components.
- `python -X utf8 scripts/ci/check_generated_surfaces.py --check` passed: 75 HTTP operations, seven signal projections, OpenAPI SHA-256 `12e52bb72a3be1c220d1badf3af0124be7d5c08c9f2c875249a9c3ab43eb7e3d`. After the final regeneration, TypeScript `corepack pnpm lint`, `corepack pnpm typecheck`, and `corepack pnpm test` passed; the latter ran six tests.
- The first `python -X utf8 scripts/ci/check_documentation.py` run rejected this journal's status because it did not say `active` or `historical`; the status was corrected to `active draft`, retaining the explicit statement that no gate status changed.
- Windows `cargo clippy --offline --locked -p rrd-core -p rrd-query -p rrd-contract -p rrd-engine -p rrflow-cli --all-targets -- -D warnings` passed. The repeated `python -X utf8 scripts/ci/check_documentation.py` and `python -X utf8 scripts/knowledge/render_navigation.py --check` passed after the status correction.
- An attempted `cargo test --offline --locked -p rrflow-cli --test installed_reopen --test installed_crash --no-run` was rejected before compilation because `installed_crash` requires the `crash-test` feature; compile those targets with the feature enabled before relying on them.
- `cargo test --offline --locked -p rrflow-cli --features crash-test --test installed_reopen --test installed_crash --no-run` subsequently compiled both drivers. This is compilation only, not crash or 100-cycle execution, and it preceded the final flight/retrieval consumer correction.
- After the first flight correction, `cargo test --offline --locked -p rrd-engine --lib flight` passed eight tests, including malformed reference denial and a real engine prompt flight that refused an unrepresentable reference without advancing the runtime cursor. After the additional retrieval and context corrections, `cargo test --offline --locked -p rrd-engine --lib` passed 337 tests, with four subprocess fixtures ignored, and the primary-executable installed lifecycle command above passed again.
- The subsequent engine-only strict Clippy run failed `items_after_test_module` in the two newly added test modules. After moving them to the ends of their files, `cargo clippy --offline --locked -p rrd-engine --all-targets -- -D warnings`, `cargo test --offline --locked -p rrd-engine --lib flight` (eight passed), `cargo test --offline --locked -p rrd-engine --lib text_results_require_the_expected_kind_and_canonical_reference` (one passed), and `cargo fmt --all -- --check` passed. The move changed only test placement.
- The first `python scripts/ci/build_execution_inventory.py --check` reported drift after staging the new source and journal files. Running `python scripts/ci/build_execution_inventory.py` regenerated the replaceable local inventory; a repeated `--check` passed with 2,352 records. It is not a semantic or completion authority.
- Candidate and merge-group CI, the public delimiter-bearing mutation/BM25 path, and cross-platform capacity/soak remain outstanding. The public contract currently prevents that path, as recorded in POAM-005.

This record remains provisional until dependency integration, the remaining
review and candidate/merge-group evidence are complete.

## Change checklist and review boundary

- [x] Bind this prerequisite to OBJ-02/OBJ-07/OBJ-14, Gate E and POAM-005 in PR 266 before source changes.
- [x] Map each replaced result producer, index transition, live continuation and prompt-flight/retrieval consumer to the existing engine/query/core authority. No native storage-key format is removed.
- [x] Retain actual failures, generated-surface results and the installed-process oracle above; the candidate and merge-group pipelines remain mandatory.
- [x] Identify the public identifier and explicit graph-query gaps in POAM-005 without closing it. No roadmap completion status changed.
- [ ] Complete any remaining review, refresh after dependency PR 265 lands, run candidate/merge-group CI and record their exact revisions and results.

The complete current changed-file bytes were read mechanically and hashed; this
is not a claim that all unchanged lines in those files were semantically
reviewed. The implementation diff and its generated schema changes were
inspected, with generated surfaces independently verified. Full-file semantic
review and any portions of the wider repository outside the named scope are
not claimed. Files explicitly read in full as text include `README.md`,
`docs/roadmap/rrflow-1.0-execution/change-authoring.md`,
`crates/kernel/rrd-core/src/result_identity.rs`,
`crates/compute/rrd-query/tests/query/identity_consistency.rs`,
`crates/persistence/rrd-lsm/src/maintenance_limits.rs` and
`crates/persistence/rrd-store/src/repository/projection.rs`; other files were
inspected by their relevant sections and diffs. The whole-file byte read and
hashes are retained locally at `target/canonical-identity-changed-file-read.json`;
it is a working inventory, not independent review evidence. The exact paths in
this working package are:

```text
crates/adapters/rrflow-cli/tests/installed_crash.rs
crates/adapters/rrflow-cli/tests/installed_lifecycle.rs
crates/adapters/rrflow-cli/tests/installed_reopen.rs
crates/adapters/rrflow-cli/tests/support/capacity.rs
crates/adapters/rrflow-cli/tests/support/storage_history.rs
crates/authority/rrd-engine/src/engine/context.rs
crates/authority/rrd-engine/src/engine/flight.rs
crates/authority/rrd-engine/src/engine/memory_estate.rs
crates/authority/rrd-engine/src/engine/query.rs
crates/authority/rrd-engine/src/engine/retrieval.rs
crates/authority/rrd-engine/src/engine/retrieval_query.rs
crates/authority/rrd-engine/src/engine/subscription.rs
crates/authority/rrd-engine/src/engine/tests/context.rs
crates/authority/rrd-engine/src/engine/tests/flight.rs
crates/authority/rrd-engine/src/engine/tests/index_foundation.rs
crates/authority/rrd-engine/src/engine/tests/subscription.rs
crates/compute/rrd-query/fixtures/plan-vectors.json
crates/compute/rrd-query/src/execute.rs
crates/compute/rrd-query/src/index.rs
crates/compute/rrd-query/src/live.rs
crates/compute/rrd-query/src/physical/graph.rs
crates/compute/rrd-query/src/plan.rs
crates/compute/rrd-query/tests/graph_traversal.rs
crates/compute/rrd-query/tests/index_catalogue.rs
crates/compute/rrd-query/tests/live_query.rs
crates/compute/rrd-query/tests/query.rs
crates/compute/rrd-query/tests/query/identity_consistency.rs
crates/kernel/rrd-core/src/lib.rs
crates/kernel/rrd-core/src/result_identity.rs
crates/transport/rrd-contract/src/lib.rs
crates/transport/rrd-contract/src/memory_context.rs
crates/transport/rrd-contract/src/websocket.rs
crates/transport/rrd-contract/tests/public_contract.rs
crates/transport/rrd-contract/tests/websocket_contract.rs
crates/transport/rrd-server/tests/http_process.rs
docs/evidence/change-journals/gate-e/README.md
docs/evidence/change-journals/gate-e/e-01-canonical-result-identity.md
docs/poam/rrflow-1.0-alpha/poam-005.md
docs/reference/protocol/public-contract.md
docs/reference/protocol/signal-catalogue.md
docs/reference/protocol/subscriptions.md
docs/reference/query/index-catalogue.md
docs/reference/query/live-query.md
docs/reference/query/multi-model.md
docs/reference/sdk/dotnet.md
docs/reference/sdk/go.md
docs/reference/sdk/java.md
docs/reference/sdk/python.md
docs/reference/sdk/typescript.md
sdks/dotnet/src/Rrflow.Rrd.Client/Generated/OperationId.g.cs
sdks/dotnet/src/Rrflow.Rrd.Client/Generated/SignalCatalogue.g.cs
sdks/go/endpoints_gen.go
sdks/go/signal_catalogue_gen.go
sdks/java/src/main/java/io/rrflow/rrd/OperationId.java
sdks/java/src/main/java/io/rrflow/rrd/SignalCatalogue.java
sdks/python/src/rrd_client/generated/endpoints.py
sdks/python/src/rrd_client/generated/signal_catalogue.py
sdks/typescript/src/generated/endpoints.ts
sdks/typescript/src/generated/rrd-openapi.ts
sdks/typescript/src/generated/signal-catalogue.ts
```
