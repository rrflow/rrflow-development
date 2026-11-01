# Controller integration with landed plan-owned attunement

**Status:** active integration evidence; local verification passed, protected landing pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-attunement-integration`
**Owner:** OBJ-14 / G-01b / G-06 with D-02/D-03 dependencies; native blocker at C-07 / POAM-035
**Pull request:** [206](https://github.com/rrflow/rrflow-development/pull/206)

## Bound change and traceability

Driver: the SWAG controller working session. Starting revision
`f278fa6f4233f32b069eff4e5fa00bf50ba12aac`, tree
`9c27d87922d47afb2d1532bd3b9f4c0dbfea488f`, ordinarily merges public main
`a76faf7046b48819a06c31219e9db973b7f6a06b`. PR218 and its PR208 dependency
landed on that main. The PR brief bound this exact integration before edits.
The existing controller branch lacks their versioned plan/source-replan contract;
the target combines both owners without introducing another lifecycle authority.

Git identified 29 conflicts: the OpenAPI digest, public-contract assertions,
generated SDK/protocol projections, authored digest citations, SDK assertions,
conformance count and storage-fixture identity. Authored resolutions retain
main's semantic replan assertions and the controller assertions. Counts and
projections are derived from the combined catalogue. Five endpoint/schema
generators and the signal generator produce 71 operations with OpenAPI SHA256
`f1856602eb3b5ba419f600b3be0e9626d62ad533032fe220a2728795bd28048a`.
Earlier audit hashes in SDK documentation retain their historical meaning.

The unchanged endpoint-proposal/budget modules remain the controller authority.
Incoming attunement v2 sequencing, source observation, authorized successors,
leases and receipts remain at their landed contract/engine/client owners.
The semantic auto-merges in engine errors, invocation authorization, module
exports, operation references and HTTP dispatch preserve both capabilities.
No implementation was moved or deliberately deleted. Existing live desktop and
binary receipts remain bound to their original source; this integration does
not rebuild or qualify those artifacts.
Eight controller/attunement authority and test files were compared in full to
their respective parents after normalizing line endings; all matched. The
exact paths and hashes are retained in `preserved-owners.json` with local logs.

The existing `refresh_storage_contract` example regenerated only the synthetic
storage contract fixture's catalogue/manifest/result digests. Its `synthetic`
classification and measured-looking test fields are unchanged. This refresh
executed no workload and supplies no native evaluation, routing or training data.

## Review and engineering decisions

Reviewed repository instructions, README owner entry points, CI admission and
change-authoring procedure; Gate D/G and POAM-035/039; previous controller and
landed plan/source-replan journals; all conflict hunks and the semantic
auto-merge diff. Read the storage refresh example and SDK/signal generators at
their affected boundaries. Generated output is checked mechanically against its
canonical owner, not represented as a manual line-by-line audit. The prior
implementation journals retain their full source reviews. Git supplies the path
inventory and ancestry; this record does not duplicate it.

No external research or source adaptation is needed for deterministic source
integration. Preserve existing native diagnostics; no new runtime trace,
resource measurement, model invocation or credential operation is required.
Stop on semantic loss, unexplained fixture changes, altered authority, or a
need to weaken a protected oracle. A repair beyond compatibility integration
requires a separate package at its existing owner.

## Failures and verification

The first public-contract run compiled and ran 46 tests: 44 passed and two
failed on the stale OpenAPI digest and conformance operation count. The actual
combined catalogue was then exported; derived identities were regenerated.
The first Python generator invocation failed because `ruff` was absent from
the plain shell PATH. Running through the SDK's locked `uv run --frozen`
environment succeeded. No generator or dependency constraint was weakened.

Python SDK tests passed 8 with the installed-process test skipped; strict mypy
passed. Ruff found one overlong line in main's new installed-attunement test.
Wrapping that dictionary changed no behavior; lint and format then passed.
Go SDK tests passed. TypeScript lint, typecheck and all six tests passed.
All conflict markers and unmerged index entries were removed, and initial
working/staged whitespace checks passed.

Additional acceptance commands and results:

- `cargo test --locked -p rrd-contract -p rrd-inference -p rrd-client`: 207
  tests passed across the reported suites, none failed or ignored. This count
  does not establish installed-client coverage without an installed binary.
- `cargo test --locked -p rrd-engine --lib`: 268 passed, four ignored. The
  combined source-replan, positional checkpoint and controller-budget tests pass.
- `cargo test --locked -p rrflow-eval --test storage_contract`: 11 passed.
- `cargo clippy --locked -p rrd-contract -p rrd-inference -p rrd-client -p
  rrd-engine -p rrd-server --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- TypeScript `pnpm generate:check`, Python locked `scripts/generate.py --check`,
  Go `go run ./cmd/generate --check`, Java and .NET generation checks: passed.
- `python scripts/ci/check_generated_surfaces.py --check`: 71 operations and
  seven signal projections passed.
- Navigation generation, documentation/workflow policy and execution inventory
  generation/check passed. Final whitespace and unresolved-index checks passed.

Local command logs and exact results are retained under the operator's
`controller-attunement-integration` evidence directory dated 2026-10-30.
The complete merge diff and authored resolutions were reviewed against both
parents; generated full-file equality is established by generation checks.

## Remaining evidence and checklist

The original protected run `40982529726` failed macOS APFS creation with
`Resource busy`; its cause remains unresolved. Landed main and unrelated green
checks do not repair that failure. Keep this candidate draft under its recorded
stop condition; no retry-until-green, reduced oracle or branch-protection bypass.

This package does not prove selected-worker execution, per-prompt handoff,
ordinary-edit recovery of arbitrary effects, RROFLOW delivery, independently
reviewed learning, model superiority, quantization or release qualification.
Only Connect executes in the landed attunement phase implementation; other
phases remain explicitly unavailable. Configurable plan admission is separately
owned by PR221. Native capacity, installed merged-source workflow, full candidate
and protected merge-group checks have not run for this integration.

Checklist: pre-edit PR brief; exact baseline and incoming main; canonical
ownership and preserved behavior/tests; regenerated derived surfaces; retained
first failures; linked journal/navigation and final local checks. Roadmap and
POA&M acceptance status is unchanged.
