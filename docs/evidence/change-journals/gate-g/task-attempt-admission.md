# Shared task-attempt admission at the existing budget owner

**Status:** active internal prerequisite; controller completion and execution remain missing
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-attempt-admission`
**Owner:** G-01a/G-01b, POAM-009; OBJ-12 prerequisite
**Pull request:** [203](https://github.com/rrflow/rrflow-development/pull/203), after [201](https://github.com/rrflow/rrflow-development/pull/201) and its dependencies

## Baseline, brief and scope

Public main was `44cbfa7346fe458d84daead188d33cbac7392efd`. An isolated worktree
ordinarily merged PR201 `2c3f6bff9db9c87731899ebe4640bd3008324799`, producing
opening head `6aac41672fd1b98d880ccd0804fb836666573efc`. The SWAG controller
working session opened the draft PR with its change brief before implementation.
Dependency landing, refreshed full candidate CI and protected merge remain
required; no skipped draft check is acceptance evidence.

Before this change, the durable reservation function consumed an embedding
request directly. Adding a controller-specific ledger would duplicate task
accounting. The embedding entry now derives its own request identity and uses
one common internal admission path. `TaskAttemptAdmission` carries scope, task,
attempt, request digest, cancellation and the monotonic start captured before
identity preparation. The deferred effect-specific repetition calculation runs
only after replay, prior-stop and budget checks permit it. Request/equivalence
digests are validated before publication.

The existing budget and attempt keys, serialized records, runtime/control CAS,
initial deadline, step count, active-attempt fence and completion behavior remain
at `engine/reasoning_budget.rs`. The embedding result type remains unchanged.
There is no serializable admission request, public operation, new authorization,
provider dispatch, registry or second budget. Future effect-specific callers
must authorize and bind their complete relevant identity before reserving.

## Inspection and replacement map

Read the existing reservation/completion and cancellation guard, persistence CAS,
public configure/execute/read/Stop, repetition checks and embedding identity,
task-open validation, native transport observation and relevant contract types.
Followed README/AGENTS, change-authoring, Gate G and the executable dependency
spine, and the endpoint-selection input/effect owner. The two changed Rust files
and resulting diff were reviewed at the changed behavior; no whole-repository
audit is claimed.

The replacement is confined to `reserve_task_attempt`: effect-specific digest
preparation delegates to `reserve_admitted_task_attempt` at the same owner. The
original 14 reasoning-budget tests are the unchanged characterization oracle.
Added two tests to the existing engine test module, updated the inference
reference and this journal, and regenerated discovery. No source was deleted,
no public type/schema changed and no migration or alternate behavior lane exists.

No external dependency or algorithm is introduced; repository source suffices
for this extraction. No new runtime trace, GPU call, latency or resource claim
is relevant to the reservation invariant. Stop conditions are changed embedding
behavior, reset budgets, changed replay/storage identity, duplicate authority,
invented observations or bypassed checks.

## Retained-input oracle

The new tests read the actual PR198 maintenance task and captured controller request
under `rrd-contract/tests/fixtures/endpoint-selection-observed/available/`.
Both files are checked against their retained SHA-256 index. A fresh embedded
engine/store opens that real task text; its test estate/session identity does not
claim to be the captured deployment. No provider is installed or called.

A reservation survives close/reopen unchanged. Repeating its identity reports
the recorded unresolved attempt; altered bytes under the same attempt conflict.
An embedding attempt sees that same task's active reservation and cannot acquire
another step. Malformed request identity leaves no budget; malformed or failed
equivalence calculation leaves a configured budget unchanged. Valid equivalence
reserves one step and records the fingerprint without extending its deadline.
Faults are explicitly in-memory validation probes, not captured provider output,
training rows or model-quality evidence.

## Validation and remaining work

The unmodified baseline passed all 14 focused budget tests. The first common
admission implementation passed all 16. Review then retained the pre-preparation
monotonic start and added task-file hash verification. The first owning-suite
compile exposed a missing `Instant` test import; that failure is retained and
the import was corrected. Local commands use a shared Cargo output cache, with the changed
worktree as source; no prebuilt binary is claimed as validation of changed code.

No installed process kill, remote provider execution, source snapshot/replan,
typed controller completion or end-to-end user workflow is established here.
No routing labels, training, independent review or 29 GB measurement were added.
The active controller goal, G-01b, POAM-009 and OBJ-12 remain open. Controller
completion/replay and task/source/capability/credential authorization must compose
this same budget before the adapter can become an admitted runtime operation.

Checklist: pre-edit brief; canonical-owner extraction and characterization;
retained real input with no invented result; focused then owning checks; source
and diff review; failures and limits retained; immutable journal and generated
discovery. No roadmap or qualification status change.

Final validation: `cargo test --locked -p rrd-engine -j 2` passed 306 tests,
including all 16 focused budget cases. Three declared cases remain ignored in
this invocation: attunement and installation subprocess child fixtures, and the
privileged foreign-owner test that native CI runs explicitly. Strict
`cargo clippy --locked -p rrd-engine --all-targets -j 2 -- -D warnings` passed.
Both changed Rust files pass rustfmt; staged whitespace passes. Documentation
policy passed with 740 statuses and 738 classified coordinates; workflow policy
passed. Navigation checks passed with 18 indexes, 739 nodes and 2,849 edges;
inventory generation/check passed with 1,968 records. No public schema or SDK
source changed, so no generated API update or cross-language rerun was needed.
The native/cross-platform and optional-feature candidate lanes remain required
before landing, and the exact post-commit push identity is recorded separately.
