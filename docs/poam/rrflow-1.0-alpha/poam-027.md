# POAM-027 — change-package and repository enforcement

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-027`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Candidate CI contains deterministic policy/test commands and one `pipeline / ci-gate`, but
the procedure-to-diff binding is not machine-readable and no external record proves that
pull requests, the stable check, reviews, deletion denial, or force-push denial are
required. On 2026-09-11, GitHub returned HTTP 403 for both rulesets and `main`
branch-protection APIs on private `rrflow/rrflow-development`, stating that GitHub Pro or
public visibility is required.

## Impact

A direct push or inadequately journaled change can bypass the intended review/evidence
process even while local checks pass; installing a client Git/editor/provider hook would
remain unportable and would create a forbidden parallel lifecycle mechanism.

## Owning gates

J-01, J-02

## Closure evidence

Maintain one checked-in deterministic change-package validator that binds changed paths,
starting revision, owner/gate, complete-file review, tests, observability decision,
remaining failures, and generated inventory to one linked package journal under
`docs/evidence/change-journals/`; run it in candidate CI. Derive Git baseline, digest,
line-count, navigation, and final-path facts mechanically so the author maintains intent,
risk, failure oracles, and evidence rather than another prose repository mirror. This is an
effect and evidence boundary, not a schema for exploratory or private AI reasoning. On
an eligible private-repository plan or equivalent organization control, require pull
requests/reviews and exact `pipeline / ci-gate`, deny deletion/force pushes, and record the
repository, ref, rule identity, API evidence, and an attempted bypass. No client hook or
workflow event becomes RRFlow runtime authority.

## Execution decision

POAM-027 execution decision (2026-09-12): the first remediation package is
`POAM-027a`. Its separate active-change record is bound to clean commit
`b9c46c35febf81d548ec4b06ef9184df988ae693` and permits only the deterministic
change-plan validator, its mutation tests, the candidate-CI/workflow-policy
binding, CI documentation, package evidence journal, and generated inventory.
It must reject a mixed plan/code commit, stale digest, incomplete line review,
undeclared committed/staged/unstaged/untracked path, and missing research,
failure, acceptance, observability, or stop-condition evidence. This does not
close POAM-027: the validator is not implemented at this planning revision and,
after implementation, eligible non-bypassable server-side protection remains
separate evidence.

POAM-027 convergence update (2026-09-14): package journals are now individual
coordinated evidence records, their parent listings are generated, and the
execution-map coordinate is a link portal rather than a growing journal/status
authority. `AGENTS.md` accepts the linked record plus regenerated index as the
literal journal obligation. ADR-0002 limits closed planning contracts to
repository effects and claimed evidence. Further mechanical generation of the
active change envelope and eligible non-bypassable server-side protection
remain open, so the parent lifecycle status is unchanged.
