# C-07: bounded installed transaction trace capture

**Status:** active draft prerequisite; no C-07 or alpha gate passed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-bounded-transaction-trace-capture-20261107`
**Owner:** C-07g, OBJ-10/11, [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md), [PR #295](https://github.com/rrflow/rrflow-development/pull/295).

The starting draft revision was `4b0228c3feceba3c6d9bda7c2b06361d4989cd11`,
tree `5365cb8f72929b17d81c32f46cbee399f608b43e`. The
[earlier diagnostic record](c-07-transaction-begin-diagnostic-20261107.md)
describes the transaction timing and the failed ARM run's limits. The PR brief
was amended before this change to specify a bounded redacting reader and its
installed-process consumer. The sustained driver does not yet enable it.

The existing process helper terminated a child when its captured stderr
exceeded 1 MiB. Enabling one event per transaction on an entire large workload
could therefore stop the process and also retain unreviewed stderr. The new
test-owned reader drains in fixed chunks and passes them to a private observer.
The observer parses bounded lines, retains only allowlisted transaction fields
and the slowest events within a caller-selected event budget, and reports
rejected, oversized, incomplete, dropped and failed-reader counts. A safety
ceiling of 1,024 events and a 16 KiB line limit bound memory; process ID and a
capture-local observer ordinal distinguish restarts. The ordinary untraced
capture, output limit and its failure semantics remain in use by default.
Opted-in kill observations explicitly identify redacted stderr.

Changed `crates/adapters/rrflow-cli/tests/installed_reopen.rs` and
`crates/adapters/rrflow-cli/tests/support/process.rs`; created
`crates/adapters/rrflow-cli/tests/support/transaction_trace.rs` and this
record. The generated Gate C evidence index is updated by the navigation
renderer. These are test-harness changes; no new public engine API, durable
state, retry behavior, workload budget or benchmark scoring is introduced.

Read the complete three source files above, the earlier diagnostic journal,
and the repository change-authoring routine; reviewed their complete source
diff. Reviewed the installed sustained driver entry and lifecycle boundary
where this capture will later be consumed. This is not an exhaustive codebase
review. The research and source decisions in the previous journal remain
applicable; no upstream code was copied. Trace output is diagnostic only:
request digests can be guessable, synchronous emission and pipe handling may
change timings, selection intentionally omits events, and a killed process
may lose data before it reaches the reader. Counters cannot prove events were
emitted or received. Raw stderr is discarded when this mode is selected, so
an unrelated startup error may have only a rejection count rather than a
diagnostic message.

On Windows with `CARGO_TARGET_DIR=D:\localdev\devlabs\rrflow\target\c07-diagnostic-trace-build`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, and
`CARGO_PROFILE_TEST_DEBUG=0`:

- `cargo fmt --all` passed.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen`
  passed **39**, failed **0**, ignored **14**. The installed executable was
  started, killed, reopened and replayed; sanitized created and replayed events
  were identified across the two readers. A separate child wrote 6,000 events
  totaling more than 1 MiB; the reader retained two and accounted for 5,998
  dropped events. A deliberately panicking observer failed the test-owned child
  and the test verified it was reaped. Malformed, foreign, oversized and
  incomplete lines were tested.
- `cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings`
  passed.
- `cargo check --offline --locked -p rrflow-cli --test installed_crash --features crash-test`
  passed, compiling the other integration target sharing the modified process
  helper. This is not a crash test execution.
- `python scripts/knowledge/render_navigation.py`, its `--check` mode,
  `python scripts/ci/check_documentation.py`, and `git diff --check` passed.

The source-built executable used by the installed test had SHA-256
`8ea85a3d72317fe8108b31fee3c3efd4aabf49ffa93fa9c4e6376e9b03155709`;
the test executable had SHA-256
`5cbe015883ea2b8b3c505ecbf5ba325b477208d182271572fc9046278dfa6ff4`.
These identify the local uncommitted test build, not a candidate or release.

The actual sustained driver still starts its child without the new collector.
After the ordered PR #294 dependency lands, integrate explicit
characterization-only admission, a single campaign-wide budget and sanitized
reporting through the existing lifecycle, test cleanup and denial for
qualification, and run a bounded installed characterization. Then compare
instrumented and uninstrumented overhead before interpreting latency.
The original ARM capacity failure, interrupted source visibility, full native
matrix, soak, candidate CI and merge-group CI remain unresolved. No roadmap
or POA&M status changed.

Change checklist: owner and actual driver identified; PR brief amended before
implementation; behavior and unchanged paths described; readback, saturation,
privacy and failure tests run; no external dependency or upstream source added;
whole changed sources and diff reviewed; this linked journal and generated
navigation record the result. The PR remains draft until its consumer and
dependency integration are complete.
