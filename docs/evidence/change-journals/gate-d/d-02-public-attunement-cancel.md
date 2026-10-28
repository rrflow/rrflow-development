# D-02 public attunement cancellation

**Status:** active partial implementation with local evidence; D-02 and alpha remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-public-attunement-cancel`
**Owner:** OBJ-14, D-02a/b and POAM-040

## Brief, baseline and implementation

Starting public development `main` revision `65800fb9ef31c63faf51d2c4f684b2aae2353921`, tree `ff917ae7901836947049a92b4280eb64041867a8`. [PR 172](https://github.com/rrflow/rrflow-development/pull/172) held the implementation brief before source edits; the branch starts with an empty bootstrap commit. Driver: this Codex session. C-07 storage PR 168 had merged, but no C-07 qualification is claimed.

Before this package, only the embedded engine could request attunement cancellation. The public client could read one installed job but could not request cancellation. The operation catalogue, generated schemas and endpoint identifiers, HTTP dispatch and internal client now expose `attunement-cancel` using the existing `CancelAttunement` and `RrdEngine::control_attunement` path under EstateAdmin. The transition and its principal-scoped idempotency receipt remain the engine's existing atomic control transaction. The public result validates the expected job, plan digest, next revision and `cancel_requested` state. The result does not claim a worker or an external resource has stopped. No source scanning, external activation, worker implementation, source-bound replan, memory effect or new lifecycle authority is added.

## Review and decisions

Read in full: `docs/roadmap/rrflow-1.0-execution/change-authoring.md`, `crates/transport/rrd-server/src/http/handlers/attunement.rs`, `crates/transport/rrd-server/src/http/handlers/knowledge.rs`, `crates/transport/rrd-server/src/http/handlers/endpoint_configuration.rs`, `.github/PULL_REQUEST_TEMPLATE.md`. Read relevant sections and symbols in `README.md`, ADR-0002, `docs/objectives/rrflow-1.0-alpha.md` OBJ-14, Gate D project workflow, POAM-040, generated-surfaces design, `engine/attunement.rs` including transition and receipt code, `rrd-contract/src/attunement.rs`, catalogue and schema export in `rrd-contract/src/lib.rs`, operation reference, server router/envelope, Rust client operations, installed HTTP test, contract tests, SDK generators, and generated-surface checker. Full-file review of the changed files and complete diff is recorded below after validation.

[Amazon's idempotent API discussion](https://aws.amazon.com/builders-library/making-retries-safe-with-idempotent-APIs/) supports stable caller identity and atomic effect/receipt handling; this package reuses the existing RRFlow control transaction, not upstream source. No performance or new telemetry claim is made. Existing durable status/receipt and independent installed HTTP readback provide the local oracle.

## Oracle and local results

The first `cargo test -p rrd-client --test rrflowdb_local installed_attunement_cancel --locked` failed to compile with `E0599`: `RrdClient::cancel_attunement` was absent. After implementation, that command passed twice on Windows, including after formatting. It starts an installed rrflowKV estate and HTTP server, reads the pending job, rejects a closed-session request and a wrong plan, records `cancel_requested`, rejects changed intent and a stale revision, closes and reopens the engine, reads the job, and replays the same request under a new authenticated session. This test does not kill a shipped process or prove source freshness.

`cargo test -p rrd-contract --test attunement_contract --locked`: 14 passed. The endpoint test verifies mutation, path, EstateAdmin and request/response identities. `python scripts/ci/check_generated_surfaces.py --check`: passed, 57 HTTP operations and OpenAPI SHA-256 `35716b8b26b83a92673ea7a0682bd986f31ed58d3fd193abadab8917ae01e566`. TypeScript, Python, Go, Java and .NET endpoint generators and the signal projections were run from the one contract export. TypeScript dependencies were restored from the local locked cache with no downloads.

The initial local `pnpm` wrapper failed because its managed executable link was broken; `COREPACK_ENABLE_NETWORK=0 corepack pnpm install --offline --frozen-lockfile --ignore-scripts` succeeded. The first Python generator invocation selected an unavailable Python >=3.15 from the project environment; `uv run --offline --no-project --python 3.11 --with ruff python scripts/generate.py` succeeded. These were tooling failures, not product behavior.

The initial combined Rust package run passed the client suite and reached the contract suite, where `public_contract` failed two tests because the frozen OpenAPI digest and shared SDK endpoint-count fixture still described 56 endpoints. The contract constant, fixture and expectation were updated to the newly generated 57-operation digest and count. The focused `cargo test -p rrd-contract --test public_contract --locked` then passed all 42 tests. The complete `cargo test -p rrd-contract -p rrd-client -p rrd-server --locked` passed 179 tests across 32 test binaries/doc-test suites; the local full log is `target/attunement-public-cancel-suite.log` at the session checkout. Strict `cargo clippy -p rrd-contract -p rrd-client -p rrd-server --all-targets --locked -- -D warnings` passed.

The initial TypeScript `corepack pnpm exec tsx --test tests/*.test.ts` run failed one stale endpoint-count assertion. The corresponding five SDK test expectations were updated; TypeScript then passed six tests, `corepack pnpm exec tsc --noEmit` and `corepack pnpm exec biome check .` passed, and Go `go test ./...` passed. Python and Java runtime tests were not run: the locked Python 3.15 interpreter and Java toolchain were unavailable locally. `dotnet test tests/Rrflow.Rrd.Client.Tests/Rrflow.Rrd.Client.Tests.csproj --no-restore` failed before tests: the repository pins SDK 10.0.402 while only 10.0.401 is installed. The full candidate CI remains responsible for those SDK lanes.

The first navigation generation failed because PowerShell wrote a UTF-8 byte-order mark before the journal title; the journal was rewritten as UTF-8 without a byte-order mark. `python scripts/knowledge/render_navigation.py` and its `--check` passed; `python scripts/ci/check_documentation.py` passed; `cargo fmt --all -- --check`, `python scripts/ci/check_workflow.py`, and `git diff --check` passed. An attempted `python scripts/ci/check_ci_workflow.py` failed because that file does not exist; the correct workflow checker was then run. The failing checks are not counted as passes.

## Paths, checklist and outstanding proof

Authored changes are in `crates/transport/rrd-contract/src/attunement.rs`, `lib.rs`, `operation_reference.rs` and its `attunement_contract.rs` and `public_contract.rs` tests, the shared `fixtures/rrd-sdk-conformance-v1.json`; five SDK catalogue unit tests; `crates/transport/rrd-server/src/http/router.rs` and `handlers/attunement.rs`; and `crates/transport/rrd-client/src/operation.rs` and `tests/rrflowdb_local.rs`. Generated or synchronized changes are the five SDK endpoint projections, six signal/reference projections, TypeScript schema and six protocol/SDK OpenAPI digest citations. This journal and its generated index/inventory are evidence paths. No implementation path was moved or deleted.

- [x] Outcome, owner, baseline, scope and stop conditions bound in PR before implementation.
- [x] Missing public behavior characterized; canonical engine transition reused.
- [x] Authentication, request identity, replay and restart covered by the installed HTTP test.
- [x] Primary research and trace/resource decisions recorded; no third-party code adapted.
- [x] Affected Rust package suites, strict Clippy, TypeScript and Go checks, generated surfaces, documentation and navigation.
- [x] Changed implementation and fixture diffs reviewed; every changed file was reread in full by a UTF-8/hash inventory, and generated output was checked against the canonical export.
- [x] Final execution inventory was regenerated and checked before commit.
- [ ] Exact candidate CI and merge-group CI are pending. Python, Java and .NET runtime checks remain for candidate CI.
- [ ] Source identity refresh and replan; inventory execution; public remaining job actions and receipt/journal read; cancellation acknowledgement and resource drain for real workers; actual process-kill proof; memory effects; C-07 native matrix and soak remain owned by their existing gates.

No objective, roadmap, POA&M or release status changes. Candidate and merge evidence are pending; local tests do not establish integration or alpha completion.
