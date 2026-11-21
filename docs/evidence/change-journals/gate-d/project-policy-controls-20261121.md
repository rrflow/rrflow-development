# Governed live project policy: engine implementation

**Status:** active bounded implementation evidence; local Windows acceptance passed; candidate and queue pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/project-policy-controls-20261121`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), D-01/D-06 and [POAM-077](../../../poam/rrflow-1.0-alpha/poam-077.md)

## Brief and traceability

[PR458](https://github.com/rrflow/rrflow-development/pull/458) starts from
ffb4c4b1779212aa8fa80e2d5908f3685f98d4ab. The
[starting-boundary record](project-policy-controls-start-20261121.md) records the
before/after oracle and source review before implementation. This session drives
the engine prerequisite and dependent Connectome controls; neither package alone
establishes the complete project-to-learning journey.

Project numeric policies were readable only from sealed installation settings.
The engine now exposes authenticated project read, pure preview and exact-digest
apply. Each edit is a full numeric settings snapshot with a half-open valid
interval. Native project control transactions append immutable history and CAS
the head atomically. Installation settings supply revision zero; selected edits
replace only those numeric project rules, retaining installed ceilings and the
explicit user-source binding. User requirements remain independently enforced.

| Existing behavior/source | Canonical destination and replacement evidence |
|---|---|
| User temporal change/revision types in contract scoped_configuration/publication.rs | Scope-neutral ConfigurationChange and ConfigurationRevision; existing user publication/history tests retained. No compatibility alias. |
| Bounded hash-linked history selection in engine scoped_configuration/store.rs | One history.rs selector consumed by the user store and new project_store.rs. Missing required user coverage still refuses; project revision zero is the declared installed baseline. |
| Sealed project rules in scoped_configuration/resolution.rs | Current or explicitly historical selected project revision, plus unchanged user/installed constraint intersection. Query and context admission stamps bind selected/read revisions. |
| Read-only admin endpoint preview in engine invocation.rs | ConfigurationPreview serves both endpoint and project previews. Existing endpoint operations, permissions and tests remain. |
| Existing installed correction driver | Preserved and joined by project_configuration::project_policy_edits_are_live_isolated_and_temporal_after_restart through the real primary binary and authenticated HTTP SDK. |

New modules: contract scoped_configuration/project.rs; engine scoped_configuration/
history.rs, project.rs and project_store.rs; engine tests/project_configuration.rs;
client operation/project_configuration.rs; server http/handlers/project_configuration.rs;
CLI tests/installed_lifecycle/project_configuration.rs. Their module owners,
contract catalogue/OpenAPI/reference, server dispatch, admission resolver and
shared user-type consumers are updated together. All five SDK catalogues, schema
and seven signal projections are regenerated. The strict SDK fixture count moves
from 84 to 87 operations. The product version remains 1.0.0.

Additional source review covers those new modules in full, existing scoped
contract/engine modules, endpoint preview implementation and its test patterns,
client configuration operations, server handlers and changed catalogue/dispatch/
invocation regions. Documentation owners and each changed region are reviewed
against the final diff. This is not a review of unrelated engine implementation.

## Decisions and limits

The existing sourced ADR-0006 owns composition and terminology; no new external
algorithm or research claim is introduced. Existing control transactions and
authenticated audit carry writes; reads never rewrite policy or refresh truth.
Project history is bounded at 4096 reads. A failed source or history read cannot
become empty policy. Source/store revisions are independent; there is no atomic
transaction across the global and project stores or cancellation of admitted work.

Preview checks composition at its disclosed valid-time point. Apply verifies
current project state and the observed user revision, and checks current-time
composition if the edit is active. Later global edits can create a visible
conflict; state read remains usable for a governed repair. An expired plan cannot
publish. A lost-response retry only returns its exact current publication;
superseded plans cannot resurrect prior settings. Project bearer authority never
publishes global settings. Unknown effects remain unexecutable.

Only the eleven numeric query/recall settings are editable. Full installation
configuration, source rebinding, organization/behavioral policies, controller
integration and vector freshness remain outside this package. No throughput,
whole-platform parity or alpha qualification is claimed. D-06/POAM-077 stay open.

## Verification and surfaced failures

Windows, CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target, CARGO_BUILD_JOBS=4.
- Existing focused shared history/resolution tests: 3 passed.
- New engine project tests: 3 passed (concurrent CAS/history, expired/tampered
  plans, read-only authority cannot preview/apply).
- Installed project edit driver: passed. It covers explicit approval, foreign
  identity, user drift, stale project edits, idempotent readback, live query/recall
  limits and admission stamps, isolation, backdated correction, repair and restart.
- Entire installed lifecycle target: 17 passed, 31.04 seconds.
- Strict clippy across contract, engine, server, client and primary CLI all targets
  passed, including the final shared TTL constant name.
- Selector tests: 7 passed. Code-shape and whitespace checks passed.

First installed run exposed the preview adapter being classified as mutating;
its missing idempotency-key error was repaired by the shared read-only admin
preview invocation, preserving authorization. The broad contract run exposed a
stale SDK endpoint-count expectation; it is updated to the catalogue's 87, and
the exact failed golden test passed. No assertion, permission or CI gate weakened.
Initial journal encoding/metadata failed navigation/documentation checks; UTF-8
and the evidence status were corrected. A Windows relink attempted while the
long engine suite still held the test executable produced LNK1104; the running
suite was not killed. After it exited, the focused configuration run passed
37 tests and strict clippy passed again.

Final local checks:
- `cargo test --locked -p rrd-contract -p rrd-engine --lib --tests`: 804 passed
  across 42 targets; 12 existing opt-in tests ignored.
- `cargo test --locked -p rrd-client -p rrd-server --lib --tests`: 70 passed
  across 17 targets; one existing live-SDK harness test ignored.
- `cargo test --locked -p rrd-engine configuration --lib`: final 37 passed.
- `cargo clippy --locked -p rrd-contract -p rrd-engine -p rrd-client -p
  rrd-server -p rrflow-cli --all-targets -- -D warnings`: passed.
- TypeScript `pnpm check`: generated schema, lint, typecheck and 7 tests passed.
  The installed launcher selected Node 24.21 despite an attempted fnm override;
  it emitted the minimum-version warning. Hosted CI uses the declared toolchain.
- Generated parity: 87 operations, seven signal projections, OpenAPI
  45a1377eddcc481533db43c70756e40963a468e319126082a372358a9868c7bd.
- Documentation, generated navigation, code shape, formatting and whitespace
  checked; the new installed driver is in the exact CI selection inventory.

No full six-language live SDK run is claimed locally (Maven is absent); candidate
CI owns that lane and native Unix qualification. New source and changed regions,
wire projections, documentation and complete diff were reread for this bounded
package; the change checklist and replacement map above are satisfied. Candidate and queue evidence will be attached
to PR458 for the exact source revision. This record does not claim pending CI.

## Joined handoff

Connectome currently pins an older engine/SDK, has no scoped-policy editor, and
caches query budgets at connection. The dependent UI package must update the
single engine/SDK input, use these public operations, retain exact native previews
for approval, display conflicts/provenance and read effective budgets for the next
query/prompt. Its real-process test and desktop observation must prove edit,
apply, query and restart, followed by a reviewed parent pin. The same session
continues that work; the engine prerequisite is not the completed UI journey.

Scratch D:/.usr/Temp/rrflow-project-policy-20261121 and the checkout-specific build
cache are retained. Tests own their temporary fixtures/processes. No containers,
additional worktrees or unrelated services/data are removed.
