# Independent document writes at one read stamp

**Status:** active implementation evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/disjoint-document-transactions`
**Owner:** [Gate C, C-02](../../../roadmap/rrflow-1.0/gate-c.md) and [POAM-071](../../../poam/rrflow-1.0-alpha/poam-071.md), OBJ-02.

## Change brief

Baseline `b08542f7ec32f942321b27877473b3bc9bf01edf`. Two installed, authenticated sessions began at cursor 7. The commit for document `left` advanced it to 8. The commit for `right` received HTTP 409. The public API, embedded store contention test and installed CLI campaign all reach the same semantic commit planner. The current store test even requires one disjoint writer to fail.

TX retains the begin stamp, operation identity and prepared intent. DB/KV validates the written identities and publishes the next contiguous log positions in one physical transaction. Record puts with unbounded valid-to and no governed receipts are the initial scope. Preserve conflicts on overlapping identity, including effective-time corrections and retirements, and preserve schema/catalogue constraints. All other operations retain their existing admission semantics. Preserve the original audit read across export, verify, restore and exact replay. QL, MX, LSM and CLD require no new engine or format implementation.

Read the actual transaction entry points, stamped commit preparation, version addressing, archive reconstruction and both existing concurrency tests. The source map is `RrdEngine::commit_transaction`, `prepare_semantic_commit`, `RuntimeRepository::commit_at_read`, `RuntimeCommitPager::next_commit`, `ReplayState::accept`, the store snapshot suite and the installed sustained campaign. These owners carry the replacement behavior.

Run the existing threaded MX/KV case against the new oracle, the full snapshot and archive suites, and the installed process campaign for distinct and overlapping writers after reopen. Check exact idempotent replay and original read stamps. Record the tested target and commands below. Stop on weakened constraints, ambiguous publication, altered commit identities, lost historical reads or failed recovery. No external research or benchmark comparison is needed to establish this correctness property.

## Implementation and verification

[PR 497](https://github.com/rrflow/rrflow-development/pull/497) keeps the original transaction digest, audit read stamp and prepared intent. In `access/semantic_commit/conflicts.rs`, the existing authenticated record-version reader selects each distinct target. Any publication after the begin cursor on that identity causes a conflict, including a tombstone or a correction with an earlier valid time. The current schema still enforces uniqueness. `repository/runtime/commits.rs` retries a definite physical write conflict against a fresh physical snapshot when another runtime commit took the log positions. It never retries an ambiguous I/O result or reruns an external function. Publication attempts use the existing repository contention bound.

Archive export previously reconstructed the content-addressed commit from its position in the log. Export and replay now distinguish the read position in the sealed audit from the contiguous publication positions. Restore authenticates the historical data stamp against the restored log and validates document conflicts. The source control catalogue is outside a logical runtime archive.

The inspected production paths included `transaction.rs`, `access/semantic_commit/{plan,conflicts}.rs`, `access/runtime_state.rs`, `repository/runtime/commits.rs`, `archive/{export,format}.rs`, and the installed sustained campaign. The diff carries the store/compiler-facing changes, fixtures, installed CLI oracle, public protocol and database references. Existing audit and readback receipts provide observability. The production path adds no instrumentation writes or per-record conflict-index writes.

Local Windows checks used `CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target` and four build jobs.

- `cargo test --locked -p rrd-store --test snapshot` passed the two-backend thread barrier, same-identity refusal and independent original audit reads.
- `cargo test --locked -p rrd-store --test logical_archive --test catalog_revision` passed five archive and two catalogue tests. The round trip preserved both stamps, distinct publication positions and byte-identical re-export.
- `cargo test --locked -p rrd-store --lib replay_rejects_unbound_or_future_read_cursors_even_with_a_resealed_audit` passed. `cargo test --locked -p rrd-engine --lib transaction_stamp` passed both tests.
- `cargo test --locked -p rrflow-cli --test installed_reopen -- sustained::campaign::installed_disjoint_writes_from_one_stamp_conflict_without_partial_effect --exact --ignored --nocapture` passed. Two sessions read cursor 16, published 17 and 18, and a third overlapping writer received HTTP 409. Historical preview excluded the first writer's new value. Both winning values survived forced termination and reopening. An exact retry returned the second receipt with its replay flag. The existing mixed-model refusal remains covered.

Eight independent Python SDK clients also issued concurrent HTTP commits through one barrier, with no retries. Each started from cursor 7 in a freshly installed scratch project and wrote one distinct document. A killed and reopened process independently queried the written keys. The previous executable (SHA-256 `7c2c933ed6d1c879d3141d76dbdbbb467f19857ecdffb87dbddbbd61a4f7dd1f`) acknowledged one of eight, returning seven conflicts. The candidate executable (`93f54bd743be217b169f1fd824ffb61616eee054696cfd2132bfd27bedd08cfc`) acknowledged all eight at cursors 8 through 15. Both readbacks matched exactly. The driver and raw receipts are retained on the development machine under `D:/.usr/Temp/rrflow-disjoint-497/`. This is one contention wave, with no sustained throughput or storage endurance claim.

POAM-071 remains open for mixed-model writers. No CLD codec, storage-format benchmark, cloud deployment or release qualification was changed or run.

The complete Windows `cargo test --locked -p rrd-store --all-targets` run passed 299 tests with four pre-existing ignored cases. This run included the backdated correction, retirement, catalogue, schema, unique-value collision and disjoint-update cases. `cargo fmt --all -- --check`, `python scripts/ci/check_code_shape.py`, generated navigation and `python scripts/ci/check_documentation.py` passed. The first public baseline error was HTTP 409 on disjoint identities. Local post-change checks found no new runtime error. The required remote candidate and queue results belong to the linked PR.
