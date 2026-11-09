# Installed worker interruption acceptance harness

**Status:** active harness implementation; installed interruption trial pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-interruption-harness`
**Owner:** G-01b/G-02a / POAM-009; PR229; active SWAG session

## Change checklist and boundary

The pre-edit [PR229 brief](https://github.com/rrflow/rrflow-development/pull/229)
starts at clean `8b9ce1fb1ac9322bed1c07dee526ff67e8331ab1`. The existing runner
could prove accepted worker output and restart but rejected worker fault modes.
It now accepts explicit `--worker --worker-interrupt` for one HTTP proof: hold
the real provider's worker response before the engine receives it, kill only
the runner-owned engine child and reopen the same installed estate without a
provider credential mapping or another model operation.

The existing `FaultRelay` gains a bounded `hold_call` parameter, defaulting to
its previous first-call behavior. In this mode only call two is held and dropped;
call one is the unmodified selector exchange. The actual selected binding must
match the explicitly relayed binding; an abstention/different choice stops the
trial rather than inventing a selection. Both response limits come from their
existing per-binding policies. Review corrected an initial top-level worker
response-limit lookup before execution; no bound or validator was relaxed.

Before kill, the harness requires the actual successful provider-response
barrier, exactly two calls, a reserved worker without output, two spent steps,
the original budget deadline and unchanged task. It records the owned PID,
actual exit and caller transport loss. Credential-free reopen must retain the
same selector attempt, reserved worker, budget and task. Neither a successful
response nor an unexpected assertion is accepted as the expected transport loss.
Existing selector faults, native baseline, admission and read-only recovery
modes remain available. Incompatible interruption combinations reject before
creating an output directory.

Reviewed the full proof runner and existing relay, the prior installed native
receipts, task reserve/completion and shared-capacity boundaries, public budget
uncertainty contract and POAM-009. Authored runner/relay/test changes and the
existing CI insertion point were reread with the full diff. Changed paths are
the two inference proof files, their new focused test, one unconditional test
command in the existing CI verification lane, this journal and its generated
index. No engine/runtime API, model policy, dataset, dependency or parallel
execution authority changes.

## Local validation

`python scripts/inference/test_installed_controller_fault_relay.py -v` passes
all four cases: unchanged default first-call hold with exact byte delivery,
second-call hold/drop with first-call delivery, invalid ordinal denial and
incompatible CLI argument rejection without filesystem effects. These are
explicit local HTTP correctness bytes, not model output or training examples.
The log SHA-256 is
`2e56b9ffad8cbbe4269907d1df457cec35dee8ffd4022807e9d8b3944ee20adb`.

`python -m py_compile` passes for the runner, relay and test.
`python scripts/ci/check_workflow.py` and `git diff --check` pass. The new test
is registered beside the existing inference evidence-tooling tests; no second
pipeline, candidate-admission change or reduced check is introduced. No local
test failure occurred. Raw commands/logs and the pre-edit brief are retained
under `D:/projects/swag/.swag/evidence/20261031/worker-interruption/`.

`python scripts/ci/build_execution_inventory.py --check` passes with 2,221
path records. `python scripts/ci/check_documentation.py` passes with 860
document statuses and 858 classified coordinates. Generated navigation is
current and the final whitespace check passes. Rust suites were not repeated:
this increment changes only the Python proof tooling, its CI registration and
evidence, with no engine implementation change.

## Live acceptance and limits

The configured BF16 endpoint's authenticated model-list request returned 200
and listed the supplied service alias. This is reachability, not weight
attestation or inference evidence. The planned trial uses the existing
provenance-bound `c75c1896` engine, frozen contract helper, explicit short-worker
profile and captured real PR task from the prior native experiment. Freeze this
committed runner/relay and record their hashes before the new provider calls.
Then independently run the existing read-only recovery against the preserved
estate. A failed live oracle remains a failure; do not substitute another estate
or weaken validation to obtain a success.

No new protocol research, algorithm adaptation, production trace or resource
measurement is needed for a controlled installed-process interruption.
Stop before kill unless the provider barrier and reservation are proven, and
stop on unexpected acceptance, lost state, extra calls or credential exposure.
The original uncertainty records remain intact: this does not reconcile them,
clear capacity, reset effort/deadlines, retry work or prove remote termination.
It covers process loss after provider completion and before engine acceptance;
broader fault qualification, UI/project effects, RROFLOW, reviewed learning,
model quality and public release remain open. No roadmap or POA&M status closes.
