# H-05a signal contract and SDK projection journal

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h05a-signal-contract-sdk-projection`
**Owner:** H-05a slice-2 implementation receipt; not runtime, roadmap, or release authority

This record proves the bounded public-contract and SDK projection package. The
[kernel catalogue journal](h05a-kernel-signal-catalogue-journal.md) proves the
one semantic source, the [engine data-flow architecture](../../../architecture/engine-data-flow.md#metric-instruments-and-cardinality)
owns the accepted signal semantics, the [Gate H record](../../../roadmap/rrflow-1.0/gate-h.md)
owns acceptance, and the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
owns dependency and completion status.

## Package and outcome boundary

The final package is
`H05a-02b-generated-signal-contract-and-sdk-projection-v1`. It advances a
second executable prerequisite for H-05a, OBJ-09, and OBJ-10: the exact
kernel-owned signal catalogue and its fingerprint are deterministic OpenAPI
extensions, a generated protocol reference, and read-only identities in all
five supported SDKs.

This is discovery and conformance plumbing. It does not emit a metric or trace,
activate a diagnostic level, install a subscriber or exporter, capture content,
read a clock or environment variable, mutate engine state, add a public
operation, prove diagnostic/release binary parity, define build identity,
measure overhead, repair storage performance, or prove a running engine. H-04,
H-05a, and H-05 remain open.

## Starting state and planning boundary

- Starting revision: `078dd2228e801cc651ecd7882abb28139fc70f97`.
- Starting tree: `d80c781cc71b577a39386488a7d8344e961ce325`.
- Branch: `agent/connectome-temporal-runtime-visualizer`, with a clean starting
  worktree.
- Corrected planning-only commit:
  `bf8a111a1fe7e6b02816b0d8c3c76c147ec83752`; tree
  `69a571b8a6f61487f65425cbeff3dbda8425e892`.
- The preceding planning-only commit
  `078dd2228e801cc651ecd7882abb28139fc70f97` began from implementation
  revision `e4ea8aea1f018c2966bf8405cfec4e1eaad523be` and proposed a normal
  `rrd-core` edge. The workspace architecture oracle rejected that coupling.
  All work was placed in a recoverable stash, the clean revision was restored,
  and the corrected plan was committed alone before the work was reapplied.
- The incremental-development remote `development` resolves to
  `https://github.com/rrflow/rrflow-development.git`.
- The promotion-only remote `origin` resolves to
  `https://github.com/rrflow/rrflow.git`.
- `Cargo.lock` began at SHA-256
  `316533e7512dbb9029e494386503ca8430b0c69853b3b78ef02319dcbab93a27`
  and is unchanged.
- The planning validator accepted the planning commit with zero post-plan
  paths before corrected implementation began.

The committed active-change record permits exactly 27 implementation paths and
4 evidence paths. It binds the test-first sequence, zero-workspace-dependency
guard, one generated Rust bridge plus six outward signal projections, five
public SDK tests, regenerated OpenAPI-dependent files, two supporting
documents, nearest indexes, and the Git-backed execution inventory. No
objective, roadmap, POA&M, product version, benchmark policy, release, or
promotion record is in that envelope.

## Change brief and authority

At baseline, `rrd-core` owned and validated a v1 catalogue with four diagnostic
levels, nine bounded metric attributes, twenty-two metric instruments, durable
trace descriptors derived from their existing owners, and fingerprint
`239df2ced5974d4351ca01565369646964cb65c63d0fbae01dddccb4c3ac5a07`.
`rrd-contract`, OpenAPI, the coordinated protocol reference, and all SDKs
exposed none of it. The central generated-surface checker validated 33 endpoint
descriptors only and had no deliberate write mode. The public-contract record
also contained a pre-existing stale OpenAPI digest.

The target is a generated one-way projection. The central renderer validates
the kernel's test-verified catalogue fixture and generates a private Rust
snapshot inside `rrd-contract`. The contract parses that snapshot and embeds
the value and exact fingerprint as `x-rrd-signal-catalogue` and
`x-rrd-signal-catalogue-sha256` without a production kernel dependency. The
same renderer then exports OpenAPI once and deterministically derives the
reference and five SDK files. A dev-only focused test compares OpenAPI back to
the live kernel API. Generated files carry canonical compact JSON and
fingerprints rather than restating signal names or policy. Existing operation,
schema, authorization, mutation, runtime, persistence, and
`DiagnosticSnapshot` behavior remain unchanged.

