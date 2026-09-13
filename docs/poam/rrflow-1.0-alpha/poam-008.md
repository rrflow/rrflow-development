# POAM-008 — functions, routines, and automation convergence

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-008`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

A-07.1 supplies the canonical function-catalogue and transaction-function-binding vocabulary
and isolated `engine/function/` boundary. Accepted C-03 replaces inline definition bytes and
the whole-catalogue control record with content-addressed binary artifacts, typed immutable
definitions/bindings, digest-only membership revisions, and one CAS head through the common
rrflowMX/rrflowKV transaction port. Exact schema and runtime profile/build identities are
enforced; transaction execution seals a prepared receipt; recovery reuses it without guest
execution; known-outcome replay requires every exact prepared receipt to exist; and the
accepted receipt, derived event, domain/index effects, semantic audit, outbox, cursor, and
outcome publish atomically. Old fields/keys have no reader, historical identity revision
reset is rejected, standalone receipts replay/reopen, public maxima fit one physical batch,
runtime-build substitution fails closed, and the complete semantic key closure recovers
all-or-none at prepared, WAL-appended, WAL-synced, and visible-before-acknowledgement
boundaries. This is still not the engine-event, event-trigger, routine, host-event-adapter,
skill, install, or public-surface system. Receipt identity/effect authorization is not yet
complete across security/transport coordinates, and supported-target runtime determinism
remains unqualified. The unused `rrd-maintenance` crate still implements a fixed
operation-specific state machine that directly owns a storage port, private scope/events,
cursor-zero replay, JSON-wrapper records, hardcoded classes, and reduction policy with no
focused tests. The project-command target distinguishes discovered facts, inactive
candidates, installed bindings, prepared activities, observations, and accepted receipts,
but no implementation exists.

## Impact

Automation or project commands can still split into parallel authorities, exceed unproven
routine/activity limits, repeat uncertain external effects, lose useful safety behavior
during cleanup, or mistake functions, traces, fixed machinery, and process wrappers for
generic restartable routines.

## Owning gates

A-07, C-04, D-01, D-03, D-06, F-03, H-02, H-04, H-05, I-01 through I-07, J-01 through J-05

## Closure evidence

Retain C-03's typed content-addressed state, explicit runtime builds, schema-bound prepared
receipts, physically admitted atomic publication, direct removal, and fault/reopen evidence.
Closure still requires same-stamp effect-complete authorization and full receipt
identity/trace coordinates, supported-target runtime qualification, offline
install/retirement, and cross-surface proof; JSON sandbox functions never become implicit
DataFusion UDFs. Every preserve/generalize row in the context-maintenance matrix needs
focused generic-routine evidence; capability/activity contracts,
discovery-without-execution, exact preview/apply, prepared/effect/receipt crash gaps,
uncertainty reconciliation, re-inventory, redaction/resource bounds, restart, denial,
idempotency, optimistic conflict, budget, activation ownership, rollback, update, and
uninstall conformance must pass. Only then are the parallel maintenance/process authority
and command escape hatches absent.
