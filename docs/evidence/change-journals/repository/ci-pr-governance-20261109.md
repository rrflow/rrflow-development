# Pull request limits enforced by candidate CI

**Status:** active change-journal evidence; a repository process contract change, not alpha or release evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-pr-governance-20261109`
**Owner:** [CI and runner operations](../../../operations/ci.md#pull-request-limits); the repository engineering procedure in `AGENTS.md`

## Why

On 2026-11-09 the repository held 27 open pull requests, 20 of them drafts,
most 130 to 420 commits behind `main`; eight were stacked on each other; two
(#298, #299) had merged into `main` with no file changes while their
implementations sat uncommitted in worktrees; five worktrees held 14 to 514
staged files that no pull request carried. `AGENTS.md` described a careful
process and twelve agent sessions followed its letter. The owner asked for
limits grounded in how teams that ship actually work, enforced by the
machine.

## What the evidence supports

- Google's reviewed changes have a median of about 24 lines and are approved
  in under four hours (Sadowski, Söderberg, Church, Sipko, Bacchelli, "Modern
  Code Review: A Case Study at Google", ICSE-SEIP 2018). The authors note that
  fast review and small changes reinforce each other; small size is one
  factor, not proof of fewer defects.
- The Cisco/SmartBear study of 2,500 reviews reports review effectiveness
  falling past 200 to 400 lines; it is vendor-run and measures defects found
  in review, not shipped defects.
- DORA's trunk-based development capability sets three or fewer active
  branches and at least daily merges to trunk; its 2023 report called the
  recent results mixed, so this is correlational guidance.
- Open-source projects that admit AI agents commonly cap a contributor at
  about three open pull requests (Wessel et al., 2026). Practitioners running
  parallel agents size concurrency by pull requests awaiting review and report
  conflicts growing faster than linearly with agent count (Osmani; CircleCI;
  the AgenticFlict dataset measures frequent, substantial conflicts in
  agent-generated contributions).

## Change

Starting revision: `development/main` at `953fe774`.

- `scripts/ci/check_pr_governance.py` (new): a pure `evaluate` over pull
  request facts with six rules (non-empty diff, base is `main`, merge base
  at most 3 days old, at most 400 changed lines, at most 5 open pull
  requests, at most 3 per author). The `integration` label, applied by the
  owner, lifts the size and count rules only. Pull requests opened before
  2026-11-10 keep the first three rules and are not counted, so the backlog
  can be absorbed. `scripts/ci/test_pr_governance.py` (new) has 10 cases.
- `.github/workflows/ci-reusable.yml`: the `changes` job gains
  `pull-requests: read` and an unconditional "Enforce pull-request limits"
  step after lane selection; the verify policy step runs the governance
  tests.
- `scripts/ci/check_workflow.py`: requires the step, its permission and
  both scripts, and refuses a conditional step.
- `scripts/ci/check_session_end.py` (new): fails while a worktree has
  uncommitted or unpushed work.
- `AGENTS.md` and `docs/operations/ci.md`: the limits, their evidence and
  the session-end rule.

## Evidence

Local, Windows x86_64, Python 3.11: `test_pr_governance.py` 10 OK;
`test_ci_lanes_policy.py`, `test_candidate_admission.py`,
`test_linux_arm64_policy.py` OK; `check_workflow.py` OK; documentation
policy and navigation check OK; `check_session_end.py` run on this worktree
before push. Not run here: the governance step on a live pull request; this
pull request's own candidate run is the first, and it touches the workflows,
so it takes the full matrix.
