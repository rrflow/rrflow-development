# POAM-041 — plan-conformance and change, interrupt and restart evidence stop at installation

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-041`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

The parent ledger owns priority and lifecycle status. This detail records a
verified gap against [OBJ-14](../../objectives/rrflow-1.0-alpha.md#required-outcomes);
it cannot close a roadmap gate independently.

## Observed deficiency

Verified at development `main` `04fae276`:

- `InstallationResult.action_results` is built by zipping the plan's own
  actions with the produced outputs ([installation](../../../crates/authority/rrd-engine/src/engine/installation.rs)
  and [restore publication](../../../crates/authority/rrd-engine/src/engine/restore_publication.rs)),
  so `validate_for(&plan)` confirms structure, not that the applied effects
  match the plan. `validate_for` is otherwise tested only against a contract
  fixture, and the bootstrap test compares the journal with a literal list
  rather than with the plan.
- No test changes a source, configuration or dataset during attunement, because
  the Inventory phase returns `phase-unavailable`.
- Attunement has one kill boundary, after the `Start` claim, run through the
  engine test executable rather than the shipped `rrflow`, and no multi-phase
  resume test.
- No CLI test edits the project between `install plan` and `install apply`;
  restore and repair interruptions are in-process failpoints, not process kills.

Installation and storage already have strong evidence: real process kills at
write-ahead-log, visibility and receipt boundaries on three operating systems,
100 kill-and-reopen cycles, installation killed at eleven stages, previewed-path
confinement and acknowledgement ledgers reconciled independently of the engine.

## Impact

A test suite can pass while applied work diverges from the approved plan,
because the comparison is derived from the plan itself. The change, interrupt
and restart behaviour of attunement and later governed effects is unproven.

## Owning gates

D-02, D-03 and D-11; I-03 routines; the J independent history checker.

Roadmap owners: [Gate D](../../roadmap/rrflow-1.0/gate-d.md#project-workflow-build-packages),
[Gate I](../../roadmap/rrflow-1.0/gate-i.md) and
[Gate J](../../roadmap/rrflow-1.0/gate-j.md).

## Closure evidence

An independent checker derives the expected effects from the approved plan
alone and compares them with independently observed artifacts and the read-back
audit trail and receipts; the engine does not grade its own run. Tests
deliberately change a source, a configuration and an open-question answer
between plan and apply, kill the shipped `rrflow` at every job transition,
restart and resume, and require exactly-once effects. Both pass at an exact
revision for attunement and for at least one project mutation and one memory
write.
