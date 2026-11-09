# Installed discovery and observation composition

**Status:** active candidate integration; local validation passes, candidate and protected acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-job-discovery-observation-composition`
**Owner:** OBJ-14 / D-02 / H-04/H-06; H-05 dependency remains PR235

## Bound change and before/after oracle

The SWAG controller integration session drives
[PR248](https://github.com/rrflow/rrflow-development/pull/248). Its published
composition brief precedes this ordinary merge of observation candidate
`74ace47c877f56e4c385f74902f1799aeabaaf1a` into discovery candidate
`59e48ebfc24025a063619ce14714e545b8f4775b`. Their common accepted baseline is
`e6f3632b32f1c9be33c1951886a9d9a2be648ddf`.
[PR235](https://github.com/rrflow/rrflow-development/pull/235) must land first.
PR248 was returned to draft before reconciliation. The original live CI runs
retain their own source identities; they cannot qualify this combined source.

The merge preview and actual merge expose 23 conflicts: generated SDK/signal
projections, the contract digest and its current citations, three synthetic
storage-fixture identities, and the SDK corpus endpoint count. The authored
engine, adapter, transport and client changes merge without semantic conflict.
Both capabilities must compose in one engine before a UI can discover a
successor and correlate its authorized observation. This change neither adds
a project executor nor bypasses the separate worker PR229 transition hold.

## Preservation and generation

The conflict resolver retains the automatically merged authored content,
temporarily selects a compilation digest, and regenerates every derived SDK
and signal surface from the combined executable contract. It does not choose
one parent's generated API as the final result. The existing storage-contract
generator refreshes dependent fixture identities; the SDK corpus count comes
from the actual canonical export.

The combined OpenAPI digest is
`1c508a4b2819d4470e6717a304f9990595da25ed09ef729d5fa0b76324bc2975`,
with 75 operations. Full parsed comparison equals the observation parent's
export plus `/v1/attunement/jobs/list` and its derived count. Every other API
value is unchanged. Exact Git mode/blob comparison preserves 2,097
non-overlapping parent entries. Fixture comparison against both parents permits
only the catalogue, manifest and result digests and endpoint count; outcomes
and synthetic evidence classification remain unchanged.

No implementation is moved, removed, replaced or duplicated. Discovery and
observation source/tests retain their existing canonical owners. Shared
authored changes are the two test-module registrations and the combined
contract/reference/server dispatch entries. This new journal and its generated
navigation accompany the integration; historical journals remain unchanged.

## Validation and failures

The focused combined engine suite passes 50 tests, with two existing ignored
helpers. Both larger source-edit/interruption cases finish successfully after
the test runner's 60-second notice; that notice was not treated as failure.
All 170 contract/adapter checks and canonical parity pass. Pinned Node 24.22.0 and
Corepack-selected pnpm 12.14.0 pass SDK generation parity, lint, typechecking
and TypeScript tests.

The first whitespace check finds CRLF introduced by the private fixture
rewriter. Converting that one fixture back to LF fixes whitespace; the parsed
comparison still permits only the count change. The failed check is retained.
No failed behavioral assertion, model result or fixture outcome is discarded.

The primary build and repository documentation/navigation/inventory checks pass.
Owning engine/server/client/evaluator suites report 516 passes and four existing
ignored helpers. One SDK entry returns early without its live-daemon manifest;
the combined candidate's hosted SDK harness remains required. Strict six-package
all-target Clippy, formatting and CI policy checks pass.
An initial documentation command
used a nonexistent script name; the actual owning `check_documentation.py`
then passed. That command error is retained separately from test results.

The combined actual Windows process trial passes discovery, stale-source
rejection, fresh successor acceptance and paginated discovery, authorized
delivery to the native RROFLOW receiver, abrupt restart, and both replan and
delivery replay. It leaves one receiver event and one delivery attempt.
The copied primary executable SHA-256 is
`dcfec4ca881d13a02d98608f45c2c1dc941a931aa4c704e02d6f3623adf8d976`;
the receiver SHA-256 is
`786b6c48e64fc7c5962b1b4dc427af1b06867f8c05a54669e096ebf27a9b82bb`.
The trial uses a copied real SWAG document and controlled edits, without a
model call or learning admission. Only harness-owned processes are terminated.
This is an uncommitted-source debug build, not a release. The retained inventory
binds 823 native source/fixture/configuration files at digest
`85b93531d1cd8b89682fd2df8361e1da14194787c6eeee570abf1b036bbb820f`;
it must remain unchanged through publication.

The first joined trial reaches reopened readback, then fails its full-object
comparison because a fresh read has a later `status.observed_at_unix_ms`.
Complete parsed comparison finds that timestamp is the only changed field.
The corrected harness requires monotonic observation time and exact equality
of every other field. The separately frozen observation source remains exact,
including its historical timestamp. Both trials and the correction are retained;
no production code or failed source/publication check is changed.

Executed acceptance commands include:

- `cargo test -p rrd-engine --lib --locked -- engine::tests::attunement`
- `cargo test -p rrd-contract -p rrd-observation --all-targets --locked`
- `cargo build -p rrflow-cli --bins --locked`
- `cargo test -p rrd-engine -p rrd-server -p rrd-client -p rrflow-eval --all-targets --locked`
- `python scripts/ci/check_generated_surfaces.py --check`
- `pnpm --dir sdks/typescript check` with the pinned toolchain
- `cargo clippy -p rrd-contract -p rrd-observation -p rrd-engine -p rrd-server -p rrd-client -p rrflow-eval --all-targets --locked -- -D warnings`
- `cargo fmt --all --check` and `python scripts/ci/check_workflow.py`
- `python scripts/ci/check_documentation.py`
- `python scripts/knowledge/render_navigation.py --check`
- `python scripts/ci/build_execution_inventory.py --check`

Raw logs, commands and hashes are retained in private dated composition evidence.
The original discovery head's [candidate run 41519736377](https://github.com/rrflow/rrflow-development/actions/runs/41519736377)
passes all 23 jobs. That result remains bound to `59e48ebf`, not this merge.
The combined owning-suite log SHA-256 is
`913287fdb3a7f316c6c0a8dcb0aecfa312d731b3236b3ad0b202743048d5368e`;
strict-lint log SHA-256 is
`ce243cf640e571d3771d8eab3fe403a0a29b3c1c481ac14be1ed8d6524876888`.
Other native platforms, optional-feature partitions and live multi-language SDK
conformance are not rerun locally; they remain required combined-candidate CI.

## Review, limits and handoff

Reviewed owners include engineering instructions, README, change-authoring and
CI rules, POAM-040, both discovery journals and the observation refresh journal.
The full merge conflict inventory, canonical generator, complete shared authored
deltas and parent preservation checks bind this review. Unchanged implementation
files are covered by exact parent blob comparison and the earlier package
reviews. The shared authored diffs and complete new journal were reread;
generated values were checked through canonical and parsed structural parity.
This is author review, not an independent approval.

No new dependency, algorithm, provider policy or external research is needed
for source composition. Existing bounded operation/request audit and durable
receipts provide diagnostics. Private raw logs, retained requests and binary
identities remain separate from this public summary. Controlled edits and
synthetic contract fixtures are correctness evidence, never training labels.

Stop for semantic conflicts, lost parent behavior, altered fixture outcomes,
expanded authority, changed publication heads or failed owning checks. After
local validation, publish the ordinary merge to the existing draft PR, retain
the dependency on PR235, then require combined candidate and protected CI.
Connectome acceptance, governed project effects, independent learning review,
model/29 GB qualification and native/container release remain open. No objective,
roadmap or POAM status changes.

## Links

- [Discovery on accepted main](d-02-job-discovery-on-main.md)
- [Observation refresh evidence](../gate-h/h-05-observation-accepted-main-refresh.md)
- [Public discovery contract](../../../reference/protocol/public-contract.md#installed-job-discovery)
