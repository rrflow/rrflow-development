# POAM-015 — public-contract discovery consistency

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-015`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Public-contract discovery is not self-consistent: the frozen sample capability advertises
`POST /v1/backups/create` while the catalogue, server, and Rust client use
`POST /v1/backups`; runtime capability limitations cite retired `F5`, `G06`, and `G09`
labels; and successful claim/data scope plus field/metric vector-address alternatives
remain.

## Impact

Generated artifacts can pass while a sample or runtime description directs a client to a
nonexistent route or implies a superseded plan, and alternate pre-release branches can
survive as accidental compatibility promises.

## Owning gates

A-07, C-03, H-04, J-01

Roadmap owners: [Gate A](../../roadmap/rrflow-1.0/gate-a.md),
[Gate C](../../roadmap/rrflow-1.0/gate-c.md),
[Gate H](../../roadmap/rrflow-1.0/gate-h.md), and
[Gate J](../../roadmap/rrflow-1.0/gate-j.md).

## Closure evidence

Trace every affected fixture, type, server/client path, and test; derive available surface
bindings from the executable catalogue; remove retired labels and superseded successful
branches directly; then pass negative old-shape tests plus contract/router/OpenAPI/client
cross-surface parity.
