# Worker candidate reconciliation with current main

**Status:** active local reconciliation evidence; candidate CI and installed proof pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-main-reconciliation-20261106`
**Owner:** OBJ-06, G-01b/G-02a and POAM-009; PR229

## Scope and failure oracle

[PR229](https://github.com/rrflow/rrflow-development/pull/229) was conflicted
against public development main. Its change brief was updated before editing.
This reconciliation starts at worker head
`5ddd7c2c03a335024ab111e56b8776fb93149520` and merges main
`1fd7f163de3448c694e09c916b328540303eab1c`, with merge base
`5075f49c73518712209cee58a75b6bba835aebcf`. An isolated worktree preserves the
earlier worker checkout and its installed receipts. Stop on an independently
advanced candidate, an unexplained semantic mismatch, or loss of authority.

The initial merge preview reported 20 conflicts. They comprise the canonical
OpenAPI digest, its generated SDK/reference projections, authored digest
citations, and the LFG/SWAG reference. Conflict regions were resolved without
replacing whole source files, preserving automatic merges. The combined
catalogue then regenerated all five SDK endpoint projections and seven signal
projections. The resulting 79-operation OpenAPI identity is
`4186a8f279fd213b592d54d5061717a8f27ce0a85aeba145aeda5ef24c025f2e`.

Main's request extension semantics remain intact: unknown endpoint-proposal
fields are ignored, never admitted as LFG evidence, credentials, endpoints or
candidate facts. The merged proposal characterization test verifies this.
Worker admission and readback remain distinct from selection and task success.
The reference preserves both the experimental worker boundary and main's new
extension explanation. A staged-object comparison verified all 92 main-only
paths are byte-identical to main. The other 22 paths overlap the candidate;
this is not an exhaustive audit of imported main code.

## Review, research and resource decisions

Read the repository instructions and prior reconciliation and float-refresh
journals. Reviewed relevant README, alpha/roadmap/POAM and change-authoring
sections, the complete authored conflict hunks, the automatically merged
proposal test, exporter and SDK generation code, and the worker/flight contract
and engine entry points. No full-file review of every imported source is claimed.
No upstream implementation was adapted; external research is unnecessary for
this merge repair. Existing authority, attempt, input and configuration
identities remain the debugging boundary. Rust compilation uses one build job
and an isolated target directory. Historical provider/native receipts are not
rewritten or relabeled as evidence for this source revision.

## Validation

- Combined locked contract export succeeded.
- `python scripts/ci/check_generated_surfaces.py --write` and `--check` passed:
  79 HTTP operations and seven signal projections.
- `cargo fmt --all -- --check` and whitespace checks passed.
- Python SDK: eight tests passed; two installed attunement tests skipped because
  no `RRFLOW_TEST_BINARY` was supplied to that SDK run. Go SDK `go test ./...`
  passed. These are not installed all-language conformance evidence.
- Navigation rendering and `python scripts/ci/check_documentation.py` passed.
  Execution-inventory rendering and checking passed with 2,535 path records;
  this is mechanical inventory, not source-review coverage.
- `cargo test --locked -j 1 -p rrd-contract -p rrd-client --lib --tests`:
  205 passed across 36 result summaries, with no failures or ignored tests.
- Pinned Node 24.22.0/pnpm 12.14.0 TypeScript `check` passed generation, lint,
  typecheck and seven tests.
- The separate engine-only worker command began recompiling dependencies with
  a different unified feature set. Its verified owned compiler was interrupted;
  exit 101 is not a test result. The replacement command includes the same
  contract/client package set with `-p rrd-engine --lib reasoning_budget::worker::`.
  The replacement passed all nine worker tests. The subsequent same-package
  full library run passed the engine's 352 tests, with four ignored and no
  failures. Three ignored cases are native subprocess fixtures; the fourth is
  the privileged ownership fixture explicitly run by native CI.
- `cargo test --locked -j 1 -p rrflow-eval`: 57 passed, no failures.
- `cargo run --locked -j 1 -p rrflow-eval --example refresh_storage_contract`
  passed; no synthetic fixture bytes changed and no native workload executed.
- Candidate and protected merge CI have not run on this reconciliation.

## Joined journey and next boundary

The same installed driver remains
`scripts/inference/installed_controller_proof.py --worker`. Extending that driver
through the public UI remains unimplemented. The installed workspace
status first reported engine `connected` and model endpoint `unavailable`;
the model tunnel and observation sink were stopped. An SSH probe succeeded,
and the existing owned launcher restarted both services with readiness checks.
The real browser driver then reported engine `connected`, endpoint
`model_listed`, three knowledge requirements and no page errors. Authenticated
model discovery is not inference, weight attestation or a successful project
trial. Earlier installed receipts retain their exact original source and
executable identities; the running engine was not replaced by this merge.
Credentialed readback of the existing worker observation after receiver restart
also passed: one receiver event, acknowledged replay without duplication, exact
retained source readback, unchanged task budget and an open task. No model call
was requested. The fresh receipt is separate from the prior receipt; this
verifies the existing installed stack, not the reconciled candidate.

`PromptFlightProvider` still supports only `Observe`; a prepared flight invokes
no provider. Its synchronous commit cannot wrap a model call. The next package
must bind the flight's actual prompt, task/context identity and admitted worker
attempt, reuse existing authority/budgets, and recover retained or uncertain
attempts without redispatch. Connecting unrelated task text to a flight or
copying a draft into a second execution ledger would not satisfy this boundary.
The UI must expose that admitted result and its correlation through the same
public operations. Ordinary-edit successor plans, verified project effects,
triggered LFG/SWAG cycles and reviewed learning remain subsequent engine-owned
boundaries; none is established by this reconciliation.

No roadmap or POAM completion status changes. This journal and its regenerated
index provide the reconciliation record; CI, landing and installed joined
workflow evidence remain separate requirements.
