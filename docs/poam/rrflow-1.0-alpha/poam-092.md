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
