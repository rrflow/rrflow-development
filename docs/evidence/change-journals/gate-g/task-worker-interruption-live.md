# Installed worker interruption and independent readback

**Status:** active installed fault evidence; reconciliation and full delivery pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-interruption-live`
**Owner:** G-01b/G-02a / POAM-009; PR229; active SWAG session

## Source, scope and review

The [harness package](task-worker-interruption-harness.md) was committed clean at
`2f7a1a344958c29891a2934a1965dfdfc8c0acc9`, tree
`38d247457c9ac92b486d440c22bdd2bc41052779`, before this actual provider trial.
Its runner SHA-256 is
`c9f5f38daee112cc482e8c61a3e7d1c68568a985ca31509822f8df2fdfbbf521`;
relay SHA-256 is
`af3b9eb9a9e42bd8fb081d8ac9556578c9484e8f01c44009d3ca412767deca7b`.
The installed primary remains clean `c75c1896`, executable SHA-256
`60a708117899a022b04818c096af17a925c60eae06c554c39d00e5fe09e028fa`.
The contract helper remains the prior `43d18059` build. No new engine build is
claimed. The configured short-worker profile and captured public PR229 task are
the same supplied inputs as the preceding native trial; this run uses HTTP.

Reviewed AGENTS/README, Gate G and POAM-009 owners, the complete runner and relay,
focused tests, CI diff, prior native journal and current PR brief. The pre-edit
publication brief limits this increment to this immutable record, its generated
index and adjacent byte-preserved receipts. The actual provider responses were
inspected; selected receipts were scanned against the private credential and
credential-bearing field names before copying. Credentials, session tokens,
installed estates and environment files remain local. No new algorithm,
resource measurement, runtime behavior or roadmap-status change is introduced.

## Actual experiment

From the clean harness source, run
`python scripts/inference/installed_controller_proof.py --worker --worker-interrupt`
with the frozen build receipt, contract helper, explicit profile, actual task,
private credential reference and a new output directory. Exact argument arrays,
source hashes and logs are retained under SWAG
`.swag/evidence/20261031/worker-interruption/`. The trial estate is
`.swag/evidence/20261031/installed-worker-interrupted/`.

The command exited zero at 18:48:03 UTC on 31 October 2026. Its log SHA-256 is
`123ded07ee201c6e4ea25541653bdf87a09f1762df454a435f6e4d2a016d3269`.
Exactly two actual provider calls occurred. The selector response passed through;
the worker response was held before delivery. Both returned HTTP 200. The
[actual worker bytes](task-worker-interruption-receipts/relay-2-provider-response.json)
are 1,806 bytes with SHA-256
`535b144a8c7864eef27f8d183ec2384539042f133dd64d570c8af41dcf25a6c9`.
They were not accepted as an engine worker result or assessed for routing quality.

Before interruption, public reads showed a reserved worker without output, two
spent steps, the original deadline, the same task, and that worker as the active
uncertain attempt. The runner killed only its owned engine PID 77016, observed
exit code 1, then released/dropped the held response. The original caller observed
`ConnectionResetError`; it did not return an accepted response. The independently
started engine reopened without provider credential mapping and retained exact
selector, worker and budget equality and the unchanged task. No model operation
was requested on reopen.

The [trial result](task-worker-interruption-receipts/result.json) SHA-256 is
`6c73987744fbca320647861aa2c627d1ec707a45f0eee1f5d1630a45440b32ea`.

## Separate recovery and remaining limits

The existing `--recover-proof` mode then reopened that same estate in a separate
invocation with only source-proof and output arguments, no provider credentials.
It exited zero at 18:48:21 UTC. Actual selector, worker and budget values equal
the trial's records; recovery requested no provider call. The
[recovery result](task-worker-interruption-receipts/recovery.json) SHA-256 is
`555cfddcec7063b080073c738f73c158f93d8422e240395c5982a2eb9d8b3dd2`.
Its log SHA-256 is
`7904677ca91a1b823b00894f215b92dce77ab2820725a2a6ef79d71301ce41bb`.
The [receipt manifest](task-worker-interruption-receipts/manifest.json) binds 14
records; SHA-256 is
`c3ce80a6850efffd5d31d98b578153b75620be14dda0b32f203a0f5dcebc8277`.
Raw JSON bytes retain their line endings; do not normalize them after hashing.

Navigation regeneration passes with 860 nodes and 3,108 edges. Execution
inventory generation/check passes with 2,238 paths; documentation policy passes
with 861 statuses and 859 classified coordinates. The staged receipt bytes
are checked against the manifest, and whitespace is checked before commit.
No Rust suites or additional model calls are needed for this evidence-only
increment; implementation checks belong to the preceding harness journal.

Both actual commands and explicit cross-result/task equality assertions passed.
No live retry, replacement estate or relaxed validator was used to obtain this
result. A preparatory shell command used an invalid working-directory spelling;
the process was not created, and the corrected commit command succeeded before
the live trial. No engine or model defect was exposed in this experiment.

The worker remains reserved with `execution_may_continue: true`. This proves
honest retention after process loss, not terminal reconciliation, capacity
release, remote cancellation, replay safety for arbitrary providers, or a
usable resume operation. Existing uncertain estates remain untouched. Candidate
CI, dependency landing, remaining policy/credential qualification, complete
UI/project effects and RROFLOW correlation, independently reviewed learning,
model qualification and native/container release remain open. No training
record, consent, capability ranking or 29 GB claim follows from this evidence.
