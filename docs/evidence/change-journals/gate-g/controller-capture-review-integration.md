# PR188: integrate the checked review dependency

**Status:** active local integration evidence; dependency landing and fresh candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-capture-review-integration`
**Owner:** G-06a observation and review preparation
**Pull request:** [188](https://github.com/rrflow/rrflow-development/pull/188)

## Scope and source identity

The resumed working session found PR180's candidate checks all passing and its
protected merge-group checks pending. PR188 had independently become ready and
was running candidate CI. A read-only `git merge-tree` preflight reproduced
conflicts in the shared CI block, Gate G index and execution inventory. Merely
waiting for PR180 to land would leave those same conflicts unresolved.

The PR188 brief was updated before this ordinary dependency merge. Its starting
head was `323e6e74ac1c065d053453e2c41e9a1ddf1eedd4`, with a clean worktree.
The exact queued PR180 candidate is
`45a0cd82733abc93a1374796c47bf5d68bd2bbcc`; its parents are current main
`57b9a35583b2d5b6d2d34281904646a509cb32f8` and the checked PR180 head
`21fb12c2a48f97aef7d0b66368a2903a0f153327`. Those identities and ancestry were
verified before merging. A queued commit is not reported as merged.

The existing package brief, owning instructions, conflict files, inventory
generator's input/output ownership, merged source diff and tests were inspected.
The implementation trace is unchanged: observation capture stays in
`capture_observations.py`; explicit decisions stay in `review_decisions.py`;
paired packets stay in `review_outcomes.py`. Their characterization suites are
retained at the same paths. There is no source deletion or movement.

## Resolution and checks

The CI conflict is resolved by retaining all three test commands. Generated
index/inventory conflicts are resolved from their owning source files with the
repository generators, not by manually selecting status or digest rows. This
new receipt leaves both earlier package journals unchanged.

- Eight capture tests, seven decision-review tests and 15 outcome-review tests
  passed on the combined tree: 30 total.
- `python scripts/ci/check_workflow.py` passed with the existing full topology.
- `git ls-files --unmerged` was empty after resolution.
- Diffs confirmed the collector and its tests match PR188's original head;
  both review implementations and their tests match the queued PR180 source.

Documentation/navigation/inventory and whitespace checks are required before
publication. The merge's independent upstream changes are already on main;
this package does not author their runtime behavior. No new algorithm, library,
trace or GPU experiment is needed for this integration-only change.

## Candidate and landing limits

At the pre-push check, previous-head run `40801207290` had 20 successful jobs,
zero failures and remaining queued jobs. The source integration requires fresh
CI; earlier successes are not substituted for acceptance of the combined head.
PR180's own queue and checks remain untouched. Landing order stays PR180 then
PR188, enforced by the operator follower's exact-head checks and normal queue.
Stop if either dependency identity changes, checks fail, another conflict occurs
or protected landing would need a bypass. Runtime learning, training admission,
model dispatch and qualification remain outside this package. Roadmap/POA&M
acceptance is unchanged.