Work stops if projection requires changing telemetry semantics, copying signal
names into a generator, accepting a digest mismatch, adding any production
workspace or third-party dependency to `rrd-contract`, changing `Cargo.lock`,
introducing runtime activation or an effect, changing any of the 33 endpoint
descriptors, broadening into H-04 model generation or later H-05 runtime work,
weakening the storage promotion gate, or overstating completion.

## Complete baseline and owner reads

The active-change record binds baseline SHA-256, line counts, full line ranges,
and symbols for every read below. Each file was read completely before
implementation:

- `README.md`; the root `Cargo.toml`; `docs/objectives/rrflow-1.0-alpha.md`;
  `docs/roadmap/rrflow-1.0.md`; `docs/roadmap/rrflow-1.0/gate-h.md`;
  `docs/poam/rrflow-1.0-alpha.md`; and
  `docs/poam/rrflow-1.0-alpha/poam-026.md`;
- `docs/architecture/system-overview.md`;
  `docs/architecture/engine-data-flow.md`;
  `docs/research/rrflow-build-debug-trace-optimize-architecture-research.md`;
  `docs/roadmap/rrflow-1.0-execution/change-authoring.md`;
  `docs/roadmap/rrflow-1.0-execution/generated-surfaces.md`;
  `docs/reference/protocol/public-contract.md`; and the superseded
  `docs/roadmap/rrflow-1.0-active-change.json`;
- `crates/kernel/rrd-core/Cargo.toml`, `src/lib.rs`, `src/telemetry.rs`,
  `fixtures/telemetry-catalogue-v1.json`, and
  `tests/telemetry_contract.rs`;
- `crates/transport/rrd-contract/Cargo.toml`, the complete 6,304-line
  `src/lib.rs`, `src/diagnostic.rs`, the complete 1,802-line
  `tests/public_contract.rs`, and `src/bin/rrd-contract-export.rs`;
- the complete 1,860-line
  `crates/authority/rrd-engine/tests/workspace_architecture.rs`, including
  dependency metadata construction and every architecture assertion;
- `scripts/ci/check_generated_surfaces.py`,
  `scripts/ci/build_execution_inventory.py`, and
  `scripts/knowledge/render_navigation.py`;
- TypeScript's complete 25,503-line generated `rrd-openapi.ts`,
  `generated/endpoints.ts`, `scripts/generate.ts`, `package.json`,
  `src/index.ts`, and `tests/client.test.ts`;
- Python's `scripts/generate.py`, `pyproject.toml`, root and generated
  `__init__.py`, `generated/endpoints.py`, and `tests/test_client.py`;
- Go's `cmd/generate/main.go`, `go.mod`, `doc.go`, `endpoints_gen.go`, and
  `client_test.go`;
- Java's `scripts/generate.py`, `pom.xml`, `package-info.java`,
  `OperationId.java`, and `RrdClientTest.java`; and
- .NET's `scripts/generate.py`, runtime and test project files,
  `Generated/OperationId.g.cs`, and `RrdClientTests.cs`.

The complete changed files and complete final diff are reread after formatting,
generation, and evidence reconciliation. Exact generated-byte checks supplement
but do not replace that review.

## Paths and implemented behavior

### Canonical public projection

- Kept `crates/transport/rrd-contract/Cargo.toml` byte-identical: its
  `rrd-core` edge remains dev-only, and production has no RRFlow workspace
  dependency.
- Added generated
  `crates/transport/rrd-contract/src/generated/signal_catalogue.rs` containing
  only private canonical JSON and fingerprint constants derived from the
  kernel fixture.
- Changed `crates/transport/rrd-contract/src/lib.rs` so
  `openapi_document()` parses the private generated JSON, uses its generated
  fingerprint, and inserts the two `x-rrd-*` extensions. Invalid generated JSON
  returns a contract error rather than falling back. The canonical OpenAPI
  digest is now
  `8f9efc7be194e4900812f93b422e252fab187facf854c9459f1c84be70971f8b`.
- Added `crates/transport/rrd-contract/tests/signal_catalogue_projection.rs`
  to prove exact JSON and fingerprint equality with the kernel, the 4/9/22
  cardinalities, exact top-level field set, and absence of runtime/effect
  authority.

### One renderer and seven replaceable outputs

- Changed `scripts/ci/check_generated_surfaces.py` into a deterministic central
  signal renderer while preserving endpoint parity. No arguments and `--check`
  validate the kernel fixture and compare all seven outputs without writing.
  `--write` writes the private Rust bridge before contract export and may then
  write only the six declared outward signal outputs. It validates live
  OpenAPI equality, v1 shapes, positive limits, uniqueness,
  descriptor fields, bounded metric dimensions, both SHA-256 identities, and
  all 33 endpoint descriptors. The renderer embeds no metric or trace semantic
  names.
