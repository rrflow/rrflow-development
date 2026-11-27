# POAM-088 — managed instances have fixed driver timings and are not registered or stoppable in every form

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-088`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Read on 2026-11-19 in the working tree of
[the instance registry and stop package](../../evidence/change-journals/gate-d/instance-registry-and-stop-20261118.md),
which gave `rrflow serve` and the internal `rrd-server` one
[stop request](../../reference/deployment/installed-lifecycle.md#stop-request)
and removed `rrd-server`'s own shutdown files and its literal 10 ms poll. What
that package left:

- **Fixed timings in the local process driver.**
  `rrflow/engram/src/local_process` waits on four literal 10 ms
  sleeps (`driver.rs` identity discovery, `lifecycle.rs` readiness and owned
  exit, `preparation.rs`) and four fixed bounds in `deployment.rs`
  (`PROCESS_DISCOVERY_TIMEOUT` 3 s, `PROCESS_START_STABILITY` 250 ms,
  `PROCESS_PREPARATION_TIMEOUT` 10 s, `PROCESS_STOP_TIMEOUT` 5 s). None has a
  stated reason or an override. Only the readiness and graceful-stop timeouts
  come from the deployment catalogue.
- **A managed instance is registered only when told where.** The driver starts
  `rrd-server` with an empty environment, so the server finds no per-user
  state directory and reports the
  [instance registry](../../reference/deployment/installed-lifecycle.md#instance-registry)
  as not updated. `rrd-deployment-catalog` passes no `--instance-registry`.
  An instance the driver starts is therefore not listed by `rrflow instances`
  unless the catalogue author adds the flag by hand.
- **A listener that is not plain loopback HTTP cannot be asked to stop.**
  A serving record names only `http://` on a loopback address. `rrd-server`
  with mutual TLS or a non-loopback bind publishes none, so no stop request
  reaches it; the driver's authenticated kill is what remains.
- **Nothing removes a registry entry.** An entry whose project folder is gone
  is reported as missing for ever.

## Impact

A managed instance can be slower to start or stop than its host needs with no
way to tune it. A client that finds instances through the registry does not
see one the driver started. A TLS-served managed instance is always killed,
never stopped cleanly. A user's registry grows with every project they remove.

## Owning gates

D-01 owns the one primary command tree and its lifecycle. D-11 owns removing a
registry entry with the rest of an installation.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when every wait in the local process driver is a named default with a
documented override or a stated protocol invariant; the generated deployment
catalogue registers a managed instance and a test lists it through
`rrflow instances`; a managed instance served with mutual TLS stops cleanly
through a request its supervisor is authorized to make; and uninstall removes
the registry entry.
