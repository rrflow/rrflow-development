# Controller reconciliation with the startup recovery candidate

**Status:** active documentation reconciliation; prerequisite and candidate landing pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-startup-candidate`
**Owner:** Gate G/I controller composition; PR206; driver: active SWAG session

## Source and change checklist

The pre-edit [PR206 brief](https://github.com/rrflow/rrflow-development/pull/206)
binds ordinary composition of startup candidate
`974c9d65d28750f89edacf4f4955d4125d2e23ca` into clean controller head
`952e0928c55f809b4f9fcce4dcf3808878a0bd65`. Merge
`d4d2aea226b09593bb6a9a5973ab63c99edeb33b`, tree
`db80d2f3bfe1880cb435adf27e67646792a685d6`, preserves both parents without
conflicts. It includes landed main `906bec82`. PR224 and PR228 are merged;
PR227 is ready, with candidate run
[41227938401](https://github.com/rrflow/rrflow-development/actions/runs/41227938401)
started but not yet accepted or landed at this observation.

The complete incoming delta is three documentation paths: the Gate D index,
`d-03-candidate-check-admission.md` and `d-03-startup-landed-main.md`. There is
no production, configuration, SDK, workflow, lockfile or generated contract
change. The startup/Inventory/storage/controller implementations remain at
their original owners. `git diff --exit-code <baseline> HEAD -- . ':!docs'`
passes against both `952e0928` and tested implementation `9e167aad`.

Reviewed README ownership and identical AGENTS instructions, the prior
[composition journal](controller-publication-integration.md) and
[installed SDK journal](controller-publication-installed-sdk.md) in full,
the two incoming journals, the complete incoming diff and current PR/run
observations. This increment adds this immutable journal and regenerates its
Gate G navigation. No source is moved, deleted or replaced; no second owner
or endpoint selection rule is added. The final diff is confined to that scope.

## Acceptance and limits

`python scripts/knowledge/render_navigation.py` passes with 846 nodes and 3,073
edges. Regenerated execution inventory passes its `--check` with 2,163 records.
`python scripts/ci/check_documentation.py` passes with 847 statuses and 845
classified coordinates; `git diff --check` passes. No check failed in this
increment. Review of the full changed-path list confirms the declared scope.

No unchanged Rust suite or provider call is repeated for this documentation
merge. The 353 engine and 208 contract/inference/client passes, seven installed
CLI lifecycle passes and two real Python SDK replan passes remain bound to
the source and artifact in the prior journals. No new binary is built and no
historical observation is reassigned. This is prerequisite reconciliation,
not worker completion, model qualification or distribution evidence.

No new research, tracing or resource measurement is necessary: the source
equality and prior canonical-owner review establish this bounded scope.
Stop on unexpected non-documentation changes, missing evidence or weakened
dependency controls; none occurred. Prior failures and uncertainties remain
retained. Product version remains `1.0.0`; no roadmap gate or POA&M closes.

Keep PR206 draft until PR227 has a verified main destination; then reconcile
with that destination and mark ready to start its full candidate pipeline.
Candidate and protected acceptance are required before landing, not before
the ready transition that launches candidate CI. PR226 is not closed by ancestry alone.
Worker execution belongs to PR229, so its separate UI/project-effect and
learning requirements are not silently added to or claimed by this proposal
package. Public release and the complete user journey remain unfinished.