- Added generated `docs/reference/protocol/signal-catalogue.md` and generated
  TypeScript, Python, Go, Java, and .NET signal-catalogue files. Each carries
  the same canonical compact JSON, kernel catalogue fingerprint, and OpenAPI
  digest. The TypeScript parsed object is recursively frozen.
- Regenerated TypeScript OpenAPI types plus TypeScript, Python, Go, Java, and
  .NET endpoint outputs solely for the changed OpenAPI digest. Central parity
  still reports 33 unchanged method/path/authentication/mutation descriptors.
- Changed TypeScript and Python public exports to expose the generated values;
  the other three generated files are directly public in their package idioms.

### Independent language oracles and supporting records

- Added one test in each supported language. Each parses the public generated
  JSON, proves version 1, exact 4/9/22 counts and fingerprints, and rejects
  runtime/effect keys; TypeScript additionally proves deep immutability.
- Changed `docs/reference/protocol/public-contract.md` to record the tested
  digest, extensions, generated kernel-to-contract bridge, dependency
  independence, discovery surface, and non-authority boundary.
- Changed `docs/roadmap/rrflow-1.0-execution/generated-surfaces.md` to record
  central signal generation while retaining the explicit H-04 gap: endpoint
  generators and complete operation/model generation are not yet converged.
- Changed `scripts/ci/build_execution_inventory.py` with exact H-05 path
  assignments and path-to-generator classifications, not a broad glob or prose
  audit.
- Added this journal and regenerated only its nearest Gate H index, the nearest
  protocol index, and the exact Git-backed file inventory. The protocol index's
  hand-maintained table had no renderer markers and therefore omitted the new
  coordinate; it was converted in its declared path to the repository's
  standard generated-index region before regeneration.
- No implementation was deleted, moved, merged, renamed, or rewritten, so the
  implementation traceability procedure is not applicable.

## Research and adaptation decision

Research was required and completed in the linked architecture research
package. This slice retains OpenAPI 3.1 specification-extension behavior,
OpenTelemetry metric identity semantics, and failure isolation between
observability metadata and application results. It adapts semantic guidance
only. No vendor SDK, exporter, upstream code tree, public compatibility model,
runtime topology, fetch, or new dependency was imported.

## Trace, resource, and debugging decision

- Runtime trace: **preserved**. Existing trace descriptors are exposed; no
  `RuntimeTraceEvent` is created, sampled, propagated, persisted, or changed.
- Resource evidence: **added as identity only**. The public projection exposes
  the metric catalogue and bounded-cardinality policy but records no sample,
  throughput, latency, allocation, RSS, disk, or benchmark result.
- Debugging evidence: **added as discovery only**. SDK authors can inspect one
  fingerprinted vocabulary. No diagnostic level is activated and no client
  gains mutation or lifecycle authority.

## Test-first oracle and surfaced failures

The focused Rust test and five language tests were authored before their
implementations. They could not pass on baseline inventory:

The first plan selected a direct normal `rrd-core` edge. Focused contract and
SDK checks passed, but the full workspace architecture suite passed 27 of 28
tests and rejected that edge in
`public_contract_and_client_stay_implementation_free`. Complete review of the
1,860-line guard, root dependency arrows, and system overview confirmed this
was an intentional layer boundary, not a stale whitelist. The invariant was
kept unchanged. Work was preserved in a recoverable stash, the repository was
returned to clean revision `078dd2228e801cc651ecd7882abb28139fc70f97`,
corrected plan `bf8a111a1fe7e6b02816b0d8c3c76c147ec83752` was
committed alone and validated with zero post-plan paths, and only then was the
work reapplied and replaced with the generated fixture bridge. The corrected
normal graph has no RRFlow workspace package.

1. The first planning-validator run after only the Rust test rejected every
   still-missing declared package path. This was the expected fail-closed
   partial-package behavior; the guard was not bypassed.
2. The first Rust compile exposed an ambiguous `BTreeSet` collection in the
   test (E0283). The oracle was made explicit rather than weakening a type.
3. The compiled focused Rust oracle then failed both tests because OpenAPI had
   neither signal extension. After projection, both passed.
4. The existing public-contract suite then passed 32 of 33 tests and rejected
   the frozen old digest: actual `8f9efc...`, expected `1d18655a...`. Only after
   that failure was retained was `OPENAPI_DOCUMENT_SHA256` updated; all 33
   tests then passed.
