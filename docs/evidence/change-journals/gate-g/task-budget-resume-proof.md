# Installed same-task recovery proof harness

**Status:** active harness evidence; actual provider acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-budget-resume-proof`
**Owner:** G-01b/G-02a / POAM-009; PR230

## Boundary and source

The pre-edit increment in [PR230](https://github.com/rrflow/rrflow-development/pull/230)
starts at clean `6a4cbb042a13f5a7c41b53d11d62b54a85df3bed`.
The [resume implementation](task-budget-resume.md) has engine and HTTP checks,
but the preceding [actual terminal trial](task-worker-terminal-live.md) proves
only that another task can use released capacity. This increment prepares the
missing installed same-task recovery oracle. It changes no engine semantics.

`scripts/inference/installed_controller_proof.py` adds an explicit
`--worker-terminal-resume --resume-profile PATH` mode. The supplied profiles
must differ only in the selected worker's output allowance, which increases
after the actual terminal rejection. `installed_task_resume_proof.py` carries
public-operation assertions; the existing relay still delivers actual provider
bytes without replacing output. Its tests cover the fourth forwarded call,
argument exclusions, profile restrictions and effort-preservation assertions.

## Required actual history

The runner must observe an actual length-terminated worker response, reject
resume while that response is held, retain the stopped attempt and original
budget across reopen, deny unauthenticated/stale resume, and explicitly resume
the same task at its current budget digest. Resume preserves every budget field
except the stop marker, Stop replay marker and observed clock. It dispatches
nothing. Profile replacement uses normal configuration preview and approval.

A fresh selector and worker then spend two further steps under the original
deadline. Both original attempts remain unchanged; replay dispatches nothing.
Exactly four provider calls are allowed. Credential-free reopen and independent
read-only recovery compare all four attempts and the final budget. A provider
failure, abstention, unexpected terminal condition or unequal readback stops the
proof. No automatic trial retry, budget extension, historical lease clearance,
task completion, training record or routing-quality claim follows.

## Review and verification

Review follows README, alpha objectives, Gate G, POAM-009 and change-authoring.
The resume implementation, helper, relay test file and changed runner branches
were inspected, along with the existing public-call/reopen machinery and prior
terminal-trial journal. Scope is these three Python files, this journal and
generated navigation/inventory. No files or behavior are removed or relocated.
No external algorithm/research, telemetry authority or performance measurement
is introduced. Actual operation receipts and provider envelopes own diagnostics.

`python -m unittest discover -s scripts/inference -p test_installed_controller_fault_relay.py`
passes all 14 tests in 12.608 seconds. Transport bytes and in-memory budget
perturbations are explicitly test probes, not generated model/training data.
The budget oracle starts from retained actual terminal-trial control bytes.
Python compilation passes for all three changed modules. Navigation generation
passes with 868 nodes and 3,127 edges; inventory generation/check passes with
2,339 paths. Documentation checks pass with 869 statuses and 867 coordinates;
whitespace checks pass. The frozen executable's independently checked SHA-256
matches its build receipt.
The already recorded Rust suites are not repeated for this proof-only change.

The harness must be committed before actual calls. Installed evidence will have
its own immutable journal and exact source, profile, executable and receipt
identities. The frozen engine remains the clean-source `6a4cbb04` executable,
SHA-256 `d4cc99d8527b97ba145023e0b1cc4a7c26a0682f6edc8c2d5d8d9ff1ad4ffec0`.
Actual same-task BF16 recovery, UI composition, unknown-outcome reconciliation,
ordinary-edit/project effects, RROFLOW outcomes, reviewed learning and release
qualification remain open. No roadmap gate, maturity or version changes.
