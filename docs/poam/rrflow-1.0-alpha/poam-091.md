# POAM-091 — attunement tests give fixed times that a slow machine overtakes

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-091`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Read on 2026-11-20 at `aeb76407b`. The engine stamps an attunement transition with the caller's time plus the real time the step took, and refuses a transition that is not later than the previous one (`elapsed_time` in `rrd/engine/src/engine/attunement.rs`; the check in `attunement/control.rs`, `inventory/publication.rs`, `parse/publish.rs` and `replan.rs`).

The tests under `rrd/engine/src/engine/tests/attunement/` supply fixed times of the form `BASE + n`, usually one second apart. A step that takes longer in real time than the gap to the next fixed time is recorded after it, and the next step is refused with `attunement transition time must advance`. This failed two Parse step tests on the push run of `aeb76407b` ([journal](../../evidence/change-journals/gate-d/parse-step-test-clock-20261120.md)), and `parse_step.rs` now reads an advancing clock.

Twelve other files in that folder still supply fixed times: 172 uses in `adaptive_selection.rs`, `control.rs`, `crash.rs`, `derivation.rs`, `discovery.rs`, `inventory.rs`, `inventory_delta.rs`, `observation_delivery.rs`, `parse_policy.rs`, `replan.rs`, `replan_receipts.rs` and `startup_binding.rs`.

## Impact

A loaded or slow runner can fail any of these tests without a defect in the engine, and a failure of that kind turns `main` red.

## Owning gates

D-02 owns attunement execution and its tests.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when every test in the folder takes its times from a clock that includes real elapsed time, or asserts an exact time only where the engine is given a fixed clock for the whole step, and the folder's tests pass.
