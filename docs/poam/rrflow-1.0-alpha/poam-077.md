# POAM-077 — the engine's configuration cannot be tuned after install, and most parts have no settings

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-077`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

A code read at `de435238` on 2026-11-11 found these gaps.

**The configuration cannot change after install.**
- `EngramConfiguration` is immutable once installed. Its contract says "Mutating it requires a future governed configure-plan/configure-apply operation" (`crates/transport/rrd-contract/src/deployment.rs:643-645`).
- Only the endpoint profile can be revised (`crates/authority/rrd-engine/src/engine/endpoint_configuration.rs:106`).

**Most parts have no configuration at all.**
- The configuration has clock, storage, reasoning, recall and query sections (`deployment.rs:646-655`).
- It has no vector, BM25, fusion, embedding or graph section.

**Several values are hardcoded:**
- LSM compaction trigger, target size and levels (`crates/persistence/rrd-lsm/src/database.rs:29-32`);
- segment row groups and compression (`crates/persistence/rrd-lsm/src/segment/format.rs:19-30`);
- vector residency (`crates/authority/rrd-engine/src/runtime/vector_residency.rs:16-17`);
- context fusion constants ([POAM-052](poam-052.md)).

**BM25 parameters have two owners:** `crates/transport/rrd-contract/src/lib.rs:739-804` and `crates/compute/rrd-query/src/bm25.rs:77-88`.

**No record of where a value came from.** A request's value is intersected with the sealed ceiling, but nothing records which level supplied the value.

## Impact

The 2026-11-20 documentation review also found a design contradiction:
ADR-0005 prescribed personal/org engrams while the instance topology reserves
engrams for projects. [ADR-0006](../../decisions/0006-scoped-policies-and-preferences.md)
supersedes that design with global **User policies and preferences** and
project-scoped configuration. The durable global source, authorized binding
and temporal resolver are still unimplemented; correcting terminology does
not close this gap. Users must be able to define enforced minimum requirements
as well as overridable preferences.

No contributor can be tuned or measured on a real installation without reinstalling. The owner's requirement that every threshold be configurable and measured cannot be met. That requirement is decision 3 and package J9 of the [one-engine joins plan](../../roadmap/one-engine-joins-engineering-plan.md).

## Owning gates

D-01 owns the installed lifecycle and its configuration. The configure-plan and configure-apply operations belong to that lifecycle.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when all of the following hold:
- A governed operation revises the sealed configuration.
- Each contributor has its own typed section.
- The hardcoded values above are settings in it.
- A read shows each effective value, its unit, its source and whether it applies now or after a restart.
- An installed test changes one setting and observes the effect.
- The ADR-0006 installed acceptance proves shared user configuration across
  two isolated project instances, governed project overrides, required user
  policies, explicit conflicts and unavailable-source behavior, temporal
  correction/restart, concurrent revision checks and configuration survival
  across project shutdown. The physical persistence owner is bound before code.
- Connectome explains effective values through the same public contracts,
  distinguishing user/project scope and enforced requirements from defaults;
  no client store becomes a second authority and no project sharing is implied.
