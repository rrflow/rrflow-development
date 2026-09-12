# C-06h rrflowKV adversarial qualification engineering plan

**Status:** active supporting work-package evidence; implementation committed at
`5d9a203816ac3b2ad045170ed57d751387756c6b`; C-06 remains open for the
unplanned C-06i package
**Coordinate:** `rrflow://rrflow-instance/data/work-package/c-06h-rrflowkv-adversarial-qualification`
**Owner:** executable qualification of the C-06g projected reader and existing rrflowKV recovery paths
**Authority:** subordinate to [`rrflow-1.0.md`](rrflow-1.0.md) and the machine-bound [`rrflow-1.0-active-change.json`](rrflow-1.0-active-change.json)

## Outcome

C-06h adds an independent, deterministic MVCC state-machine oracle shared by
ordinary integration tests, a configurable stress executable, and a locked
coverage-guided fuzz workspace. It attacks the accepted C-06g projected read
generation together with existing WAL, flush, reopen, compaction, garbage
collection, fault-injection, cancellation, and resource-limit behavior.

It changes no product API, file format, transaction authority, persistence
semantics, dependency closure, or DataFusion boundary. A failure that requires
production-code changes stops this package and requires a separate repair plan.

## Executable boundaries

| Path | Responsibility |
|---|---|
| `crates/persistence/rrd-lsm/tests/support/projected_read_model.rs` | Decode bounded operation programs; maintain an implementation-independent version history; compare point, range, multi-range, keys-only, and key-value reads; inject typed failures; retain exact replay coordinates and resource counters. |
| `crates/persistence/rrd-lsm/tests/projected_read_adversarial.rs` | Run fixed seeds plus directed all-boundary and stream-lifetime programs on stable Rust. |
| `crates/persistence/rrd-lsm/examples/rrflowkv_stress.rs` | Run configurable deterministic histories outside unit-test case budgets and print one stable evidence summary. |
| `crates/persistence/rrd-lsm/fuzz/fuzz_targets/projected_read_state_machine.rs` | Feed coverage-guided byte programs through the same independent oracle under libFuzzer and AddressSanitizer. |
| `crates/persistence/rrd-lsm/fuzz/fuzz_targets/segment_v4_open.rs` | Mutate the frozen authenticated segment-v4 fixture; accept only a typed open error or a segment that remains fully readable. |
| `scripts/check_version.py` | Keep product workspace/version parity exact while separately validating an explicitly marked, unpublished, `0.0.0`, self-contained Cargo-fuzz workspace below a declared product crate. |

The nested fuzz project is not a root-workspace member or runtime dependency.
Its own lockfile pins `libfuzzer-sys` and test-only dependencies; the root
`Cargo.lock` remains unchanged. The release-version checker does not broadly
ignore nested manifests: an unmarked, publishable, release-versioned,
mislocated, root-member, or non-isolated fuzz manifest fails policy.

## State-machine model

Each six-byte operation selects one bounded action over eight realistic key
families: audit/evidence, incoming edge, outgoing edge, record, runtime,
scalar, term, and vector. The model owns expected versions and sequence
visibility without reading implementation output. Operations include:

- single put/delete and atomic multi-family writes;
- flush, protected compaction, reopen, and garbage collection;
- retained-snapshot and current-snapshot differential verification;
- projected reads across complete and disjoint ranges with bounded batches;
- a pinned read interleaved with write, flush, compaction, and collection;
- explicit cancellation/drop and active-view release;
- write failures at prepared, WAL-appended, WAL-synced, and visible boundaries;
- flush failures at WAL-sync, segment-sync, successor-WAL-sync, and manifest
  publication boundaries;
- compaction failures at segment-sync and manifest-publication boundaries; and
- output-row and page-request resource exhaustion.

Unexpected storage errors, partial multi-family visibility, sequence drift,
snapshot drift, tombstone resurrection, duplicate/out-of-order projected keys,
incorrect terminal outcomes, retained view leases, or resource-limit bypasses
panic with seed, operation, input length, digest, and exact input bytes.

## Research adaptation

