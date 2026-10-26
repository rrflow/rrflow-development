# D-02 candidate capacity dependency refresh

**Status:** active candidate integration evidence; acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-capacity-dependency-refresh`
**Owner:** OBJ-07 / D-02 planning dependency; C-07f / POAM-035 owns the fixture

## Brief, baseline and scope

[PR136](https://github.com/rrflow/rrflow-development/pull/136) published its
refresh brief before this ordinary merge. Starting revision was
`337d136210ff59b2359f898e717166c087dd5bfc`; public main remained
`dd225072bcc639c89b0eef968625cc73303ea998`. Driver: current Codex session.
The dependency is [PR131](https://github.com/rrflow/rrflow-development/pull/131),
exact head `6911f397b33b9282acad2c5be437cb811ba6740c`.

Before: PR136's candidate still uses per-file metadata settlement and periodic
whole-volume synchronization. Its macOS capacity fixture times out before the
selected manifest operation resumes. After this merge: it carries PR131's
bounded batching repair at the same test-support boundary. No new engine,
API, workload, permission, time-limit or acceptance behavior is introduced.
Stop for weakened failure oracles, semantic conflicts or unqualified landing.

Inherited implementation is `crates/adapters/rrflow-cli/tests/support/capacity.rs`
and `capacity_metadata.rs`, plus PR131's immutable journal and POAM entry.
This package adds this journal, the new observed failure to POAM-035 and
regenerated navigation/file inventory. No implementation is deleted or moved.
Existing journals remain unchanged. The inventory conflict is resolved by its
generator, not by choosing one branch's derived projection.

## Review, traceability and research

Read in full for this refresh: change-authoring procedure, original PR136
journal, PR131 journal, PR131 complete Rust diff including the new metadata
helper and its tests. Targeted reads: README status/identity, CI candidate
chain, execution-map navigation, POAM-035 current capacity history, inventory
generator entry/index handling, installed-crash test filters. The preceding
failure investigation also inspected the capacity fill/settlement paths and
APFS helper. This is a bounded review, not a full-engine audit.

Traceability: old `fill_data` periodic synchronization maps to the same helper's
explicit interval/capacity barriers; `Filler::fill_metadata` maps to the bounded
metadata scheduler. Tests preserve delayed allocation, fresh post-settlement
denial, noncapacity errors and aggregate byte/file/time caps. Installed native
operation failure and acknowledged-history readback retain their owners. The
repair changes fixture scheduling, not production durability. Preserve its
phase timing diagnostics; no new telemetry is required for this integration.

Research used GitHub's [required-status troubleshooting](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks)
and [enqueue API](https://docs.github.com/en/graphql/reference/pulls) to diagnose
landing. No new storage algorithm or external source adaptation. PR131 owns
the earlier APFS research. No repository rule was disabled or bypassed.

## Observed failures and acceptance

PR136 run [40218400444](https://github.com/rrflow/rrflow-development/actions/runs/40218400444),
macOS job `121244432897`, fails `bounded joint pressure` in capacity.rs:313,
then the controller join fails at line 1205. The preceding WAL and segment
cases reach their expected native failure classes. The failing fresh manifest
case stops in fixture preparation; it is not evidence of lost acknowledged
records. Full log retained locally at `target/pr136-macos-domain-crash.log`.

PR131 run [40201872215](https://github.com/rrflow/rrflow-development/actions/runs/40201872215),
attempt 2, completed successfully on exact head `6911f397...`, including the
three native capacity jobs, reopen histories and `pipeline / ci-gate`.
Nevertheless, normal CLI merge failed because auto-merge is disabled, and
direct normal enqueue rejected `Required status check "pipeline / ci-gate"
is expected`. The head check is green and from required GitHub Actions app
15368; the current potential merge commit has no check rollup. A later skipped
draft-event run exists. Its causal role is unproven. A fresh draft event was
allowed to finish before marking ready, starting run
[40261615460](https://github.com/rrflow/rrflow-development/actions/runs/40261615460).
This is a fresh required candidate, not fabricated success or a gate bypass.

The first inventory regeneration rejected the unresolved generated-file index.
After staging the conflict for mechanical replacement, regeneration was rerun.
The generated result must pass the drift check before commit.

Local acceptance results are recorded below. Native APFS,
owned disk-image qualification, Connectome walkthrough and a 24-hour soak were
not run locally. No alpha, D-02 or C-07 completion is claimed. Dependency-ready
full candidate CI and normal merge remain required.

## Change checklist and status

Owner/baseline/brief bound before merge; implementation replacement mapped to
its existing fixture owner; research and diagnostic decisions recorded;
original failure preserved; generated conflict resolved mechanically; focused
and owning tests required; complete integration diff reviewed before commit;
linked journal and projections supplied. POAM-035 gains this observed candidate
failure and remains open. Roadmap checkbox and maturity status do not change.

## Local results

- `cargo test -p rrflow-cli --test installed_crash --features crash-test --locked -- capacity:: --nocapture`: 18 passed, one owned-volume test ignored, seven filtered out. Windows; retained `target/pr136-capacity-focused.log`.
- `cargo test -p rrflow-cli --test installed_crash --features crash-test --locked -- --nocapture`: 24 passed, zero failed, two owned-volume tests ignored; 17.85 seconds. Retained `target/pr136-installed-crash.log`. Both runs use the shared target cache and dev/test debug level zero.
- `python -X utf8 scripts/ci/check_documentation.py`: passed, 611 statuses and 609 classified coordinates.
- `python -X utf8 scripts/knowledge/render_navigation.py --check`: passed, 17 indexes, 610 nodes and 2417 edges.
- `python -X utf8 scripts/ci/check_workflow.py`: passed, all current workflow topology requirements retained.
- `git diff --check`: passed. Exact-byte comparison of inherited Rust files and PR131 journal against `6911f397...`: identical after checkout line-ending normalization.

The POAM append initially changed line endings across the file; diff review
caught and corrected that incidental churn. An initial journal patch contained
a nonexistent context line and was rejected without edits, then corrected.
Final inventory and whitespace checks run after the journal is staged. Clippy
is not repeated locally for unchanged PR131 Rust bytes; its candidate and
prior package lint results remain attributed to their own revision. No new
runtime source or threshold changes were needed during integration.

Final `cargo fmt --all -- --check`, execution inventory `--check` (1734 records),
and staged whitespace checks passed. Full integration diff reviewed; inherited
code remains byte-identical to PR131. This records local integration acceptance
only; all new candidate and merge results remain observable on the PRs.