5. Before generation, TypeScript failed on its missing root export; Python
   failed collection on a missing import; Go failed on undefined constants;
   and Java failed on the missing `SignalCatalogue` class. Generation and
   public export wiring satisfied those failures.
6. The planned .NET `dotnet test ... --no-restore` command returned exit zero
   while discovering and running no xUnit v3 tests. It is recorded as a false
   no-op, not acceptance evidence. The repository pins SDK 10.0.111 while the
   host exposes 10.0.112; the exact 10.0.111 packages already cached locally
   were extracted into a resolved temporary directory without a network fetch.
   Locked restore passed, and the actual xUnit executable through `dotnet run`
   discovered and passed all five tests under .NET 10.0.11. A supplemental
   installed-SDK run selected and passed the new signal test alone.
7. The first attempted xUnit selection used unsupported `--filter-class`.
   Runner help identified the valid `-class` option, which then selected and
   passed the test.
8. Initial generated Python quoting was not Ruff-stable. The renderer now
   chooses a deterministic shortest valid literal; generated output formats
   without drift.
9. Initial Go output used one roughly 10 KiB raw-string line and was not
   gofmt-stable. The renderer now emits deterministic chunks and stable
   alignment.
10. An adversarial one-character edit to the generated TypeScript signal
    fingerprint made central `--check` fail specifically as stale output.
    `--write` restored it, and the next check passed. This proves the checker
    detects and repairs drift rather than merely parsing files.
11. The first navigation render updated the Gate H journal index but not the
    protocol index because that older index was still a manual table without
    generated markers. The declared index was converted to the standard
    generated region; the renderer now owns all sibling protocol links.
12. The first documentation-policy run rejected the generated catalogue's
    status because coordinated records must classify themselves as `active` or
    `historical`. The renderer, rather than its output alone, was corrected to
    emit `active generated signal-contract reference`, so future generation
    remains policy-valid.
13. The first final Rust formatting check rejected the generated bridge's
    one-line fingerprint constant. The Rust renderer was corrected to emit the
    exact multiline rustfmt form; regeneration, exact check, and formatting
    then agree byte-for-byte.
14. A first attempt to reconstruct the pinned .NET toolchain used the Debian
    packages' wrong `usr/share/dotnet` root and therefore resolved the installed
    10.0.112 SDK. The package layout was inspected, `DOTNET_ROOT` was corrected
    to the extracted `usr/lib/dotnet`, and the rerun proved SDK 10.0.111,
    runtime 10.0.11, locked restore, real discovery, and all five passing tests.
15. An adversarial one-character change to the private generated Rust
    fingerprint was rejected as a stale bridge before contract compilation or
    OpenAPI export. Deliberate write mode restored it; subsequent check mode
    passed and left the SHA-256 of all seven generated files unchanged.
16. A final inventory refresh was first invoked with an unsupported `--write`
    argument. The command failed without changing the inventory; the script's
    declared interface was then used, where ordinary invocation writes and
    `--check` verifies without mutation.
17. The first local result commit summary exposed an unintended executable-bit
    loss on `scripts/ci/check_generated_surfaces.py`. That revision was not
    pushed. The declared script mode was restored, direct Ruff lint then passed
    without suppressing `EXE001`, and the still-local result commit was amended
    only after regenerating its journal-backed inventory.

Every intermediate error exposed a real boundary or tooling mismatch. None was
hidden by a compatibility path, relaxed equality, altered count, skipped
assertion, or weakened promotion rule.

## Acceptance evidence

- `cargo test -p rrd-core --test telemetry_contract --locked` passed all 4
  catalogue/fixture tests and an explicit before/after digest comparison proved
  the fixture was not rewritten.
- `cargo test -p rrd-contract --test signal_catalogue_projection --locked`
  passed both focused tests.
- `cargo test -p rrd-contract --test public_contract --locked` passed all 33
  existing public-contract tests after the expected digest failure.
- `cargo test -p rrd-contract --all-targets --locked` passed every contract
  unit and integration target, including the new projection test.
- `cargo clippy -p rrd-contract --all-targets --locked -- -D warnings` passed.
- The exact normal-dependency guard passed: `rrd-core`, `rrd-engine`,
  `rrd-server`, Tokio, tracing, and OpenTelemetry are all absent.
  The dev-only kernel oracle remains available to tests, and `Cargo.lock` is
  byte-identical to baseline.
