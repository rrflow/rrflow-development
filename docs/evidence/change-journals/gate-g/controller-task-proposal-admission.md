# Engine-admitted experimental controller operation

**Status:** active implementation-stage evidence; installed qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-task-proposal-admission`
**Owner:** G-01a/G-01b and G-02/G-05 task-attempt prerequisites; OBJ-12 and POAM-009 remain open
**Pull request:** [206](https://github.com/rrflow/rrflow-development/pull/206), dependent on [205](https://github.com/rrflow/rrflow-development/pull/205)

## Brief, baseline and canonical ownership

Driver: SWAG controller session. Baseline `a92127e337e4745ca133d30ca13fe09b5ef1ffea`.
The existing PR brief precedes implementation and covers dedicated authorization,
configured native transport/credentials, bounded concurrency, durable completion,
the executable public handler and installed-process proof in this same draft.
Prior journals and actual provider captures remain unchanged. No gate advances.

Reviewed README/AGENTS, Gate G, the LFG/SWAG owner, change-authoring and CI owners;
endpoint policy/packet/receipt contracts, native chat transport, task reads,
session/security authorization, engine composition and invocation, budget
reservation/completion/control persistence, server dispatch/envelopes, installed
serve and build provenance, contract catalogue/reference/schema export and SDK
generators. Existing real-task projection and completion tests characterize the
ledger; they do not establish a model answer to a newly constructed packet.

## Behavior and failure oracle

Previously there was no engine entry or installed credential composition for
the native chat adapter. `propose_task_endpoint` now requires initialized
security and three explicit actions, derives a bounded packet from the actual
open task and installed policy, guards source/configuration/session/security
bytes, reserves existing task effort, and invokes the configured adapter once.
Model, endpoint, prompt, candidate facts, generation settings and limits come
from configuration. Experimental network and unverified-identity permissions
are explicit; service aliases and declared capabilities remain unqualified.

The canonical security repository exposes a validated exact-byte snapshot for
the existing atomic control CAS. Task reads expose an internal bounded stamped
record helper. These extend their existing owners, not alternate authority.

An engine-owned capacity index is committed in the same transaction as budget
and attempt. Aggregate and per-binding limits admit before dispatch. Known
completion releases capacity atomically; uncertain execution retains it through
reopen. This is admission state, not a second task lifecycle or retry scheduler.
Completion checks run on errors as well as successful output. A failed cleanup
read leaves the attempt unresolved instead of releasing unknown capacity.
Authority/input invalidation does not erase remote uncertainty or spent effort.

Trusted process composition installs one immutable bounded credential map.
The CLI accepts a bounded explicit reference-to-environment mapping; no arbitrary
runtime resolver, environment scan, public secret field or logged secret is added.
Missing credentials deny before task reservation. Public dispatch uses the same
authenticated engine operation and dedicated `EndpointPropose` permission.
The contract catalogue contains 63 executable operations. The offline example
derives proposal request digests using the same typed Rust contract.

## Validation and surfaced failures

The first compile exposed a private budget-key helper and the wrong error mapper
for a contract identifier. Both were corrected at their owners. The focused
reasoning-budget suite then passed all 26 cases, including atomic capacity
reservation, conflict rollback, known-failure cleanup, uncertain reopen retention,
and denial before reservation when security is uninitialized. Controlled failure
classifications and recombined retained tuples remain test inputs only; no model
answer, human label, consent, training record or quality metric is fabricated.

All-target `cargo check --locked -p rrd-engine -p rrd-server -p rrflow-cli` passed.
The first OpenAPI export rejected missing operation-reference text, which was
added with the actual handler semantics. A Windows text-decoding failure interrupted
authored digest updates; explicit UTF-8 resumed unfinished updates. Incidental
CRLF changes were normalized to the existing LF bytes. Generators passed for
TypeScript, Python, Go, Java and .NET plus signal projections. OpenAPI SHA-256:
`53e10f4dd635a203e6966ad8bd81be71d6e365277c089126ca67d8f6cc8e4e9f`.
The first TypeScript test run correctly rejected the old 62-operation assertion;
all five SDK count assertions now reflect the executable 63-operation catalogue.

The broad six-package test command subsequently exposed an old 53-action
assertion and then a stale catalogue digest in the CLI's existing storage-validator
fixture. The action assertion now expects 54. The owning
`refresh_storage_contract` example changed only the current test fixture's
catalogue digest and dependent hashes; it explicitly refuses native evidence.
No provider capture, workload value or measured result changed. The broad run
completed the contract, inference, security, engine and server suites before the
CLI fixture failure. The corrected CLI/evaluator run passed 102 tests with eight
declared native/campaign/subprocess skips. Three engine skips remain assigned to
native ownership/child-process CI and are not counted as passes.

The final focused engine suite passed all 27 cases, including independent denial
of each of the three required permissions using actual engine authorization and
an engine-issued test token. Strict all-target Clippy passed for contract,
inference, security, engine, server and CLI. Remaining documentation tests for
those five library packages passed. TypeScript generation/typecheck/lint and six
client tests passed; Python's five client tests and Go's tests passed. All five
SDK generation checks and generated-surface parity passed. Java/.NET runtime
tests remain for candidate CI; their generators and parity were checked locally.

Documentation initially rejected the journal's missing active-status marker,
then its stale generated index; both were corrected using the navigation owner.
Documentation, navigation, inventory, workflow and whitespace checks passed.
An observation handle disappeared during the CLI rerun; process inspection
confirmed its Cargo/Python processes had stopped without a terminal test result.
Only that unfinished validation resumed, retaining the partial log. Local logs
are retained under the operator's
`.swag/evidence/20261029/controller-operation-plan/` directory.
An authenticated operator preflight reached the existing private BF16 service;
this is explicitly not an engine-dispatch or loaded-weight attestation receipt.

`scripts/inference/installed_controller_proof.py` is the repository-owned next
proof driver. All model/profile/task inputs are supplied files. It uses a
source-identified copied native binary, a new isolated estate, real public
authorization/configuration/task operations, one actual endpoint attempt and
restart/readback/replay checks. Task context is an explicitly recorded bounded
projection of an actual PR body. Credentials/session tokens are never retained
in its receipts. It records owned process identities and pre-call intent so an
interrupted observation is recovered from the estate rather than blindly retried.
Only syntax is checked at this stage; the driver has not yet supplied live proof.

Key validation commands (from the repository root unless stated):

```text
cargo test --locked -p rrd-contract -p rrd-inference -p rrd-security -p rrd-engine -p rrd-server -p rrflow-cli -j 2
cargo run --locked -p rrflow-eval --example refresh_storage_contract
cargo test --locked -p rrflow-cli -p rrflow-eval -j 2
cargo test --locked -p rrd-engine --lib reasoning_budget
cargo test --locked --doc -p rrd-contract -p rrd-inference -p rrd-security -p rrd-engine -p rrd-server
cargo clippy --locked -p rrd-contract -p rrd-inference -p rrd-security -p rrd-engine -p rrd-server -p rrflow-cli --all-targets -- -D warnings
cargo fmt --all -- --check
python scripts/ci/check_generated_surfaces.py
python scripts/knowledge/render_navigation.py --check
python scripts/ci/check_documentation.py
python scripts/ci/build_execution_inventory.py --check
python scripts/ci/check_workflow.py
git diff --cached --check
```

SDK-local checks used the installed pinned pnpm module for `generate:check`,
`typecheck`, `lint`, and `test`; `uv run --frozen pytest` and
`uv run --frozen python scripts/generate.py --check` in `sdks/python`; installed
Go 1.26.9 for `go test ./...` and `go run ./cmd/generate --check` in `sdks/go`;
and `python sdks/{java,dotnet}/scripts/generate.py --check` at the root.

## Limits and next evidence

The PR stays draft until installed authenticated BF16 execution, durable readback
after process restart, replay/denial/Stop/uncertainty evidence and required CI
exist. No new GPU training occurred. Uncertain remote calls require explicit
future reconciliation; this stage does not promise remote termination or free
capacity automatically. Existing terminal transport failures retain task stop
semantics. This bounded task-text operation does not implement repository-source
dependency discovery, ordinary-edit successor plans, selected-worker effects,
Connectome/RROFLOW integration, independent review, qualified learning, scale-out
image delivery or measured quantization. The full original goal remains active.

Research/resource decision: use existing repository transaction, authorization
and transport owners; no new vendor algorithm or performance claim. Observe actual
usage/digests and configured bounds, never estimated scores. Stop dependent calls
when authority, identity, credentials, capacity or completion cannot be established.
