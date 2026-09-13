# Gate I — add explicit automation scaffolding without automatic hooks

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/gate-i`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

| Done | ID | Required change | Owning boundary | Acceptance evidence |
|---|---|---|---|---|
| [ ] | I-01 | Define one canonical engine-event envelope with producer, action, target, scope, stamp, idempotency key, provenance, and authorization coordinates; directly converge the current kernel `RuntimeEvent` representation into that vocabulary without a parallel event log or compatibility type. | `rrd-contract`, `rrd-core` | Golden, lowering, commit, and replay tests reject ambiguous identity, duplicate mismatches, unbounded payloads, and events outside the authenticated estate while proving public and persisted forms are one semantic event. |
| [ ] | I-02 | Implement triggers as persisted conditions over canonical committed events; a trigger may request an authorized operation but cannot commit independently. Keep transaction function bindings as the separate pre-commit validation/proposal mechanism defined by the governed-function contract, with no `Trigger` type, field, operation, or persisted marker shared between them. | `rrd-engine` | Match/non-match, denial, duplicate, ordering, recursion-depth, and restart tests prove deterministic bounded behavior; direct-convergence tests reject every old `FunctionTrigger*` shape and show transaction bindings cannot consume committed-event cursors or schedule routines. |
| [ ] | I-03 | Implement routines as versioned resumable graphs of authorized engine operations with explicit inputs, checkpoints, budgets, cancellation, verification, and terminal status. Model, process, network, and external MCP effects are prepared fenced activities: adapters return bounded observations, while `RrdEngine` alone accepts receipts and advances state. Express error resolution and context projection maintenance as routine definitions, not operation-specific repositories or state machines. | `rrd-contract`, `rrd-engine`, outward activity adapters | Generic kill/restart, retry, compensation, stale-input, denial, maximum-step, prepared/effect/receipt crash-gap, lost-ack, uncertain-outcome reconciliation, redaction, and resource-limit tests plus both template corpora prove no busy loop, silently repeated mutation/effect, direct storage access, private event log, adapter-authored completion, or client-owned state. |
| [ ] | I-04 | Implement host-event adapters as stateless translators that submit typed events only when explicitly installed and configured; ship no editor/provider-owned automatic hook. | outward adapters | Claude/OpenAI/reference adapter conformance produces the same envelope; uninstall removes the adapter cleanly and leaves canonical state readable. |
| [ ] | I-05 | Implement skills as versioned instruction/resource packages referenced by identity and digest, resolved through governed context rather than executed as storage or lifecycle code. | `rrd-contract`, `rrd-engine` | Install/resolve/update/retire tests prove provenance, authorization, version pinning, prompt-budget enforcement, and no implicit mutation. |
| [ ] | I-06 | Add previewable install/configure/retire/uninstall scaffolding for governed functions, transaction function bindings, triggers, routines, host-event adapters, skills, and optional capability/activity bindings after their individual contracts pass. Default function runtimes/artifacts come only from the manifest-verified distribution; attunement may propose an inactive project function but cannot compile, fetch, activate, or execute it. | `rrflow-cli`, adapters | Fresh/existing project tests show exact planned artifacts/schemas/files/records/invocation closures, explicit consent, preview without execution, idempotent apply, retirement that blocks new work without deleting receipts, clean uninstall of only unreferenced RRFlow-owned scaffolding, offline readiness, and no session-start loop. |
| [ ] | I-07 | Require every project-development routine to bind the latest complete authorized project-tree snapshot, then react to committed inventory, schema, dependency, workload, and failure signals by scheduling only the required incremental attunement phases and evaluating eligible capability, routine, and skill activation under estate policy. | `rrd-engine`, `rrd-attunement` | Missing/stale inventory returns `inventory-required`; changed-since-plan entries and paths outside the root are denied; adding one language, framework, data source, or recurring failure triggers the minimal bounded work, survives restart, records its decision evidence, and never performs an ad hoc client scan, blanket reinstall, or unauthorized activation. |

## Exit condition

Gate I exits only when automation is explicit, bounded, replayable, removable,
and subordinate to `RrdEngine`; installation alone is never evidence that a
trigger, routine, adapter, or skill worked.

## Required end-to-end corpus

The Gate I end-to-end corpus must include the generic `error-resolution`
vertical slice defined by the
[automation flow](../../architecture/engine-data-flow.md#first-complete-automation-proof).
It must prove one committed diagnostic can activate, resolve governed skills
and context, select or reject graph/BM25/vector/TurboQuant/DataFusion work,
invoke an attuned verification capability, survive restart without repeating
effects, and expose the same persisted result through MCP and the other public
surfaces.
