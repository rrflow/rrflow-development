# POAM-020 — canonical project, estate, and instance topology

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-020`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Installed topology has three conflicting active representations: `.rrflow/instance.toml`
plus `ProjectAuthorityBinding` makes a path/manifest shape the project authority and can be
created by server, CLI, MCP, SDK, and test startup; public `PLATFORM_TERMS`/`ResourcePath`
mirrors a superseded historical organization/estate/project hierarchy while accepting
arbitrary resource-kind order; and `EstateAuthorityState` defines another
organization/account/project/environment/instance/node/shard hierarchy. Environment is
absent from the public resource vocabulary, cluster identities use another loose string
family, and none of these paths proves one project/estate/instance binding across rrflowMX
and rrflowKV.

## Impact

Startup can manufacture authority outside installation, logical identity and authorization
can depend on the caller or historical word order, an estate can be mistaken for a
multi-project fleet, physical placement can diverge from semantic scope, and clients can
appear connected while resolving different project/database authorities.

## Owning gates

A-07, B-04, C-02, C-03, D-01 through D-03, D-06, H-04, H-07, J-01, J-03, J-05

## Closure evidence

Freeze one project ↔ estate ↔ instance relationship graph and operation-specific resource
grammar; introduce only the D-01 locator and engine-persisted installed-estate binding;
preserve strict identity, containment, digest, and cluster safety checks; remove
`.rrflow/instance.toml`, `InstanceManifest`, `rrd-server initialize`, startup
`ensure_dedicated*`, the private JSON binding, historical-table conformance, arbitrary path
ordering, and the duplicate estate topology; then pass fresh/existing preview/apply, MX/KV
semantic, unauthorized/nested/neighbor/escape/foreign/move, cross-surface identity,
clustered-placement, restart, and offline release tests.
