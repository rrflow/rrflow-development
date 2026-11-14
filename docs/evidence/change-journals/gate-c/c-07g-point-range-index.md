# C-07g: indexed immutable point-read candidates

**Status:** active component implementation evidence; native full-profile and soak acceptance remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/c-07g-point-range-index`
**Owner:** [C-07g](../../../roadmap/rrflow-1.0/gate-c-07.md), OBJ-02 prerequisite; [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)

## Starting identity and brief

[PR141](https://github.com/rrflow/rrflow-development/pull/141) was opened with
the change brief before source edits. Public main was
`9a441efd5cbef924fc01f6cc47b6e6c5c844d4e6`, tree
`bd882e3734fbd96dd849454951fd345957438a3c`; opening commit was
`9d1945605ef309f73aef428cdbe2a0f4b655d730`. The current Codex session owns
implementation and handoff. This package has no source dependency on
PR134/139/140; final native qualification must integrate admitted repairs.

`Database::get` previously called `Segment::select_version` on every immutable
segment. Each segment checked its own bounds, but dispatch and metadata access
were linear in estate size. A private, derived balanced interval index now
visits only overlapping candidate ranges. It stores two ordinals per segment,
copies no keys or pages, and allocates nothing during lookup. Segment selection
still authenticates pages and chooses the newest visible sequence, including
tombstones. Equal sequences retain the first live-segment winner.

The index is rebuilt at open, flush, compaction and both snapshot installs;
publication changes the segment set and index together. It is never persisted
or a separate authority. Open charges construction to its existing recovery
deadline; flush/compaction check elapsed limits after construction and before
publication. Maximum retained index storage on a 64-bit target is 16 bytes per
segment, with old/new indexes briefly coexisting during publication. Worst-case
fully overlapping ranges still require all candidates; construction sorts
ordinals in O(n log n). No new public API, format, settings, default, deadline,
corpus or runtime dependency is introduced. `get_many` retains segment-locality
selection; this change does not claim to optimize every read path.

## Research and retained native diagnostic

The [RocksDB read-path description](https://github.com/facebook/rocksdb/wiki/MultiGet-Performance)
describes candidate-file selection before filter/page lookup. Its level-specific
layout is not copied: RRFlow permits overlapping segment ranges and retains its
own snapshot/publication authority. The implementation uses a general balanced
interval search over existing authenticated descriptors, with no upstream source
or dependency. [Per-operation performance counters](https://github.com/facebook/rocksdb/wiki/Perf-Context-and-IO-Stats-Context)
and [CPU profiling methodology](https://www.brendangregg.com/methodology.html)
informed collecting bounded deltas rather than treating cumulative counters as
causal proof.

The failed 04559542 estate remains unchanged. A full private copy at
`/home/devcontainers/rrflow-c07-native-79663cdd/window-readback-diagnostic-20261026`
was served with its original binary SHA-256
`be77466dffe12cd2eee91bc3f5996702089c69481fcefd26310064e40f33c5ca`.
The sibling `window-readback-diagnostic-20261026-evidence` retains intent,
result, server output and perf records. Only the diagnostic server was stopped.
It is not a resource-enforced qualification run.

For 128 vectors, 128 stamped record queries and 128 stamped relation queries
across four separated corpus windows, means were 10.171, 14.192 and 17.565 ms.
The diagnostic checked presence/row counts, not every independent value; it
does not substitute for the full corpus oracle. Counter deltas were 699 page
loads, 200709 cache hits, zero evictions, 42738 filter checks and 53 handle
reopens. The estate had 1477 segments. The CPU profile contained only 83
samples, zero lost; `Segment::select_version` was the largest named symbol
(8.43%). This supports investigating dispatch overhead, not a complete causal
or end-to-end speedup claim.

Local retained result `target/c07-window-readback-result.json` is 33716 bytes,
SHA-256 `b86b4f40a5e141a249a6718a3766dd0fa17104239a856f7046488b317dbaa736`.
`target/c07-window-profile.log` has SHA-256
`7cdcd8d6c9a6d8bfc291481e40dc2e9b0e2dc02652ae1796e8fa97239eb7e3d1`.
Neither file contains credentials. These operator artifacts are not build or
runtime dependencies.

## Change traceability and review scope

Source changed: `crates/persistence/rrd-lsm/src/database.rs`, specifically
create/open, `flush_memtable_inner`, both snapshot-install implementations,
compaction publication and `get`. Private module added:
`crates/persistence/rrd-lsm/src/database/point_index.rs`. Characterization and
lifecycle coverage remains in `tests/point_read_selection.rs`, extended with
independent expected histories across publication/reopen/import. Existing
`tests/hybrid_segment.rs` supplies the randomized independent MVCC oracle.
No implementation is moved/deleted; linear point enumeration is replaced at
the same canonical database boundary. No client or semantic-store authority
changes.

Complete new module and point-read test file were reviewed. Related source
inspection covered `Segment::select_version`, `SelectedVersion`, manifest
construction/order, snapshot-bundle tests, projected read-view composition,
and all database segment-set mutation sites. This is not a full-engine audit.
Owner reads were README, supplied AGENTS instructions, alpha objectives,
Gate C/C-07, POAM-035, execution map/change-authoring and the PR template.
Complete diff review includes all publication sites and ordinal lifetimes.

## Failure oracles and corrections

- Private index tests compare every candidate to exhaustive binary-range
  selection, including empty, nested, overlapping and duplicate ranges, and
  propagate candidate errors. The 1477-disjoint-range case checks 5909 lookup
  keys: maximum 19 visited nodes, versus 1477 descriptor dispatches in the old
  linear enumeration; retained index bytes are 23632. This is deterministic
  metadata-work evidence, not measured native engine throughput.
- Initial lifecycle testing incorrectly expected zero source-page ownership
  with the ordinary enabled cache. The new lifetime assertion now runs with
  cache bytes zero; cached behavior remains covered by existing tests.
- The next run exposed an implementation bug: sorted manifest ordinals did
  not match live segment order before reopen. Historical key `key-001` at
  snapshot 10 returned absent. Index construction and lookup were corrected
  to follow the live segment order.
- The first wider suite independently caught the same ordering requirement
  during successive flushes: `none_and_adaptive_lz4_match_the_independent_mvcc_model`
  failed seed `0x4d595df4d0f33173`, operation 96, before reopen. Flush must
  derive its descriptors from the live segment vector, not the previously
  sorted manifest. This was corrected; the new lifecycle fixture now includes
  consecutive publications without reopening to normalize order.

Local failed logs remain under `target/c07-point-index-integration.log`,
`-integration-after.log`, `-context.log`, and `-suite.log`. The first failed
suite exited 101 after 109.344 seconds; it did not proceed to lint. These are
development failures, not an accepted capacity result.

## Acceptance commands and limits

Commands ran on Windows with `CARGO_TARGET_DIR` set to the shared local
`target/endpoint-profile-build` and development/test debug information disabled.

| Command | Actual result |
|---|---|
| `cargo test -p rrd-lsm --locked database::point_index -- --nocapture` | 3 private index tests passed; 19 maximum visits, 23632 retained bytes. |
| `cargo test -p rrd-lsm --locked --test hybrid_segment --test point_read_selection` | 8 passed after live-order correction; 10.453 seconds including build. |
| `cargo test -p rrd-lsm --locked` | 229 passed, 0 failed, 3 ignored across 32 targets; 159.734 seconds including build. |
| `cargo clippy -p rrd-lsm --all-targets --locked -- -D warnings` | Passed after replacing the new test helper's complex parameter type with `ExpectedHistory`; final 0.672 seconds. |
| `cargo test -p rrd-lsm --locked --test point_read_selection` | Final helper-only follow-up: 3 passed; 3.313 seconds including build. |

The three ignored cases are two subprocess child entry points exercised by
parent tests and the explicitly opt-in encoder working-set characterization.
The package suite includes existing native kill/reopen and fault-history tests;
these are storage-component tests, not the installed full-capacity workload.
The initial strict lint rejected the helper type complexity. The documentation
checker initially rejected this new journal's missing active/historical marker
and noncanonical coordinate; both metadata fields were corrected. Exact
command outcomes are retained in `target/c07-point-index-final-checks.json`,
`-lint-fix-checks.json` and their named logs. Initial failures are retained,
not overwritten.

Final repository verification passed: navigation (17 indexes, 611 nodes,
2422 edges), inventory (1739 paths), documentation (612 statuses, 610
coordinates), workflow policy, workspace formatting and `git diff --check`.
The exact commands/results are retained in
`target/c07-point-index-repository-final.json`. Full installed
100k readback with this implementation, native comparative latency, other
capacity profiles and actual 24-hour soak were not run in this package; they
need a newly frozen integrated primary executable and harness. No prior
candidate's workload evidence is transferred to this changed binary.

## Related native blocker evidence

PR140's macOS installed-domain-crash job
[121584561114](https://github.com/rrflow/rrflow-development/actions/runs/40326533968/job/121584561114)
passed on candidate `b623bd441d5b83abefcae845f872688767eeab39`.
Its retained log `target/pr140-macos-passed.log` has SHA-256
`43d588ba04c29f64e18a8374bdc69faa5345f53fad63c478df7005f05181e13e`.
There are 32 native capacity receipts, each with zero unresolved calls and no
violations, covering four repetitions of fresh/existing WAL, segment, manifest
and current-pointer cases. All 32 data-fill phases report zero periodic
barriers. This proves that job's fixture/failure/recovery checks; the candidate
pipeline, full native capacity matrix and soak are separate qualifications.

## Checklist and handoff

- Owning objective, package, baseline, current/target behavior, scope and
  stop conditions were in the PR before source edits.
- Research, trace/resource ownership, replacement mapping, independent oracles
  and surfaced failures are recorded above.
- Preserve page/I/O counters and all authentication. Add no public signal
  solely for this private acceleration. Maintain bounded index metadata.
- Evidence/index/POAM linkage and repository checks must pass before commit.
- Stop on incomplete candidates, stale ordinals, incorrect MVCC/tombstones,
  hidden errors, altered workload limits or unproven completion claims.

C-07 and POAM-035 remain open. No objective or roadmap completion status
changes. The latest full-size frozen run still failed after 100000 ACKs and
64724 independent verifications; no actual 24-hour soak started. Next steps
are candidate CI, normal integration of admitted repair PRs, and a fresh frozen
installed corpus/profile run. Component evidence cannot clear that gate.
