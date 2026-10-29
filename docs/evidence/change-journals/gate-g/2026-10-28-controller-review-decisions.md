# G-06a prerequisite: explicit controller label decisions

**Status:** active local implementation evidence; PR179 dependency and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-review-decisions-20261028`
**Owner:** G-06a offline review evidence
**Pull request:** [180](https://github.com/rrflow/rrflow-development/pull/180)

## Scope and authoring checklist

This package began from development/main
`81fce08e14497bbc280790beefe4666716355aec`. A draft PR with the bounded brief
preceded implementation. The PR179 packet implementation at
`6f37ae55644e6807f2fb25fac227d5e061ffdc90` is an explicit dependency, integrated
with an ordinary merge in this isolated worktree. PR180 cannot land before
PR179; neither the merge nor local checks proves runtime integration.

Previously a packet could be inspected but there was no executable validation
boundary for completed reviewer decisions. The new tool reconstructs the
packet from its original export, verifies every decision, and publishes
immutable development-label evidence. Accept/correct/exclude semantics are
checked; excluded tasks retain provenance without a target. Declared reviewer
and label-author identities must differ. That distinction does not authenticate
people or prove independent judgment. Pending drafts, stale packet identities,
missing/duplicate tasks and unsupported assertions fail before publication.

The owning chain is README/alpha objectives, Gate G and controller-training.
Source review covered the complete PR179 packet builder/publisher and reference,
the new `scripts/inference/review_decisions.py` and
`scripts/inference/test_review_decisions.py`, the new review-decisions reference,
the reusable CI workflow and changed/generated diff. The pre-edit brief names
the source-adaptation decision: existing explicit-review practice becomes an
offline repository-owned boundary, with no sibling code import or canonical
state. No new ML method/dependency needs external algorithm research here.

Created files are the validator, its tests, reference and this journal. Changed
files are the outcome-review link and existing CI test block. Navigation and
file inventory are regenerated. No source is removed. There is no engine
operation, schema/SDK, training scheduler, network request or checkpoint
activation change. Stop conditions are runtime-authority expansion, fabricated
review/admission or unrelated effects. Roadmap/POA&M status is unchanged.

## Validation and retained evidence

| Check | Actual result |
|---|---|
| `python scripts/inference/test_review_decisions.py` | Seven tests passed: complete decisions, authority flags, stale packet/author identity, missing/duplicate/unknown tasks, semantics/evidence, timestamps/version, immutable round trip and changed-source rejection. |
| `python scripts/inference/test_review_outcomes.py` | All 15 dependency tests passed. |
| `python scripts/ci/check_workflow.py` | Passed; full candidate topology unchanged. |
| `python scripts/ci/check_documentation.py` | Passed after generated indexes, 705 statuses before this receipt. |
| External optional UI `npm test` | Six Node tests and TypeScript build passed. |
| External optional UI `npx playwright test` | Two tests passed, covering the existing dashboard and actual review page, draft download/resume, stale rejection, complete export and mobile overflow. |
| Actual browser-export file through the validator twice | Same content-addressed result both times; all 21 tasks explicitly excluded by a synthetic assistant test, admission false. |

The browser test initially used text-content matching on a textarea whose
value had correctly restored. It failed, was corrected to a value assertion,
then passed. A subsequent review-page-only run passed after fixing fixed-sidebar
placement and capturing the fresh pending-review screen. These were UI/test
corrections, not weakened packet or decision checks.

The optional SWAG UI is a separate local file author, not a build/runtime
dependency or Connectome integration. Its complete test export declares
`reviewerKind: assistant`, identifies itself as synthetic, and excludes every
task. No human review or useful semantic labels are manufactured by this test.
The source packet contains 21 real development tasks and 63 retained model
outputs. The test proves transport and validation, not label correctness.

- Source packet digest:
  `e54bfeeef61fef3cd0e31f00d4c45460ce67336badf51138692102fa99d34bb6`.
- Executed validator digest:
  `6e4674b0e0e1c7a94e65a41f2fb14114f372ac2ea32905daa7b13935b67caaac`.
- Synthetic browser review-file digest:
  `545db44ac495c147f80fa0bf571097cd98e1484c6aa1c6a9f2cb589e54563c58`.
- Repeated output digest:
  `c29d18aed738d293de1abceed5b88625208b06e0a7ab5d4e32117d5c866af67f`.

Local operator evidence is retained under the SWAG workspace's dated
`review-ui-test-only` directory, separate from dataset review/admission inputs.
Raw task/model files are not published in this PR. Text fields and review bytes
are bounded; source/reference authenticity remains unverified. No runtime trace
or model-resource experiment is relevant to this local file-validation effect.

## Handoff and limits

The working session owns PR180 follow-through after PR179 lands. Full candidate
CI and merge remain unproved. No training, quantization, installed learned
routing, governed improvement routine or hosted integration was performed.
The validator always retains `identityAuthenticated`, `trainingAdmitted`,
`acceptanceDataset` and `deploymentQualified` as false. Consent, leakage/split
admission and fresh acceptance evidence remain required at their existing
owners; relabelling this development set cannot make it unseen.

Pre-edit scope/dependency, first failure oracle, complete-file/diff review,
source adaptation, focused checks, generated projections, bounded resources,
explicit unrun checks and remaining owners are recorded. Any later candidate
CI result or merge supplies its own revision-bound evidence; this receipt
does not predict it.
