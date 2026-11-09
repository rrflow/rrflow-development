# Installed worker and RROFLOW observation proof

**Status:** active local installed evidence; candidate publication and protected CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-observation-installed`
**Owner:** G-01b/G-02a, H-05 and POAM-009; PR229

## Candidate and scope

This follows the [worker/observation reconciliation](task-worker-observation-main.md)
on clean source `5319a0c5cb7f6f924d38ba313b2ab856561e0a10`. The
[structured receipt](task-worker-observation-installed-receipt.json) carries the
exact binary, request/response, configuration, source and received-event identities.
The Windows developer executable is 173,427,200 bytes with SHA-256
`365c0efce1dc3c3559565100ef77c9e237af48ed129d2824dc6973910bb888c7`.
That is executable size, not serving memory or a model footprint.

`python scripts/build_rrflow.py --profile dev --output <new-private-receipt>`
completed both compilation-input capture and embedded-provenance verification.
`cargo build --locked -j 1 -p rrd-contract --example endpoint_selection -p rrd-client --example task_model_call`
built the existing typed helpers. Source remained clean during compilation.
The additional commit containing this journal does not change the binary's
recorded source identity.

## Actual model and recovery

The named installed driver is
`scripts/inference/installed_controller_proof.py --worker --native-client <built-helper>`
with the exact build receipt, explicit profile, actual public PR229 title/body/head,
existing credential source and a fresh private evidence directory. The profile's
expired local forwarding address was replaced by a newly owned connection to
the configured endpoint. This changed only its connection address; the effective
profile and original digest are retained. Model settings and answers were not
substituted with expected results.

The installed primary executable authorized one real controller selection and
one worker draft through the native client. Both returned HTTP 200. Missing
credentials and unauthenticated worker admission were denied. The retained
selection, worker result and budget survived process restart; same-attempt
replay was rejected. A separate `--recover-proof` run, after the owned forwarding
process was stopped and without provider credentials, read the same history
without requesting a provider call. Both driver invocations exited zero.

The response's service alias identifies the configured service, not independently
attested weights. This is one previously exposed development task, not a frozen
unseen benchmark or an independently reviewed correctness outcome.

## Actual local receiver and correlation

A bounded private harness reused the retained public-operation and owned-process
helpers to reopen that exact trial estate. It installed an explicitly configured
local RROFLOW receiver, appended its authenticated observation binding through
engine configuration preview/apply, and exported the accepted worker attempt.
It invoked no selector or worker operation. Receiver revision is
`b4c04ceec6686db056b53bc48de44137c5c2f885`; its executable digest and build-info
are in the receipt. No hosted service is implied.

The receiver retained one actual metadata event. Its configuration hash identifies
the earlier worker execution, while the export receipt binds the later authorized
sink configuration. Task, attempt and source-response identities match. The wire
event excludes the worker's draft and credentials. After both owned services were
terminated and reopened, readback was identical and acknowledged delivery replay
created no duplicate. The model budget was unchanged and the task remained open.
The harness exited zero and all its owned processes were stopped. Existing user
services and the model endpoint were left running.

## Review, validation and limits

This evidence-only increment adds this journal, the generated structured receipt
and its navigation entry; it changes no runtime code. The complete new receipt and
journal were reviewed against the retained source results. Raw evidence and
connection configuration remain private; the public receipt retains their hashes
and the actual metadata event without private infrastructure coordinates.
Navigation, documentation and whitespace checks pass. The preceding code candidate
has 649 passing Rust tests, four explicit ignored tests, strict Clippy and the six
TypeScript tests; those retain their exact earlier source/worktree scope.

No new algorithm, external research adaptation, telemetry store or learning
lifecycle is added. The original source/attempt/delivery identities remain the
debugging authority. The local installed trial advances worker-to-observation
integration evidence. It does not establish UI execution, ordinary-edit native
replanning, verified project changes, template fulfillment, hosted availability,
model quality, reviewed training labels, quantization equivalence or total serving
memory. Candidate and merge-group CI, PR229 publication and the dependent project
workflow remain open; no roadmap or POAM gate is marked complete.
