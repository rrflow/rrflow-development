# Typed System One controller through engine selection

**Status:** active package evidence; integrated engine and installed driver passed, desktop integration pending

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/system-one-controller-20261120`

**Owner:** [kernel contract](../../../roadmap/kernel-contract.md), K07 and [Gate G](../../../roadmap/rrflow-1.0/gate-g.md) G-01b.

Starting revision: `081282df0af594ba3ded2554c9ee5acd2ab88308`, tree
`b7104b59a97127cb0879e1528333d43e1c0aa983`.
Pull request: [#440](https://github.com/rrflow/rrflow-development/pull/440).
Driver: this RRFlow implementation session; no dependency PRs or overlapping
agent work. The audit record and PR brief preceded implementation.

The audit found that endpoint selection only prepares chat-completion requests.
Its narrowed allowance validation and worker native-adapter guard already exist.
This package adds an explicitly configured System One Choice wire protocol at
the same engine admission boundary, with real worker consumption as its oracle.
It does not establish generic internal reasoning decisions, provider quality,
cold recall, rrflowCLD suitability or rrflowLSM parity.

Baseline evidence on Windows, four Cargo jobs, isolated target directory
`D:/.usr/Temp/rrflow-selective-recall-audit-20261120/target`:

- `cargo test --locked -p rrd-core temporal::`: five selected tests passed.
- `cargo test --locked -p rrd-contract --test endpoint_proposal --test router_contract --test reasoning_tree_contract`: twelve tests passed.
- Worktree was clean; no open merge-blocked or Main-is-red issue was returned.

Primary research: [System One API](https://docs.typesafe.ai/api/system-one)
and [Choice](https://docs.typesafe.ai/primitives/choice), read 2026-11-20.
The native wire supports typed questions; endpoint and permitted effort can be
one choice, avoiding a dependent second request. This is a design decision,
not a latency or quality measurement. No upstream source is copied.

Scope: contract policy, prepared inference transport, engine proposal consumer,
focused and joined driver tests, generated contract surfaces and this evidence.
No protocol fallback, automatic retry, compatibility configuration, external
credential discovery or network activation is authorized by configuration alone.

## Change brief and replacement map

Current behavior: `prepare_controller` sends only chat completions. Target:
explicit native typed selection must influence the real worker's endpoint and
reasoning effort, after semantic narrowing, and retain accepted receipts across
restart. The existing admission, authority rechecks, budget/capacity reservation,
completion validation and durable attempt records remain the only effect owners.

`PreparedEndpointChat` is replaced by `PreparedEndpointController`; the existing
chat wire and nine retained-response/denial tests continue through its `chat`
constructor. The native constructor uses the same bounded HTTP execution code,
with a separate wire codec, not another dispatcher. Policy configuration now
requires `controller_protocol`; chat generation fields belong to its chat
variant. No old configuration decoder, automatic protocol selection or retry is
retained. Examples and fixture builders use the new canonical interface.

The native options are the allowed endpoint/effort pairs plus abstentions.
Unknown effort support supplies no selectable pair. A test with two configured
candidates narrows to one before native request construction. Provider choice
and distribution validation cannot enlarge that allowance. A selected effort
then passes through the existing worker mapping to its actual HTTP request.

Implementation paths: `rrd-contract/src/endpoint_policy.rs`, `lib.rs` and
`protocol.rs`; `rrd-inference/src/endpoint_chat.rs` and its new private
`endpoint_chat/system_one.rs`; `rrd-engine/src/engine/endpoint_proposal.rs`.
Characterization and acceptance paths: contract endpoint policy/worker tests,
inference endpoint chat tests and example, engine endpoint configuration and
reasoning-budget proposal/worker tests, CLI `tests/support/loopback_worker.rs`
and `source_worker.rs`. The generated OpenAPI/SDK surfaces and their digest
citations follow the contract owner. No dependency or version changes.

Stop conditions: a new effect authority, undeclared provider/network activation,
required scope expansion into another roadmap package, or a missing consumption
oracle. Native Noul/Score assessment and recipe/branch/context reasoning are
outside this package. A valid controller response is not proof of model quality.

## Research, diagnostics and resource decisions

No upstream implementation is adapted. The reviewed API documents a typed
Choice, at most 255 options, a keyed distribution, confidence and input/output
usage. This adapter uses that interface and requires the exact configured model
identifier, rather than guessing a resolved alias. Confidence is range checked;
no invented confidence threshold or quality claim is introduced.

Existing request, response and canonical-choice digests, HTTP status, usage,
elapsed time and durable attempts provide the trace. No private reasoning trace
or new logging sink is introduced. The native wire has no chat generation token
or temperature parameter; it remains subject to request/response byte bounds,
deadline and engine budgets. A joint endpoint/effort question is one request;
comparative latency, token savings and answer quality were not measured.

## First failures and corrections

- Source characterization: the baseline's decoder only understood chat envelopes.
  The new native engine and installed oracles cannot pass on that implementation.
  No baseline native-provider execution is claimed.
- The first native negative test accepted an old choice after its positional
  option name changed meaning under narrowing. Option names now hash their
  canonical choice; the same response is rejected after narrowing.
- A contract test found that Serde's tagged unit variant ignored unsupported
  generation fields. The native protocol is now a closed empty struct variant,
  and that test passes without weakening its assertion.
- Compilation found an old test mutating a removed top-level generation field;
  it now tests protocol changes against configuration approval instead.
- The full contract suite exposed the expected OpenAPI digest change. All
  projections were regenerated from the owner, and duplicate stale current
  digest citations were removed. Generated-surface parity then passed.
- Documentation validation rejected an unsupported `draft` metadata status;
  the evidence uses `active` while describing the candidate as draft. Inventory
  drift after navigation generation requires regenerating the inventory last.
- A Windows scratch edit initially used the default text encoding. The affected
  documentation was reconstructed from its UTF-8 Git source with only the intended
  section/digest changes; the diff was checked. No source content was discarded.

## Acceptance evidence

Windows development profile; the target directory and four-job limit are above.
These results are controlled protocol or local verification evidence, not a
live-model trial or release qualification.

| Command / driver | Result |
|---|---|
| `cargo test --locked -p rrd-inference --features endpoint-chat --test endpoint_chat` | 12 passed, including malformed native outputs, option limit and narrowed response rejection |
| `cargo test --locked -p rrd-contract --test endpoint_policy` | 6 passed |
| `cargo test --locked -p rrd-engine --lib system_one_controls_worker_effort_and_reopens_without_replay` | Passed, including two-candidate narrowing, selected high effort, both receipts across reopen and replay refusal |
| `cargo test --locked -p rrd-contract` | 203 passed |
| `cargo test --locked -p rrd-inference --features endpoint-chat` | 49 passed, 1 existing LFG live-service test ignored because its explicit service was not launched |
| `python scripts/ci/check_generated_surfaces.py --check` | 83 operations; all projections agree with `ecfce11ae9a7102165c116a6c73837fe0dd3940ce3f72120ee67acfa224cbc83` |
| TypeScript generator `--check`, `tsc --noEmit`, and `tsx --test tests/*.test.ts`, via Node 24.22.0 | All passed; 7 tests |
| `python scripts/ci/check_code_shape.py` | Passed; no ratchet growth |
| `python scripts/ci/check_documentation.py` | Passed after metadata correction |
| `cargo test --locked -p rrflow-cli --test installed_lifecycle installed_source_read_choice_is_explicit_and_reopens` | Passed in 6.50 s after compilation; actual installed engine/public client, controlled provider responses |
| `cargo clippy --locked -p rrd-contract -p rrd-inference -p rrd-engine -p rrflow-cli --all-targets --features rrd-inference/endpoint-chat -- -D warnings` | Passed |
| `cargo test --locked -p rrd-engine` | Failed: 522 passed, 1 failed, 12 ignored in the library; integration targets not reached |

The named installed driver is
`cargo test --locked -p rrflow-cli --test installed_lifecycle installed_source_read_choice_is_explicit_and_reopens`.
It runs the installed process and public client with a controlled native
controller response, source/context admission and worker readback after restart.
Its accepted worker draft survived restart. The full engine suite failed in
the unchanged concurrent serving-record test with Windows error 5 during
publication (`serving.rs:161`). This is recorded as
[POAM-092](../../../poam/rrflow-1.0-alpha/poam-092.md); the log remains in
`engine-tests.log` in the session scratch directory. No green full-suite claim
is made. Candidate CI remains pending, and the PR stays draft for the baseline
repair in a separate bounded package.

## Review coverage and handoff

Owner records read: `AGENTS.md`, `README.md`, agent session standard,
change-authoring routine, kernel contract, Gate G and inference reference.
Production files read in full: endpoint policy/selection contracts, shared
endpoint transport and new native codec. Engine review covers proposal admission,
LFG checkpoint, dispatch, completion and the changed constructor. Test review
covers the complete new native module, existing endpoint chat/policy tests and
the changed configuration/worker/installed-driver functions. Generated schema
diffs show only the intended protocol union and digest changes. This is scoped
review, not a whole-codebase line-review claim; unrelated test bodies, storage,
Connectome and other engine paths are not claimed re-audited by this package.

Change checklist: owning outcome and boundaries recorded; replacement behaviors
and tests mapped; primary research and diagnostics decisions recorded; first
failures preserved above; focused checks passed; installed/owning-suite/lint
results recorded above; the implementation and generated diffs were reviewed,
and the candidate remains blocked on the named baseline defect. The session retains only its Cargo target
and scratch logs under `D:/.usr/Temp/rrflow-selective-recall-audit-20261120`;
no containers, stashes, worktrees or unrelated processes were created or removed.

No roadmap completion-status change; POAM-092 is added as Open. Live-provider performance/quality,
native Noul/Score and reasoning-step consumers remain unverified. The immediate
next action is to repair the serving-record failure, integrate it and finish the candidate gate on this branch;
the existing Gate G owner retains the broader reasoning integration work.


## Dependency integration and remaining-consumer review

The Windows repair [PR442](https://github.com/rrflow/rrflow-development/pull/442)
merged at `2aa15d6000b93be383a7fcff0752837e510fb66d` after green candidate and
merge-group CI. The final repair passed 275 LSM tests (3 existing ignored),
15 serving tests and 590 full engine tests (12 existing ignored); no failures.
It is integrated into this branch by ordinary merge `84a1305f7`.
POAM-092 is Closed with its repair and exact native evidence linked.

The final consumer audit read `task_assessment.rs` preparation and reply
validation and `endpoint_configuration.rs::load_endpoint_configuration`.
Assessments reuse the selector binding but require chat JSON with cited byte
ranges. Their policy now rejects a native controller before publication or
use; configuration is validated again when loaded. The new endpoint-policy
regression confirms the chat configuration is accepted and the same policy
with native System One is rejected. Its focused run passed. No citations are
invented and no alternate protocol is selected. Native assessment and generic
recipe/branch/context decision consumers remain outside this package.

Additional changed files: `rrd-contract/src/task_assessment.rs` and
`tests/endpoint_policy.rs`; the inference reference explains that admission
limit. The original caller/read authority and assessment reply checks remain.
Full integrated contract, inference, engine, installed-driver and strict lint
checks are running; their final results will be recorded below.

The Connectome audit found its SDK pins still name `cc5ebe0fd17bd9cb1a72a0d40811ba381869d202`;
its typed process fixture constructs the old selector shape. A separate
Connectome PR must refresh those pins and fixture before claiming a live
native desktop/controller journey. No Connectome files changed here.


## Integrated local acceptance

With the merged Windows repair and assessment protocol guard, the final source
checks passed: contract 204 tests; inference 49 tests with its one existing
live-service test ignored; full engine 591 tests with 12 existing ignored;
installed `installed_source_read_choice_is_explicit_and_reopens` passed in
6.45 s. The installed driver uses the actual engine/public client and a
controlled native controller, then verifies accepted worker readback after
restart. It does not call a live model. Logs are `controller-final-contract.log`,
`controller-final-inference.log`, `controller-final-engine.log` and
`controller-final-installed.log` in the same session scratch directory.
Generated surfaces and documentation checks pass. Strict affected-crate all-target lint with `-D warnings` passed in 20.72 s (`controller-final-clippy.log`).

The user clarified the wider acceptance target during verification: a live
Connectome prompt journey that exposes constraint/schema, lexicon and behavioral
state; modular SWAG/classification; concurrent ephemeral work and measured
physical writes; complete capability comparisons with SurrealDB, HydraDB and
Qdrant; optional hosted synchronization/free tier; and rroflow as SDK/signal
manager. These remain owning-record implementation requirements, not completion
claims for this controller package. The code audit identified the existing
SurrealDB/Qdrant inventories and kernel-contract K0-K16 owners; no duplicate
capability or planning authority is created here.

Read-only SWAG inspection at public revision
`ec1f0c2eb40b48b973ee3ab98535a3d0c4e9bca5` found its separate raw `answers`
and final `swag.decisions` envelope in README/decision-contract. Its tool status
reported unavailable engine/model endpoint, no verified loaded weights, and no
inference performed. A SWAG adapter must consume its final rule/preference
outcome and preserve provenance, not ignore it and consume only raw answers.
That module adapter is not implemented or claimed by this TypeSafe-compatible
Choice package. The controller wire research is not a SWAG integration test.

## Paired desktop launcher collision

Connectome PR107 now passes 87 ordinary native tests, all 31 installed-process
tests (436.32 s), strict native lint, frontend checks, production build and 33
render smoke cases locally. The focused native System One/source-worker test
passed independently. Its fixtures were regenerated by the three real engine
drivers, then verified by the full process run. These are local, not CI results.

Actual desktop acceptance through `python scripts/dev/connectome_dev.py` opened
the native window but first waited on a slow cold Vite load. HTTP reads to its
index and client timed out, then recovered without code changes. Once loaded,
the Project engines UI reported a digest refusal: the shared Cargo target had
placed Connectome beside the engine debug binary, which its existing native
resolver treated as the bundled sidecar. The debug hash differed from the
canonical sidecar hash embedded by Connectome. The refusal was correct; the
launcher environment was wrong. No project was installed in this attempt.

The PR brief now includes `scripts/dev/connectome_dev.py`,
`scripts/dev/test_connectome_dev.py` and the owning development-loop reference.
The canonical destination is the existing launcher: isolate the application
Cargo target, expose its path in the report, and refuse an explicit target
collision. Preserve the client's sidecar verification. The launcher and its
complete test file, Connectome `src-tauri/src/rrd/engine.rs`, `build.rs`,
`window.rs`, startup/main and Vite configuration were read for this diagnosis.
The first launcher build accidentally duplicated UI output in the engine target;
subsequent runs use the existing separate Connectome target. No other session's
process or data was removed. Cold-start latency is not qualified by this fix.


## Desktop consumer and exact development engine selection

Connectome PR107 merged through successful candidate and merge-group gates as
`7cbe1f10b248e6477dc2c5b46581a639a6337344`. Its source tip
`e6d54d42abb41f467d9bb475cdc0d36b76bab29b` consumes this engine's SDK at
`36dccea2c8b47f6e431078fbdbb54165546236ef`. Native Windows/Linux/macOS checks,
frontend checks and all installed-process drivers passed in GitHub run
44808683576; merge-group run 44808724615 also passed.

The separate startup package, Connectome PR108, restores route-derived sidebar
access to existing tools. Actual desktop use installed a fresh project and
opened live graph/vector/storage summaries, but exposed a second executable
selection mismatch: Cargo prepended `debug/deps` to PATH and selected a prior
process-test executable there. The launcher now sets `RRFLOW_DEV_ENGINE` to
its private staged binary's absolute path; the debug desktop consumes that
binding and refuses conflicting saved executable settings. Release sidecar
verification is unchanged. No stale test executable is removed to mask this.

The parent environment suite passes all ten tests. The child exact-selection
unit test passes, and its new real-process driver with empty saved executable
installs, serves, verifies path and SHA-256, authenticates and reads a fresh
project through the same native lifecycle. The existing two-project isolation
driver also passes on Windows. These two drivers used debug engine SHA-256
`bf7b95e06ba92eea62f6a97ffb1e021c6f466b4be54420eb902a882421be0a4d`.
Child frontend checks/build and all 35 smoke cases pass; native unit suite
88 passed, strict all-target lint passed. The child journal records exact
commands, before-failure evidence and limitations. The first documentation
check of POAM-093 failed missing required headings/title, then gate links;
those record defects were corrected without changing the check.

POAM-093 remains Open: the final explicit-binding desktop journey was not
repeated after the owner stopped Computer Use. The native process proof is
recorded separately from visual acceptance. Hosted synchronization, full
inherited capability parity, project-link consent/federation, enforced pod
isolation and joined prompt/model behavior are not completed by these packages.
No roadmap gate is marked complete. The six old official-remote branches remain
on origin; their 2026-11-17 absorption record is historical evidence, not a new
behavioral verification by this session. Development branch cleanup is tracked
by actual pull-request state and the session-end check.


## Multi-project boundary checked during integration

The current project registry isolation test passed against the checkout engine;
its source in `connectome/src-tauri/src/rrd/client/process.rs` was read around
`project_registry_keeps_parallel_engrams_and_sessions_isolated`. It checks
independent endpoints/credentials, data and sessions, reconnection and targeted
stop. This is narrower than physical resource isolation or a noise comparison.
The native lifecycle registry and `runtime/shell/workspace.tsx` expose project
lifecycle and explicitly do not grant context sharing.

Read `crates/operations/rrd-kubernetes/src/resources.rs` through its complete
resource renderer, `controller.rs` through apply/cleanup and the resource
application path, and the corresponding rendering test. The renderer has a
single RRD StatefulSet, retained volume, non-root security context, CPU/memory
limits and ingress/egress policy. The controller reports readiness from ready
replicas, forces server-side apply, and performs direct cleanup. These are
existing implementation inputs, not live-cluster or hostile-isolation proof.
The deployment owner already records those limitations. `eval/lab/pod.sh` was
read: it runs comparison systems with a shared Docker socket; it is an evaluation
harness, not the application's project-pod manager.

The H-10 owner and `rrflowdb-capability.md#project-links` require two-party
consent, revocation and two stamped/audited reads. A source search of Rust
crates for project-link/federated-read symbols found no implementation, consistent
with that owner's explicitly missing project-link record. H-07e owns the
friendly separate controls for context sharing, sync and physical placement.
Those controls are not implemented by startup navigation. No inference about
BEAM equivalence or a performance comparison is made.
