# LFG endpoint request extensions

**Status:** active implementation evidence; candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/lfg-endpoint-request-extensions`
**Owner:** forward-compatible task endpoint proposal request evidence

**Package:** Gate G-01/G-03 and POAM-009 integration prerequisite; no roadmap
or alpha status change. **Driver:** LFG/RRFlow integration working session.
**Starting development main:** `e6f3632b32f1c9be33c1951886a9d9a2be648ddf`.
**Draft:** [PR246](https://github.com/rrflow/rrflow-development/pull/246).

## Change brief

The owner requested additional request fields rather than a decoder that rejects
them. `EndpointSelectionRequest` previously denied all unknown fields. The
request now accepts and ignores them, so the typed request, operation digest,
authorized controller packet, selected endpoint, budgets and permissions do not
gain new caller-controlled inputs. In particular, sending an LFG observation as
an unknown field does not make the engine read or admit it. Prompt or credential
contents are not retained by this typed request. Normal transport body limits and
known-field validation remain in force. A separately implemented and authorized
engine path is necessary before LFG observations can affect a proposal.

Read for this package: `AGENTS.md`, `README.md`, Gate G, the inference reference,
change-authoring procedure, `rrd-contract/src/endpoint_proposal.rs`, endpoint
selection tests, `rrd-engine/src/engine/endpoint_proposal.rs`, reasoning-budget
proposal tests, native client example, controller-proposal client tests,
generated-surface tooling, SDK generators and their configuration. No external
code was adapted. This compatibility change uses existing HTTP body bounds and
records no additional trace content; resource and model-quality claims are out
of scope. No model call is needed for the decoding oracle.

Implementation changes `EndpointSelectionRequest` deserialization and its
OpenAPI-derived schemas, adds contract and engine tests, and corrects the
inference-reference status note. All five SDK endpoint and signal projections
are mechanically regenerated because they bind the OpenAPI digest. This journal
and its navigation index are evidence, not another implementation authority.

## Evidence and limitations

- Before the implementation change, the focused contract test failed with
  `unknown field endpoint`, reproducing the reported behavior.
- After the change, `cargo test --locked -p rrd-contract --test endpoint_selection`
  passed all nine tests. The new test checks that injected endpoint and credential
  fields decode to exactly the original typed request and do not serialize back.
- `cargo test --locked -p rrd-engine proposal_tuple_rejects_cross_attempt_config_and_usage_substitution`
  passed the targeted engine test. It checked that a request carrying an extra
  endpoint still validates against the same authorized proposal tuple.
- The initial `cargo fmt --all --check` showed formatting changes needed in the
  new test; `cargo fmt --all` applied them. The first generated-surface check
  then reported the stale TypeScript OpenAPI digest; all SDK projections were
  regenerated through their owning generators and the signal projections through
  `scripts/ci/check_generated_surfaces.py --write`.
- The first full `rrd-contract` suite then failed its deliberate OpenAPI digest
  golden check, and the generated-surface check flagged stale reference
  citations. The canonical digest and six current reference citations were
  updated to the newly exported schema; historical evidence was not rewritten.
- Inspection of the first regenerated TypeScript schema showed that merely
  omitting `additionalProperties` did not expose an index signature to typed
  clients. The schema now explicitly sets `additionalProperties: true`, with
  a schema assertion and a TypeScript payload test. All projections and the
  canonical digest were regenerated again. Unknown values remain ignored by
  the engine, not promoted into the controller packet.
- After the schema correction, `cargo test --locked -p rrd-contract` passed
  its full suite and `cargo test --locked -p rrd-engine --lib` passed 316 tests
  with four native subprocess fixtures ignored. The TypeScript SDK's
  `corepack pnpm check` passed generation, lint, typecheck and seven tests,
  including the extra-field type test. Its first run failed import ordering;
  that was corrected before the passing run.
- `python scripts/ci/check_generated_surfaces.py --check`,
  `python scripts/knowledge/render_navigation.py --check` and
  `python scripts/ci/check_documentation.py` passed. The tested OpenAPI digest
  is `9ddcf2c3965d140af0495f661f09a426d37892e2e10396889366f1da4133e656`.
- `cargo clippy --locked -p rrd-contract -p rrd-engine --all-targets -- -D warnings`
  passed on the local Windows host. Implementation and generated diffs were
  reviewed; generated digest changes are derived from the contract exporter.
- The tests above are in-process contract/engine evidence, not a real provider
  call, installed runtime, live LFG connection, or routing-quality result.
  Candidate CI, the remaining cross-platform and installed-runtime checks,
  and merge evidence remain pending and are owned by the pull request.

No access rights, engine dispatch, LFG admission, provider capability or user
subscription is inferred from an accepted extra field. Release gates remain open.


## Local integration with job discovery, 2026-11-03

Starting candidate `919ea4aad371ef20c3909707d145edc33d43a8db`; prospective dependency #248 `c4a3b96a4d1a18fbf19f819f37ce237c0b749873` is in the protected merge queue. The PR brief was updated before this local merge preparation. Seven hand-authored conflicts contained only the two competing current OpenAPI digests; their surrounding text was identical. The other twelve conflicted SDK and signal paths are replaceable projections. They were regenerated from the combined catalogue and contract rather than choosing either PR's generated content as the final result. No historical digest citations were rewritten.

The combined contract exports 75 operations including `attunement-job-list` at `/v1/attunement/jobs/list`; its SHA-256 is `978e20e8128dea9d17367f66492200d675372d53c3cef92b381ff0b001bcbbc2`. `cargo test --offline --locked -p rrd-contract --test endpoint_selection --test attunement_discovery` passed 14 tests. `python scripts/ci/check_generated_surfaces.py --check`, `python scripts/knowledge/render_navigation.py --check`, `cargo fmt --all --check` and the TypeScript SDK's `corepack pnpm check` (seven tests, lint, typecheck and generation) passed on Windows. The first Python generator invocation failed because `ruff` was absent from PATH; rerunning with this checkout's `sdks/python/.venv/Scripts` on PATH succeeded. `cargo test --offline --locked -p rrd-contract` also passed the full contract suite. `python scripts/ci/check_documentation.py` passed. `cargo test --offline --locked -p rrd-engine --lib` passed 332 tests with four ignored; the focused discovery and proposal tests also passed. Python, Go, Java and .NET generated-endpoint checks passed. `cargo clippy --offline --locked -p rrd-contract -p rrd-engine --all-targets -- -D warnings` passed. The final CI and integration status are recorded with the candidate.

This remains local speculative integration until #248 lands and actual main is checked. It does not establish that an external LFG call was performed or that the adaptive plan is implemented. No gate status changes.

## Current-main reconciliation, 2026-11-04

The isolated `agent/lfg246-main-integration` candidate merges public main
`746a0a462e512563b6d3aaa89520f8ab5cde9051` into PR head
`5f69e1ee81eaa1246c9e84fb509eb2be4d4a8a23` without rewriting history. Git
identified 19 conflicts: seven authored contract/reference digest citations
and twelve replaceable SDK/signal projections. The authored hunks were
inspected, source changes preserved, and the combined contract exported to
SHA-256 `c4375b035939ca40618f6e56eb1d52ea11347b742d003508f40df5d73c441d38`.
All five SDK endpoint/schema projections and the signal references were
regenerated from that source. Generated-surface parity reports 78 HTTP
operations and seven verified signal projections. No generated side was
accepted as final by merge policy.

Windows local combined-tree checks: the full `cargo test --offline --locked
-p rrd-contract` suite passed; TypeScript typecheck and six `client.test.ts`
tests passed, including acceptance of a future field in the generated type;
Go `go test ./...`, navigation and documentation policy passed. Execution
inventory initially reported drift, then regeneration and recheck passed with
2,368 path records. The real engine's cross-attempt/configuration/usage test
is running. Offline TypeScript dependencies came from the existing pinned
pnpm store; endpoint generation used pinned Ruff 0.17.1. This is local candidate
proof, not candidate/merge-group CI or evidence that any LFG call ran. The
unknown fields remain ignored and do not authorize a capability. Roadmap and
POA&M statuses do not change.

The combined engine command `cargo test --offline --locked -p rrd-engine
--lib proposal_tuple_rejects_cross_attempt_config_and_usage_substitution --
--nocapture` passed one test, including the ignored unknown `endpoint` field
remaining absent from the typed request. `cargo fmt --all --check`,
`git diff --cached --check`, and `python -X utf8 scripts/ci/check_workflow.py`
also passed. These checks do not replace required candidate and merge-group CI.

2026-11-10 candidate correction after run 42750047347: the rrflow-cli installed-lifecycle test builds an `EndpointSelectionPolicy` literal that lacked this package's new `lfg` field, so the lifecycle, service-protocol and ARM64 jobs could not compile; the literal now sets `lfg: None`. The Linux-only `lfg_diagnostics_are_bounded_and_report_invalidated_checkpoint` failure (one event instead of two) passes on Windows and is being reproduced on Linux separately.
The Linux-only diagnostics failure captured one event where two are expected; the test passes on Windows and on Ubuntu WSL2 with the same tree. The one path on which a completed transport can leave no second event is a packet that fails `validate_for` before the checkpoint, so that rejection now emits an `advisory_rejected` diagnostic naming the error, and the test's assertions print the captured events. The next candidate run shows the real second event.
