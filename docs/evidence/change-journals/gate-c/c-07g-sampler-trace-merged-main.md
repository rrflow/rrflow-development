# C-07: integrate sampler correlation after trace merge

**Status:** active draft integration evidence; native sampling ownership and qualification remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-sampler-trace-merged-main`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR171](https://github.com/rrflow/rrflow-development/pull/171)

The PR brief declared this refresh before effects. Clean draft head
`fc4540cf158276f0767a7594f9151a27c88f7688` already contained the exact tracing
candidate from PR178. Public main `81fce08e14497bbc280790beefe4666716355aec`,
tree `0d92234da88dc5d4c0f14e15b01e6efa13a11f4f`, has now merged PR176 and
PR178. Their ordinary merge into this draft reported no conflicts. Existing
source, tests, journal nodes and evidence were preserved; navigation and
execution inventory are regenerated. No new source behavior, threshold or
campaign format changed in this integration. Prior trace, failed 100k run and
initial missing-trace test are retained in
[c-07g-installed-trace-correlation](c-07g-installed-trace-correlation.md)
and [c-07g-sampler-native-outcome](c-07g-sampler-native-outcome.md).

On Windows x64, rustc 1.99.1, isolated `target/c07-sampler-correlation-build`,
locked offline dependencies and disabled dev/test debug info:

- `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::installed_sampler_trace_matches_server_and_store -- --exact --ignored --nocapture` passed the real installed mixed-family query and restart trace join. The two idle small-fixture calls recorded wait/assembly/profile/service nanoseconds `3700/349700/34100/1888000` and `2100/347900/40500/2233800`.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::installed_capacity_workload_smoke -- --exact --ignored --nocapture` passed; it reports unqualified debug build, one interrupted acknowledged write and cancellation of an injected asynchronous sample, with restart and replay. It is not capacity qualification.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen sample_task::tests` passed three tests for bounded ownership, error and async cancellation behavior.

Reviewed the PR brief, merged source and journal history, and integration diff.
No upstream adaptation or performance conclusion is made. Final navigation,
documentation, workflow, format and inventory checks precede commit. Native OS
sampling can still block the current-thread runtime and is not proved bounded
under interruption; the draft remains subject to its existing stop conditions.
The prior loaded slow call's cause remains unknown. Fresh integrated candidate
CI, native matrix and a genuine 24-hour soak remain unrun. No objective, roadmap
or POAM status changed. Checklist: merged prerequisite confirmed, exact parents
recorded, no source conflict, real cross-boundary tests rerun, prior failures
retained and unproven acceptance explicitly identified.

The first `python scripts/ci/check_documentation.py` check rejected this record because its status line did not declare `active` or `historical`; the status was corrected and the check rerun.
