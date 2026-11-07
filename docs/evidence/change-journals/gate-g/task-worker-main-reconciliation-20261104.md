# Worker execution candidate: current-main reconciliation

**Status:** active reconciliation evidence; qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-main-reconciliation-20261104`
**Owner:** OBJ-06, G-01b/G-02a, H-05 and POAM-009; PR229

## Change brief and review

[PR229](https://github.com/rrflow/rrflow-development/pull/229) previously
implemented engine-owned selected-worker calls with controlled and one earlier
real-provider trial. It became conflicted against main. This reconciliation
starts at worker head `fb633aa6c19da24df0da858aeeaf89b1a7d70d92` and public
development main `746a0a462e512563b6d3aaa89520f8ab5cde9051`, with merge base
`8f3cbfbe8a52bc2c25e19ec9436ba464d1861515`. The PR brief was amended before
the integration to preserve both candidates, regenerate combined projections
and fixtures, and stop on semantic or authorization mismatch. No provider,
project mutation, or rollout is authorized by this merge. The prior real-model
receipt remains bound to its own source and executable, not this new tree.

The worker's production consumer is its engine task-attempt operation and
installed controller driver. The intended joined journey additionally requires
a model-backed prompt flight: `PromptFlightProvider` is still `Observe`, and
the synchronous flight commit cannot simply wrap a provider call as a single
transaction. A future bridge must bind the flight, task and selected attempt,
use existing worker admission/budgets/uncertainty, and durably correlate
outcomes without redispatch or a new authority. No such bridge is claimed here.

Read completely for this reconciliation: the RRFlow joined-delivery skill,
`docs/evidence/change-journals/gate-g/task-worker-observation-main.md`, and
`docs/evidence/change-journals/gate-g/task-worker-observation-installed.md`.
Reviewed relevant portions of `README.md`, ADR-0002, alpha objectives, the
roadmap, POAM-032, change-authoring, `flight.rs` contract/engine,
`endpoint_worker.rs`, worker tests, exporter/generator scripts, and the authored
conflict hunks; the 12 generated conflict paths were classified and regenerated; this is not a line-by-line review of all imported main code.
The merge has 26 overlapping paths, including three auto-merged source/test
paths reviewed through their diffs; the other paths are digest citations,
fixture hashes, or generated projections. No upstream source was copied.

## Research and debugging decisions

[Temporal's model-call integration](https://docs.temporal.io/develop/python/integrations/openrouter)
documents that a remote call can be repeated after the remote result arrives
but before it is durably recorded; it does not guarantee deduplication for
RRFlow's providers. That supports retaining uncertain external outcomes rather
than treating a timeout as proof of no execution. [OpenTelemetry's sampling
specification](https://opentelemetry.io/docs/specs/otel/trace/tracengram-probability-sampling/)
allows spans to be dropped at several points; sampled traces therefore cannot
replace a durable attempt receipt. This reconciliation retains existing
operation, configuration, attempt and response digests and adds no telemetry
authority or new external dependency. Resource decision: use locked offline
builds and one Rust build job to avoid the earlier multi-job memory failure.

## Reconciliation and initial failures

A `git merge-tree --write-tree` preview found 23 content conflicts: one
contract digest, seven reference digest citations, three fixture digest
files, and twelve generated SDK or conformance paths. A temporary `--ours`
checkout of complete files mistakenly discarded nonconflicting main changes
within `lib.rs`. Contract export failed with missing `result_identity_version`
(`E0609`). The merge index was used to reconstruct every conflicted file and
preserve its automatic merges; only conflict regions were selected as
temporary generator inputs. The combined export then succeeded with digest
`f14071c71318062cd2b1a9fca491a16d91b4481e98c3e0bcefeeaf98359ed580`.
This failure was local and the incorrect tree was never committed or pushed.

The broad engine suite reported 350 passed, one failed, four ignored. The
failure occurred while validating the merged SDK conformance fixture before
installation identity comparison: it retained endpoint count 76 from the
worker parent although the combined exporter has 79. A structural comparison
found all other fixture fields equal to main; the expected count was updated
to 79 without weakening validation. The failing test passed on rerun, and the independent
`refresh_sdk_installation_identity -- --check` verified that no identity
rewrite was required.

The first Python generator could not find `ruff`; using the existing pinned
SDK environment on PATH succeeded. An initial TypeScript dependency command
pointed to a nonexistent `pnpm.cjs`; the pinned Node 24.22.0 and pnpm
12.14.0 `pnpm.mjs` installed the locked packages from the local store.
Generated Python, Java, .NET, Go and TypeScript endpoint outputs and seven
signal projections were refreshed from the combined executable catalogue.
`cargo run --offline --locked -j 1 -p rrflow-eval --example
refresh_storage_contract` regenerated the existing synthetic storage
fixture identities; it is not a native capacity workload.

## Validation and remaining work

- `cargo test --offline --locked -j 1 -p rrd-engine --lib
  reasoning_budget::worker:: -- --test-threads=4`: nine passed, zero failed.
- `cargo test --offline --locked -j 1 -p rrd-contract --test websocket_contract`:
  eight passed, zero failed.
- `cargo test --offline --locked -j 1 -p rrflow-eval`: passed, including the
  generated synthetic storage fixture checks.
- `python scripts/ci/check_generated_surfaces.py --write` and `--check`:
  79 HTTP operations, seven signal projections, combined digest above.
- Locked Node 24.22.0/pnpm 12.14.0 TypeScript typecheck and six tests passed;
  `go test ./...` passed. Navigation and documentation passed before this
  new record was added; both must be rechecked after finalizing it.
- A staged-blob comparison against main verified all 149 main-only paths are
  byte-identical to main. The 26 overlapping paths were reviewed as described
  above; no exhaustive source audit is claimed.
- Broader engine suite: 350 passed, one failed, four ignored before correcting
  the fixture count as described above; after correction the full rerun passed
  351 with four ignored. `cargo test --offline --locked -j 1 -p rrd-contract
  --tests` then passed. `cargo fmt --all -- --check` passed.
- The independent installation identity check passed. No read-only planner
  output was written and no installation took place in that check.

The changed paths are the 175-path main integration plus the conflict-derived
projections, generated fixtures and this record; Git and the execution
inventory provide the mechanical list. No existing implementation was deleted
or moved by this repair. Project-source freshness, prompt-flight provider
execution, current-revision real-provider/installed readback, C-07 capacity
and soak, candidate CI and merge-group CI remain unproven here. No roadmap
or POAM status changes.

## Change checklist and publication

The PR brief binds the outcome, source identities, owned effect scope,
unchanged authority, failure oracle, acceptance commands and stop conditions.
The conflict review covers the authored digest regions and auto-merged
operation reference, contract tests and router; the generated paths were
regenerated and checked. The binary blob comparison establishes exact
main-only preservation, not an independent full-source review. The recorded
research, debugging and resource decisions are above. This journal and its
nearest generated index are the evidence additions. Remaining installed
worker readback and protected CI are not yet run on this new revision; do not
interpret prior receipts as those results.
