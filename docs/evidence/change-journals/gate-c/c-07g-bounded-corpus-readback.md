# C-07g bounded corpus readback windows

**Status:** active implementation evidence; full native qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-bounded-corpus-readback`
**Owner:** OBJ-02 / C-07g / POAM-035 / PR139

## Baseline and pre-effect brief

[PR139](https://github.com/rrflow/rrflow-development/pull/139) published the
brief before implementation. Driver: current Codex session. Public main was
`dd225072bcc639c89b0eef968625cc73303ea998`; the branch fast-forwarded to its
explicit PR134 dependency `db0a7a192a04ad820740f753fe70cad2e9dbc983` before
opening revision `caf33d19f7efac2b631717b4a10ab0a9c50aa8e0`. No source was
force-pushed and no main/official-repository push was used.

The [retained native outcome](c-07g-proof-read-native-outcome.md) acknowledged
100000 corpus operations but verified only 38960 before its original two-hour
setup limit. It remains failed. Source inspection found a serial verification
loop despite the frozen point's four-client allowance. That schedule prevents
overlap of independent reads; it does not prove scheduling is the only engine
or setup cost. This package changes the qualification driver's schedule, not
production engine performance, workload size, operation or setup deadlines,
resource limits, durability, API, record schema or acceptance thresholds.

## Implementation and replacement map

`tests/support/sustained/corpus.rs::seed_inner` retains sequential writes and
all expected values/digest grammar. The complete independent readback uses
windows of at most min(point clients, frozen queue allowance, 64). Session
rotation occurs only after the previous window has completed. `verify_window`
constructs each bounded expected record and calls the unchanged `Workload::verify`
under the original per-operation timeout. That oracle still retrieves vectors,
queries records and relations at the vector's known cursor, and compares every
value against independently generated input. The original outer setup timeout
still spans ingestion and all verification.

The new private `corpus_window.rs::ordered` owns a finite set of polled futures.
It validates cardinality before polling or allocating them; every unfinished
future is dropped synchronously when its owner fails or is cancelled. There
are no spawned tasks, detached reads or new dependencies. Results retain input
ordinal order despite completion order. The rolling readback digest and progress
records therefore keep their original grammar/order. A failed partial window
emits no successful window evidence and cannot create complete-corpus evidence;
previous successful windows remain retained. At most one admitted window of
requests/results exists, rather than a corpus-sized result list.

The existing installed workload smoke now exercises the same two-client window
and checks every returned ordinal/effect digest. Its later mixed operations,
retained-reader/backup checks, abrupt termination and reopen remain in place.
The old serial verifier maps one-for-one to this unchanged public API oracle;
no expected value is learned from an engine response and no verification is
replaced by an acknowledgement count.

Exact changed source: `crates/adapters/rrflow-cli/tests/support/sustained.rs`,
`sustained/corpus.rs`, and new `sustained/corpus_window.rs`. Additional paths:
this journal, POAM-035 and generated journal navigation/file inventory. No
engine source, SDK, Cargo dependency, profile or historical receipt changes.
Stop for weakened acceptance, unbounded work, session rotation with active
reads, missing source evidence or an unreviewed production change.

## Research, review and diagnostics

Full reads: corpus.rs, new corpus_window.rs and tests, existing management.rs,
prior bounded-worker-readback journal, PR134 terminal outcome and native
staging/preparation/launch scripts. Targeted reads: workload verify/verify_key
and query, sustained session rotation/native smoke/campaign setup, storage
campaign client-versus-queue validation, query pipeline, C-07 owner, README,
AGENTS and existing change-authoring procedure. Changed source and complete
package diff are reviewed before commit. Unread modules are not claimed audited.

Research decision: existing source and retained native evidence support this
bounded scheduling correction; no new upstream algorithm or source is adapted.
Trace/resource/debug decision: preserve bounded native counters, exact frozen
identities, original deadlines and independent progress receipts. Component
overlap is not a native throughput claim. A fresh exact-source characterization
is needed to determine whether full setup now completes; the previous failed
estate and evidence must remain untouched.

## Local verification and limitations

Windows, locked dependencies, shared `target/endpoint-profile-build`, dev/test
debug level zero:

- `cargo test -p rrflow-cli --test installed_reopen --locked -- corpus_window --nocapture`: two passed. Barrier-gated work requires overlap; forced reverse completion preserves canonical order; admission precedes polling; failure and outer timeout release all four owned futures.
- `cargo test -p rrflow-cli --test installed_reopen --locked -- --nocapture`: 14 passed, zero failed, seven dedicated native/campaign tests ignored. The dedicated smoke below was then explicitly run.
- `cargo test -p rrflow-cli --test installed_reopen --locked -- sustained::installed_capacity_workload_smoke --exact --ignored --nocapture`: one passed, 7.29 seconds test execution. Real installed process, mixed-family readback, backup/read-lifetime checks and termination/reopen. This is debug smoke, not release capacity qualification.

Logs are `target/c07-corpus-window-focused.log`, `c07-corpus-suite.log`,
`c07-corpus-native-smoke.log`, and `c07-corpus-clippy.log`; command receipts are
`target/c07-corpus-commands.json`. No compile or behavioral failure occurred in
these first tests. The initial whole-workspace formatter skips this test-support
subtree; the new helper and corpus owner were explicitly formatted with Rust
2021, and the small parent diff was formatted without rewriting unrelated code.

Full native corpus, release profile matrix, 100-cycle native reopen, current
Connectome walkthrough and actual 24-hour soak have not passed for this new
candidate. Dependency and full candidate/merge-queue checks remain required.
The promised before/after outcome here is bounded concurrent verification with
unchanged independent results and cleanup, not complete C-07 or alpha delivery.

Checklist: owner/brief/baseline before edits; exact serial-to-window replacement
map; source/failure evidence and research decisions; finite resource ownership;
focused then owning/native smoke tests; surfaced limits recorded; linked journal
and generated projections; no roadmap checkbox or maturity change. POAM-035
remains open and gains this attempted remedy with its qualification limits.

Final local results: strict primary/installed-reopen Clippy passed with
`-D warnings` (28.47 s). Workspace formatting and explicit corpus/helper
formatting passed. Documentation: 616 statuses/614 coordinates; navigation:
17 indexes/615 nodes/2433 edges; inventory: 1744 records. Workflow policy and
whitespace checks passed. Full source/diff review preserved production,
profiles, all original deadlines and all workload verification calls.
