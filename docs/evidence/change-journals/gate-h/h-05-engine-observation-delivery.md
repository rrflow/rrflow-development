# H-05: engine-owned observation delivery

**Status:** active draft implementation evidence; full acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-05-engine-observation-delivery`
**Owner:** H-05/H-07 optional transfer and I-03 outward-effect prerequisites

## Brief and baseline

[PR235](https://github.com/rrflow/rrflow-development/pull/235) was opened before
source edits from public main `6c069e1462d2b9cbb0237e9c117c379d1482bbaa`, with
opening commit `e2aad006ff3fc3ac976d4a245c59bd2f34269881`. Driver: the SWAG
controller working session. The engine had authorized task/job reads and endpoint
configuration, but no durable RROFLOW exporter. Existing observation-source
configuration is inactive; session subscriptions do not supply an unattended
export checkpoint.

The package adds explicit prepare/deliver/read capabilities at `RrdEngine`,
using the existing control store and audit transactions. An optional profile
policy binds destination, credential reference, source scopes, network/metadata
grants and finite limits. A bundled stateless HTTP adapter performs one exchange
without owning retries, storage or task lifecycle. No provider ranking, model
name, endpoint host, secret, training consent or success label is inferred.

Scope: contract/profile/security/catalogue, engine persistence and source
projection, `rrd-observation`, HTTP handlers, validated native operation markers,
generated public projections, tests and this reference/evidence record. No
installation default, model training, generic routine executor, automatic
learning or public release is added. Stop if the implementation requires a
second authority, arbitrary caller-supplied observations, unsupported identities,
or a claimed outcome without actual source evidence.

## Implementation and review boundary

Preparation freezes an actual authorized public response under atomic source,
configuration, security and session guards. Job revision and plan identity avoid
using a changing read timestamp as a precondition. Historical source bytes remain
local and are available through authorized readback; the sink receives references
and digests only. A later source revision neither rewrites the observation nor
blocks project lifecycle work.

Delivery saves its attempt/lease before I/O. Retry uses the same retained event
and bytes. Only exact destination/event acknowledgement, before lease expiry and
under unchanged authority/configuration, can publish an acknowledged receipt.
Lost replies or rejected completion fences remain uncertain. Acknowledged replay
does not dispatch again. Current permissions and process-owned credentials are
required after restart. Credentials and raw sink failures are not serialized.

The bounded index shares the existing control transaction with each record.
Cleanup respects its 64-transition ceiling, and serialized records respect the
one-MiB control-value bound. Uncertain records are retained by default. An
explicit approved `retire_expired_uncertain` policy permits audited retirement
after expiry, without converting uncertainty to success. A new preparation after
retirement gets a fresh engine-generated remote identity. This affects export
history only, not the original task's state.

Owner records read include AGENTS, the README/objective/gate warp chain, the
change-authoring procedure and endpoint-binding reference. Implementation review
covers endpoint-profile hashing/configuration CAS, process-owned credentials,
source public reads, security/session authorization, reasoning-attempt persistence,
control transactions, operation catalogue/reference lookup and generated native
bindings. New contract, adapter, engine and test files were read in full. Final review
compares complete file bytes and inspects the complete change; its method and
limits are recorded below. No moved/deleted
implementation or sibling-source runtime dependency is introduced.

Research/adaptation: reuse the repository's authorization, transaction, source
read and pinned HTTP APIs. No external algorithm adaptation or new third-party
version. Existing control audit and bounded public delivery receipts provide
diagnostics; no source content, credential or invented resource metric is logged.
No performance or memory qualification is claimed.

## Verification in progress

Evidence logs are retained locally under
`D:/projects/swag/.swag/evidence/20261031/observation-delivery/`.

- Four focused `rrd-contract` tests pass: optional-policy hashing, explicit bounds,
  source identity and durable job preconditions. Five adapter protocol tests
  passed earlier in development; final-source rerun remains due.
- Initial engine check exposed two incorrect `CorrelationId` display uses; they
  now use the canonical string accessor.
- Adding historical readback exposed an invalid contract-to-core hash reference;
  it now uses the contract's existing hash helper, with no dependency added.
- The first installed-engine run failed all eight tests before delivery because
  new operation references were not in the sorted lookup table order. The
  canonical table was corrected; no fallback lookup was added.
- The next run passed seven tests. Its remaining test supplied a read timestamp
  before the real job-control completion. The corrected test derives later
  observations from the returned engine timestamp; production validation remains
  intact. The corrected run passed all eight installed-engine tests. Its retained
  log SHA-256 is `a2f2b41e56bef0fe8fe23260358edfcd790656fbd83327ca9f5a4c7612e6b3f4`.
- The broader engine library run passed 297 tests, failed the SDK corpus's stale
  operation count, and ignored four existing subprocess helper entries. The
  canonical exported catalogue supplies the corrected count of 74, and the
  existing read-only installation-identity generator confirms the identity
  projection. No captured dataset or model result was edited. The corrected
  engine library suite passes 298 tests, with four existing helper entries
  ignored; log SHA-256
  `8462157365c5a7c43ac504e09b6bd04e11f4f0397920e0303bb56a0b0052f321`.
- The full contract run then exposed two intentional protocol golden changes:
  the security catalogue now has 57 actions (three observation actions added),
  and the reviewed OpenAPI document has the digest below. The additions retain
  old profile hashes when no observation policy is present, add three operations
  and their schemas, and extend action enums. The golden constant and assertion
  were updated after reviewing that delta; the check was not removed or weakened.
  The contract/adapter rerun passes all 165 tests, with none ignored.
  `cargo check -p rrd-server -p rrd-client --locked` passes with the new HTTP
  dispatch and native markers. Strict all-target Clippy passes for contract,
  observation adapter, engine, server and client; workspace formatting passes.
  The engine library suite was rechecked after the final protocol-fingerprint
  golden update: 298 passed, four existing helper entries ignored. Final log
  SHA-256: `cd94f762754db48bd786d66bc9a696c40d8336cf13f164c6bbdd46da2c5a07e5`.
  The 165-test contract/adapter log SHA-256 is
  `2ebe0a4751805f184e5e5e91d810ff28e15ddae41431450c5f2b5acf28cf7b3b`.
  The documentation check rejected the initial
  journal status because it lacked the required active/historical classification;
  it now explicitly declares active draft status. Regenerated navigation and the
  documentation-policy check pass.
- All five language SDK projections regenerated. Generated-surface parity passes
  for 74 HTTP operations and seven signal projections, with OpenAPI SHA-256
  `057f6fea8ff25639c61012710793bb87e5672d2331af85117da30029c49629db`.
  TypeScript `tsc --noEmit` passes. Current digest citations were updated without
  rewriting historical conformance results.
- The default local Node/pnpm pair could not read the repository's current
  lockfile. Its declared Node 24.22.0/pnpm 12.14.0 toolchain installed all 44 locked
  TypeScript packages from cache, without downloads or lockfile changes. A cached
  Windows shim was unusable; the installed package's documented Node launcher
  worked. These were toolchain setup failures, not product behavior.

- `cargo test -p rrd-client --test observation_delivery --locked` passes all
  three public transport scenarios. They install an actual temporary engine,
  approve its endpoint configuration through the native client, and exercise
  prepare/deliver/read through the authenticated HTTP server and real outbound
  loopback HTTP. Acknowledged replay survives closing/reopening the engine
  without dispatch; a dropped receiver reply remains uncertain and explicit
  retry sends identical event bytes after reopening. A third scenario executes
  the bundled deterministic embedding adapter on local file bytes and exports
  the resulting actual accepted task attempt. Stale source/configuration and
  mismatched payload requests are rejected without creating a delivery or
  adding a network attempt. Historical response, payload and acknowledgement
  digests are checked against actual bytes. Retained log SHA-256:
  `adfd3714520cf86543c7aaec604f890f7acbded139973eba586244f81ff1be53`.
  The new test file was read completely; package-wide final review remains open.
- Inventory verification initially reported drift. Its canonical generator
  rebuilt 2,201 path records and the subsequent inventory check passed.
  Documentation policy also passed (854 statuses, 852 coordinates); whitespace
  checking passes. `cargo test -p rrd-client --locked` reports 29 passes,
  zero failures/ignored. One existing external-daemon conformance entry returns
  early without `RRD_SDK_CONFORMANCE_MANIFEST`; that variable was absent, so its
  external harness was not exercised. All three new transport scenarios ran.
  Suite log SHA-256:
  `8eb3253fc73d83a99bb3c3da7c8d022b4cc29c911489b50cfa05a9b8a53c188f`.
  `cargo clippy -p rrd-client --all-targets --locked -- -D warnings` passes;
  log SHA-256 `23480563d4d5b8f4d1725bfe677b1195dce3884526f82c15a73f84fbefbf0f49`.

## Installed receiver round trip

### Retention correction after the first installed proof

The new installed-engine regression fills the 128-record history, lets it
expire, approves a cap of one, and prepares a fresh observation. The initial
run fails with `ConfigurationLimit`: one transaction's 58-deletion limit
prevented any cleanup from committing. Log: `retention-cap-before-fix.log`,
SHA-256 `6b6d99b404bffa1dc891fdba515f4f2e954763bf4f301d3d36d74ef78a938120`.
Preparation now checks whether all eligible retirement can satisfy the cap,
validates the new record before retirement, and retires in at most three
bounded transactions. Every batch guards source/configuration/authority/index;
only the final batch publishes the new record with its index. A concurrent
conflict can preserve audited cleanup progress without publishing the new
observation. All nine focused tests then pass, including reopening with exactly
the new cap and preserving unresolved records without an explicit retirement
grant (`retention-cap-fixed.log`). Two further tests pass: eight concurrent
preparations cannot overrun a one-record cap, and a job can advance during
outbound delivery while the acknowledged historical observation remains intact.
All 11 focused tests pass (`preparation-concurrency.log`). POAM-008 records this
local correction with acceptance still pending. Earlier installed-binary
evidence below remains bound to its original executable; the changed engine
requires a rebuilt trial.

### Original installed executable evidence

Both actual Windows binaries were built with locked dependencies and copied to
an isolated evidence directory. RRFlow's local uncommitted implementation is an
unqualified debug build, SHA-256
`882dcdbb48b073eac9cd2e885b2f7f43daea5a412fcbddace09fb8541ff0b0c9`.
The receiver reports candidate revision `b4c04ceec6686db056b53bc48de44137c5c2f885`;
Git verifies its tree is identical to merged `d552e8ab`. Its executable SHA-256 is
`786b6c48e64fc7c5962b1b4dc427af1b06867f8c05a54669e096ebf27a9b82bb`.

The local harness uses the primary CLI's actual identity/plan/apply/serve path,
process-owned credential mapping, public operation catalogue, explicit approved
endpoint policy, fresh loopback receiver/devspace and JSONL storage. The source
is the installed job for an isolated project containing the existing deployment
readiness document. No provider, training worker or external account is used.
The receiver's declared footprint uses an actual startup working-set observation;
that sample is not peak-memory or resource qualification.

The first run reached acknowledged delivery but the harness incorrectly required
the receiver's nullable payload field to be absent. The second corrected that
check but used POST for session close; the public catalogue requires DELETE.
Both failures and their original harnesses/binaries are retained. The corrected
harness follows each catalogue method and verifies absent/null content fields.
It changes no engine/receiver behavior or access controls.

Run `installed-roundtrip-20261101T012030Z` passes. It retains exact installation,
configuration, prepare/delivery/read requests and responses plus receiver reads.
Event `71ee6ea7-9811-8d26-acc5-3b354403ffd5` has the same instance/job and source
response digest as the engine receipt. After terminating and reopening both owned
child processes, historical engine readback and receiver storage are unchanged.
Acknowledged replay leaves one engine attempt and one receiver event. All trial
processes were stopped; a subsequent process inventory found none remaining.
The result record SHA-256 is
`16bd1721e4afea02cffd7d87c5bd8b869a4e328ecba1a2e9115c4e616004d6a4`.
This establishes isolated installed sender/receiver transport and restart
persistence, not a completed development task, hosted deployment, model-quality
result, dataset approval or release qualification.

These are isolated installed-engine and protocol fixtures, not model evaluation
or training observations. The public transport tests use the test executable as
installation identity and a loopback protocol receiver; they do not qualify the
shipped CLI binary. The separate installed trial above exercises actual binaries
and RROFLOW. Source/configuration concurrency, remaining owning suites, final
review and actual development-task outcome correlation remain required. No
full-workflow acceptance follows.

## Dependency and handoff

### Owning-suite corrections and rebuilt installed trial

The first complete engine/server/client command passed the 301 engine library
tests (four existing subprocess helpers ignored), then exposed three architecture
failures: the exact grouped-package and engine-composition lists omitted the new
adapter, and new target sources were not yet in the Git index. The failure log
`owning-engine-server-client.log` has SHA-256
`6e1bbf822adc1d9d7c1181cdb266a5fdf9041e6e85e9875719142c9727ac75fd`.
The adapter is now explicitly declared in those lists and in the components
outward clients cannot import. A new architecture check constrains it to the
contract/kernel/serialization/HTTP dependencies and an optional full-engine-only
consumer. The new target sources were staged for clean-checkout inclusion; the
tracked-source check was preserved. All 33 architecture checks pass, log SHA-256
`49fa5ed46e401f9cb9de96625f9a657b6814ae0a6d9d3a005fca009c3712ecc7`.

The next owning run passed engine checks and 21 HTTP lifecycle tests, then failed
the process kill/recovery matrix because this isolated build directory lacked
`rrd-estate-controller.exe`. This is an explicit test prerequisite, not a skipped
test or a changed recovery assertion. Failure log
`owning-engine-server-client-corrected.log` SHA-256:
`6083baace0e17f67fc0f27d250e3b5d33e6edef92f4a74108e7b080adb11b7cb`.
`cargo build -p rrflow-cli --bins --locked` then built the declared CLI binaries;
build log SHA-256
`bdee38efe0f7ec305df8fb98e7af2675cfb93fd3eba82065aa2d4a1d1205511a`.
A staged contract-test file had CRLF bytes reported by `git diff --cached --check`;
normalizing that file to LF clears the check without changing test behavior.

The rebuilt engine executable includes the bounded-retirement fix. Its SHA-256 is
`147770113e31631c20b5b54642afb50fac27715991eeb1ee0265519e5e34b11a`.
The same installed-binary harness passes again in
`installed-roundtrip-20261101T013715Z`, with actual receiver executable unchanged.
Event `02afb39b-8133-8100-aed9-a44fe6fcc850` survives abrupt restart of both owned
processes; acknowledged replay retains one receiver event and one engine attempt.
The result SHA-256 is
`57e6a7eb442b20306af751d9592cead463f9fc8f1425505b4b115218684040b0`.
Subsequent process inspection found no trial executables remaining. This is
metadata-transport and restart evidence for this exact local debug binary; no
model call, development-task completion, training consent or release is inferred.

With the required binaries built, `cargo test -p rrd-engine -p rrd-server
-p rrd-client --locked` completes successfully: 427 reported passes, zero failures
and four existing subprocess helpers ignored. One reported Rust SDK entry returned
early without its external manifest; it is not counted as executed conformance.
Log `owning-engine-server-client-complete.log` SHA-256:
`d78e0573625d6219db6d87d14244af1c6dadfe9e8d25500dfd8f6668742faa26`.

A separate local wrapper then reused `scripts/ci/run_sdk_conformance.py` and its
private-manifest creation/verification helpers, built the actual daemon example,
checked corpus and retry-endpoint identity, set the private manifest environment
and ran the owner's Rust command. Actual Rust daemon conformance passes against
corpus `42e2a637a7852dfc4b86ef5993a95f2e59747ad4a7c63f0304425f5c290f90d2`.
The owned daemon shut down successfully and the private temporary directory was
removed by the repository helper. No manifest credentials were retained in logs.
Log `rust-actual-daemon-conformance.log` SHA-256:
`efcfe7f5f95b7b19057ea4a76ea03fd9b18d4784870ae12a0c8872109434e804`.
Only Rust was selected; this does not claim six-language SDK conformance.

Strict all-target Clippy passes for contract, observation, engine, server and
client, log `final-retention-clippy.log` SHA-256:
`170a1e83ff34c4b8f9aa7665863c01d6adc4d8f994a7911711f77b6772949796`.
Workspace formatting initially caught an edition-2024 formatting invocation on
the architecture test; applying the repository's declared edition 2021 restores
the existing layout and `cargo fmt --all --check` passes. Generated-surface parity
still verifies 74 operations and seven signal projections at the unchanged
OpenAPI digest. Review corrected the endpoint reference's stale claim that only
configuration validation exists: profile persistence itself still dispatches
nothing, while explicit experimental proposal/export operations can call approved
bindings. No general provider activation or release qualification is inferred.

Receiver [RROFLOW PR43](https://github.com/rrflow/rroflow-development/pull/43)
merged as `d552e8ab7e898b21dd5de98caab77f8ed992226e`, candidate `b4c04cee`, after
its six CI jobs and release-plan builds passed. Release publication jobs were
skipped. Post-merge main CI `41257683331` also passes all six jobs. This proves
receiver acceptance, not deployed engine integration.

### Final author review and publication handoff

The new contract, adapter, engine/persistence, handler and test files were read
in full. Changed existing engine, client, binding and router implementations were
read with their owning control/configuration/source boundaries. The complete
patch was reviewed, including optional-policy hashing, the three authorization
and catalogue entries, generated SDK routes/schemas and reference corrections.
No unrelated implementation change was found. This is author review, not an
independent reviewer approval.

The final full-file comparison read all 51 changed files (82,826 lines) and
compared 77,608 unchanged lines with the opening commit. Large shared contract
and test files use exact unchanged-region comparison plus inspection of every
changed region; this does not claim a fresh manual line-by-line audit of their
unchanged implementation. Generated files are reviewed through their owner,
complete generated delta and canonical whole-file parity. The lockfile was
parsed completely: only the bundled adapter package and engine dependency were
added; all third-party packages are unchanged. The shared JSON corpus was parsed
and compared completely: only the generated endpoint count changed, from 71 to
74. Seven protocol/SDK references change only their current OpenAPI digest;
historical audits remain intact.

`source-review.json`, SHA-256
`2c758350553d5d38b831d2d32357f85eec7157cd269aae039d55d85956a7c6c1`,
retains each path, whole-file SHA-256, changed spans and comparison method under
the local evidence directory. All 51 hashes match `retention-validation.json`
(SHA-256 `4ed3d972469bea70bc8169f74115e04c4c4eb6193dcddad289ead81f7589a50a`)
before this final journal-only update. Those immutable receipts are local
validation evidence, not publication, independent approval or release receipts.
Fetching public development main still resolves to `6c069e14`; no integration
merge is needed for this candidate.

Change checklist: the preimplementation PR brief binds H-05/POAM-008; owner and
implementation review, first-failure evidence, bounded implementation, owning
checks, generated projections, source review and linked journal are recorded.
No implementation was moved or deleted and no first-party dependency escapes
the repository. The driver remains the SWAG controller working session. Next
publish this bounded implementation to PR235, start its full candidate pipeline,
and land only after required checks. Exact candidate revision, CI and merge state
come from the PR; none are asserted complete in this prepublication record.

The exporter has no source dependency on PR229, PR230, PR231 or Connectome PR30;
it uses existing accepted task/job reads. Those packages share catalogue/profile
surfaces and must refresh by ordinary merges and regeneration after either lands.
They remain prerequisites for actual development-task/UI outcome proof, not for
this bounded transport candidate. No roadmap, alpha, POA&M or release gate is
marked complete. Other-language SDK conformance and native candidate lanes await
required CI; full workflow, training approval and public deployment remain open.
