# POAM-006 — installed product and attunement lifecycle

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-006`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Install and attunement are contracts without an engine-persisted job, deterministic
plan/apply implementation, generic bundle-resident template, committed project-tree
inventory, or real phase executors. The public `rrflow` binary has no
install/serve/ready/verify/repair/version command; checkout-only `dev up` builds companion
binaries and `dev doctor` can report READY from source/supervisor state. `rrd-server`
directs operators to nonexistent `rrflow init`, while `RrflowKvStore::open` can create
absent storage and reconcile state.

## Impact

RRFlow cannot become or operate as a repeatable installed product; cold start, normal open,
diagnostics, and process supervision can manufacture conflicting lifecycle truth, and
internal components can appear healthy without one usable estate or executable.

## Owning gates

D-01 through D-11, J-01 through J-05

## Closure evidence

The D-01 single-primary-executable fresh/existing lifecycle passes deterministic plan,
no-write preview, apply, authenticated ready, commit, close/reopen, and baseline read-only
verification without checkout companions; old initializer/supervisor/doctor/create-on-open
success paths are absent. D-02 through D-05 prove durable attunement and inventory/index
work; D-11 proves safe repair/restore/uninstall; native release qualification passes in J.
