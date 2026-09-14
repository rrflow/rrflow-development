# C-07 native batch-write tail attribution

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-batch-write-tail-attribution`
**Owner:** C-07/J-04 diagnostic prerequisite evidence; the roadmap and POA&M retain lifecycle authority

This record documents one bounded diagnostic package. It locates the measured
native authoritative-write delay without changing durability, promotion policy,
roadmap order, or release status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md),
[Gate C owner](../../../roadmap/rrflow-1.0/gate-c.md), and
[POAM-012](../../../poam/rrflow-1.0-alpha/poam-012.md) for acceptance and lifecycle
state.

## Package and baseline

- **Gate/package and prerequisite advanced:**
  `C07-diagnostic-01-native-batch-write-tail-attribution-v1`; adds the bounded
  phase and thread-resource evidence needed before selecting a native-write
  optimization. It advances only the diagnostic prerequisites of C-07, J-02,
  J-04, and POAM-012. It does not complete a gate or alpha objective.
- **Starting revision/tree:**
  `9d6b03f7d1ef60a4e9fe6d363d210e91e1817adb` /
  `b3a7abdfd17a53104bfb2203db65dbff8804f77f` on
  `agent/connectome-temporal-runtime-visualizer`, with a clean worktree and
  `Cargo.lock` SHA-256
  `316533e7512dbb9029e494386503ca8430b0c69853b3b78ef02319dcbab93a27`.
- **Planning commit:**
  `a97d527463e67674d9842f4eec35f3e5ac6c630d`, tree
  `29ae00e9cae53997fec2b3caa63ba1c1a2e2b56a`, whose sole parent is the starting
  revision. That planning-only commit contains only
  `docs/roadmap/rrflow-1.0-active-change.json`.
- **Development destination:** `development` resolves to the private
  `https://github.com/rrflow/rrflow-development.git`. Official `origin` is a
  promotion-only remote and is not authorized for this package.

## Change brief and traceability

Before this package, benchmark format 5 retained whole semantic-append latency
but no raw phase below it and no p99.9. A clean characterization found 257
physical mutations per semantic batch, no maintenance event, and one
`fdatasync` per authoritative batch, but could not separate WAL preparation,
write, sync, memtable work, allocation pressure, locks, or scheduling.

The package keeps the existing transaction and physical-write algorithm and
adds a process-local collector that is bounded and explicitly activated. One
accepted profiled write records semantic batch identity; begin and commit mutex
waits; validation, preparation, encoding, maintenance, WAL record/reservation/
write/sync, memtable apply, bookkeeping, and enclosing wall/thread-CPU times;
maintenance deltas; byte/sequence dimensions; and Linux calling-thread resource
deltas. The concrete `RrflowKvStore` supplies adapter lock context. The
provider-neutral `StorageEngine`, semantic repositories, WAL bytes, sequence
allocation, acknowledgement, durability, recovery, and maintenance policy do
not change.

Diagnostics remain off by default. The disabled path takes no diagnostic clock
or `getrusage` sample and allocates no diagnostic record. Collected records
contain no key, value, claim, prompt, project, provider, or user content. They
are replaceable benchmark evidence, not canonical state, telemetry authority,
or a schema of reasoning.

The package would stop if attribution required weakening `Authoritative`
durability, adding a second write path or lifecycle authority, changing a
storage or semantic contract, silently dropping evidence, making diagnostics
unbounded or always-on, changing Cargo/version/status/promotion policy, or
claiming causality that the retained measurements cannot support. None of
those conditions was crossed.

No implementation file was deleted, moved, merged, or replaced. The canonical
destinations remain `RrflowKvStore` for the semantic adapter, `Database` for
physical transaction authority, `WalWriter` for WAL publication and sync, and
the new `write_diagnostics` module for bounded measurement types and probes.

## Complete reads and changed paths

The planning record binds exact baseline digests, line counts, reviewed spans,
and symbols for every complete read. The author read in full: `README.md`,
`AGENTS.md`, alpha objective, canonical roadmap and Gate C/J owners, POA&M and
POAM-012, engine-data-flow architecture, execution portal and complete
change-authoring/implementation-navigation procedures, build/debug/trace/
optimization research, benchmark and historical-promotion records, persistence
scenario matrix, Gate C journal index, workflow, complete `rrd-lsm` library,
database, WAL, batch, memtable, transaction and owning tests, complete
`rrflow_kv` adapter, claim repository, benchmark example and bridge test, and
the change-plan/navigation/inventory generators. Every final changed file and
the complete staged diff are reread before commit.

