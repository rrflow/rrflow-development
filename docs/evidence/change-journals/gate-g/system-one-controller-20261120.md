# Typed System One controller through engine selection

**Status:** active package evidence; controller and installed driver passed, draft held for a Windows baseline failure

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
