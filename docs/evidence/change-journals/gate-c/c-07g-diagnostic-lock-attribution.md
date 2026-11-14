# C-07 storage-diagnostic lock attribution

**Status:** active implementation evidence; original failure, candidate CI and native qualification remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-diagnostic-lock-attribution`
**Owner:** OBJ-02, C-07g, H-05 and POAM-035

## Brief and baseline

[PR178](https://github.com/rrflow/rrflow-development/pull/178) contained the brief before implementation and its scope additions before the HTTP integration test and operational documentation. Driver: this Codex session. Starting public `development/main` `89ec21eaa7fc129e6c532a3bb9f0ce1b57a9f3f9`; bootstrap `06f47f5b323bbb425fb695bbfa8386bac1c1171d`. The prior [Windows sampler result](c-07g-sampler-native-outcome.md) remains immutable: the public diagnostics call took 1.269 seconds during a failed 2-second sample-gap test. It retained no server-side lock-wait or profile time, so no cause is established.

Reviewed README, C-07 and POAM-035, the retained sampler outcome, `RrdEngine::read_storage_diagnostics`, the server router and diagnostic handler, `RrflowKvStore::physical_store_evidence`, `Memtable::profile` and mutation paths, existing storage/HTTP tests, CLI tracing setup, the native CI controller-fixture commands, and current PR171 status. No upstream implementation was copied. The [tracing enabled documentation](https://docs.rs/tracing/latest/tracing/macro.enabled.html) explains metadata-filter limitations; the implementation uses event/span-specific enabled checks and documented target filters. No default tracing, new storage authority, public operation or change in sampling threshold was added.

## Behavior and results

The opt-in server span hashes the request ID and contains a service elapsed-time event and success flag. The real rrflowKV diagnostic emits a second event with mutex wait, in-lock assembly interval, memtable profiling interval, key/version/segment counts. It does not emit payloads, credentials or raw request IDs. Clock reads are conditional on tracing. Event emission occurs after the database guard is dropped; the held-assembly interval ends just before drop and is not a measurement of release overhead. Events are observations, not committed audit records. The [operation guide](../../../operations/ci.md#c-07-storage-diagnostic-attribution) identifies the exact filters, interpretation, capture limitations and current sampler's reused ID.

Native Windows 11 Pro 10.0.26200 x64, rustc 1.99.1, isolated `CARGO_TARGET_DIR=D:/localdev/devlabs/rrflow/target/c07-diagnostic-trace-build`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_BUILD_JOBS=3`:

- `cargo test -p rrd-store --lib physical_diagnostic_trace_attributes_real_mutex_contention --locked -- --nocapture` passed. A real database mutex was deliberately held; one run measured `mutex_wait_ns=109215400`, `mutex_held_assembly_ns=363800`, `memtable_profile_ns=18300`, independently observed outer elapsed `109801200` ns. The captured event matched real counters, the measured components fit inside the outer elapsed, and the actual trace writer could acquire the database mutex, proving emission happened after release in this test. The same test also compares the trace-enabled snapshot to an ordinary untraced snapshot.
- `cargo test -p rrd-store --lib characterize_diagnostic_profile_at_observed_memtable_size --locked -- --ignored --nocapture` passed on an unoptimized test build: 19,440 keys, 38,880 versions, zero segments and no competing load. Three actual `memtable_profile_ns` values were 851500, 683800 and 717700; lock waits 1300, 700 and 300 ns. This small different fixture shows only that this scan under those conditions was sub-millisecond; it does not reproduce the original 1,980 segments, workload, installed binary or contention, and cannot establish the earlier RPC cause.
- `cargo test -p rrd-server --test http_process authenticated_storage_diagnostic_trace_correlates_store_without_secrets --locked -- --nocapture` passed against the real authenticated HTTP server and rrflowKV. The test captures a service event and a store event under the exact request digest, checks measured intervals against service time, and excludes the test request ID, API key and session token from captured text. The companion server unit test also passed.
- `cargo test -p rrd-store --all-targets --locked` passed. `cargo test -p rrd-server --all-targets --locked` initially failed the controller kill-matrix test because the isolated directory lacked `rrd-estate-controller`; after the CI-prescribed `cargo build -p rrflow-cli --bin rrd-estate-controller --locked` with `CARGO_PROFILE_DEV_DEBUG=0`, the full server suite passed, including all 19 HTTP and four local-driver tests. Both outcomes are retained here.
- A first server/store test using the shared `D:/localdev/devlabs/rrflow/target` returned no store trace event; rebuilding into the isolated target passed. The cause of the shared-build discrepancy is not established, and no claim is based solely on that build. The first documentation check found a broken relative link in this journal; it was corrected. `cargo clippy -p rrd-store -p rrd-server --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`, documentation, navigation and workflow checks passed. Inventory and diff checks run before commit.

## Checklist and outstanding evidence

- [x] Brief, exact scope, failure oracle and research/trace choices recorded before changes; original failed run retained.
- [x] Controlled real-mutex wait, real HTTP/store event correlation, no raw credential/request ID in test capture, counter preservation and post-unlock emission tested.
- [x] Narrow tests followed by affected store and server suites; initial missing-fixture failure recorded and corrected.
- [ ] Sampler request IDs remain reused; one-to-one trace correlation, native bounded loaded reproduction with trace collection, causal repair, exact integrated candidate CI, native capacity matrix, and actual 24-hour soak are not established. This work is not an optimization, complete C-07, or proof of end-to-end one-engine operation.
- [ ] Candidate CI and merge queue remain required for this PR. No objective, roadmap or POAM lifecycle status changed.

Changed `crates/persistence/rrd-store/src/rrflow_kv.rs`, its test-only dependency and `Cargo.lock`, the server diagnostic handler and HTTP integration test, operational CI guidance and POAM-035 detail; added this journal and regenerated navigation/inventory. No implementation was moved or deleted.
