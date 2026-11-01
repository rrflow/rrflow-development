# Task-bound controller proposal semantics

**Status:** active implementation-stage evidence; full operation incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-task-proposal-semantics`
**Owner:** G-01a/G-01b and G-02/G-05 task-attempt prerequisites; POAM-009 and OBJ-12 remain open
**Pull request:** [206](https://github.com/rrflow/rrflow-development/pull/206), dependent on [205](https://github.com/rrflow/rrflow-development/pull/205)

## Brief and inspected boundaries

Driver: SWAG controller working session. The pre-implementation PR brief binds
one complete experimental proposal operation, not a sequence of substitute
capabilities. This record covers its initial semantics stage, not completion.
Starting public main was `44cbfa7346fe458d84daead188d33cbac7392efd`; ordinary
merge of PR205 `169b010a714ae7ea09895b2f82504d07ea9ecf0f` produced opening head
`d02d8e460899a3ce360f8d8a51247072ed62390d`. No original journal or capture is
modified. PR196 subsequently landed; PR205's fresh candidate remains pending.

Reviewed README/AGENTS, Gate G and the executable dependency spine, change-authoring
and CI rules, inference reference, endpoint selection/policy/configuration types,
task records/reads, reasoning budget reservation/completion/public readback and
persistence, engine session/security authority, store control CAS, native chat,
installed CLI command/serve and their relevant tests. These expose the missing
engine operation, embedding-specific completion and absent installed credential
configuration. Existing core owners remain authoritative.

## Behavior and traceability

New `rrd-contract/src/endpoint_proposal.rs` defines bounded request, task projection,
packet, proposal and reported-token usage types. The request names expected task
and configuration identities and a finite read allowance; no endpoint or secret
is supplied. Packet checks bind the exact configured controller/allowance and
open-task projection. The projection excludes the task's creation replay key.
Receipt checks reject substituted task/attempt/configuration/packet identities,
invalid choice, failure HTTP status and inconsistent reported usage. The sole
purpose is experimental proposal, not worker execution or qualification.

`task.rs` exposes its existing scope validator only within the contract crate;
no task validation semantics change. `lib.rs` exports the new Rust types without
adding an operation or altering the existing generated SDK contract. Native
`endpoint_chat.rs` projects provider usage into the canonical type; its private
wire decoder still tolerates provider metadata. Unknown counts stay unknown.
Impossible partial totals and overflowing input/output sums now fail decoding.
Existing choice/network/response/credential behavior and retained captures remain.

The engine's existing reasoning-budget test module opens the retained actual
PR198 task in a fresh engine, reads its real task version, and tests changed/closed
task projection and request/receipt tuple substitution. Tuples combining captured
provider fields with this new test packet are explicitly in-memory consistency
probes: they are not persisted receipts, causal claims about those old calls,
human judgments, training rows or evaluation observations. The inference test
perturbs actual retained output to exercise partial-count failures. No new
provider call, fake response, reviewer identity, consent or quality label is added.

## Validation and failures

The first `cargo check --locked -p rrd-contract -j 2` failed because reusing
`ReadTask` exposed its required `max_storage_keys`. The proposal request now
carries and validates that bound explicitly. The corrected contract/inference
check with `rrd-inference/endpoint-chat` passed. The initial read of a guessed
fixture path failed without mutation; tests use indexed existing captures.
An accidental empty file outside the worktree was immediately removed; it is
not a source input or retained artifact.

`cargo test --locked -p rrd-engine engine::tests::reasoning_budget::proposal -j 2`
passed both new tests. Other engine tests were filtered, not counted as passes.
`cargo test --locked -p rrd-contract -p rrd-inference --features
rrd-inference/endpoint-chat -j 2` passed all 182 tests, with no failures or ignored
cases. This includes existing capture hash/provenance and native transport checks.
No GPU/training, new remote benchmark or installed engine call ran in this stage.
Compiler and test logs are retained under the operator's
`.swag/evidence/20261029/controller-operation-plan/` directory.

Strict all-target Clippy passed for contract, inference and engine with the
native chat feature. Generated surface parity passed: still 62 operations,
OpenAPI `c1c8c59f888c3c1425789feec81c008d3aa0cd0676f6155a526ccd96aca2c606`,
and unchanged signal catalogue/projections. The new Rust types are deliberately
not registered as an operation or advertised by generated clients in this stage.
Formatting, navigation generation/check, documentation policy, execution inventory
generation/check and workflow policy passed; staged whitespace checks passed.
The projection contains 1,972 current/generated/planned paths; documentation
validation covers 742 statuses and 740 classified coordinates. Full engine tests
were not repeated for this stage's test-only engine change; its two focused tests
passed and the full suite remains required for the completed operation candidate.

## Remaining implementation and acceptance

The same draft must still implement dedicated authorization, engine-built packet
admission, explicit experimental configuration, bounded private credential
resolution, binding concurrency, typed durable completion/readback and atomic
task/configuration/security freshness guards. Then add the actual public handler,
catalogue and generated clients together and prove one installed BF16 call with
Stop, uncertainty, replay and restart evidence. A valid tuple is not dispatch
authority; missing capability evidence cannot satisfy dependent hard requirements.

Source-change successor plans, worker effects, Connectome/RROFLOW integration,
independent routing review, qualified training, measured quantization and clean
native/image delivery remain the original goal. No gate, POAM or objective is
closed. Required candidate and protected merge-group CI are not replaced by these
local checks. Keep this package draft until the complete promised operation and
its acceptance evidence exist.

Research/source adaptation: no new vendor method, dependency or algorithm chosen.
Trace/resource/debug: existing bounded validators and retained byte identities;
no automatic prompt, response or secret logging; actual deployment measurements
deferred to the installed proof. Stop dependent effects if current admission or
atomic completion cannot be supported at the canonical owners.
