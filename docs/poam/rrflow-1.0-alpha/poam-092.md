# POAM-092 — Windows serving-record replacement fails during concurrent reads

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-092`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-20, `cargo test --locked -p rrd-engine` on Windows failed
`engine::tests::serving::a_requester_waiting_on_a_stale_record_never_sees_the_project_unserved`
at `serving.rs:161`: publication returned
`Storage("Access is denied. (os error 5)")` while another thread repeatedly read
the serving record. The library result was 522 passed, 1 failed, 12 ignored.
The starting revision was `081282df0af594ba3ded2554c9ee5acd2ab88308`, with the
[controller candidate](../../evidence/change-journals/gate-g/system-one-controller-20261120.md)
working changes; serving and filesystem implementation/test files were unchanged.

`ServingPublisher::publish` retries the entire publication once for any error.
Its replacement uses `AnchoredPath::rename_to`, whose Windows implementation
calls `MoveFileExW` with replacement and write-through flags. The failing log
does not identify which filesystem operation returned access denied. A passing
rerun alone cannot establish the cause or repair this defect.

## Impact

A new serving process can fail to publish over a stale endpoint record under
concurrent inspection. Removing the old record first is not a valid repair:
a stop requester interprets absence as completion.

## Owning gates

D-01 owns served lifecycle;
the anchored filesystem owns any native replacement repair.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Identify the failing native operation and cause, retain a reproduction, preserve
atomic old-or-new visibility and containment, then pass the concurrent serving
test and the owning filesystem/engine suites on Windows. Do not weaken the
reader assertion or hide the failure with an unbounded retry.


## Verified repair

[PR442](https://github.com/rrflow/rrflow-development/pull/442) merged through
its green candidate and merge-group gates at
`2aa15d6000b93be383a7fcff0752837e510fb66d` on 2026-11-20.
The [repair journal](../../evidence/change-journals/gate-d/windows-atomic-publication-20261120.md)
records a deterministic held-reader reproduction at the replacing native
operation, the one FileRenameInfoEx implementation, source/ancestor pinning,
exclusive-winner preservation and directory publication checks.
Final repair revision `0bf5a213998fce1a40691c1649f581d8aa24a2f5` passed 275 LSM
tests (3 existing ignored), all 15 serving tests, and subsequently the full
engine suite: 590 passed, zero failed, 12 existing ignored across 13 targets.
The final full-suite log is `publication-final-full-engine-tests.log` under
`D:/.usr/Temp/rrflow-selective-recall-audit-20261120`.
Candidate [44797439370](https://github.com/rrflow/rrflow-development/actions/runs/44797439370)
and merge-group [44798441599](https://github.com/rrflow/rrflow-development/actions/runs/44798441599)
both passed. The original engine oracle and containment assertions were not
weakened. The parent ledger was closed on that evidence; no broader storage
qualification was claimed.

## Reopened: false absence after the native repair

On the later PR448 worktree based on `95cd44d86065e76c242ec5e5a7cc9266cb208044`,
`cargo test --locked -p rrd-engine --lib` produced 522 passed, one failed and
12 ignored in 417.42 seconds. The same concurrency oracle failed in its reader
at `serving.rs:149`: `a look found no serving record during replacement`.
The writer's replacement did not report the earlier access-denied error.
Serving and filesystem source were unchanged by PR448's timestamp work.
Log: `cold-load-time-engine-suite.log` in the existing session scratchpad.

[Alarm449](https://github.com/rrflow/rrflow-development/issues/449) and
[repair PR450](https://github.com/rrflow/rrflow-development/pull/450) own this
investigation; the [new journal](../../evidence/change-journals/gate-d/serving-record-false-absence-20261120.md) records its native-open reproduction and subsequent checks. A focused single test and all 15 serving tests passed, but do
not explain the observed false absence. The earlier native repair remains;
its passing checks do not qualify this newly observed failure. Parent status
is Open again. Preserve the atomic old-or-new oracle and identify the exact
failing operation before claiming another repair.
