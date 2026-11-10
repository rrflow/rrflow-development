# D-03 background indexing research and main reconciliation

**Status:** active draft integration evidence; interrupted-refresh repair remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-background-indexing-research-and-main-refresh-20261106`
**Owner:** D-03 and [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md); [PR 252](https://github.com/rrflow/rrflow-development/pull/252) owns this candidate

## Brief, source and scope

The PR brief was amended before reconciling draft head
`f43d7bc152dab0e7f0e046e7618a02217d0cf578` with public main
`1fd7f163de3448c694e09c916b328540303eab1c`. Objective: carry the existing
interrupted-refresh repair toward one real project indexing flow, not create a
new index authority or claim that the repair exists. The ordinary merge
conflicted in `rrd-core/src/authenticated_log.rs`. Main's byte-based
`digest::from_hex` accepts the same canonical lowercase hex and rejects invalid
multibyte inputs without slicing; the resolution retains it alongside the draft
generation-proof tests and main's independent digest tests. The draft's test
fixtures now use main's canonical project/subject IDs and its typed event-ID
constructor. No generation selection is wired to a production consumer.

Read the installation preview/admission and inventory walk, source proposal,
engine Inventory publication and prior-record reader; the PR 252 history,
existing interrupted-refresh oracle and POAM-006; the project-indexing plan,
Gate D/H owners and the change-authoring procedure. This is a focused source
review, not a whole-codebase review. Current preview hashes admitted content,
apply validates that binding and Inventory later re-reads and rechecks it;
the walker retains and sorts entries and does not claim an atomic OS snapshot.
The source rechecks enforce the current contract. A faster preview needs an
explicit narrower installation/effect binding and its own acceptance tests,
not removal of checks or a claim that an unverified scan is complete.

## Research and design decisions

[SurrealDB](https://surrealdb.com/docs/reference/query-language/statements/define/indexes)
supports background initial index builds and status. Its
[3.2 release history](https://surrealdb.com/releases/3.2) describes resumable
scan checkpoints and a later fix for write contention and lease expiry that
could prevent progress. [Qdrant](https://qdrant.tech/documentation/search/low-latency-search/)
documents a visibility/latency tradeoff and temporarily disappearing results
when indexed-only queries omit parts being optimized. [PostgreSQL](https://www.postgresql.org/docs/current/sql-createindex.html)
documents extra scans and waits for concurrent index builds. The Rust
[notify](https://docs.rs/notify/latest/notify/) documentation describes missed
events, unsupported filesystems and polling. None is proof of RRFlow performance
or a reason to copy an upstream runtime.

The implementation target is a bounded durable backlog with per-scope source
identity, partial coverage and explicit freshness; work resumes or reconciles
after restart and continues under configured resource limits. Notifications
are hints and a lost/overflowed watch requires reconciliation. Reads must not
mix old completed heads with partially published new facts or silently claim
complete/current results; authorization and deletion changes fail closed at
the relevant boundary. Source effects stay under RrdEngine and existing store
transactions. Compare the existing atomic control/read-stamp substrate against
the actual omission threat before adding a second authenticated index: the
draft's isolated proof module has no production consumer and passing its tests
does not establish a necessary design or a repaired workflow. Trace progress,
backlog, lag, work and errors through existing bounded signals and receipts;
no signal or resource policy was implemented in this reconciliation.

## Checks and limitations

The first `cargo test --offline --locked -p rrd-core` on Windows failed five
generation-proof tests because their old `project:...` identifiers no longer
passed main's canonical grammar; 65 passed and one was ignored. After fixing
the fixtures the full `rrd-core` suite passed (70 unit tests passed, one
ignored, plus the package integration and doc tests). The focused
`cargo test --offline --locked -p rrd-store --lib
selection_event_range_authenticates_inclusion_but_not_omission -- --nocapture`
passed its deliberate-corruption characterization; it demonstrates the
existing omission gap, not a repair. `cargo test --offline --locked -p
rrd-store --test installed_lifecycle` passed all six tests. The first
documentation-policy check failed because this record's status omitted the
required active/historical classification; after correction, navigation
generation, navigation `--check`, `python scripts/ci/check_documentation.py`
and `cargo fmt --all -- --check` passed. Candidate CI, merge-group CI, full
installed indexing, C-07 capacity/soak and performance comparisons have not been
run. The separate
2,048-file engine interruption regression is recorded below.

On Windows, the test build of the reconciled code took 6m 38s. The command
`cargo test --offline --locked -p rrd-engine --lib
interrupted_refresh_does_not_expose_partial_facts_as_completed_snapshot --
--nocapture` then failed one test after 72.28s, at the original assertion:
the completed-snapshot root returned at HEAD was
`fbd74d959620a5cbb377f7e1e4ed38dc387abc92004f941c0301c88777200a9d`,
while its expected committed root was
`ee18a00a3326e4f79d505f979066a4328723d3962be0dc556b32c4b4c7b76009`.
The engine sources under test match staged tree
`b0b754faa9bb97f78b3a8be07448d24103a214b4`; a subsequent edit to the
separate store lifecycle test fixture did not change engine source. This
reproduces the known mixed-generation defect; it does not show full indexing,
production repair or acceptable performance.

The prerequisite parser draft #241 remains separate and conflicted. No roadmap
or POAM status is closed; no changes were made to the user's project or ACLs.