Created:

- `crates/persistence/rrd-lsm/src/write_diagnostics.rs`
- `crates/persistence/rrd-lsm/tests/write_diagnostics.rs`
- this linked journal

Changed:

- `.github/workflows/rrd-lsm-benchmark.yml`
- `crates/persistence/rrd-lsm/src/{lib,database,wal}.rs`
- `crates/persistence/rrd-store/src/rrflow_kv.rs`
- `crates/persistence/rrd-store/examples/engine_benchmark.rs`
- `crates/persistence/rrd-store/tests/rrflow_kv_operator.rs`
- `docs/reference/storage/rrflowkv-benchmark-harness.md`
- `docs/evidence/test-plans/persistence-scenario-matrix.md`
- `scripts/ci/build_execution_inventory.py`
- generated `docs/evidence/change-journals/gate-c/README.md`
- generated `docs/roadmap/rrflow-1.0-file-plan.jsonl`

Moved or deleted: none. The four JSON measurement artifacts remain under
ignored `target/` storage rather than becoming source or lifecycle authority.

## Research, trace, resource, and debug decisions

Fresh primary documentation was required for the exact measurement semantics:

- Rust [`File::sync_data`](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_data)
  names the existing content-synchronization call. The package times it but
  does not assume that one observed duration universally proves device or
  directory-entry durability.
