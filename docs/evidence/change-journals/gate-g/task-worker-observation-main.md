# Worker and observation delivery on the same engine candidate

**Status:** active local candidate reconciliation; publication and installed qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-observation-main`
**Owner:** G-01b/G-02a, H-05 and POAM-009; PR229

## Scope and source

This continues [PR229](https://github.com/rrflow/rrflow-development/pull/229)
and its [landed-native-client reconciliation](task-worker-native-landed.md).
The clean worker parent is `52e2254af0ad616295975f35aef1154ca5634713`;
the fetched development main parent is
`8f3cbfbe8a52bc2c25e19ec9436ba464d1861515`.
An ordinary merge preserves both histories. Its initial preview found 29
conflicted paths; a conflict-free or complete integration was not inferred.
The existing PR remains a draft. Its previously rejected description/ready
publication action was not retried during this local preparation.

The bounded outcome is one native engine containing both selected-worker
execution and the landed observation delivery/discovery operations. Endpoint
profiles preserve both optional policies, validate both, and include each in
its domain-separated configuration digest. Engine invocation permissions,
HTTP dispatch and the native client's request/response validators retain both
operation families. No worker text becomes a verified project effect.

The canonical operation catalogue drives regenerated SDKs and OpenAPI.
The combined security catalogue has 58 actions and the HTTP catalogue has
76 operations. Existing fixture configuration identities remain valid when
new optional policies are absent. Historical provider receipts retain their
original source and binary identities.

## Review and failure oracle

Review covered the root instructions and README, Gate I, the relevant Gate D
project-workflow prerequisites, POAM-009, change-authoring procedure, prior
worker journal, conflict hunks and surrounding policy validation/digest,
invocation, HTTP dispatch, native client and configuration-test code. Generated
projections are checked against their exporters rather than edited as owners.
The 29 conflict paths, generated outputs and added configuration test are the
implementation scope; imported main history retains its existing evidence.

The added configuration test previews and applies worker and observation
policies together, rejects an independently tampered policy before mutation,
reopens the persistent estate, checks exact readback and idempotent apply, and
rejects stale resurrection after revocation. Its first run exposed a malformed
test sink inheriting a model's placement machine. The engine correctly rejected
that API binding; removing the fixture's placement resolved it. All seven
endpoint-configuration tests then passed. Fixtures are protocol tests, not
observed model outcomes or training data.

The first SDK generation attempt lacked `ruff` on PATH; the existing pinned
SDK environment supplied it. During the initial broad parallel Rust build, available
physical memory fell to roughly 1.4 MiB; that build was explicitly interrupted. Only its
identified process tree was stopped. Its log remains failed/interrupted
history; a one-job run is recorded separately. Neither interruption nor
successful compilation substitutes for test results.

## Worker observation correlation repair

The joined path exposed a production composition defect in
`engine/observation_delivery/persistence.rs::source_references`: it projected
configuration identity only from `endpoint_proposal`, so accepted worker events
lost their known configuration hash. The new native HTTP test first failed with
`Null` instead of the actual worker configuration digest. The projection now
uses the retained worker output when the result is a worker attempt. The
attempt contract already validates its result kind and identity. No client
supplies this correlation and no current export configuration is substituted
for the historical worker configuration.

The regression executes controlled selection and worker exchanges through engine
admission, configures a separate loopback sink afterward, prepares and delivers
the actual metadata payload, checks task/attempt/response/configuration identity,
checks absence of draft text and credentials in the wire event, reopens and reads
both source and delivery, and proves the task stays open and its effort is
unchanged. It now passes. The sinks and model replies are explicitly controlled
protocol fixtures, never real model quality or external service acceptance.
The worker test's profile replacement helper also preserves observation policy.

The source parent on development main could not produce worker-output attempts;
its selector and attunement record projections remain unchanged. Historical
worker receipts from the worker-only candidate are not rewritten or described
as having been exported. Existing observation validation still reconstructs
references from the retained source and checks its receipt/payload hashes.

The first regression compilation exposed an incorrect optional-task assumption
in the new test; `TaskRecord` is required, and the assertion now reads its status
directly. The documentation checker also rejected a status lacking the required
`active` prefix; that metadata was corrected. These failures remain recorded.

## Validation and remaining boundary

Final local validation on the composed worktree:

- `cargo check --locked -p rrd-engine -p rrd-client -p rrd-server --tests`: passed.
- `cargo test --locked -j 1 -p rrd-contract -p rrd-client -p rrd-engine -p rrd-inference`:
  649 passed, zero failed across 57 target summaries, four explicitly ignored.
  Three ignored entrypoints are subprocess fixtures; the fourth is the native
  privileged ownership test reserved for its CI lane. No privileged-test pass
  is claimed here.
- `cargo clippy --locked -j 1 -p rrd-contract -p rrd-client -p rrd-engine -p rrd-server --all-targets -- -D warnings`: passed.
- All five endpoint generators and `python scripts/ci/check_generated_surfaces.py --write/--check`:
  passed; 76 operations, OpenAPI SHA-256
  `7bd0c7d7be778546369ed203633ccbfba0672442e4f24e952d48a38ef04aa3cc`.
- TypeScript `pnpm check` under Node 24.22.0 and pnpm 12.14.0: generated parity,
  lint, typecheck and all six tests passed. An earlier generator invocation used
  an older shell toolchain and warned; qualification uses the required versions.
- Rust format, navigation, documentation policy and whitespace checks passed.
  The whitespace check admits CRLF via `core.whitespace=cr-at-eol`; default
  whitespace reporting flags existing byte-preserved evidence, not just this
  change. Historical receipt bytes were not normalized.

The complete source conflict diff, native correlation repair and its test were
reviewed after generation. No unmerged index entries remain. The broad interrupted
run, malformed test fixture, regression compile error, reproduced null correlation
and corrected passing runs remain distinct logs in the private evidence workspace.
Candidate CI, protected merge-group CI, installed-primary-binary execution and an
actual provider/sink trial on this composed source have not run. Earlier live
receipts are not reassigned to this revision.
The existing request, task-attempt, configuration and observation identities
remain the debugging boundary; no extra event store or lifecycle is added.
This is integration of existing code, so no new algorithm or external research
claim is introduced. Resource scope is local compilation and controlled tests;
no model, training, deployment or external observation-delivery effect is run.

No roadmap or POAM status is closed. Template preview and typed answers in SWAG
do not yet implement native source inventory, project changes, verification or
fulfillment. Those retain D-03, I-03 and I-07 ownership. Full UI-to-project
acceptance, independently reviewed outcomes, checkpoint comparisons and total
serving-memory qualification remain open. Stop on lost behavior, weakened
checks, conflicting authority, missing evidence or an unapproved publication;
preserve the failed history instead of converting it into success.
