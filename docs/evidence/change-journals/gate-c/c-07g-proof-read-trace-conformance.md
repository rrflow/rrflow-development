# C-07g proof-read trace conformance

**Status:** active corrective candidate evidence; native qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-proof-read-trace-conformance`
**Owner:** OBJ-02 / C-07g / POAM-035 / PR134

## Failure and pre-effect scope

[PR134](https://github.com/rrflow/rrflow-development/pull/134) candidate
[40265699310](https://github.com/rrflow/rrflow-development/actions/runs/40265699310)
failed its optional `rrd-engine` job 121387504542 at exact source
`db0a7a192a04ad820740f753fe70cad2e9dbc983`. The trace fixture expected
16 decoded values but measured 14. The full job log is retained locally at
`target/pr134-121387504542-failed.log`; its command was
`cargo test -p rrd-engine --all-targets --all-features --locked --no-fail-fast`.
This is a real missed cross-boundary test in the proof-reuse package, not a
runner-capacity notice or an unexplained intermittent failure.

The driver (current Codex session) added the corrective brief to PR134 before
editing source. Starting revision is that failed candidate. Scope is the
existing `runtime_query_trace.rs` fixture, this new immutable journal, POAM-035
and generated navigation/inventory. No runtime, API, budget, cache, profile or
native-campaign source changes. Stop for a different measured value, changed
query results, trace mismatch, redaction/causality regression or an unrelated
owning-suite failure; preserve such failures rather than weakening assertions.

## Characterization and replacement

Full file read: `crates/authority/rrd-engine/tests/runtime_query_trace.rs` and
`crates/persistence/rrd-store/src/access/proof_nodes.rs`. Targeted reads:
`access/versioned_read.rs::read_versioned_expanding`, engine
`runtime/query.rs` storage/execution trace attributes and
`runtime/read_evidence.rs`, engine Cargo features, the exact failed job log,
README/AGENTS, change-authoring procedure, POAM-035 and previous native outcome.
An exploratory guessed authentication-module path did not exist; the actual
inline versioned-read owner was used and no missing-code claim followed.

The old exact-count assertion maps to an exact 14-value execution assertion
for both MX and KV, plus equality of both storage-finish and query-finish
trace counters with their execution evidence. The same fixture retains
selected versions, result identity, cursor, logical equivalence, causal tree,
secret redaction, backend physical signals and typed failure checks. It does
not change production metering to satisfy a test. The proof-reuse package's
separate direct-read budget and corruption oracles remain in place.

Research decision: source and reproducible candidate evidence establish this
stale expectation; no external algorithm or source adaptation is involved.
Trace/debug decision: strengthen projection consistency while preserving the
exact reduced physical-work count. Resource decision: no production change.
The frozen `04559542` full-corpus characterization retains its own source,
binary and evidence identity; this test-only correction does not retroactively
qualify it or alter the running workload.

## Verification

Windows, locked dependencies, `target/endpoint-profile-build`, test/dev debug
level zero. Before edit:
`cargo test -p rrd-engine --test runtime_query_trace --locked -- --nocapture`
reproduced exactly 14 versus 16: two passed, one failed, exit 101.
After edit the identical command passed all three tests (0.06 seconds tests).
Logs: `target/pr134-trace-before.log` and `pr134-trace-after-0.log`.
Owning all-feature suite and strict lint results follow below before commit.

Checklist: owning outcome and pre-edit PR brief; exact failed revision/log;
full fixture and targeted owner review; direct old-to-new assertion map;
research/resource/trace decisions; first failure then focused pass; new linked
journal rather than rewriting previous evidence. No roadmap checkbox or
maturity changes. POAM-035 and full native profile/24-hour acceptance remain
open. PR134 still requires a corrected full candidate and normal merge queue.

Documentation validation initially rejected a missing active-status prefix and
a Windows default-decoding error in the POAM heading. The heading was restored
from its exact Git bytes, the new text uses explicit UTF-8, and navigation was
regenerated. Neither authoring error changed runtime or historical evidence.

The same failed candidate's `engine-authority` job 121387504453 later failed
at the identical assertion, also 14 versus 16. Its complete log is retained at
`target/pr134-121387504453-failed.log`; it is the same correction's second
candidate lane, not an uninvestigated failure.

The owning command `cargo test -p rrd-engine --all-targets --all-features
--locked --no-fail-fast` passed locally in 76.44 seconds: 269 tests passed,
zero failed, two ignored across 13 targets. The ignored installation subprocess
fixture is invoked by its parent tests; the privileged Windows ownership
fixture was not explicitly run locally and remains required in native CI.
Full output is `target/pr134-trace-after-1.log`. Focused before/after, owning
suite and candidate failures are separate evidence; a local pass is not a
candidate-CI pass. Formatting, documentation (616 statuses/614 coordinates),
navigation and workflow policy passed after the documented authoring fixes.

Strict lint `cargo clippy -p rrd-engine --test runtime_query_trace --all-features
--locked -- -D warnings` passed (36.75 seconds command; log
`target/pr134-trace-after-2.log`). Inventory passed with 1743 records;
whitespace checks passed. Complete changed fixture/diff and new journal were
reread. The exact source change is test-only; no new native qualification or
Connectome/provider acceptance is claimed by this correction.
