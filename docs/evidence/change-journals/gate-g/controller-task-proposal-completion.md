# Durable controller proposal completion

**Status:** active implementation-stage evidence; controller operation incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-task-proposal-completion`
**Owner:** G-01a/G-01b and G-02/G-05 task-attempt prerequisites; POAM-009 and OBJ-12 remain open
**Pull request:** [206](https://github.com/rrflow/rrflow-development/pull/206), dependent on [205](https://github.com/rrflow/rrflow-development/pull/205)

## Scope and baseline

Driver: SWAG controller working session. Baseline
`6e039ce8c36dd978af49f71e140281479809128b` contains the preceding typed-semantics
stage. The existing pre-implementation PR brief covers durable typed completion,
atomic freshness guards and generated projections in the same complete-operation
package. This stage keeps that scope; it does not substitute bookkeeping for the
promised installed controller operation. No prior journal or capture is rewritten.

Reinspected task budget reservation/completion, persistence and public readback;
the task-attempt contract, service error classification, task/session transitions,
store control CAS and existing reasoning-budget tests. Reviewed current generated
surface tooling and the SDK generators before changing the exposed read schema.
The relevant owners and initial broader source review remain in the PR brief and
[semantics-stage record](controller-task-proposal-semantics.md).

## Canonical change and before/after oracle

Previously completion only retained embeddings and every completion error
permanently stopped the budget. `finish_admitted_task_attempt` now uses a private
typed output at the same `AttemptRecord`/budget/control-runtime transaction. The
existing embedding entry delegates to it; embedding payload digests remain
unchanged. No second ledger, scheduler, provider registry or public output-injection
entry is added. The proposal writer remains engine-internal.

Public `TaskAttemptSnapshot` adds optional typed proposal and invalidation fields;
validation rejects two output kinds, misplaced invalidation, wrong attempt,
invalid output and mismatched digest. Readback also binds proposal task and scope.
`Invalidated` records changed inputs or authorization while retaining spent time,
reserved steps and original deadline. A fresh attempt needs fresh admission;
invalidation itself is not retry permission. Stop/deadline remain stronger.
Existing terminal failures keep their stop semantics. Endpoint failures can retain
an explicit uncertain remote outcome, with no automatic resubmission.

Completion obtains a current runtime read stamp and invokes its admission check.
Engine-owned control guards carry exact expected bytes. Existing control/runtime
CAS compares them atomically with the budget/attempt/event, using equal-byte
replacements that preserve the guarded authority's value. Duplicate keys and
resource limits remain store-enforced; serialized guard bytes count toward write
admission. Conflicts recheck completion authority without invoking a provider.
Time is reobserved after admission and on every retry. The future controller
operation must derive the relevant guards from actual authorization/configuration;
this generic primitive cannot establish permission by itself.

Five new tests reuse the prior real-task/provider projection helper in isolated
estates. They prove supplied typed-output persistence/reopen/replay, charged
invalidation followed by another bounded reservation, a real session closure
between guard read and commit, a real task closure between runtime read and commit,
and stopped/uncertain completion without accepted output. The session and task
cases require a first CAS conflict and second admission decision. Existing budget
tests remain. These supplied tuples test ledger behavior: they are not observations
of a provider answering the new packet and never enter a learning dataset. Reopen
within a test process is not installed-process crash qualification.

## Generated surfaces and failures

The readback schema changes, while the catalogue still contains 62 operations.
All five SDK generators and signal projections were regenerated from the contract.
Current authored citations use OpenAPI
`af8488aa64d477c1b2d6f7c705c5ce131a7b3b15e3f4d04755621dccc6cd2526`;
historical evidence retains its original digest. No model/provider/endpoint choice
is embedded in runtime code, and no new dependency or product version is added.

The initial engine check exposed the persistence helper's private visibility;
it was restricted to its parent module. Initial focused-test compilation exposed
missing engine helper/result imports; these were corrected. An orchestration
print failed on Windows console encoding after dependency installation succeeded;
generation resumed from the not-yet-run steps with escaped progress output. The
first parity check correctly rejected stale signal projections, then the owning
renderer regenerated them and parity passed. All failures/logs are retained in
the operator's `.swag/evidence/20261029/controller-operation-plan/` directory.
Strict Clippy subsequently identified the private output enum's large embedding
variant. Both output payloads now use boxed storage; serialized payloads and
their digest definitions are unchanged. The engine suite and strict lint are
rerun after that adjustment rather than inheriting the earlier success.

Focused reasoning-budget tests passed: 23, including the five new completion
tests. SDK generation/checks passed for TypeScript, Python, Go, Java and .NET.
TypeScript typecheck/lint and all six client tests passed. Generated parity passed
with the new digest and unchanged signal semantics. No GPU run, training data,
new model response, reviewer identity, consent or quality score was created.

The full `cargo test --locked -p rrd-contract -p rrd-inference -p rrd-engine
--features rrd-inference/endpoint-chat -j 2` passed 495 tests. Three declared
cases remain ignored by that invocation: attunement and installation subprocess
child fixtures, and foreign-owner validation explicitly run by native privileged
CI. No ignored case is counted as a pass. Subsequent test-only assertions also
check unchanged guarded bytes on successful publication and Stop taking
precedence over invalidation; production code is unchanged by those assertions.

After the enum storage correction, all 313 engine tests passed with the same
three declared skips. Strict all-target Clippy passed for contract, inference
and engine with the native chat feature. The strengthened focused suite passed
all 23 cases. Formatting, documentation policy (743 statuses, 741 classified
coordinates), navigation, inventory generation/check (1,974 paths), workflow
and staged whitespace checks passed. A failed journal patch anchor made no change
before the corrected result entry. All validation is local stage evidence;
completed-operation candidate CI and installed proof remain outstanding.

## Remaining work and gates

Dedicated controller authorization, explicit experimental-policy admission,
configured credential resolution, eligible packet construction, binding concurrency
and actual installed endpoint execution remain in this same draft. Add its public
handler/catalogue only with the executable admission path. Prove real Stop,
uncertain result, replay and process-restart behavior against the configured BF16
service; these bookkeeping tests cannot replace that evidence.

Project-source dependency discovery/successor plans, selected-worker effects,
Connectome/RROFLOW integration, independent outcome review, qualified learning,
native/image delivery and measured quantization remain the original goal. This
stage closes no gate or POAM. Required full candidate/protected merge-group CI
still applies to the completed package after dependencies land. No additional
algorithm/vendor research or performance qualification is claimed; the smallest
oracle is the actual engine's atomic state transition under controlled interleaving.
