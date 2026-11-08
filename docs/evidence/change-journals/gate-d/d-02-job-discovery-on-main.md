# Installed job discovery on accepted main

**Status:** active candidate evidence; local validation passes, candidate and protected acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-job-discovery-on-main`
**Owner:** OBJ-14 / D-02a/D-02b / H-04/H-06 / POAM-040

## Bound package and provenance

The SWAG controller integration session drives
[PR248](https://github.com/rrflow/rrflow-development/pull/248), whose pre-edit
brief binds this package to accepted main
`e6f3632b32f1c9be33c1951886a9d9a2be648ddf`. An empty branch bootstrap commit
allowed the brief to be published before implementation. The package adapts
only discovery commit `8d47500869329f1a5604821138f1a6c8efc349a5` from
[PR231](https://github.com/rrflow/rrflow-development/pull/231); its worker and
task-budget ancestors are absent. The complete original discovery journal is
retained unchanged as historical provenance. Keep PR231 until replacement
acceptance and protected landing are proved.

The baseline canonical export contains 71 operations and no
`attunement-job-list`. Clients cannot discover installed job identities through
that public boundary. The target adds the original bounded authenticated list
operation over the installed initial job and committed successors, using
existing engine authorization, lineage and control sequence. Discovery does
not execute, approve or freshly verify source. The existing replan preview and
explicit acceptance remain the source-edit authority.

## Implementation traceability

The original engine discovery module, contract module and contract tests are
byte-identical to their source commit. Native client/HTTP carriage and
catalogue/reference additions are the original authored delta. Main's test
module registration needs contextual adaptation because its chronology comment
changed. The discovery engine tests retain every behavioral assertion but now
derive follow-up times from returned completion timestamps, matching accepted
main's chronology rule. No runtime decision changes result from that adaptation.

Canonical generation produces 72 operations. Full parsed OpenAPI comparison
finds exactly the new `/v1/attunement/jobs/list` path and the derived
`x-rrd-endpoint-count` change from 71 to 72; every other value is unchanged.
OpenAPI SHA-256 is
`57ef2f38fecf0a0d7a96fa5b9fa46c548b260bdc82c371d084456b39a708e6d1`.
All five language SDKs and seven signal projections follow the local canonical
export. The SDK conformance corpus changes only its derived endpoint count.
Worker execution and task-budget recovery operations are not introduced.

The existing storage-contract generator refreshes three synthetic protocol
fixtures. A full parsed comparison proves the only differences are the
candidate catalogue digest, dependent manifest digest and dependent result
digest. Every outcome, case and evidence-class value is unchanged. These
fixtures are not native qualification receipts or model-training examples.

Source, contract, native client, HTTP server and tests stay at their existing
canonical paths. No implementation is moved, deleted, duplicated or replaced.
The POAM-040 evidence link and this generated journal index identify the current
package without closing the owner gap. Source comparison receipts retain exact
file hashes and the test adaptation separately.

## Validation and surfaced corrections

The original five contract and five installed-engine discovery tests pass on
accepted main. The five engine tests pass again after chronology adaptation.
Generated parity verifies 72 operations and seven signal projections.
`cargo test --locked -p rrd-contract -p rrd-engine -p rrd-client -p rrd-server -p rrflow-eval --all-targets`
reports 662 passes and four existing ignored helpers, with no failures. Its
log SHA-256 is
`c4c1fdcf57d33cfbe6854f2b0846b19806b6a27ad30afc093ac8cf882e21e15f`.
That count includes the shared SDK conformance entry, which returns early
without its live harness manifest; full SDK conformance remains required in
candidate CI. The focused native-client HTTP/replan/reopen test also passes
separately, and `cargo build --locked -p rrflow-cli --bins` succeeds.
`cargo clippy --locked -p rrd-contract -p rrd-engine -p rrd-client -p rrd-server -p rrflow-eval --all-targets -- -D warnings`
and `cargo fmt --all --check` pass. Strict-lint log SHA-256 is
`494efcbc9cfb16cd4cc3e19c8d295949ac9b68de221b5599b529cbe296abd242`.
With pinned Node/Corepack, `pnpm --dir sdks/typescript check` passes generation
parity, lint, type checking and all six tests. Its log SHA-256 is
`d15d80827a35ea58c530946fa22914caede7ec156112d2fca94844979d5343a8`.
`python scripts/ci/check_workflow.py` passes. Documentation policy validates
934 document statuses and 932 coordinates; navigation validates 18 indexes,
933 nodes and 3,333 edges. `build_execution_inventory.py` and its `--check`
validate 2,323 path records. Whitespace checks pass.

The actual Windows primary binary passes an isolated authenticated public-HTTP
trial using a copied real SWAG deployment-readiness document. Discovery returns
the initial job without a client-derived ID. An edit after preview causes
explicit acceptance to return `failed_precondition` with reason
`attunement-source-changed`; discovery still returns the original job. A fresh
preview and explicit acceptance commit one successor, which is discovered by
pagination. Abrupt termination and reopening of only the harness-owned engine
preserve the instance and successor. Same-key acceptance replay returns that
successor with `idempotent_replay: true`. Requests, responses and source
identities are retained in private `installed-discovery-20261102T201440Z` evidence.
The tested primary SHA-256 is
`a5803f1ba6fb7747aad2fce512e0406fc73af7c0053be0a874970c112b2410eb`.
It was built from the uncommitted candidate source, not an accepted release;
the post-trial native-source inventory is retained and checked unchanged before
publication. No model was invoked and no learning label was approved. This
trial does not establish desktop, full-project or release acceptance.

The initial patch check rejected the test module hunk against main's chronology
comment before applying anything. Adapting only that registration preserves
main. The shell's Node 24.21/pnpm 10 cannot read the pinned pnpm 12 multi-document
lockfile; Node 24.22.0 with Corepack-selected pnpm 12.14.0 completes the frozen
install without a dependency or lockfile change. A first schema comparison
incorrectly expected the root endpoint-count metadata to remain unchanged.
The corrected structural oracle allows exactly the measured 71-to-72 count
change plus the new path; it does not waive any existing endpoint difference.
The fixture comparison initially named the manifest identity parent `build`
instead of its actual `candidate` field; correcting that assertion's path
verifies the same catalogue-only delta without changing any fixture result.
The final status wording made its generated journal-index entry stale;
documentation checking rejected that drift. Regenerating navigation after the
status update resolves it without changing implementation or acceptance rules.

Private dated `job-discovery-main` evidence retains the source patch, rejected
patch log, baseline/new canonical exports, generation and validation commands,
raw results and SHA-256 identities. Protocol fixtures and controlled source
edits are correctness tests, not training data or routing-quality evidence.

## Review, checks and remaining acceptance

Reviewed owners include engineering instructions, README/change-authoring/CI
requirements, POAM-040's explicit job-list decision, the complete original
discovery journal and authored delta. Engine/contract discovery implementations
and their tests were read in full; shared owner contexts and native/HTTP changes
were inspected at affected symbols and through their complete deltas. Exact
whole-file comparison covers unchanged reused modules. This is author review,
not independent approval.

No external algorithm, dependency or upstream adaptation is needed. Existing
request/operation IDs and authorization audit supply diagnostics. Summaries
contain no credentials or raw source; bounds cover page traversal and encoding,
not throughput or serving memory. Stop on an actual unmerged semantic
dependency, lost reusable behavior, expanded authority, unsafe concurrency,
unexpected source/publication changes or failed owning checks.

Local implementation, owning validation and journal are complete; full
exact-candidate and protected CI remain required before landing. Other native
platforms, all optional features and the live multi-language SDK harness are
delegated to those required CI partitions, not claimed by the local commands.
PR235's outcome-delivery acceptance proceeds separately; later composition
must regenerate combined public surfaces. Connectome integration, real project
execution, independently reviewed learning, model qualification and native/
container delivery remain open. No objective, roadmap or POAM gate closes.

## Links

- [Original discovery implementation and evidence](d-02-installed-job-discovery.md)
- [Public discovery contract](../../../reference/protocol/public-contract.md#installed-job-discovery)
- [POAM-040 owner](../../../poam/rrflow-1.0-alpha/poam-040.md)