- `python3 scripts/ci/check_generated_surfaces.py --check` passed 33 HTTP
  operations, seven exact signal projections, catalogue fingerprint
  `239df2ce...`, and OpenAPI digest `8f9efc7b...` without writing. An explicit
  before/after digest comparison proved check mode did not mutate any of the
  seven files.
- TypeScript generation, Biome, TypeScript compilation, and all 5 tests passed.
- Python generation, Ruff format/lint, mypy over 11 source files, and all 5
  tests passed.
- Go generation, gofmt, vet, and ordinary tests passed, including the signal
  test.
- Java generation passed; Maven discovered 5 tests, passed 4, and explicitly
  skipped the existing manifest-gated conformance case because no conformance
  manifest was supplied.
- .NET generation and locked restore passed. The actual xUnit v3 runner under
  the exact pinned SDK discovered and passed all 5 tests; the new signal class
  also passed alone under the installed SDK. The no-discovery `dotnet test`
  result is not counted.
- `cargo test -p rrd-engine --test workspace_architecture --locked` passed all
  28 assertions, including zero production `rrd-contract` workspace
  dependencies, tracked targets, source closure, and single engine authority.
- `go -C sdks/go test -race ./...` passed the client package; generator and
  conformance command packages correctly report no test files.
- `python3 scripts/ci/check_change_plan.py` accepted exactly 31 post-plan paths
  for corrected plan commit
  `bf8a111a1fe7e6b02816b0d8c3c76c147ec83752`.
- `python3 scripts/ci/check_documentation.py` passed knowledge ownership, 262
  document statuses, 260 classified coordinates, generated parent indexes,
  and local links.
- Navigation check passed 14 indexes, 261 nodes, 1,086 edges, and zero pending
  updates; all 6 navigation tests passed.
- Execution inventory check passed all 1,145 current, generated, and planned
  path records; all 11 knowledge-export tests passed.
- Version policy retained `1.0.0`; `cargo fmt --all -- --check`, focused Ruff
  format/lint for the central renderer, and `git diff --check` passed.

## Methodical benchmark separation

The supplied strict benchmark failure is real and remains unresolved by this
package. All records report correctness and sequence 16,384. Aggregate native
write throughput is 105,306 operations/s versus Fjall's 144,868 operations/s,
a failing ratio of `0.7269154837184973`. Native has a better write p50 and p95,
but severe trial-to-trial tail events (individual p99 values from roughly 45
to 111 ms) depress sustained throughput. Native otherwise wins the supplied
read throughput, read p95, recovery, peak RSS, and disk dimensions. The strict
all-dimensions policy therefore correctly fails on write throughput alone.

The preceding kernel journal records the complete current/historical harness
and storage-path review. The evidence supports host/durable-sync tail variance
as the leading explanation for that historical run, not a proven hardware root
cause. No benchmark implementation, threshold, result, roadmap state, or
POA&M item changes here. Signal discovery is not storage optimization evidence.

## Checks not run and remaining errors

This slice does not run or claim live D-01 installed-engine signal collection,
runtime level activation, W3C propagation, exporter or capture behavior,
diagnostic/release build parity, a build manifest, runtime overhead, fuzz,
Miri, Loom, sanitizer, profiler, fixed-hardware benchmark qualification,
installation, daemon, deployment, offline bundle, signing, provenance, or
platform qualification. Those behaviors remain owned by later H-05a/H-05 and
release packages.

The provided native/Fjall write-throughput failure remains an open performance
diagnostic outside this change. The Java manifest-gated conformance case is not
evidence without its manifest. The .NET test project requires its executable
xUnit v3 runner for real discovery; the planned `dotnet test` command alone is
not evidence and should be corrected in a future tooling package.

## Final reconciliation and status

The repository architecture, Go race, documentation, generated-navigation,
inventory, knowledge-export, version, format, and diff-integrity checks pass
with exactly the 31 declared paths. Every new or changed hand-authored source,
test, and document was reread completely. The complete `rrd-contract` and
25,503-line TypeScript OpenAPI sources were read at baseline; their final exact
diffs were reviewed, and the generated OpenAPI file changed only its digest
header. All seven generated signal files were inspected, reproduced
byte-for-byte, parsed or compiled in their owning language, and remained
unchanged during an explicit check-mode digest comparison. The complete final
repository diff was reviewed after evidence generation.

No roadmap checkbox, objective outcome, POA&M status, product maturity,
version, release state, benchmark-promotion state, or official-promotion state
changes. H05a-02b is only a deterministic discovery prerequisite. The result
commit and development push are pending at the time this immutable receipt is
authored; the official `origin` is not a push target.
