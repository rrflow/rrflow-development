# POAM-085 — Parse cannot run on macOS

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-085`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Read on 2026-11-18 at `cc5ebe0fd`. Parse, the phase that indexes a project's code, starts its worker under a native supervisor. That supervisor exists for Windows (`attunement/windows_supervisor.rs`, a job object) and Linux (`attunement/linux_supervisor.rs`, a direct child with a parent-death signal and an address-space limit). On every other platform `attunement/parse/worker.rs` returns `no native parse supervisor on this platform`, and `attunement/evidence.rs` leaves Parse out of `EXECUTABLE_PHASES`, so a derived plan never selects it. The Parse step tests (`tests/attunement.rs`, `mod parse_step`) are compiled for Windows and Linux only.

## Impact

On macOS an installed engram connects and inventories a project but never parses it. Code knowledge, and everything built on it, is unavailable there. The engine reports this as unavailable rather than failing, so no macOS run shows it as a failure.

## Owning gates

D-02 owns attunement execution.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when a macOS supervisor contains the worker (it dies with the engine, and a memory bound is enforced or its absence is stated in the plan), Parse is in `EXECUTABLE_PHASES` on macOS, and the Parse step tests pass in a macOS qualification run cited by its run id.