The design adapts testing behavior rather than source trees, formats, APIs, or
configuration surfaces:

- [RocksDB stress testing](https://github.com/facebook/rocksdb/wiki/Stress-test): combine randomized operations with an independent expected-state check and preserve replay inputs.
- [FoundationDB simulation and testing](https://apple.github.io/foundationdb/testing.html): make operation and fault schedules deterministic and replayable while not claiming simulated coverage for real devices or operating systems.
- [FoundationDB client testing](https://apple.github.io/foundationdb/client-testing.html): drive rare failure paths deliberately and verify post-failure state.
- [Rust Fuzz Book structure-aware fuzzing](https://rust-fuzz.github.io/book/cargo-fuzz/structure-aware-fuzzing.html): fuzz valid stateful operation sequences so coverage reaches recovery and compaction behavior.
- [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz): retain corpus/artifact replay and run the real targets with sanitizer instrumentation.
- [Loom](https://github.com/tokio-rs/loom): not used because this package's filesystem, mmap, process-failure, and io_uring behavior is outside Loom's model.

## Observed execution

The first compile exposed two stale draft fields in `SegmentIoPolicy`; the
oracle now uses `max_request_bytes` and explicitly denies backend fallback.
The first directed run exposed an incorrect report assertion; the report
correctly distinguishes nine successful writes from ten injected failures.
Strict Clippy then required the canonical divisibility method and a named
projected-row collection type. No production storage defect or format change
was required.

Observed bounded evidence on Linux:

- three adversarial integration tests passed, including 256 generated
  operations across four fixed cases and all ten injected boundary classes;
- the stress executable passed 1,536 operations, 283 injected failures, 382
  reopens, 189 compactions, 264 garbage collections, and 1,334 projected reads;
- the projected state-machine fuzzer completed 512 sanitizer executions with
  no crash, timeout, or oracle mismatch;
- the segment-v4 fuzzer completed 4,096 sanitizer executions with no parser
  panic or accepted-but-unreadable segment; and
- the complete `rrd-lsm` suite passed 94 tests and strict all-target Clippy;
  seven direct version-policy probes rejected malformed fuzz-package shapes;
  and the architecture, locked workspace compilation, documentation, workflow,
  version, inventory, knowledge-export, generated-surface, and formatting
  checks passed.

These bounded runs do not prove absence of defects, real-process crash
durability, ENOSPC behavior, cross-platform lifetime safety, steady-state
maintenance, performance, or competitor equivalence. C-07 and Gate J retain
those obligations.

## Reproduction

```text
cargo test -p rrd-lsm --test projected_read_adversarial --locked -- --nocapture
cargo run -p rrd-lsm --example rrflowkv_stress --locked -- --seed 14592251008053203194 --cases 16 --operations 96
cargo +nightly fuzz check --fuzz-dir crates/persistence/rrd-lsm/fuzz
cargo +nightly fuzz run --fuzz-dir crates/persistence/rrd-lsm/fuzz projected-read-state-machine crates/persistence/rrd-lsm/fuzz/corpus/projected_read_state_machine -- -runs=512 -max_len=256 -timeout=10
cargo +nightly fuzz run --fuzz-dir crates/persistence/rrd-lsm/fuzz segment-v4-open crates/persistence/rrd-lsm/fuzz/corpus/segment_v4_open -- -runs=4096 -max_len=128 -timeout=10
cargo test -p rrd-lsm --locked
cargo clippy -p rrd-lsm --all-targets --locked -- -D warnings
ruff check scripts/check_version.py
python3 scripts/check_version.py
cargo test -p rrd-engine --test workspace_architecture --locked
cargo check --workspace --all-targets --locked
```

## Handoff

C-06i is the only remaining C-06 slice. It must compare revision-bound
compression, value placement, persisted filters, mixed-family interference,
and cache policy on fixed hardware, retaining only measured improvements. It
must not change correctness semantics to improve a benchmark. After C-06
closes, D-01 builds the first installed primary `rrflow`/`rrflow.exe` walking
lifecycle; F-01 later adapts the qualified projected stream into stamped,
bounded DataFusion `RecordBatch` execution.
