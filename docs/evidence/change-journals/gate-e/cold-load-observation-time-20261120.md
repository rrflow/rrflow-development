# Separate cold-load observation time from vector valid time

**Status:** active repair evidence; candidate and merge-queue checks pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/cold-load-observation-time-20261120`
**Owner:** OBJ-13 / [E-04](../../../roadmap/rrflow-1.0/gate-e.md) serving evidence and [POAM-026](../../../poam/rrflow-1.0-alpha/poam-026.md) diagnostics

## Brief and historical source

[PR448](https://github.com/rrflow/rrflow-development/pull/448) was opened with
its exact scope, consumers, failure oracle and stop conditions before editing.
Baseline main: `95cd44d86065e76c242ec5e5a7cc9266cb208044`; opening commit:
`87989f182cfa8e4532479e08d3604cf2b20a36a9`. Independent PR447 was in candidate
CI; this package changes none of its source or evidence paths.

The owner's historical recovery request led to review of closed, unmerged
PR315 head `d26ad75bed82fe529f804b5b0fee3c19f50b6c73`. Its retained
`runtime/coldlight.rs`, vector-residency source classification and query Cargo
dependency match their historical blobs. The current
[trigger journal](e-04-rrflow-cld-triggers-20261110.md) explains its split into
separate trigger and index-decoder packages. That is supported by the current
source comparison below, rather than inferred from Git ancestry.

| Historical behavior | Current canonical boundary and review |
|---|---|
| Cold mapped/owned/cache-bypass counters, warm sources ignored | Complete `rrd-engine/src/runtime/coldlight.rs`; exact historical blob, including its counter test. `runtime/vector_residency.rs::is_cold_source` and runtime exports remain. |
| One process-local owner and diagnostic projection | `engine/core.rs` state, both constructors and methods at lines 354-373; `engine/diagnostic/storage.rs` lines 1-42 and 273-280; contract `diagnostic/sections.rs` cold snapshot and `diagnostic/snapshot.rs` lines 1-65 retain the fields/invariants after module splits. |
| Both vector search paths report cold work | Complete `engine/vector/search/operations.rs`, with `search/results.rs` lines 43-100. General and active-generation paths retain source binding, exact rerank, resource counts and the shared read stamp. |
| Bounded decoded row retention | Complete historical query-executor patch compared with current `rrd-query/src/execute/read_path.rs` lines 266-565. The RawValue envelope, version/kind/coordinate validation, row-limit conditions, total-row accounting and skipped-tail parser remain. This does not establish new malformed-tail or performance coverage. |
| Public projections and owning prose | Historical contract/export changes and journal/POAM patch read; current cold fields retained. Generated-surface check at PR447 source `388a7bfd4` passed 83 HTTP operations and seven signal projections. It is not a claim that old API bytes should be restored. |

The prior fixture-only counter test did not assert the timestamp supplied by
an actual search. Both current search paths used `request.valid_at` as
`last_trigger_at_unix_ms`. This confuses historical data validity with the time
the operation observes a cold acquisition. The defect is retained old behavior,
not proof that PR315's implementation was missing.

## Repair at the existing owner

`search_vectors_at` now receives observation time separately from `ReadStamp`
and `SearchVectors`. Public direct search, hybrid retrieval, embed-and-search
and recursive retrieval pass their existing authorized operation time. Both
cold-load recording sites use that value. No wall-clock read, provider state,
wire field, persisted format, version bump or alternate search path is added.
Twenty existing internal test calls now supply a deterministic observation
time; their historical-stamp, result, budget and refusal assertions remain.

Files changed under `crates/authority/rrd-engine/src/engine/`:
`vector/search/operations.rs`, `retrieval.rs`, `inference.rs`,
`retrieval_query.rs`, `retrieval_query/stages.rs`, and
`tests/vector_index/{catalogue_reads,memory_tiers,turboquant_overlay}.rs`.
The existing memory-tier reference, POAM-026 and generated Gate E index link
this result. No implementation is removed or moved.

All eight changed source files were read, including the complete retained
historical-stamp and TurboQuant tests. Owners read: README/AGENTS/session and
change-authoring rules, Gate E, POAM-026 and the memory-tier reference. The
historical PR315 implementation patches and current destination ranges above
were examined; no complete repository review is implied.

## Failure and acceptance

Scratch: `D:/.usr/Temp/rrflow-selective-recall-audit-20261120`.
Use its existing checkout-specific `target` with `CARGO_BUILD_JOBS=4`.
No external research or upstream adaptation is needed for this local timestamp
confusion. Existing public storage diagnostics are the observation boundary;
retained test output supplies the debug trace.

Before changing production, extending
`engine_vector_memory_tiers_are_physical_bounded_and_restart_safe` to read
public storage diagnostics reproduced `Some(1)` versus expected `Some(701)`.
`cargo test --locked -p rrd-engine --lib engine_vector_memory_tiers_are_physical_bounded_and_restart_safe`
failed with exit 101 in 0.05 seconds. Log: `cold-load-time-before.log`.

After the repair, the same command passed in 0.08 seconds
(`cold-load-time-after.log`). The expanded oracle proves cached access does
not trigger cold counters; required-approximate cold access reports time 701
through public diagnostic readback; general-path access at valid time 2
reports operation time 703 and the same hits; reopen clears process-local
counters; the subsequent cold search returns the same hits and records time
800. It retains the original physical tier, pressure and restart assertions.

Before this repair, at `388a7bfd4`, the existing memory-tier fixture passed
without the new diagnostic assertions, and all eight `rrd-query --test
index_catalogue` tests passed. Logs: `pr315-memory-tiers.log` and
`pr315-query-index.log`. Those results cover existing query-index behavior,
not a new decoder speed or quality claim.

The full library command `cargo test --locked -p rrd-engine --lib` failed:
522 passed, one failed, 12 existing ignored in 417.42 seconds. The concurrent
serving reader reported absence during replacement at `tests/serving.rs:149`,
then its join failed at line 164. Log: `cold-load-time-engine-suite.log`.
This is not a passing package suite. One focused serving test passed in 2.66
seconds and all 15 serving tests passed in 1.71 seconds; neither explains or
repairs the full-suite failure. Logs: `cold-load-time-serving-focus.log` and
`cold-load-time-serving-suite.log`.

[Alarm449](https://github.com/rrflow/rrflow-development/issues/449) holds this
PR in draft. Serving source and its anchored filesystem were unchanged by this
package. POAM-092 records an earlier native replacement repair; this new
absence observation requires investigation and renewed evidence at that owner.
The timestamp repair must not be made ready by hiding or filtering that failure.

`cargo clippy --locked -p rrd-engine --all-targets -- -D warnings` passed in
32.70 seconds (`cold-load-time-clippy.log`). Navigation regenerated 21 indexes,
1,282 nodes and 4,510 edges with one index changed; the execution inventory
contained 3,691 files. Documentation checks passed 1,285 statuses and 1,281
coordinates (`cold-load-time-docs.log`). `cargo fmt --all -- --check` and
`git diff --check` passed. No gate or product maturity status advances.

During
editing, a guarded mechanical caller update expected 20 calls in one file,
found 19, and stopped before writing; the twentieth was in the separate
TurboQuant file. A subsequent write introduced CRLF in that file; its original
LF endings were restored without changing content. Neither is hidden as a
passing test result.

## Checklist and limits

The package brief preceded edits; scope and production consumers were traced;
the new public-path assertion failed before the fix; all existing tests remain;
source/diff review, journal, navigation, package checks and exact candidate/queue
checks are required before landing. Stop on an unexplained authorization,
result, read-stamp or diagnostic change. No roadmap or POAM status is closed.

This is real persistent-engine and public diagnostic evidence on Windows, not
a Connectome desktop comparison, native cross-platform qualification,
competitor benchmark, full cold-history implementation or proof that every
historical branch is reconciled. Continue the historical recovery after this
bounded diagnostic repair lands.


## Repair prerequisite and current-main integration

PR450 repaired the reproduced native-open false absence at the existing
filesystem boundary and merged as `fe3217570e46a03a8277de6e2376bdba65acb148`.
Candidate 44840099007 and merge-group 44840639640 passed. All six changed paths
match repair source `906d68422f859b89d5547851d90e84912eed7b7a` exactly;
`root-merged-450.bundle` verifies complete history. Its remote branch was
automatically deleted; its local branch was deleted after that comparison.
Alarm449 is closed. POAM-092's detail/ledger now record this renewed repair
without removing the original failure or PR442's earlier evidence.

PR448 ordinarily merged current main, including PR447 and PR450, producing
`1ab971781a32ed857ec242cef96e392309bc65b6`. The cold-memory focused oracle and
full engine package are being rerun on this combined revision; their results
are required before readiness. No passing rerun on unrepaired production is
being substituted for the failed run above.


## Combined revision results

On `1ab971781a32ed857ec242cef96e392309bc65b6`, with the same isolated
Cargo target and four build jobs, the focused cold-memory test passed one
case. `cargo test --locked -p rrd-engine` then exited zero: 591 passed,
zero failed and 12 existing ignored across 13 targets, including the full
523-test library result. Logs are `cold-load-integrated-focused.log` and
`cold-load-integrated-engine.log` in the session scratchpad. This passing
combined run follows the causal filesystem repair; it does not erase the
original failure. Documentation checks passed 1,287 statuses and 1,283
coordinates (`cold-load-integrated-docs.log`). Navigation regeneration made
no further index changes.

The same revision built the real CLI and Rust database example with
`cargo build --locked -p rrflow-cli --bin rrflow -p rrd-client --example rrflowdb`.
`python scripts/ci/run_rrflowdb_standalone.py` with those two scratch-target
executables exited zero: fresh installation, authenticated task/document and
relationship reads and writes, abort, independent wrong-state denial, offline
verification and reopened cursor 9. Primary SHA-256:
`769e0af7991e63a2e311ce6d2e5329828fb35bec78e0eee35e4ae5d483d0eea6`;
example SHA-256:
`629f6bfb7bae1aa2c273fb2f748e204fbbb844f76899abb52fe0556af6898e2a`.
Logs: `pr189-standalone-build.log` and `pr189-standalone-real.log`.
This is a small native Windows process corpus, not capacity, desktop UI,
cloud sync or full historical absorption evidence. Exact ready-candidate and
merge-group CI remain required before this package lands.
