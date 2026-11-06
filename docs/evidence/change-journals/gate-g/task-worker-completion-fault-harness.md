# Installed worker completion-fault proof

**Status:** active acceptance harness; actual provider qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-completion-fault-harness`
**Owner:** G-01b/G-02a / POAM-009; PR229; active SWAG session

## Boundary, baseline and brief

Start: clean `e35283303f4c5823c612686d32fb6bee1ac368ac`, tree
`d773755431c8c54029b9d407afc2be53bd665d11`. The pre-edit PR229 description
binds this remaining installed worker acceptance scope. Existing engine tests
cover completion invalidation, but the installed proof rejected worker fault
modes. Its first oracle, `--worker --fault-case configuration --output
target/worker-fault-before-edit-probe`, reported the incompatible-mode error
before creating an estate or calling a provider.

The existing runner now admits `--worker --fault-case` for configuration,
session, Stop and disconnected delivery. It relays the actual selector response,
holds request two's actual provider response, and verifies the engine's reserved
worker and original deadline before one controlled intervention. The configuration
case applies a public revision; session closes the dispatch session; Stop uses
the observed budget digest; disconnect drops delivery. Output must be withheld.
The runner asserts unchanged selector/task, two spent steps, unchanged deadline,
nondecreasing charged time, expected invalidated/stopped state, and exact
credential-free reopen readback. It sends no successor model request.

The actual selected binding must match the explicitly relayed binding. An
abstention, malformed response, timeout or different choice fails the trial;
the runner never supplies a replacement choice. Stop/disconnect uncertainty
remains retained. No attempt or shared capacity is cleared. The existing
interruption, selector-only, native, admission-only and recovery paths remain;
incompatible combinations fail before filesystem/provider effects.

## Review, implementation and checks

Read AGENTS/README, Gate G/POAM-009 and the change-authoring procedure. Read the
complete installed proof runner, relay and focused tests; inspect worker
completion admission, transport cancellation/decoding and the engine's
changed-input/authority/Stop/lost-response characterization. Prior installed
interruption evidence supplies the retained-uncertainty boundary. The changed
runner/test sections and complete diff were reread after editing.

Changed `scripts/inference/installed_controller_proof.py` and the existing
`test_installed_controller_fault_relay.py`. The relay itself is unchanged.
Its existing unconditional CI verification command discovers the new checks.
No runtime, contract, SDK, schema, model policy or training code changed.

`python -m unittest discover -s scripts/inference -p
test_installed_controller_fault_relay.py -v` passed seven tests. They cover
first/second-call byte preservation, held-response delivery/drop, invalid relay
ordinals, incompatible modes without filesystem effects and required real input
arguments. Local opaque HTTP fixture bytes are explicitly transport tests, not
model observations. `python -m py_compile` for the runner/relay/tests and
`git diff --check` passed. Navigation generation/check passed with 861 nodes
and 3,109 edges; documentation policy passed with 862 statuses and 860
classified coordinates; execution-inventory generation/check passed with
2,239 paths before committing the clean trial source.

No external algorithm or new protocol research is needed: this proves the
existing public engine boundary. Request/response digests, bounded raw bytes
and child-process exit records supply debugging evidence; no default prompt
logging or resource benchmark is added. No Rust suite is rerun for Python-only
proof changes; the exact installed engine and later candidate CI retain their
own qualification obligations.

## Required next evidence and limits

Freeze the clean committed runner and use the provenance-bound `e3528330`
engine, explicit short-worker profile and captured real PR task. Run each
distinct intervention once, retain any failure, then independently recover
that estate without provider credentials. Publish actual outcomes separately;
these seven local tests cannot establish model-backed completion handling.

Checklist: owner and prerequisite bound; before/after oracle recorded; exact
scope reviewed; no duplicated execution authority; first-party proof stays in
this repository; tests and omitted checks distinguished; immutable journal and
generated navigation supplied. No roadmap/POAM status change. PR229 remains
draft pending local acceptance and dependency landing, then full candidate CI.
Explicit uncertainty reconciliation, full ordinary-edit workflow, project
effects, RROFLOW correlation, reviewed learning and release qualification stay
open. An HTTP response or reopened UI alone never authorizes retry or proves
provider termination.
