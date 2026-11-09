# Actual installed controller completion and fault histories

**Status:** active installed experimental evidence; candidate CI and landing pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-task-proposal-installed-live`
**Owner:** G-01a/G-01b and G-02/G-05; OBJ-12 and POAM-009 remain open
**Pull request:** [206](https://github.com/rrflow/rrflow-development/pull/206)

## Baseline, brief and inspected owners

Source baseline `a8d0bceb6ffaea0ac616769eb89cb969084a7ff4`; clean-source installed
engine binary `d365d02891350590aa4495b46fc665ef313aec24`, executable SHA256
`7fe07a7198b864a2642c5a9a521e8e13ac7a7d935fd62e8477f2c075fd7680b9`.
The existing full-operation brief includes installed execution, Stop, revocation,
configuration replacement and uncertain transport. Before implementation, the PR
also received the explicit bounded relay scope, file owners, two-call/one-call
limits and failure oracle. Runtime Rust behavior is unchanged in this stage.

Read AGENTS/README and earlier admission/denial journals, the proof driver,
endpoint proposal completion/capacity logic, budget reserve/finish, session
closure/error mapping, actual profile/operation catalogue and retained responses.
The new `scripts/inference/installed_controller_fault_relay.py` is only proof
infrastructure owned alongside `installed_controller_proof.py`, not engine code,
a provider adapter, a deployment dependency or another lifecycle authority.
The reference/POAM now link this acceptance evidence without closing their gates.

## Restored access and first installed success

The operator completed the Tailscale SSH identity check. The existing custom
container was running without restart/OOM flags. A new owned loopback SSH forward
restored the configured endpoint. Authenticated model discovery and health returned
200; unauthenticated discovery returned 401. Provider metrics showed zero running
and waiting requests before a separately bounded new development trial. The old
uncertain estate was preserved; later idle metrics do not prove historical
non-dispatch or authorize erasing its capacity.

The actual installed engine admitted a task from an explicitly bounded projection
of the real public PR206 body and called the configured BF16 service. It returned
200 and retained the typed proposal, configuration/task/request/response hashes
and reported usage: 1,797 input tokens, 26 output tokens, 1,823 total. Adapter
elapsed time was 2,362 ms; budget charged 2,387 ms. Restart returned identical
attempt/budget records; repeated attempt identity was rejected and only one step
was reserved. Provider success counters increased from 43 to 44 during that trial.
This is a single observation, not latency, model-quality or weight-attestation
qualification. There is only one configured candidate in this transport profile.

## Actual response fault control

The optional `--fault-case` mode installs a trial profile whose controller URL is
an ephemeral loopback relay. The relay forwards the actual engine request bytes
to the configured provider once, without ambient proxy or redirect, within the
profile's input/output/time bounds. It retains the actual provider response, then
holds delivery while the proof invokes a real public engine action. Release or
connection loss completes the controlled history. No generated or historical
model response is substituted. Credentials remain in memory and never enter logs.

| Case | Actual installed result and restart evidence |
|---|---|
| Configuration replacement | Held response rejected as `inputs_changed`; charged effort retained, budget unstopped. A fresh request at the new configuration succeeds on the same task. Two reserved steps, original deadline retained, exact reopen readback. |
| Session closure | Held response rejected as `authorization_changed`; no accepted proposal. One charged step, budget unstopped, exact reopen readback using a newly authenticated session. |
| Stop | Real public Stop while response delivery is held; no accepted proposal. Stopped/cancelled budget, one charged step, conservative remote-execution uncertainty retained through reopen. |
| Lost response | Actual provider response retained, delivery connection dropped; no accepted proposal. Stopped/attempt-failed budget, one charged step, uncertainty retained through reopen. |

The configuration case made two real provider calls; each other fault case made
one. The baseline made one. These are six actual calls in total, not six training
examples. Stop was applied after the relay received a real completed provider
response and before engine publication; it proves cancellation of local acceptance,
not interruption/termination of remote GPU work. Ordinary repository source edit
and successor-plan execution remain distinct missing capabilities.

## Reviewable receipts and validation

[Receipt manifest](controller-installed-receipts/manifest.json) identifies 33 exact
retained public response/observation files with byte sizes and SHA256. It includes
the baseline accepted/replay/reopen records and each fault's actual provider
responses, failed public admission/completion, budget and reopen result. Credential
values and session-token fields were checked absent before copying. Private estate
databases, credential files, binaries and unsanitized process environments were
excluded. Previous failure receipts/journals remain unchanged.

Executed the repository proof against the source-identified native binary with
supplied `--build-receipt`, `--contract-example`, `--profile`, `--task-source`,
explicit credential-file/key/environment names and exclusive `--output`; repeated
with `--fault-case configuration`, `session`, `stop`, and `disconnect`.
All five complete trial runs passed. A final `--recover-proof` read of the accepted
baseline estate matched its previous attempt/budget exactly without dispatch.
Exact local commands/results are retained under the operator's
`.swag/evidence/20261029/controller-operation-plan/` and corresponding
`installed-controller-proposal-connected/`, `installed-controller-fault-*/` and
`controller-access-restored/` directories.

Both scripts passed syntax compilation; mixed fault/recovery/admission modes
rejected before process/output creation. All 33 receipt hashes passed. Driver SHA256
`75ad84d69c2ce1fec8ec1021eedef398144ba5508a2e70d841c7998b40cf9d80`;
relay SHA256 `92e4d85c0080e4d9012ddf5d9a8b0790d2e41f264c874b8daca8202fb84ada4e`.
Navigation, documentation, inventory and workflow checks passed. The first staged
whitespace check flagged Windows CRLF in exact captured JSON. Following existing
captured-fixture owners, the local `.gitattributes` preserves those bytes and marks
CR-at-EOL explicitly; no receipt content or hash is normalized to hide the failure.
The corrected whitespace check passed. Repository check commands were
`render_navigation.py --check`, `check_documentation.py`,
`build_execution_inventory.py --check`, `check_workflow.py` and
`git diff --cached --check`.
Earlier Rust/SDK suites remain the evidence for unchanged runtime source; fresh
candidate CI must still run on the final branch. No research algorithm or
performance optimization was introduced by this proof harness.

## Remaining scope

This establishes the bounded experimental task-text proposal and guarded local
completion path. It does not qualify routing quality, remote hard-stop, public
uncertainty reconciliation, selected-worker effects, repository-edit successor
plans, Connectome/RROFLOW integration, reviewed training, release/image delivery
or the 29 GB target. The development executable uses an unoptimized debug build;
its observed call time is not a release-performance result. The full goal remains
unfinished and this package still requires candidate and protected landing checks.
