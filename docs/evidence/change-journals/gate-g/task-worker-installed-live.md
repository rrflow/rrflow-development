# Installed selected-worker draft and recovery

**Status:** active installed evidence; composed candidate CI and full journey pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-installed-live`
**Owner:** G-01b / G-02a / POAM-009; driver: active SWAG session

## Scope and source

PR229's installed-proof refinement was recorded in its description before editing
the existing `scripts/inference/installed_controller_proof.py`. Starting source:
`c75c189612db7e7c8efe016fd92fb1052a3fb056`. The earlier
[admission journal](task-worker-model-admission.md) remains immutable.
Starting tree: `37a044f380f13e24f90e7b7e96a16bca305e66cc`.
This package extends that runner with explicit `--worker` mode and read-only
recovery of both attempts. It introduces no provider client, execution ledger,
project effect, training label or lifecycle authority. The public engine catalogue
supplies operations; the accepted selection receipt supplies worker identity.

Read root instructions/README, alpha and POAM owners, the PR brief, the canonical
build script, existing installed runner, endpoint proposal/worker/generation
contracts, relevant worker admission and budget replay sections, focused worker
tests and the LFG/SWAG reference. No new dependency or upstream algorithm is used.
Existing bounded receipts and hashes suffice; no unbounded tracing is enabled.

## Build failure and correction

The first raw Cargo build compiled but its follow-up assertion failed because
`build.provenance` was null. That artifact and failure remain retained; compilation
did not produce an accepted source receipt. Use the existing canonical builder:

```text
python scripts/build_rrflow.py --profile dev --target-dir target/worker-952e0928 --output <new-build-receipt.json>
```

It passed both compilation observations, source/feature-closure equality and
embedded provenance checks at clean `c75c1896`. The copied executable has one
filesystem link, 153,999,360 bytes, and SHA-256
`60a708117899a022b04818c096af17a925c60eae06c554c39d00e5fe09e028fa`.
This is an unoptimized, unsigned engineering artifact, not a qualified release.
The runner has separately measured SHA-256
`d6630f3b4cca156cc00e8b8ddf77ea6d18655945a16a480d32cf0f22aeca5b42`.
The same-source contract example was built with
`cargo build --locked -p rrd-contract --example endpoint_selection --target-dir target/worker-952e0928`.

## Actual trials, including failure

Inputs are the actual PR229 title/body captured from GitHub, an explicitly
configured private endpoint profile and an existing private credential file.
The task projection retains source URL, original body hash and explicit 4,096-byte
truncation. The profile admits one candidate with default effort; it is not a
multi-provider routing comparison. Discovery observed the configured service
alias, not loaded-weight attestation. Private credentials are never evidence.

The first installed selector returned HTTP 412: `invalid endpoint selection
choice`. No worker was dispatched. Credential-free recovery retained one spent
step, stopped `attempt_failed`, `execution_may_continue: true` and no accepted
proposal. Its estate and unresolved capacity remain intact. The raw choice was
not present in the public error, so its specific decoding cause is unknown.
The failure response SHA-256 is
`b2e2365610a36c7e6b5ef11f8b4819afc2597d13b1a2a542e536ac1d5401c1bc`;
recovery result SHA-256 is
`1beb381f3e304bc3e894a86c43252604ce04f7b4af77a279715de37d8c978d05`.

A second isolated trial clarified the operator prompt's existing choice schema:
candidate evidence metadata is not a selected effort value. No candidate,
capability, validator, model weights or observed result was changed. Its profile
SHA-256 is `88e5cd4713fa1276bd7cd37de01e36c59ea954b4c14e419fe2289ae433af6908`.
This experiment is not an automatic retry of the uncertain first attempt.
It passed:

- Actual selector acceptance and one subsequent engine-admitted worker response.
- Unauthenticated worker denial and missing-credential denial after process
  restart; neither added effort or created a worker attempt.
- Exact scope/task/selection/configuration identity and content digest checks;
  accepted worker draft equals the public task-attempt readback.
- Two total spent steps, original deadline, no active attempt and no remaining
  execution uncertainty after accepted completion.
- Worker replay rejected specifically as `ReasoningAttemptRecorded`, with no
  budget change, both before and after process restart.
- The task record remained unchanged. Text acceptance did not complete the task.
- A separate credential-free recovery read both retained attempts; the decoded
  worker attempt was identical. Every owned server stopped.

The worker returned HTTP 200 and 2,304 UTF-8 content bytes. Reported usage was
1,368 prompt / 504 completion / 1,872 total tokens, with 38,390 ms elapsed for
this worker call. These are individual observations, not a latency benchmark.
Result SHA-256: `d75800affce27396f6541b06899f15c7c65af49a04e1175018fda09b56a042fa`.
Credential-free recovery SHA-256:
`d5a79ebc70aa38ff9aaa718cf42fa08801fff201ea7dd2ea253b320788b75a11`.

Local inputs, exact command arrays and both trial estates are under SWAG's
`.swag/evidence/20261031/worker-installed-inputs`, `installed-worker-first`,
`installed-worker-explicit-choice` and their `-recovery` directories. The runner
commands are its existing explicit build/contract/profile/task/credential inputs
plus `--worker`; recovery uses only `--recover-proof <estate> --output <new-dir>`.
A profile-tagging mistake was corrected before dispatch with the original input
retained. An initial runner parenthesis error was corrected before execution.

## Package checks and review

`python -m py_compile scripts/inference/installed_controller_proof.py` passes.
Three incompatible worker-mode combinations (admission-only, selector fault and
recovery) each exit 2 before creating an estate. Actual installed trials and
credential-free recoveries above supply the behavior oracle; controlled or
generated model responses are not substituted for these calls.
`python scripts/knowledge/render_navigation.py`,
`python scripts/ci/build_execution_inventory.py` and its `--check`,
`python scripts/ci/check_documentation.py`, and `git diff --check` pass.
Navigation reports 847 nodes / 3,077 edges; inventory 2,171 paths; documentation
848 statuses / 846 coordinates. Exact commands and exits are retained in
`worker-installed-inputs/repository-validation.json`.

The complete runner and canonical build script were read; contract, engine,
test and owner files were inspected at the relevant sections, not represented
as whole-file audits. The runner diff was reviewed after execution. Changed
paths are the existing runner, LFG/SWAG reference and POAM-009; this journal is
new and the Gate G index is regenerated. No source was removed or relocated.
No Rust production code changed, so its unchanged broad suites were not rerun
for this Python proof increment. Candidate CI remains pending. The checklist
preserves one engine authority, explicit configuration, secret-free evidence,
real observations and unchanged release/learning status.

## Remaining acceptance

This proves task-text selection and worker draft admission on the named binary.
It does not prove assembled-context export in this installed trial, project
mutation, ordinary-file-edit replanning, process death mid-worker, the complete
Connectome/SDK journey, Claude/Codex subscription adapters, RROFLOW correlation,
reviewed learning, model quality, 29 GB serving, container delivery or release
qualification. Required candidate/protected CI and dependency landing remain.
Keep PR229 draft; no objective, roadmap gate or POAM closes here.
