# Reject truncated parser worker frame headers

**Status:** active bounded parser repair; candidate and merge-queue validation pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/parser-frame-eof-20261120`
**Owner:** D-04 in [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md)

## Change brief and source map

Baseline: `e708980de22028d09fcd12c7410bf560c79d3596`.
Driver: the ongoing historical-code recovery session.
Pull request: [445](https://github.com/rrflow/rrflow-development/pull/445),
opened with this scope before implementation. The separate PR444 owns the
recovery inventory and Connectome pin; no child gitlink is changed here.

The recovered stash `c53cf21040ec500c280b5d0e476311ec6da31117` includes a
single-request parser worker in its third-parent untracked tree. That worker
rejects truncated frame prefixes through `read_exact`. The current batch worker
in `rrd-attunement/src/worker.rs::run_worker` treats every `UnexpectedEof` while
reading four header bytes as clean shutdown, including after receiving one,
two or three bytes. This silently reports successful termination on a truncated
request. The current CLI maps this result directly to its process exit status.

Read one header byte to distinguish clean EOF at the boundary, then require the
other three with `read_exact`. Keep the current batch protocol and strict header,
source identity, body-size and response checks. Empty input remains a clean
shutdown. A partial header fails without generating another response; responses
to earlier complete requests remain intact. Standard interrupted-read retry
behavior and fragmented input are exercised explicitly. No source files are
opened or durable state written by the worker.

Scope: the existing worker function, its protocol tests, actual primary-binary
parser tests, this journal/index and POAM-006 evidence. No new dependency,
fallback, compatibility protocol or runtime abstraction. Stop on regression in
batching, malformed-body denial, fixed error output or request-bound identity.

Read: historical `parse.rs`, recovered worker and worker/process tests; current
`parse.rs`, `worker.rs`, `tests/worker.rs`, CLI `tests/parser_worker.rs`, CLI
dispatch in `main.rs` lines 32-46 and `facts.rs` lines 1-182; README, D-04,
POAM-006, session standard and change-authoring procedure. The full remaining
parser/installation stash is not claimed absorbed by this repair. No external
research is needed; the first-party source and reproducible protocol failure
establish the requirement. Existing process exit/output supplies debugging
evidence, so no source-bearing runtime diagnostics are added.

## Before and after oracle

Before edits, the actual checkout debug executable with SHA-256
`bf7b95e06ba92eea62f6a97ffb1e021c6f466b4be54420eb902a882421be0a4d`
returned exit 0 for empty input and each of `00`, `0000`, `000000`; it returned
exit 1 for `ffffffff`. Every output stream was empty. The partial-prefix cases
are the defect; clean boundary EOF is intentional.

After adding the test but before changing the worker,
`cargo test --locked -p rrd-attunement --test worker
worker_rejects_partial_headers_at_startup_and_after_a_response` failed with
exit 101: `unwrap_err` received `Ok(())`. This is a behavioral assertion failure,
not a missing symbol or failed compilation. After the repair, the full worker
target passed all nine tests, including partial headers before/after a response,
interrupted one-byte reads, empty shutdown and the existing two-request batch.

Scratch: `D:/.usr/Temp/rrflow-selective-recall-audit-20261120`.
Logs: `parser-prefix-before.json`, `parser-frame-before.log`,
`parser-frame-worker.log`, `parser-frame-attunement.log`, `parser-frame-cli.log`.
Four Cargo jobs use the existing root `target` in that scratch directory. No
cache, original file, user process, branch or stash is deleted by this repair.

Windows acceptance on the repaired tree:

- `cargo test --locked -p rrd-attunement --test worker`: nine passed.
- `cargo test --locked -p rrd-attunement`: 63 passed across facts, Git inventory,
  filesystem inventory, parsing and worker targets; no failed or ignored tests.
- `cargo test --locked -p rrflow-cli --test parser_worker`: all three actual
  primary-executable tests passed, 0.96 seconds. Each partial-header length is
  rejected both at startup and after a complete response; empty input succeeds.
- `cargo clippy --locked -p rrd-attunement --all-targets -- -D warnings`: passed.
- `cargo fmt --all`, `git diff --check` and `check_documentation.py`: passed.
  Navigation and execution inventory regenerated the Gate D journal link and
  3,687 path records. Candidate and merge-group acceptance remain pending.

This small protocol repair does not establish
whole-project isolation, installed source indexing, performance or complete
historical stash absorption. POAM-006 and D-04 remain open.

## Change checklist

The pre-edit PR brief binds outcome, current and target behavior, exact symbols,
source provenance, smallest failure oracle, acceptance commands, resources,
debug decision and stop conditions. Created: this journal. Changed: canonical
worker, protocol/process tests, POAM-006. Generated: nearest journal index.
Complete changed-file and diff review precedes commit. No roadmap status change.
The next recovery action is reconciliation of the remaining parser source facts
and installation/worker stash behavior at their existing owners.
