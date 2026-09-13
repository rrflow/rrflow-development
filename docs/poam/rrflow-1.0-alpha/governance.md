# RRFlow 1.0 alpha POA&M ledger governance

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/governance`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Ledger ownership

The [parent POA&M](../rrflow-1.0-alpha.md) tracks gaps observed in the checkout. The
[objective](../../objectives/rrflow-1.0-alpha.md) defines the required result and
the [system overview](../../architecture/system-overview.md) defines the component
and security boundaries. The
[engine data-flow architecture](../../architecture/engine-data-flow.md) defines the
affected system flow. The [roadmap](../../roadmap/rrflow-1.0.md) owns execution
order. A row closes only when the referenced gate records its acceptance
evidence; editing this table cannot declare a capability complete.

## Status vocabulary

- `Open`: the deficiency is present and its prerequisite gate is available.
- `Sequenced`: the deficiency is present but an earlier roadmap gate owns the
  next executable work.
- `Verifying`: implementation exists and the exact closure evidence is running
  or under review.
- `Closed`: the owning roadmap gate contains accepted evidence at an exact
  revision.

## Milestone discipline

The milestone for each row is its owning roadmap gate, not a date guessed
before implementation exposes the true work. Work follows roadmap dependency
order. If implementation reveals another material deficiency, add one row with
observable evidence and a gate mapping before expanding scope.
