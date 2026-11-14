# Selected project source in a governed worker call

**Status:** active source-context implementation; installed UI and qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-selected-source`
**Owner:** G-02b / D-03 / POAM-009 and POAM-038; PR289

## Brief and baseline

[PR289](https://github.com/rrflow/rrflow-development/pull/289) publishes the
pre-edit brief and its selected-path, disclosure and inventory-projection
amendments. Protected baseline is `5f8a7e3679318d235356f4475e9cb8349423929d`;
the tree-identical bootstrap is `e47e7b06cbe1498c828913febba59f75e4503f60`.
The existing worker accepts task text and optional recalled context, but the
observed Connectome worker request had no selected project source. This package
advances the same project-to-learning journey; it does not complete that journey.

The consumer is Connectome's existing Reasoning / Tasks worker review.
[Connectome PR39](https://github.com/rrflow/connectome-development/pull/39)
contains its pre-edit integration brief, based on protected UI PR30. Its SDK,
engine pin, implementation and installed source-aware trial remain dependent
work. The earlier desktop replan evidence retains its original binary identity.

## Canonical implementation and boundaries

`TaskWorkerSourceRequest` selects one accepted inventory entry by identity,
expected content digest and explicit byte bound. It supplies no filesystem path,
file bytes, expected global policy or expected inventory head. The engine resolves
the installed project, accepted membership and canonical path. Anchored reads,
static exclusions, applicable Git ignore layers, tracked overrides, gitlinks,
file kinds and byte limits govern the current selected path.

The existing worker configuration separately opts into source export, defaulting
to false. Explicit `ProjectSourceRead`, context-read and existing task/worker
permissions still apply. Admission, pre-dispatch validation and completion use
the existing task budget, authority guards and bounded completion transaction.
Source output records captured and completion-validated head/read identities
separately. Historical global policy remains provenance, not current path
authorization or an equivalent-work discriminator. Unrelated staging or head
advancement must not reset the selected work's repetition count. No-source
requests retain `context: null` and the exact previous repetition fingerprint.

Attempt readback rechecks present source permissions, export settings and path
eligibility before disclosing source-derived drafts. Retained observation
delivery applies that same check before returning its stored source JSON.
Authorized historical output after a content edit remains historical; unavailable
or excluded source is withheld without deleting the durable attempt.

The inventory's existing `kind` property collides with QueryExecute's built-in
record-kind field. The canonical inventory publisher now derives an optional
`source_file` map containing file content SHA-256 and unsigned byte count.
Reconstruction verifies any present map against canonical File metadata, rejects
extra/mismatched/non-file projections, and removes it before proposal validation.
Old records without the map remain loadable. Normal query field and row policies
apply; no new query operation, display-string parser or private picker API is added.
Inventory reuse may retain equal facts; publication need not rewrite every row.

Eligible unresolved jobs refresh through preview, exact replan acceptance,
successor Start, Connect and Inventory. Their previous whole-source plan cannot
simply execute another step after an edit. Completed jobs and jobs with later
effect checkpoints have no general refresh path; old inventories missing the new
projection must remain visibly unavailable until that gap is remediated.

Filesystem observations and remote dispatch are not atomic with arbitrary
editors. Observed Git-input drift gets bounded local rereads; stable exclusions
and genuine I/O errors are not silently retried. No provider redispatch, private
head injection, permission bypass, automatic model selection or second scheduler
is introduced.

## Review, research and diagnostics

Authored owners are the worker contract; engine endpoint worker, reasoning budget
and protected readback; observation delivery; installed inventory/Git observation
helpers; canonical inventory record publication/reconstruction; and relevant
engine, contract and client tests. Catalogue-owned generated schemas/SDKs follow
those types. Runtime source is not deleted or relocated. Git's final diff owns
the complete changed-path inventory; this list is not an exhaustive source review.

Review covered AGENTS/README, the alpha objective, Gate G, source inventory,
POAM-009/038, change-authoring and candidate-CI procedures, and the affected
implementation/test symbols and diffs. Independent review examined projection
integrity, disclosure, no-source compatibility and Git read-error handling.
Reviewed file digests are retained privately. Source inspection is not execution
evidence. No external algorithm or dependency is adopted; existing native
authority, query and filesystem contracts own the repair.

The final review read all 24 changed Rust files in full and their complete
diffs: 11 production engine files, six engine test files, the compute inventory
owner, two client tests, and four contract implementation/test files. No new
blocking finding surfaced. Generated/reference changes were checked against
canonical generation and their complete diffs; the 17 digest-only projections
were compared mechanically, and the TypeScript model additions were inspected.
The three changed POA&M records and this journal were read in full; navigation
was regenerated. This is a changed-file review, not a repository-wide audit.

Diagnostics reuse request/attempt identities, digests, existing audit and local
test logs. No raw project content, credential, endpoint host or private machine
path is added to public evidence. Tests use isolated files and controlled HTTP
envelopes; these are protocol fixtures, never training data or model outcomes.

## Verification and remaining acceptance

The initial focused run failed compilation on fixture paths, visibility and
imports; a following compile exposed fixture type/closure errors. After those
repairs, 11 existing worker tests passed and all 11 source tests failed their
shared typed-query lookup: `kind` was the built-in record type string, not File
metadata. The canonical projection addresses that reproduced boundary.

Focused projection tests (2), the exact legacy fingerprint test (1), and the
installed-fixture typed query/source-export denial test (1) subsequently passed.
The broader worker run progressed from 16/22 to 20/22. The remaining reopen
failure exposed an accepted source grant added to the installed initial principal:
`with_installed_binding` requires its original grant-action list. A separate
explicitly granted principal is the current path to a real restart proof; the
bootstrap-principal grant-change limitation is recorded in POAM-016. The
repetition fixture first configured after a proposal had already reserved the
budget, then mixed a future fixed clock with the host-clock configuration
deadline. Its repair must configure first, use consistent host-aligned clocks,
and retain the actual second-attempt `RepeatedWork` and no-dispatch assertions.
Removing restart or substituting only a fingerprint unit assertion does not
satisfy either oracle. Both original acceptance intents were retained.
The first contract suite also exposed its expected generated OpenAPI digest
mismatch; canonical generation and the corrected raw-byte digest resolve it.

The repaired source-bound suite passes all 11 tests with
`cargo test --locked -p rrd-engine --lib --all-features source_bound:: -- --nocapture`.
Its retained log SHA-256 is
`57adc130240d01e0e83108bc455ed1f856e772830895a667e0d05fe3190606f7`.
The final whole worker module passes all 23 tests. Its retained log SHA-256 is
`82f10d9cb1b57f1ae8dd14aa37648fc2e9eb84f07fdd41d65a6f150e3e3dc037`.
These include actual
installed-store reopen with the separately granted principal, source-derived
observation withholding/restoration, current query field/row policy, unrelated
head changes before and during dispatch, relevant source-change rejection,
tracked/ignored path controls and the real equivalent-work ceiling after staging
and replan. The repetition case configures its budget before the selection,
uses host-clock public proposal/worker calls, then verifies `RepeatedWork`,
unchanged charged/reserved effort and no second worker listener acceptance.
Separate fingerprint tests supplement that integration proof.

The persisted historical-shape regression also passes. A scoped, thread-local
`cfg(test)` guard makes the first canonical Inventory publication emit the
predecessor's strict schema and entries without `source_file`; it does not inject
a control head or add a production legacy mode. Reopening retains the exact old
proposal digest and entries. Public unchanged-source replan, Start, Connect and
Inventory then expose the typed projection. All entries remain equal; only
`previous_sha256`, `changes` and `proposal_sha256` differ in the new proposal.
The initial expectation of whole-proposal equality was rejected and corrected
to these observed semantic identities. The final log SHA-256 is
`25548e43c52bfe783b85d7025cca1157e1dfb4c8525e3707034dfedbba7fd377`.
This is a self-contained historical-format regression, not an old-binary
cross-version qualification. The source-worker reopen case additionally passes
with fresh authentication of its separate principal; log SHA-256
`7f5aaa600194ed8601ac3b6e9c3c22fb1ff60bc76661557446408ced69e3a8c1`.

The native client's observation-delivery target passes all three tests and its
task-effort target passes one. The contract all-target/all-feature suite passes,
including 47 public-contract tests. The server all-target suite passes all 34
tests after building its required estate-controller executable from current
source. The first server run's missing-executable failure remains retained.
The first TypeScript generator invocation did not run: it selected an older
Node/pnpm and dependencies were absent. The repository-pinned Node 24.22.0 and
Corepack-selected pnpm 12.14.0 were used for frozen installation and generation.
All five SDK generators and generated parity pass: 80 HTTP operations and seven
signal projections. OpenAPI SHA-256 is
`5a14cb9e11a001542a863b295990e27d58a21b0b5d98da183189f09d5869d817`;
signal catalogue SHA-256 remains
`239df2ced5974d4351ca01565369646964cb65c63d0fbae01dddccb4c3ac5a07`.
Original UTF-16 command logs remain unchanged; public fingerprints identify
those exact bytes. Dependency manifests and locks are unchanged.

The TypeScript SDK's `corepack pnpm check` passes generated parity, Biome,
`tsc --noEmit` and seven tests. Two preceding checks selected pnpm 12.16.1 and
failed the project's required 12.14.0 toolchain check. A shell-local Corepack
shim fixes nested `pnpm` resolution without changing the project or global
toolchain. The passing log SHA-256 is
`1ba198f18c9ce6c8ae25ae52ff0f2073c9aa9ebae68cd1a7a8fb87f89bf4125e`.

The full engine command
`cargo test --locked -p rrd-engine --all-features --all-targets` exits zero:
438 passed and four ignored across its targets. Its log SHA-256 is
`76ee3f1d662d1ee2c38b7a1ad126a4caadc7da1bf1a6f810522f59cff2a3a58e`.
Strict lint initially rejected two test modules followed by helper functions.
Moving those helpers before the test modules resolves the diagnostics without
changing their bodies or behavior. The final command
`cargo clippy --locked -p rrd-engine -p rrd-contract -p rrd-attunement -p rrd-client -p rrd-server --all-targets --all-features -- -D warnings`
passes; log SHA-256
`cfc111760cdb27200caf3d89b3a3e432a0e1f82051ece834617f763a1df7c72d`.
The full suite above precedes that mechanical ordering-only change.
Navigation, documentation policy, execution inventory and whitespace checks
pass locally. Candidate and protected CI remain pending.
No new installed executable or live BF16 source call is qualified by this record.
The integrated UI, real project effects, independently reviewed outcomes, generic
binary/score/choice modes, triggered assessment cycles, learning, quantization
and total-memory claims remain open. No roadmap gate closes; version stays 1.0.0.
