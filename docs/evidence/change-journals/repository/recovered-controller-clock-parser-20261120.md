# Recovered controller, installation clock and parser behavior

**Status:** active recovery evidence; complete historical absorption remains unqualified
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/recovered-controller-clock-parser-20261120`
**Owner:** [POAM-011](../../../poam/rrflow-1.0-alpha/poam-011.md), [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), [Gate G](../../../roadmap/rrflow-1.0/gate-g.md) and [Gate H](../../../roadmap/rrflow-1.0/gate-h.md)

## Brief and effect boundary

[PR452](https://github.com/rrflow/rrflow-development/pull/452) carries the
brief written before repository edits. Baseline main is
`fe8499750de12bd4f348d8d7ee803bc48b151343`; opening revision is
`6c3015cf57e4894efd12614b5ce1ea3bdcdaabd8`. This is the same owner-directed
recovery session as the [initial inventory](historical-pr-recovery-20261120.md)
and [lab/database reconciliation](recovered-lab-and-database-paths-20261120.md).

This package creates this evidence record, regenerates the repository journal
index and adds its POAM-011 evidence link. It changes no implementation,
contract, dependency, prior journal, product version or roadmap status. Its
advancement is recoverable context for the named comparisons, not a new
engine capability. The failure oracle is an unexplained source loss, changed
object or failing behavior check; that blocks the corresponding absorption
claim. Repository contracts and source suffice for this comparison; no new
upstream adaptation, research claim, telemetry or benchmark is introduced.

At the baseline, root PR448 and PR451 have passed candidate and merge-group
CI. Both development repositories reported zero open PRs and zero open
issues. The root source of PR448, `23573d30726fd9ef10ea75acdcd63d389c4b9627`,
is an ancestor of baseline main and all 14 changed paths are byte-identical
there. Its complete-history bundle verifies and its local and development
remote branches are deleted. This is Git reconciliation evidence, not proof
that every historical capability survived.

## Recovered task controller and worker

The source input is PR231, `8d47500869329f1a5604821138f1a6c8efc349a5`.
The comparison against `e2daeb9bbfb105d19f6ae0bfde3e1c9e5ec1a925` accounts
for 288 changed paths: 216 exact, 71 changed and one absent at its old path.
The absent `rrd-contract/tests/public_contract.rs` became the modular
`tests/public_contract/flight_and_tasks.rs`; its security-action cardinality
assertion progressed from 55 to the current 59. These counts classify paths;
they do not certify 288 reviewed files or complete behavioral absorption.

| Recovered responsibility | Current destination and observed behavior |
|---|---|
| Task worker execution | `rrd-engine/src/engine/endpoint_worker/{execute,packet,support}.rs` retains selected configuration/allowance/task identities, capacity reservation, bounded execution, cancellation, uncertain outcomes, completion admission and retained draft output. Current source capture also revalidates the selected source and admission identity. |
| Attempt completion and resume | `reasoning_budget/{finish,resume,public}.rs` retains read-stamp and attempt ownership, completion validation, elapsed charging, cancellation, bounded CAS and explicit resume. Source-sensitive readback and assessment support are later extensions. |
| Controller and model transport | `rrd-inference/src/endpoint_chat.rs` retains bounded request/response encoding, explicit transport/credential policy and closed choice admission. Current `PreparedEndpointController` dispatches chat or native System One selection through that boundary. This does not establish all binary/score/choice providers. |
| Rust client and public catalogue | Task-worker, task-budget-resume and attunement-job-list operations remain in the current client/catalogue modules. `examples/task_model_call.rs` uses the installed locator, credential and session with one explicit bounded attempt. |
| Installed fault evidence | `scripts/inference/installed_controller_proof.py`, `installed_task_resume_proof.py` and the fault relay remain proof drivers. Reading them or replaying relay tests is not a fresh live-provider result. |

Full current files read: engine `endpoint_worker.rs`, its `execute.rs`,
`packet.rs`, `support.rs`; `reasoning_budget/finish.rs`; inference
`endpoint_chat.rs`; client `examples/task_model_call.rs`; inference scripts
`installed_controller_fault_relay.py`, its test file,
`installed_controller_proof.py`, and `installed_task_resume_proof.py`.
Read complete old-to-current diffs, rather than every current line, for
`reasoning_budget/{resume,public}.rs`, `endpoint_proposal/capacity.rs`,
engine worker/resume/terminal-response tests, attunement discovery and tests,
contract discovery, and client catalogue dispatch. Read catalogue reasoning
lines 85-184, lifecycle 94-119 and client operation/attunement 1-35.
The large worker-test comparison retains restart, cancellation, uncertain
completion and native selection assertions; no live inference was invoked.

Read the complete current journals `task-worker-prerequisite-reconciliation`,
`task-worker-landed-main`, and `task-budget-resume-worker-candidate`, plus
`lfg-swag.md` lines 1-180. Other historical PR231 journal bodies, generated
projections and raw provider receipts have not all received a fresh full-file
review. Byte-identical client files and matching added lines remain inventory
rather than a substitute for that review. The no-source fingerprint test and
capacity deserialization behavior were observed, not rewritten in this audit.

## Installation clock stash sequence

Inputs retained by the initial recovery bundle are
`5a3e0f4aa0a69affb92509219d5f4a28be68875a`,
`04182088ee3f3f6faee00bd2b976e57e321ab6bf` and
`c0fcd87de0ad08d8344a4142ecae934b93acd8f4`.
The sequence contains 53 tracked paths. The intermediate version refines
anchor naming and preflight; the last changes the journal quotation. Their
third parents contain no additional files.

The installation clock remains engine-owned. Current `clock.rs` keeps typed
observation failures and replaces the old panic assumptions with errors.
`installation/admission.rs` observes new installation time and assesses the
recorded anchor before replay effects. `installation/storage.rs` checks the
anchor when reopening an installed result. `installation/open.rs` performs
anchored preflight before acquiring the installed storage writer. CLI apply
uses the engine boundary rather than injecting a caller clock. Verification
reports observation, anchor and rollback evidence.

The old v2 configuration/profile asset paths now resolve to the canonical v3
engram configuration/profile and fixture paths. The current digest includes
the clock and attunement-follow policy; the concrete missing-follow-digest
and unsealed-plan failures discovered during this recovery were repaired by
[PR447](https://github.com/rrflow/rrflow-development/pull/447). Old assets are
not restored as an additional implementation.

Read the full current clock module and the complete 358-line
`gate-d/d-01-engine-owned-installation-clock.md`. Read complete stash patches
for CLI installed/main, engine exports, engine-authority tests and deployment
configuration. Current source ranges inspected: installation admission 1-90,
storage 294-335, open 1-85 and 355-end, profile 142-167; deployment
configuration 1-100 and 205-249; CLI installed 1-130, 175-230 and 525-603,
with apply and readiness call-site inspection. The comparison and paths are
bound to `9d4125a76c63203e2b73208e72900054b3164ff6`; PR448 changes none of
those source files.

Current installed lifecycle 1-190, 275-307 and 610-700, support
`running_engine.rs` 1-120 and `process.rs` 365-420 were read. The old direct
abrupt-kill helper became an owned-process cancellation/termination helper;
the old explicit nonzero-exit assertion and direct preview-clock assertion
are not literally present in those current locations. Therefore this record
does not claim assertion-for-assertion preservation or power-loss/OS clock
qualification. The owned process monitor's kill/wait path was inspected;
separate crash-test evidence remains separately owned. SDK fixture and other
clock adaptation files are not all fully reread here.

## Parser stash sequence and canonical publication

Inputs are `a105804f84e80858b86050009dfbcbd606f34894`,
`c5491c0613d419439ad21ca91856fdb9c1f1337e` and
`c53cf21040ec500c280b5d0e476311ec6da31117`. The first two contain 39 tracked
paths and differ in the journal; the last contains 41 tracked paths and three
third-parent files: compute worker, its tests and CLI parser-worker tests.
Those are retained recovery inputs, not branches to blindly merge over the
current parser.

All old added authored Rust lines were compared with the current modules,
including module splits and estate-to-engram naming. This mechanical scan
found the selected-source reader, inventory record publication, CLI worker
entry, installed test additions and exports in current code. Remaining
nonidentical lines were traced to visibility/module changes, shared parse
policy validation, updated contract identity and the deliberate replacement
of raw `SyntaxNode` proposals by `SourceFacts`. The complete old-to-current
parser and parser-test diffs were read. Original digest, language, encoding,
node/depth/input/output and cancellation checks have current test oracles.
Raw syntax-node JSON is not preserved as another public API. In particular,
the old per-node error/missing ranges are not equivalent to the current
aggregate error count. This review does not establish a canonical consumer
requiring those old fields or certify that every syntax-level capability is
retained.

Full current files read at `9d4125a76c63203e2b73208e72900054b3164ff6`:
`rrd-attunement/src/{parse,facts}.rs`; engine
`attunement/parse.rs`, `parse/{worker,step,publish,records}.rs`,
`parse/records/{facts,mutations,tests}.rs`; engine
`tests/attunement/parse_policy.rs`; CLI `tests/parser_worker.rs`,
`tests/installed_lifecycle/main.rs` and `replan.rs`.
Also read contract `attunement/source_parse.rs` in full and installation
profile lines 195-255 on the baseline (the compared files did not change).

The current source path binds bytes to accepted Inventory and enforces the
parse policy. Workers receive supplied bytes under a native supervisor;
unavailable supervisors do not silently activate an unsupervised production
worker. The durable step measures the physical batch, commits source facts
with the job transition and receipt, and renews unfinished work. Facts derive
names/text from engine-verified bytes. Content/parser/chunk identity selects
reusable work; edits and departures retire old facts. Parsed fact indexes
are maintained through the current storage/query boundary.

[PR445](https://github.com/rrflow/rrflow-development/pull/445) already repaired
the recovered worker's partial-header EOF acceptance. Its boundary-shutdown
and fragmented/interrupted-read tests remain. Fresh parser results below
exercise that repair without restoring the obsolete worker wire shape.
The installed lifecycle test proves Inventory query readback after process
restart and a native Parse checkpoint. It does not independently query parsed
facts after that final Parse and another process restart; engine tests cover
the parsed-fact restart behavior. Those evidence levels remain distinct.

## Commands, results and evidence identity

Local logs are retained under
`D:/.usr/Temp/rrflow-selective-recall-audit-20261120`; Cargo target is its
`target` directory, with four build jobs. This is the existing checkout's
private build directory, not a shared target across candidate worktrees.

| Revision and command | Observed result / log |
|---|---|
| `e2daeb9bbfb105d19f6ae0bfde3e1c9e5ec1a925`: `cargo test --locked -p rrd-client --test controller_proposal --test generated_catalogue --test task_effort --test transport_faults --test rrflowdb_local` | 19 passed; `pr231-client-tests.log` |
| Same: `cargo test --locked -p rrd-inference` | 28 passed, no ignored tests; some optional targets contain zero tests; `pr231-inference-tests.log` |
| Same: `cargo test --locked -p rrd-contract --test endpoint_worker --test worker_response_identity --test attunement_discovery` | 13 passed; `pr231-contract-tests.log` |
| `9d4125a76c63203e2b73208e72900054b3164ff6`: `python -m unittest test_installed_controller_fault_relay` from `scripts/inference` | 14 passed; `pr231-relay-tests.log` |
| Same: `cargo test --locked -p rrd-attunement --test parse --test facts --test worker` | 27 passed (7 parse, 11 facts, 9 worker); `parser-recovery-compute-tests.log` |
| Same: `cargo test --locked -p rrflow-cli --test parser_worker` | 3 actual-executable tests passed; `parser-recovery-primary-tests.log` |
| Same: `cargo test --locked -p rrflow-cli --test installed_lifecycle plans::installed_engine_plans_applies_serves_discovers_authenticates_and_reopens -- --exact --nocapture` | 1 passed, 14 filtered, 17.67 seconds; `parser-recovery-installed-test.log` |

The last installed executable SHA-256 was
`2bf51faedc784687742ca596d0074ce524a9ed7bdbeb6a728928b3e1a4f84612`.
Its real storage-history ledger resolved three present logical calls and one
absent call, with zero violations. These are small Windows fixtures, not a
capacity or multi-platform qualification. No live external model, cloud sync,
Connectome visual comparison, noisy-neighbor isolation or full inherited
capability campaign ran for this package. The combined engine result
(591 passed, 12 existing ignored) and standalone database process result are
already recorded by the [cold-load journal](../gate-e/cold-load-observation-time-20261120.md),
under their actual candidate identities; they are not rerun or relabeled here.

## Checklist, remaining work and landing

The owners, current implementation, exact baseline and existing diff were
inspected before this evidence edit; the PR brief preceded the edit. Source
adaptation and behavioral destinations are above. The changed record, index
and POAM link were reread in full; `git diff --check` passed. No new runtime
defect was established by this bounded continuation; earlier observed defects
remain linked to their actual repair PRs. Read-only searches that used moved
paths or PowerShell globs were corrected; truncated output was reread in
bounded slices before a full-file claim.

`python scripts/knowledge/render_navigation.py` passed: 21 indexes, 1,286
nodes, 4,535 edges, one index updated. `python scripts/ci/check_documentation.py`
passed: 1,289 statuses and 1,285 classified coordinates. Exact-head candidate
and merge-group CI remain pending at authoring; the PR is the authoritative
landing record.
No roadmap or POAM status advances. PR231's unreviewed historical records,
remaining clock adaptation files, ancestor-only PR comparisons and most of
Connectome PR10 still prevent a complete historical-absorption claim. The
owner-requested feature parity, UI/cloud and joined reasoning-engine gaps
remain open independently of the now-reconciled active branch backlog.