- Linux [`clock_gettime(2)`](https://man7.org/linux/man-pages/man2/clock_gettime.2.html)
  defines `CLOCK_THREAD_CPUTIME_ID`; RRFlow pairs calling-thread CPU with
  monotonic wall duration and does not treat their difference as a causal
  verdict by itself.
- Linux [`getrusage(2)`](https://man7.org/linux/man-pages/man2/getrusage.2.html)
  defines `RUSAGE_THREAD` resource counters. Page faults, block operations, and
  context switches are supporting coordinates, not standalone proof of lock,
  scheduler, allocator, or device causality.
- Linux [`fsync(2)`/`fdatasync(2)`](https://man7.org/linux/man-pages/man2/fdatasync.2.html)
  constrains interpretation of the blocking content/required-metadata
  durability boundary. It does not authorize removing or relaxing the sync.

No upstream engine code, tree, type surface, format, or compatibility contract
was adapted. Runtime tracing is preserved unchanged: the package adds no
always-on span or high-cardinality event. Resource/debug evidence is added only
to the explicitly enabled in-process collector and format-6 benchmark artifact.

## Failure-first evidence and corrections

- The supplied historical format-4 failure could not be reproduced as current
  authority because its exact artifact is absent. It remains historical input,
  not a reconstructed promotion result.
- A clean format-5 characterization retained whole-append latency and zero
  flush/stall/oversized counters but could not locate the tail. `strace -f -c`
  on one trial observed 129 `fdatasync`, 128 `writev`, 21 `fsync`, and one
  `fallocate`, consistent with one authoritative sync per semantic batch plus
  setup/publication syncs.
- The first focused diagnostic test failed to compile on the deliberately
  absent measurement types and activation/take methods. Implementation then
  supplied that contract at the planned boundaries.
- The first adapter bridge and benchmark-format oracles likewise exposed the
  absent concrete activation bridge, p99.9, raw pairing, and phase/resource
  summaries before implementation.
- Initial implementation compilation exposed missing imports/methods in the
  test and one Python module-loading mistake in an auxiliary inspection. Both
  were corrected without changing scope or product semantics.
- Strict Clippy rejected an obfuscated conditional; it was rewritten directly.
- The first workspace architecture run rejected only the two new untracked
  Cargo target files. Staging those declared files made the clean-checkout
  source guard pass; no scope was added.
- A final sequence-continuity assertion initially lacked an explicit
  `Option<u64>` type. The type was made explicit, and all four benchmark tests
  passed.
- An attempted shell cleanup command was rejected by tool policy before
  execution because it contained `rm`; it caused no repository or artifact
  mutation and was not bypassed with a destructive alternative.

No correctness, WAL-format, recovery, or semantic-storage defect surfaced.

## Controlled measurement result

Both final x86_64 Linux runs used release builds, nine isolated trials,
16,384 semantic operations per trial, batches of 128, 2,048 measured reads of
width 64, and exact corpus/reopen verification. They are local diagnostic
evidence, not fixed-hardware J-04 qualification.

| Measurement | Diagnostics disabled | Diagnostics enabled |
|---|---:|---:|
| median trial write throughput | 98,356.660 ops/s | 91,320.206 ops/s |
| aggregate batch p50 | 1.342 ms | 1.330 ms |
| aggregate batch p95 | 1.529 ms | 1.636 ms |
| aggregate batch p99 | 1.582 ms | 1.704 ms |
| aggregate batch p99.9 / maximum | 1.876 ms | 1.847 ms |
| raw diagnostic samples / drops | n/a | 1,152 / 0 |

Enabled/disabled throughput was 0.9285 in this sequential pair. An earlier
pre-final characterization pair on the same unchanged write algorithm produced
a ratio near 1.004. That run-to-run spread prevents an exact instrumentation-
overhead verdict; diagnostics are therefore still opt-in and no performance or
promotion threshold is changed.

Across the retained enabled samples:

- exact batch cardinality, ordinal pairing, trial-local contiguous physical
  sequences, WAL-frame sizing, phase containment, and raw/summary derivation
  all validated;
- physical total wall p50/p95/p99/p99.9/max was
  0.919/1.258/1.368/1.725/1.790 ms;
- WAL sync p50/p95/p99/p99.9/max was
  0.636/0.967/1.066/1.220/1.352 ms;
- WAL sync was the largest measured phase in 1,152 of 1,152 samples, its wall
  duration correlated 0.9881 with physical total wall duration, and its share
  of physical time was 69.70% at p50, 77.46% at p95, and 79.71% at p99;
- summed WAL-sync off-CPU time was 99.94% of summed physical off-CPU time;
- begin mutex wait p99/max was 0.205/2.446 microseconds and commit mutex wait
  p99/max was 0.235/0.352 microseconds;
- only nine initial-reservation samples were nonzero, one per isolated trial;
  all 1,152 samples recorded zero automatic flushes, write stalls, failed
  flushes, oversized batches, automatic compactions, and failed compactions;
- Linux thread-resource totals were 20,161 minor and zero major faults, 72
  block-input and 139,392 block-output operations, 3,457 voluntary context
  switches, and four involuntary context switches.

The supported conclusion is narrow: on this workload and host, the dominant
measured physical-write delay is inside the existing authoritative
`sync_data` call, not adapter mutex contention, automatic maintenance, batch
encoding, memtable application, or initial reservation. These probes cannot
separate filesystem/device flush behavior from blocking or scheduling while
the thread is inside that syscall. The previously reported 45--126 ms extremes
did not recur, so this package does not claim their cause.

Artifact digests:

- disabled sustained:
  `b0de6cc915f3745c840d2dccbb10877a55f184ece121557da07409dee6b40e65`
- enabled sustained:
  `7b421a7deee987012ce7fd956d591767cdde4c4c28e55c983da3bfb1b39e6b8f`
- disabled smoke:
  `203415835fac39b4bea5ab47d8f26167810a176d6c4e4dedd8d1b739ea339f6d`
- enabled smoke:
  `4afdaf69b0e93bf574a00ec403c95da97e2d6f10b9ff4db6ef9045e554e0ef38`

## Acceptance evidence

Focused and package results completed before final repository regeneration:

- `python3 scripts/ci/check_change_plan.py` accepted the implementation path
  set before journal/projection closeout; the final result is recorded below.
- `cargo test -p rrd-lsm --test write_diagnostics --locked` passed 3/3.
- the exact `rrflow_kv_operator` diagnostic bridge test passed 1/1.
- `cargo test -p rrd-store --example engine_benchmark --locked` passed 4/4.
- both three-trial smoke commands exited zero; the prescribed Python
  consistency assertions passed.
- `cargo test -p rrd-lsm --all-targets --locked` passed all library,
  integration, and example targets, including 3/3 diagnostic tests.
- `cargo test -p rrd-store --all-targets --locked` passed all library,
  integration, crash/reopen, and example targets, including 3/3 adapter tests
  and 4/4 benchmark tests.
- `cargo clippy -p rrd-lsm -p rrd-store --all-targets --locked -- -D warnings`
  passed.
- `cargo fmt --all -- --check` passed.
- both final sustained commands exited zero, correctness verified, and the
  enabled run retained every expected sample with zero drops.

Final repository results:

- `python3 scripts/ci/check_change_plan.py` accepted the planning commit and
  exactly 15 post-plan paths.
- documentation policy accepted 263 statuses, 261 classified coordinates,
  parent indexes, ownership, and local links.
- navigation check accepted 14 generated indexes, 262 nodes, 1,090 edges, and
  zero drift; all six navigation tests passed.
- execution inventory check accepted 1,148 current, generated, and planned path
  records; all 11 knowledge-export tests passed.
- version policy retained `1.0.0`, and `git diff --check` passed.
- `cargo test --workspace --all-targets --locked` passed every workspace target,
  including the 28 architecture guards and the complete new diagnostic corpus.

## Completion checklist and remaining limits

- **Full-file reread and diff review:** complete after final generated output.
  Every one of the 15 final paths, including the full 1,905-line inventory
  generator, 1,148-record generated file plan, this journal, and the complete
  staged diff was reread. The review found no undeclared path, diagnostic
  content capture, second write path, contract drift, durability change, or
  status/version/promotion change.
- **Change checklist:** baseline, authority, scope, complete owner/source reads,
  first oracle, traceability, research, edit order, observability,
  implementation, focused/package tests, and sustained characterization are
  complete; generated checks, final workspace regression, and final reread are
  complete. The result commit and development handoff occur after this record
  is staged.
- **Checks not run:** no fixed-hardware or cross-platform qualification,
  production-device trace, power-loss/ENOSPC campaign, competing-writer or
  mixed AI-research workload, continuous soak, installed-product lifecycle,
  SDK/API/MCP/Connectome conformance, signed bundle, deployment, or official
  promotion test is claimed. Those are outside this bounded diagnostic package
  and remain owned by their roadmap/POA&M records.
- **Broad workspace process test:** the final workspace run's
  `local_estate_driver` binary passed four synthetic process integration tests
  in 225.88 seconds. They cover child survival, forged-PID refusal, bounded kill
  fallback, and a 12-boundary controller crash matrix. Live inspection located
  the wall time in executable authentication, not the recovery state machine:
  this debug build produced a 1,185,079,112-byte `rrd-server` and an
  840,112,920-byte controller. Test-catalog initialization hashes the entire
  server once while the other concurrent tests wait on a shared `OnceLock`;
  every running-phase `LocalProcessDriver::deployment` then streams that same
  1.185-GB executable through SHA-256 in 64-KiB chunks. Repeated controller
  launches in the crash matrix repeat the authentication. Live controller
  processes consumed one full CPU for tens of seconds with the server
  executable open. This is valid tamper-detection coverage against a
  pathologically large debug artifact, but its duration is a test-harness/build
  deficiency and is not a storage benchmark, real-world AI workload, or
  performance evidence for this package. Repair requires a separately planned
  test/package boundary; this storage diagnostic does not edit the unrelated
  estate implementation.
- **Remaining known errors:** native write throughput has not passed the strict
  Fjall comparison retained by the owning evidence; the exact source of time
  spent inside `sync_data` is not separated; the historical extreme tail was
  not reproduced; the estate process-integration suite repeatedly hashes a
  1.185-GB debug executable and lacks per-test/per-boundary duration and retry
  reporting; C-07, J-02, J-04, POAM-012, D-01, every alpha objective, alpha
  readiness, and release promotion remain open.
- **Roadmap/POA&M status change:** none. Version remains `1.0.0`; no checkbox,
  threshold, promotion rule, objective status, POA&M lifecycle, or release state
  changes.
- **Commit/development push evidence:** pending final result commit. No official
  remote, tag, binary, artifact, release, force push, or history rewrite is
  authorized or performed.
