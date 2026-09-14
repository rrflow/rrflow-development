# POAM-027 — change-package and repository enforcement

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-027`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

The checked-in `scripts/ci/check_change_plan.py` now machine-binds a package's
planning-only commit, exact baseline, complete-file review, declared post-plan
paths, failure oracle, research decision, acceptance commands, trace/resource
decisions, and stop conditions to the resulting change. Candidate CI invokes
that validator before the wider pipeline. Linked package journals and their
nearest generated indexes now carry the durable work record.

Two gaps remain. Git, digest, line-count, navigation, and final-path facts are
not yet derived automatically wherever repository tooling can own them, so an
author still maintains avoidable mechanical fields. The validator is also
bypassable outside candidate CI, and no external record proves required pull
requests, reviews, stable checks, deletion denial, or force-push denial. On
2026-09-11, GitHub returned HTTP 403 for both rulesets and `main`
branch-protection APIs on private `rrflow/rrflow-development`, stating that
GitHub Pro or public visibility is required.

## Impact

A direct push or inaccurately authored change envelope can bypass or weaken the
intended review/evidence process even while local checks pass. Installing a
client Git, editor, or provider hook would remain unportable and would create a
forbidden parallel lifecycle mechanism.

## Owning gates

J-01, J-02

Roadmap owner: [Gate J](../../roadmap/rrflow-1.0/gate-j.md).

## Closure evidence

Keep one checked-in deterministic package validator and its candidate-CI
binding. Derive mechanical baseline, digest, line-count, navigation, and
final-path facts from Git and repository tooling so the author owns intent,
risk, semantic scope, failure oracles, evidence, and stop decisions rather than
another prose repository mirror. Continue to require one linked journal per
bounded package and regenerate its nearest index. This is a repository effect
and evidence boundary, not a schema for exploratory or private AI reasoning.

On an eligible private-repository plan or equivalent organization control,
require pull requests/reviews and the exact stable candidate check, deny branch
deletion and force pushes, and record the repository, ref, rule identity, API
evidence, and an attempted bypass. No client hook or workflow event becomes
RRFlow runtime authority.

## Remediation evidence

- [POAM-027a change-plan enforcement implementation](../../evidence/change-journals/repository/poam-027a-change-plan-enforcement-implementation-journal.md)
- [POAM-027b canonical package-identity correction](../../evidence/change-journals/repository/poam-027b-canonical-package-identity-correction-journal.md)
- [POAM-027c linked execution records and generated navigation](../../evidence/change-journals/repository/poam-027c-linked-execution-records-and-generated-navigation.md)
- [Current CI and repository-enforcement boundary](../../operations/ci.md)
