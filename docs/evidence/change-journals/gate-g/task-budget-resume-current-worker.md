# Resume reconciliation with the landed worker contract

**Status:** active reconciliation; local validation passed, installed proof pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-budget-resume-current-worker`
**Owner:** G-01b/G-02a / POAM-009; PR230; SWAG integration driver

## Brief and source boundary

[PR230](https://github.com/rrflow/rrflow-development/pull/230) carries the
pre-edit reconciliation brief. The existing resume branch starts this merge at
`98996a107b28da1f4317cd501135805ae2660d4a`. Worker PR229 landed as
`f4d74af4861e8f2f008e3c3bd8fcb4e1503c0294` after all 24 protected checks in
run `42309859784` passed. Its tree equals the previously tested `a5dbb754`
candidate. Connectome PR28 subsequently landed as
`1032dd05e536e98f7b69eb55bdd64129a7472c31` after six protected checks in
run `42315069897` passed.

The production consumer remains Connectome's Tasks workspace. Its landed engine
pin lacks resume, so UI PR29 needs this native operation before its recovery
controls can be reconciled. The operation resumes only known-completed stopped
work, preserving the original allowance, deadline and attempt history. It does
not dispatch, clear an uncertain effect or grant permission for a new attempt.
The existing engine transaction and public contract remain the only authorities.

The ordinary merge has 22 overlapping paths. Preserve worker observation,
discovery, live-query identity, vector and quantization behavior while retaining
resume. Resolve the authored catalogue and test modules, then regenerate SDKs,
signal projections, current digest references and the conformance count from
their existing owners. No runtime module is moved or removed. Historical
receipts and their source identities remain unchanged.

## Failure, correction and source review

The first conflict-resolution attempt selected the incoming whole file for
20 paths. That removed two nonconflicting additions from the canonical contract:
the resume endpoint descriptor and its request-envelope schema. The subsequent
export succeeded but produced the incoming-only digest. Review caught the loss
before generators or tests ran; the attempt was stopped and independently
reviewed. A successful export was not accepted as proof of the combined contract.

The corrective patch restored exactly those two additions from the retained
merge preview. The corrected contract equals that preview with only the digest
conflict resolved. The worker test module retains both resume and observation;
the POA&M retains both entries without changing their completion status.
Independent comparison preserves all 54 resume-only and 455 landing-only paths.
This comparison is source-preservation evidence, not a new audit of every
incoming implementation or a substitute for runtime tests.

The actual combined OpenAPI digest is
`e6bd6b60e12398a5ee5ddaf23f846e74288bc128f91acc561d9099381072d80f`.
The exported catalogue contains 80 unique operations: all 79 landed operations
and one `task-budget-resume`. Its descriptor requires SessionBearer and
ReasoningWrite; its request requires scope, task identity and the expected
budget digest. Each generated language endpoint table has the same operation
set. Structural comparison shows that only `expected.endpoint_count` changes
in the conformance fixture. An initial static comparison regex also matched
the websocket helper; its assertion stopped before edits. Restricting that
comparison to the actual endpoint helper established the exact set parity.

Review followed repository instructions, README and the Gate G, POA&M,
change-authoring and CI owners. The resume implementation, recovery tests and
installed resume helper were read in full. Larger contract, transport,
persistence and proof-runner files were reviewed at the affected symbols and
through their complete reconciliation diffs. Independent review covered all
22 overlap resolutions and the generated OpenAPI/schema and SDK changes.
No exhaustive review of the incoming main tree is claimed.

## Local acceptance

All five canonical SDK generators and the signal/generated-surface renderer
exited zero. The latter reports 80 operations and seven signal projections.
The TypeScript generation ran under ambient Node 24.21.0/pnpm 10.30.3 and emitted
an unsupported-engine warning; this is not accepted as pinned-toolchain
validation. Under Node 24.22.0 and pnpm 12.14.0, the subsequent TypeScript lint,
type check and all seven tests pass. Its generation-identity check subsequently
passes under those same required versions.

The installed-proof harness tests pass: 14 tests, no real provider call.
All 11 focused engine worker tests pass, with none ignored: retained resume
effort/history, denial cases, worker observation, changed-input admission and
uncertain response handling are included. The HTTP/native task-effort case,
five catalogue cases and three observation-delivery cases also pass: 20 focused
tests in total, with none failed or ignored.

`cargo test --locked -p rrd-engine --all-features --all-targets` passes: 354
library tests and 65 tests in the additional targets. Four library fixtures
are explicitly ignored: three subprocess helpers exercised by their parent
histories and one privileged ownership fixture reserved for native CI.
The required estate-controller binary builds. The combined contract, server
and client command with `--all-features --all-targets` passes 240 tests across
45 test targets, with none failed or ignored.

The supported SDK conformance runner passes for Rust, TypeScript, Python, Go,
Java and .NET against the same hermetic real daemon and corpus digest
`803fc64d2995345666148975958aacc3d05e119fbdc81c17c73f50f47a18ad0b`.
These are protocol checks, not provider-quality observations or training labels.
The first .NET test executable reports five passing tests, but its build log
also contains a nested `nuget`/missing-SDK diagnostic. Inspection finds that
the pinned SDK 10.0.402 is unavailable; a later root probe resolves 10.0.401,
while the SDK-directory probe fails. The original run did not record its
resolved SDK version, and the nested caller remains unidentified.

The official Windows SDK 10.0.402 archive was verified against its published
SHA-512 and extracted into an isolated ignored tool directory. Process-local
tool selection resolves 10.0.402 from both repository and SDK roots. A fresh
`python scripts/ci/run_sdk_conformance.py` run passes all six languages on the
same corpus, including five .NET tests, without the earlier diagnostic.
Node 24.22.0 and pnpm 12.14.0 were verified for this rerun. No repository pin
or global tool setting changed.
Strict Clippy passes for the engine, contract, server and client with
`--locked --all-features --all-targets -- -D warnings`. The final
`python scripts/ci/check_generated_surfaces.py` check also passes. Independent
final source review finds no merge blocker and reconfirms both-parent source
preservation; staging and final tree verification remain the integrator's next
step. A clean source-bound installed recovery trial and candidate/protected CI
remain pending. A ready candidate is not established.

Navigation and execution-inventory generation/checks pass; documentation policy
reports 998 statuses and 996 classified coordinates. Formatting first rejected
the order of the resume/observation test modules. Sorting those declarations
after the engine test group completed resolves that check; no test body changes.
The unstaged whitespace check passes. The ordinary staged check flags only CRLF
endings in three incoming worker receipts and the incoming H-05 audit-landing
journal. All four staged blobs equal their landed originals, and every flagged
line has no extra trailing space/tab before its CR. The normal staged check
excluding those exact paths and a CRLF-aware check of those paths both pass;
historical evidence bytes are not normalized or rewritten.

The final documentation check rejected an edited status that omitted the
required `active` or `historical` classification. Restoring `active` fixes the
record classification; the failed check remains retained. The corrected
navigation/inventory generation and checks, documentation policy, formatting
and unstaged whitespace checks all pass.
After further journal edits the execution-inventory check correctly reports
drift; regenerate its derived inventory before the final check and commit.

SDK-tool preflight initially reports Maven absent from the shell PATH. The
existing Maven 3.10.0 and Adoptium JDK 25.0.4.1 installation supplies the missing
paths for the validation process; all supported SDK toolchains are then found
and the subsequent conformance run passes as recorded above. This Maven/JDK
repair required no download or global setting change; the later isolated .NET
download is recorded separately above.

Research/source adaptation is not needed for this source reconciliation; no
external algorithm or dependency is introduced by the resume correction.
Existing engine receipts and correlation identities own debugging. The failed
resolution, corrective review, raw exports, command exits and logs are retained
in the operator's private evidence store. No new telemetry or lifecycle ledger
is introduced. Resource use, model quality and serving memory are not qualified
by these checks.

## Remaining journey and stop conditions

After native recovery acceptance, reconcile UI PR29 and PR30 and prove the
ordinary-edit preview, stale-preview denial, approved successor and reconnect
readback through the existing UI. The current worker UI supplies no repository
context and returns a draft. Replanning creates a pending successor, not an
applied project edit. Source-bound context, governed project-file effects,
independent verification and reviewed learning outcomes remain required for
the complete journey.

Stop on changed scope, lost source/operation lineage, unknown external outcomes,
altered original budgets/history, failed checks or unreviewed parallel changes.
Do not retry the previously policy-rejected restart of an existing desktop
engine; it supplies no acceptance evidence. A fresh trial may manage only its
own child processes. No gate closure, version change, binary distribution,
automatic learning or memory-target claim is made here.
