# C-07g: share immutable read-view manifest generations

**Status:** active implementation evidence; candidate CI and native growth qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-shared-manifest-generation`
**Owner:** OBJ-02 / C-07d,e,g / POAM-035 / PR142

## Brief, baseline and implementation

[PR142](https://github.com/rrflow/rrflow-development/pull/142) described the
scope, first oracle, research, resource boundaries and stop conditions before
the original edits. This continuation updated that brief before integration.
Public development main `04fae27665c3081ef6bc193d54d8c1c9c8d3f2e0`, tree
`0effb6f859ddd0a99e24efd6016f80e06405cd40`, was merged into the original
draft at `b99a4216c6549acbbd8db340f111f7866eb85771`. Original source changes
remain in stash `a2004fd90840c8fd7061420e8b33f4b8bfed1a40`; it was not dropped.

Before, each `Database::begin_projected_read` copied the complete immutable
manifest, including unrelated segment descriptors. `Database` and the existing
read-view registry now share an `Arc<Manifest>` per immutable generation.
Publishing a new generation leaves old registered views alive until their lease
ends. Distinct allocations claiming the same digest still require equal content.
Snapshot/export interfaces that require owned values still copy at those
boundaries. Runtime paths changed: `crates/persistence/rrd-lsm/src/database.rs`
and `src/segment/reader.rs`; new tests are in the existing
`src/segment/reader_lifetime_tests.rs`. No path, data format, public operation,
dependency, transaction authority, pin admission or configuration limit changed.
Existing point-index initialization and updates remain at create/open, flush,
both snapshot installations and compaction. No source is removed or moved.

An isolated review copy applied the stash against current main. Six expected
conflicts in `database.rs` required retaining both point-index publication and
the shared generation; the resulting reviewed patch was applied to this branch.
That review copy is not a committed candidate and its tests cannot stand in for
this branch's CI. The original brief records a pre-repair allocation regression;
this continuation did not independently rerun that pre-repair test.

## Preserved native failure

Retained local files under `target/c07-indexed-failure-evidence/` were read
without editing the original estate. Campaign `170295519223ad438da894799c5f60767a1090db542bde5bf63c4edb23d7df7f`,
source `7c329d491cda5326858de0eab36a6f9912c492da`, acknowledged all 100000
corpus operations and independently verified 44556. The launcher reports exit
101, elapsed 7202526 ms and confirmed cleanup; stderr says corpus setup exceeded
its frozen deadline. No measured segment or 24-hour soak began. The original
two-hour limit, corpus and resource ceilings remain unchanged. Incomplete
verification is not evidence of a lost acknowledged write.

The retained `result.json` SHA-256 is
`1fb4b01a1403ce05e3a5454aa5706748bb5803aae74414d93d81e1d6a6f11e60`;
`progress.jsonl` SHA-256 is
`46de88f472b9ebaf061413d5f8e03078e6b04923d66188b58cad6353ea47bd47`.
An independent streaming count found 100000 intents, 100000 acknowledgements,
44556 verifications, one resource sample and one failure. The sample reported
1404 segments, 3228790960 segment bytes and 128 peak file handles with zero
handle denials. A single sample is not a whole-run resource peak. The copied
estate profiling described in the PR suggests an allocation target; it does
not prove that manifest copying caused the deadline failure.

## Research, review and evidence decisions

No upstream code or new dependency was adapted. The existing PR identifies
Rust's `Arc` ownership documentation as the relevant reference. Preserve the
existing lifetime/resource counters and native diagnostics; no new tracing
authority or instrumentation is needed for the allocation-sharing oracle.
Future native profiling must compare the frozen workload and independently
checked outputs, without exposing project contents or credentials.

Read: README bootstrap, C-07 owner and POAM-035, change-authoring procedure,
CI operations candidate policy; the complete candidate diff and lifetime tests;
database read-view capture, create/open, flush, compaction and snapshot install
publication sites; active-manifest admission/release and reclamation visit;
the current point-index implementation and tests; retained result/launch,
stderr, final progress entries and streaming progress counts. Other source
files were inspected only at relevant paths. This is not a whole-engine review.

## Checks and remaining work

On the actual integrated branch on Windows, with `CARGO_TARGET_DIR` pointing to
the isolated review build directory:

- `cargo test --offline --locked -p rrd-lsm --lib segment::reader::lifetime_tests`:
  five passed.
- `cargo test --offline --locked -p rrd-lsm --quiet`: passed, with ignored
  fixtures reported by the suite. This includes projected lifetime, snapshot,
  compaction, failure matrix, transaction scan and segment-handle tests; it
  does not claim privileged native capacity or soak acceptance.
- `cargo clippy --offline --locked -p rrd-lsm --all-targets -- -D warnings`:
  passed.

The independent review copy also passed the suite and strict Clippy, but does
not qualify this branch. `python scripts/knowledge/render_navigation.py` and
`--check` passed (17 indexes, 637 nodes, 2511 edges); `python
scripts/ci/build_execution_inventory.py` and `--check` passed (1768 paths);
`python scripts/ci/check_documentation.py`, `python
scripts/ci/check_workflow.py`, `cargo fmt --all -- --check`, and `git diff
--check` passed. Generated inventory is refreshed after this evidence update.

One exploratory Python progress-count command failed with a syntax error caused
by literal newline escapes; the corrected streaming count above succeeded.
The initial isolated review `git apply --check` failed on main's changed point
index; a three-way application exposed six expected conflicts and the reviewed
combination preserved both sides. A review-copy line-ending mistake was fixed
before testing and `git diff --check` passed. No native limits were changed.

Not run: exact-candidate CI, installed smoke and bounded native comparison,
full native/profile matrix and actual 24-hour soak. C-07 and POAM-035 remain
open; this change does not assert a speedup, clear the failed workload, or
qualify background project ingestion, planning, memory audit or alpha. The
complete PR, exact CI results and merge evidence remain pending.

Checklist: objective/owner and scope bound in the PR; retained failure and
original change preserved; current/source target and first oracle identified;
research and resource decisions documented; bounded tests, full diff review,
navigation and exact candidate CI required before landing. No roadmap status
change is made by this record.
