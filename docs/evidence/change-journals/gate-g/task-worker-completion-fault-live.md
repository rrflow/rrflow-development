# Actual installed worker completion faults

**Status:** active installed acceptance evidence; reconciliation and full delivery pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-completion-fault-live`
**Owner:** G-01b/G-02a / POAM-009; PR229; active SWAG session

## Source, scope and review

The [harness](task-worker-completion-fault-harness.md) was committed clean at
`8f8bc2b2280c31b457efbe7240c867a0ceeb4062` before these model calls. Its existing
runner now supports `--worker --fault-case configuration|session|stop|disconnect`.
The installed engine is independently built from clean
`e35283303f4c5823c612686d32fb6bee1ac368ac`, executable SHA-256
`8c323716f02e210ebc211bf4593dde84b03d1b55d50f62e710676cda5a971da0`.
The prior contract helper and explicitly supplied short-worker profile are
unchanged. The input task is the captured current PR229 title and bounded body,
with source URL/revision and truncation recorded by the runner.

The pre-edit PR brief binds these four installed interventions and subsequent
evidence publication. AGENTS/README, Gate G/POAM-009, the change-authoring
procedure, complete runner/relay/tests, worker completion admission and the
existing engine fault characterization were reviewed for the harness increment.
This record adds only the immutable journal, generated index and adjacent
byte-preserved receipts. No external algorithm, runtime change or resource
benchmark is introduced. Explicit raw provider bytes and bounded digests are
the evidence; no training labels, inferred consent or default prompt logs follow.

## Actual results

Each case used a separate installed estate and exactly two actual provider
calls. The selector response passed through; the actual worker response was
held until the engine's reservation and original deadline were read. The
requested intervention then occurred before response delivery or publication.
These are four distinct fault histories, not retries of a failed case.

| Intervention | Retained worker | Persisted stop | Execution may continue | Charged ms |
|---|---|---|---|---:|
| Apply a new configuration revision | invalidated | none | false | 16,889 |
| Close the dispatch session | invalidated | none | false | 16,054 |
| Stop using the observed budget digest | stopped | cancelled | true | 16,106 |
| Drop the held response delivery | stopped | attempt_failed | true | 16,107 |

All four trials exited zero. Public readbacks showed no accepted worker output,
the original accepted selector and unchanged task, two spent steps and the
original deadline. Reopen without provider credentials returned the exact worker,
selector and budget values. Four additional, separate `--recover-proof` commands
also exited zero; their readbacks equal the original records and request no
model operation. No original failed/uncertain estate was changed or discarded.
The runner stopped and waited for every child it owned.

The [manifest](task-worker-completion-fault-receipts/manifest.json) binds the
runner/relay/engine identities, per-case results and 64 raw receipts. Each case
includes actual provider bytes, relay observations, reservation, intervention,
withheld response, durable readback and independent recovery. Provider reports
name the configured provisional BF16 service; aliases do not attest weights.
The elapsed figures are observations of this controlled experiment, not model
quality, latency, cancellation or capacity benchmarks.

Exact command arrays, process results, full logs and estates remain in SWAG
`.swag/evidence/20261031/worker-completion-faults/`. Commands use only supplied
build/profile/task/credential paths. Private credentials, session tokens and
installed estates are excluded from publication. Selected receipts were checked
against the actual private key and credential-header fields before copying.
Do not normalize these raw JSON bytes after hashing; CRLF is part of the capture.

## Checks, observations and remaining work

The harness's seven focused tests, Python compilation, navigation, documentation
and inventory checks passed before its clean commit. This evidence increment
checks copied/staged byte hashes, exact cross-result equality, source identities,
generated navigation, documentation policy and execution inventory. Rust suites
and new model calls are not needed for receipt publication; candidate CI remains
required before landing. Whitespace checks preserve raw captures' CRLF while
checking authored code and Markdown normally.

Navigation generation/check passed with 862 nodes and 3,111 edges; documentation
policy passed with 863 statuses and 861 classified coordinates; execution-inventory
generation/check passed with 2,305 paths. All 64 copied receipt hashes and the four
independent recovery equalities were verified before staging.

An initial local publication assertion assumed an absent `worker_output` key
was serialized as null. The actual contract omits it. The corrected assertion
accepts absence as no output; already copied files were verified byte-equal
before finishing the copy. No engine, response validator or live oracle changed,
and no model trial was rerun to obtain success.

Checklist: owner/brief/source bound; actual intervention and before/after oracle
observed; complete changed-record/diff review; secrets excluded; raw byte and
identity checks; independent read-only recovery; omitted checks and limits
stated. No roadmap or POAM item closes. PR229 remains draft behind prerequisite
landing and then its own full candidate/protected pipeline.

These results verify withholding and retained state, not usable reconciliation.
Stop and lost delivery remain uncertain even when this external test relay
observed provider completion. That external observation cannot clear an engine
lease or authorize retry. A typed engine-owned reconciliation path is still
required, along with the complete ordinary-edit workflow, governed project
effects, RROFLOW correlation, independently reviewed learning, model qualification
and native/container release. The goal remains incomplete.
