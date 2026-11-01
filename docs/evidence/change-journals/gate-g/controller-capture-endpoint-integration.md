# PR188: integrate the queued endpoint-contract dependency

**Status:** active integration evidence; dependency landing and fresh candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-capture-endpoint-integration`
**Owner:** G-06a observation preparation
**Pull request:** [188](https://github.com/rrflow/rrflow-development/pull/188), after [196](https://github.com/rrflow/rrflow-development/pull/196)

## Queue evidence and pre-edit brief

PR188 candidate `d092124039f9902d270b26caa9bce81542e7128e` passed all 23
candidate checks in run `40913102114` attempt 2. PR196 candidate
`db0ae5927848a87a674616f91f5bd06d09778f32` also passed its 23 checks, in
`40914270128` attempt 2. Both entered the normal protected queue. The first
PR196 merge-group run `40928446398` remains separate from candidate success.
Earlier failure/cancellation logs stay retained; a successful retry does not
diagnose their cause.

The queue reported PR188 unmergeable while GitHub still considered each PR
individually mergeable into current main. Read-only `git merge-tree --write-tree`
between the two exact candidates reproduced one conflict in the generated Gate G
journal index. No collector implementation conflict appeared. The PR description
received the bounded integration brief before effects. The confirmed-live PR188
follower was stopped, the PR dequeued normally and returned to draft; PR196's
queue run was left alone. No bypass, duplicate CI retry or force push occurred.

## Resolution and proof

The isolated collector worktree was clean at the above head. An ordinary merge
of the exact PR196 candidate reproduced the one index conflict. Regenerating
`docs/evidence/change-journals/gate-g/README.md` from its owning journal entries
preserves both packages and resolves the conflict. The learning reference merged
automatically. PR196's existing endpoint-selection design, roadmap/POA&M links
and immutable journal remain its owned additions, not new collector behavior.

Read the existing collector/integration journals, README/AGENTS and authoring
rules, exact GitHub queue/check state, merge preview, generated index and merged
reference diff. The collector/review source and tests remain byte-identical to
the prior PR188 candidate (`git diff --cached --exit-code` for `scripts/inference`).
No source movement, training admission, schema, provider or runtime change is
introduced. This record and generated discovery are the integration evidence.

The actual collector ran with `python -S`, the existing review-decisions worktree
and retained real PR180 context, into a fresh local evidence directory. It emitted
one unreviewed observation; a fresh process returned `unchanged` and appended zero
records. No project file was edited, provider called, output label invented or
training record admitted. Local records are retained under the SWAG workspace's
`.swag/evidence/20261029/collector-endpoint-integration/`.

No new research, GPU run or performance trace is needed for a generated-index
repair. The oracle is conflict-free preservation of both owners, unchanged
collector code and reproducible real capture/deduplication. Prior candidate
success does not qualify this merged head. Keep draft until PR196 lands, refresh
main and pass fresh full candidate and merge-group checks. G-06a, G-01b and the
full user workflow remain open; no integration or model qualification is claimed.

Checklist: pre-effect brief; exact-candidate failure reproduction; canonical
generator resolution; source preservation; real observation oracle; immutable
journal; fresh required CI before landing; no roadmap completion status changes.

Final local checks passed: no unmerged index entries; unchanged collector/review
source; staged whitespace; documentation policy (736 statuses, 734 classified
coordinates); workflow policy; generated navigation (18 indexes, 735 nodes,
2,837 edges); ignored inventory generation/check (1,892 records). Its tracked
retirement marker remains unchanged. The final journal addition is included in
regeneration before committing the ordinary merge. Full CI must rerun on the
new head; no previous native result is relabeled as evidence for it.
