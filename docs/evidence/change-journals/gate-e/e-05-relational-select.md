# Native relational SELECT for rrflowQL

**Status:** active draft implementation; verification and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-05-relational-select`
**Owner:** OBJ-02/07, E-05/F, H-08 direction, POAM-005; [PR #265](https://github.com/rrflow/rrflow-development/pull/265)

Public main start `ed3027b79e41dfcdf5b27e57f07346f11f6da347`; draft bootstrap `5a235b5479d365b9aae7a751b91e13963a1fb095`. The PR brief preceded code edits and describes current and target behavior, source boundaries, oracle, research, budgets, stop conditions and separate overlapping #256 work. An explicit native `SELECT` projection lowers to the existing query AST, canonical form, binder and executor; record and relationship joins reuse existing semantic and authorization owners. SELECT does not admit the graph traversal source as a table. Existing source syntax and persisted formats remain unchanged; this does not implement PostgreSQL SQL, graph removal, full alpha or compatibility.

Read README; current query/multi-model and transactions reference; Gate F and H-08; query parser and parser tests; the focused query and engine admission fixtures and relevant binder/execution entry points; the codebase-grounded change-authoring procedure; and the #256 diff/PR brief. This is not an every-line review of the repository. Changed code: `rrd-query/src/syntax.rs`, `rrd-query/tests/relational_select.rs`, `rrd-query/tests/query.rs`, `rrd-query/tests/query/relational_select.rs`, and `rrd-engine/src/engine/tests/query_admission.rs`. Documentation: query/multi-model reference, this journal and generated index. No source implementation, test, operation, durable record, schema format or graph capability was removed or moved.

First failure on Windows: `cargo test --offline --locked -p rrd-query --test relational_select select_relationship_join_uses_the_existing_relational_query_contract -- --exact` failed with `expected FROM, found SELECT` (one selected test). The corrected parser unit test and a real rrflowKV relationship-join test passed; the latter compared independently expected row values and read stamp before/after reopen. Full package and authorized engine checks are recorded after completion.

Research: PostgreSQL's [table expressions](https://www.postgresql.org/docs/18/queries-table-expressions.html) and [recursive queries](https://www.postgresql.org/docs/18/queries-with.html) distinguish relational joins from the internal evaluation needed for relationship walks; no PostgreSQL-compatible syntax or recursive CTE support is claimed. No upstream code copied. This parser spelling reuses existing read-stamp and physical-read evidence and introduces no performance claim or telemetry authority.

Remaining: published API still contains `graph_relation`; the explicit older traversal source remains available, and the full SQL/PostgreSQL, installed process, configurable graph generation, source ingestion, C-07 capacity and soak requirements are unproved. The new SELECT source cannot admit traversal as a table. Exact candidate and merge-group CI must pass; no roadmap or POA&M completion status changes.

## Windows verification and present limitations

`cargo test --offline --locked -p rrd-query --test relational_select` passed
both parser equivalence and rejection tests. `cargo test --offline --locked
-p rrd-query --test query relational_select::select_joins_relationships_at_one_stamp_and_survives_reopen -- --exact`
selected one test and passed on a real rrflowKV store. `cargo test --offline
--locked -p rrd-query` then passed the complete query package, including the
existing temporal relationship/traversal oracle. `cargo test --offline --locked
-p rrd-engine --lib engine::tests::query_admission::public_query_uses_one_key_budget_after_unrelated_growth_and_native_reopen -- --exact`
selected and passed one test. That test exercises SELECT and the existing
native form through the public engine method; it compares rows, cursor, schema
revision and examined keys, rejects a wrong session token, checks exact and
insufficient read budgets, and repeats after native reopen. The earlier command
using the unqualified name with `--exact` selected zero tests; that was not
acceptance and was rerun with the fully qualified name. Tests used the existing
local `document-graph-journey/target` Cargo cache; they were not a clean or
installed-binary run.

The owner clarified that graph-shaped work must consume the C-06g-derived
stamped stream and configured page cache behind relational queries. This change
does not implement that storage provider. Existing native traversal remains a
separate supported query source and reads materialized semantic changes; the
SELECT form declines it. The indexed-stream, SQL/PostgreSQL profile, public
`graph_relation` naming, external PostgreSQL knowledge path, project indexing,
C-07 and native/platform qualification remain open. PR #256 owns an overlapping
query test file and is in the merge queue; integrate in order, preserving both
test modules.

`cargo clippy --offline --locked -p rrd-query --all-targets -- -D warnings`
passed on Windows. The first `python scripts/ci/check_documentation.py` run
failed because this journal omitted the required `active` status marker; the
marker was corrected and the check rerun. The HTTP fixture was added to the PR brief before its test edit; its result
is recorded below. The initial brief called it a separate process; inspection
of `start()` showed that it actually runs a server thread in the test process.
That claim was corrected in the brief and the user-facing update.

`cargo test --offline --locked -p rrd-server --test http_process
data_transaction_atomically_commits_every_public_model_and_replays_after_restart
-- --exact` selected one test and passed on Windows. The in-process HTTP server accepted the SELECT relationship join, rejected the
unauthenticated request, and returned independently checked endpoint values
and the same rows/cursor after the server and store reopened. This is real
socket traffic but is not separate-process, signed-installation, PostgreSQL
or C-07 qualification. `rrd-server/tests/http_process.rs` is an
additional changed test path. An initial format check found expected formatting
in this newly changed test; `cargo fmt -p rrd-query -p rrd-engine -p rrd-server`
corrected it.

The additional `cargo test --offline --locked -p rrd-server --test http_process
standalone_daemon_process_passes_the_shared_corpus_and_exclusively_owns_its_root
-- --exact` selected one test and passed on Windows. Unlike the thread fixture,
it launches the source-built `rrd-server` as a child, uses an installed project
with explicit fixture schema, commits two records and a relationship through
the authenticated HTTP transaction, and validates the SELECT join and cursor.
After the child exits the rrflowKV store reopens at that committed cursor. The
fixture uses its test distribution executable identity; it does not prove a
signed clean-machine release or restart the child to requery SELECT. The
separate thread fixture does requery after an engine/server restart.

The standalone test was then extended, as described in the updated PR brief,
to start a second `rrd-server` child against the same installed project and
repeat SELECT after the first child exits and the store reopens. The same exact
command above was rerun and passed one test; both children published separate
loopback addresses and the second returned the same rows and cursor. This
supersedes the earlier statement that the child was not restarted. The setup
extraction preserves the existing readiness deadline, exclusive ownership and
graceful shutdown checks.

Local release checklist: code paths and the changed test fixture were reviewed
against the current parser, engine and server entry points; existing query
canonicalization and storage formats are retained. `cargo fmt -p rrd-query
-p rrd-engine -p rrd-server -- --check`, navigation, documentation policy,
execution-inventory generation/check and `git diff --check` are run before
commit. Full candidate CI and merge-group CI remain outstanding. The joined
real-project index-to-context flow, external PostgreSQL knowledge path,
relationship-to-Arrow streaming, C-07 capacity/soak and non-Windows SELECT
process tests were not run.

Full changed-file review before commit included `rrd-query/src/syntax.rs`,
`rrd-query/tests/query.rs`, the two new SELECT test files,
`rrd-engine/src/engine/tests/query_admission.rs`,
`rrd-server/tests/http_process.rs`, and the updated query reference. The ARM
host `Clyffy-Ring` was reachable as Linux aarch64. Its non-interactive default
PATH did not contain Cargo; `$HOME/.cargo/bin/cargo --version` returned
`cargo 1.97.1`, so an exact-revision ARM test can be attempted after the branch
is pushed. No ARM test is claimed here.
