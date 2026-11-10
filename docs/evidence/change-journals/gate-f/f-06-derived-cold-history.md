# F-06 derived cold-history publication and readback

**Status:** active draft; F-06 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journals/gate-f/f-06-derived-cold-history`
**Owner:** F-06 package driver

## Scope and authority

This draft advances a prerequisite of OBJ-05, OBJ-07, F-06 and POAM-004. The
starting public main was `ffbb91f6bf06cee7f0fde6004f3e3d66bded4a36`; draft
[PR #269](https://github.com/rrflow/rrflow-development/pull/269) contains the
change brief and its refinements. Identity PR #266 overlaps the contract,
query, and engine boundaries and must land before this candidate is refreshed.
The existing rrflowKV WAL, runtime change history and control journal retain
authority. No source history is reclaimed by this package.

The baseline had bounded uncompressed query export but no background Parquet
history publication or authenticated cold-generation read. This draft adds an
opt-in installed policy, bounded source-page selection, compressed Parquet
encoding, source/policy fingerprints, a prepared control record, anchored and
synced immutable publication, a receipt chain, and authenticated status and
generation readback. The cold policy sets an offer interval; native and cold
offers share one coalescing worker, and missed or busy offers are skipped.
A stale source stamp or changed policy clears only pending
work; an ordinary later write does not invalidate the original captured page.
Status without a generation describes the recorded head and does not verify an
artifact. Selected-generation reads verify the receipt chain and file.

The affected owners are `rrd-store` runtime change paging and native admission,
`rrd-query` Parquet encoding/decoding, `rrd-engine` maintenance and governed
control records, `rrd-contract`/`rrd-server`/`rrd-client` public readback,
their generated SDK projections, and installed lifecycle evidence. No provider
or external service is needed. Changed and generated paths are mechanically
available in the PR diff; the work does not delete or move an implementation.
The changed policy remains restart-bound rather than hot-applied.

## Design and review decisions

The [Parquet format](https://parquet.apache.org/docs/file-format/) places
column chunks before the footer. Publication therefore syncs the completed
artifact before advancing its durable receipt, and does not equate an existing
partial file with a committed generation. [PostgreSQL WAL guidance](https://www.postgresql.org/docs/current/wal-intro.html)
supports retaining the durable log before considering data-page recovery.
[Parquet page indexes](https://parquet.apache.org/docs/file-format/pageindex/)
are optional; these tests do not establish predicate pushdown or byte skipping.
Tokio documents [zero-period rejection and the immediate first tick](https://docs.rs/tokio/1.53.1/tokio/time/fn.interval.html),
[skipped missed ticks](https://docs.rs/tokio/1.53.1/tokio/time/enum.MissedTickBehavior.html),
and [cancel-safe interval ticks](https://docs.rs/tokio/1.53.1/tokio/time/struct.Interval.html#method.tick).
Those facts inform the scheduler; they do not establish fairness or latency under load.
No upstream source was copied. The configured byte ceilings are accounted
bounds, not measured process RSS: native point-read can materialize one record
before its size is known and Arrow/Parquet buffers require separate measurement.

Read in full for this review included the repository change-authoring procedure,
the POAM-004 detail, the new cold-history contract, and the engine maintenance
entry point. Relevant portions of the roadmap, implementation, installation
driver, API dispatch, existing tests and generator sources were inspected.
This record makes no whole-repository or every-line review claim. Runtime
control receipts carry source/configuration identities; content is excluded
from maintenance warnings. A bounded failure class is persisted when control
storage is available; detailed measurements and reporting when that storage
is unavailable remain open.

## Local evidence and surfaced failures

On Windows x86-64, using `CARGO_TARGET_DIR=D:/localdev/devlabs/rrflow/target/f06-cold-build`:

- `cargo check --offline --locked -p rrd-engine -p rrd-server -p rrd-contract --tests` passed.
- `cargo test --offline --locked -p rrd-store --test cold_history_paging` passed one scoped paging/reopen test.
- `cargo test --offline --locked -p rrd-query --test cold_history --test parquet_export` passed four tests, including compressed stamped readback and corrupt/limit rejection.
- `cargo test --offline --locked -p rrd-engine --lib engine::cold_history::tests` passed five tests, including actual maintenance entry, replay, source and policy changes, authenticated readback, reopen, corrupt-file rejection and durable deduplicated failure/recovery status.
- `cargo check --offline --locked -p rrd-client --tests` passed.
- `cargo test --offline --locked -p rrflow-cli --test installed_lifecycle configured_cold_history_publishes_and_reads_after_process_restart -- --nocapture` passed: the installed server published a configured generation and returned it through the client before and after forced termination and restart. This does not inject termination within publication.
- After correcting the shared fixture, `cargo test --offline --locked -p rrd-contract` passed; `cargo test --offline --locked -p rrd-engine --lib` passed 336 tests with 4 ignored.
- `python -X utf8 scripts/ci/check_generated_surfaces.py --write` and `--check` reported 76 HTTP operations and matching SDK/signal projections; this is catalogue parity, not proof of every SDK behavior.
- TypeScript `corepack pnpm check` passed generation, lint, typecheck and six SDK tests. Python `uv run --offline python scripts/generate.py --check` and Go `go run ./cmd/generate -check` passed. The installed-process test passed again after the OpenAPI digest update.
- `cargo test --offline --locked -p rrd-query` passed the query package suite before the subsequent cooperative source-deadline change. Two focused cold policy and request validation tests also passed.
- The final grouped run passed `cargo test --offline --locked -p rrd-contract`, the rrflowKV cold paging test, both query cold-history/Parquet tests, all five `engine::cold_history::tests`, and the installed lifecycle cold-history test after the generated Rust-client binding and response validation were added. The 336-test engine suite and complete query suite precede those final focused changes; they are not evidence for a later integrated main.
- `cargo clippy --offline --locked -p rrd-engine -p rrd-query -p rrd-store -p rrd-client -p rrd-contract -p rrd-server --all-targets -- -D warnings` passed on Windows.
- `cargo clippy --offline --locked -p rrflow-cli --test installed_lifecycle -- -D warnings` also passed.
- After adding configurable scheduling, the paused-clock cadence test and both server worker coalescing/shutdown tests passed. The five cold-engine tests and real Windows installed-process test passed with a 25 ms interval. Earlier full-package and lint results precede this scheduler revision.
- After updating the generated OpenAPI identity to `60d01903f277eede2c2a585d20477e213a64af19471a902df87fcfb24f89b99e`, a grouped Windows run passed the full contract suite, the scheduler and worker tests, all five cold-engine tests, and the installed publication/restart/readback test. Generated-surface parity and TypeScript `corepack pnpm check` also passed for that identity.
- The first post-scheduler `cargo clippy --offline --locked -p rrd-server -p rrd-engine -p rrd-contract -p rrd-client --all-targets -- -D warnings` failed because the scheduler test module preceded another item. Moving the module to the end fixed the violation; that command and `cargo clippy --offline --locked -p rrflow-cli --test installed_lifecycle -- -D warnings` then passed. The relocation did not change the scheduler implementation.
- Review found that encoder budget errors were mapped to a generic query error and therefore the wrong durable failure class. The engine now preserves its existing typed query-error conversion and maps an already elapsed cooperative deadline to deadline exceeded. `cargo test --offline --locked -p rrd-engine --lib engine::cold_history::tests` passed six tests, including a real-engine output-limit regression that verifies resource-limit status and no generation advance. Earlier installed-process evidence predates this error-classification correction.
- `cargo fmt --all -- --check`, `python -X utf8 scripts/ci/check_documentation.py`, `python scripts/ci/build_execution_inventory.py --check`, `python -X utf8 scripts/knowledge/render_navigation.py --check`, and `git diff --cached --check` passed on the local draft. The inventory was regenerated after the journal changed; an initial check reported drift before regeneration.

The first locked query test stopped because enabling the pinned Parquet zstd
feature changed its dependency closure; an offline Cargo resolution updated
`Cargo.lock`. A closure lifetime error in the row encoder was corrected before
the passing tests. Running the Python SDK generator outside its managed
environment failed because `ruff` was not on PATH; `uv run --offline` in that
SDK succeeded. These focused tests do not establish a
candidate or release qualification.

The first full contract-suite attempt failed the shared SDK conformance corpus:
its endpoint count was 75 while the new catalogue has 76 operations. The shared
fixture was updated; historical evidence hashes were not rewritten. The
existing application-backup family list excludes cold receipts and artifacts;
restored source can be reprocessed, but existing generation identities are not
preserved across that restore. This is an explicit open boundary.

Review also found that the source loop did not check its elapsed budget while
scanning; the draft now checks a cooperative deadline between entries and maps
an expired deadline to a typed error. Individual reads, stamp validation,
compression and sync cannot be forcibly interrupted by this check.
The predecessor record was also charged against the next page after it had
been dropped, which could prevent progress when both records fit individually.
It is now independently bounded and only its digest is retained for the page;
the real-store regression covers that boundary.

The original stamped page reader verified each local change digest and its
predecessor chain but did not bind the page entries to the captured accumulator
root. A regression that changed a non-head entry and recomputed its own digest
was initially accepted on rrflowMX: `cargo test --offline --locked -p rrd-store
--lib bounded_page_rejects_a_self_consistent_change_outside_the_captured_root`
failed its rejection assertion. The reader now checks an RFC 9162 inclusion
proof for every scanned entry against the read stamp before advancing. The
same regression passed first on rrflowMX and then on native rrflowKV; the
real-store scoped paging test and six cold-engine tests also passed. Inclusion
proof checks do not make a standalone scoped artifact a completeness proof.
The installed-process publication and restart/readback command passed again
on Windows after the proof change; it did not terminate during publication.
`cargo clippy --offline --locked -p rrd-store -p rrd-engine --all-targets --
-D warnings` passed after this change.
Proof-node reads add work per cursor; their actual storage-key and elapsed-time
costs under large logs and their peak memory use remain unmeasured. The source
deadline is cooperative and the validation counter saturates at its existing
`u16` representation.

## Open boundaries and checklist

The change brief is attached to draft PR #269 and identifies the owner,
before/after oracle, source and configuration identities, acceptance commands,
primary research, diagnostic choices, and stop conditions. The implementation
is local and uncommitted; its changed paths are represented by the working-tree
diff, and generated outputs remain projections of the operation catalogue.
The smallest real-store, codec, engine, server, and installed-process checks and
the surfaced failures are recorded above. The full changed-file reread and
complete diff review have not been completed; the draft must remain uncommitted
until that procedure is finished. The full codebase has not been reviewed.

Scoped source rows do not independently prove no eligible rows were omitted;
security/control audit streams are not archived. There is no hot/cold rrflowQL
planner, safe source reclamation, orphan cleanup, complete error measurement,
per-generation paging, allocator-level memory proof, native full-disk/crash
matrix, nonblocking storage admission, starvation bounds, measured scheduling
under load, cross-platform soak,
or benchmark against Fjall and other systems.
POAM-004 records the verified coverage and cleanup gaps without a status change.
The complete diff and each changed file must be reread, generated navigation and
inventory refreshed, the installed driver and owning suites run, and current
main integrated before the draft is ready. Candidate CI, merge-group CI and
merge are not yet evidence. No F-06, C-07, alpha or release gate changes status.

## 2026-11-09 absorption onto main

The 53-file implementation that sat uncommitted was committed (051921e2) and
merged with `development/main`, 232 commits ahead of its base. Twenty-three
conflicts: generated SDK projections and reference pages regenerated from the
merged catalogue (83 operations, OpenAPI digest b3f27207…); both the `parquet`
dependency and main's dev-dependency kept; main's `installed_lifecycle.rs` with
this package's `configured_cold_history_publishes_and_reads_after_process_restart`
case re-appended; the client's `ColdHistoryRead` validation beside main's task
worker and assessment impls; `rrflow_kv.rs` moved from `try_lock` to
`try_write` after main changed the database lock to a `RwLock`; the SDK
conformance corpus endpoint count set to 83. On the merged tree: public
contract 47 passed; `rrd-lsm --lib` 59 passed; engine cold-history tests 6
passed; the lifecycle case passed; clippy and fmt clean; generated surfaces
verified; documentation policy OK. A build-and-test absorption, not F-06
acceptance: no interruption matrix, soak or capacity claim is made.

2026-11-10: merged main `cc139164` (#278, #283, #288, #308) without conflicts (`1100a66a`). On that tree: public contract 47 passed; `rrd-lsm --lib` 59 passed, 2 ignored; engine cold-history tests 6 passed; the configured cold-history lifecycle case passed.

2026-11-10 candidate corrections after run 42750056590 (the first candidate run of this package): the `rrd-query` cold-history test used `note:one` as a runtime id, which main's canonical-id rule rejects; it is `note-one`. The sensitive-read closure names three new oracles (the engine cold-history test module's `read_changes`, the `rrd-query` stamped-page test and the `rrd-store` paging test). `configured_cold_history_publishes_and_reads_after_process_restart` is listed in the offline harness `REQUIRED_CASES` and its policy test. The storage goldens (`eval/schemas/storage-*.json`, `tests/fixtures/storage/*.json`) are regenerated with `RRFLOW_UPDATE_STORAGE_GOLDENS=1` because the public catalogue gained the cold-history operation and the acceptance validator pins the catalogue identity. On this tree: `rrflow-eval` suites passed; `rrd-query --test cold_history` passed; `direct_read_source_closure` passed; the cold-history lifecycle case passed; the offline harness policy test passed.
