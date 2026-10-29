# D-02: source-bound plan and explicit replan design

**Status:** active proposed design evidence; no runtime or gate acceptance
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-source-bound-replan-design`
**Owner:** OBJ-14 / D-02 / D-03 / POAM-038 and POAM-039 / PR148

## Brief and reviewed evidence

Development main `86adada6dab6411cc58cb9aaa931e35472c0f452`, tree
`42f1a022bf06f1a5c0593203f9a553e1c7d837e0`, was the starting baseline. Empty
opening commit `6ab5bcb2295346be549be8b9e2e906a07afd6081` opened
[PR148](https://github.com/rrflow/rrflow-development/pull/148) with its brief
before documentation edits. This is a supporting design for one engine-owned
plan and source identity, not an implementation or a replacement objective.

The existing installation precondition is copied into the attunement plan;
ordinary installed engine state has no project-root anchor, the Connect
checkpoint copies that digest without revalidation, and Inventory reports
`phase-unavailable`. The proposal distinguishes that precondition from D-03
source snapshots, describes explicit immutable plan supersession after an
ordinary source edit, preserves the existing phase-level recheck requirement,
and explains the validation/publication race and uncertainty after an external
effect. It keeps question dependencies local to the effects they block and
leaves exploration and SWAG proposals adaptive under ADR-0002.

The reviewed inputs included README's bootstrap and status; ADR-0001 and
ADR-0002; OBJ-14; D-02/D-03 and the project inventory section of engine data
flow; POAM-038, POAM-039 and POAM-041; the change-authoring procedure; and the
relevant source sections of `rrd-engine` `core.rs`, `installation.rs`,
`installation/inventory.rs`, `attunement.rs`, and `rrd-contract`'s
`attunement.rs`. The new design and the complete authored diff were reread.
The source review was scoped to these boundary symbols, not a claim of a full
repository audit.

Created `docs/architecture/source-bound-plan-design.md`, linked it from the
existing source-inventory section of `docs/architecture/engine-data-flow.md`,
linked it from `docs/architecture/README.md`, and added this journal plus
generated navigation and execution inventory. No
implementation or runtime schema was changed, moved, deleted or replaced.
No roadmap, objective or POA&M status changed. Open PRs 142 and 146 are not
assumed merged or qualified.

## Research and review oracle

Reviewed primary Git documentation on status, ignore and conditional ref
updates, POSIX rename semantics, and Microsoft CreateFile sharing semantics;
the supporting design links each source next to the limited claim it supports.
These are evidence for reasoning about the proposed boundary, not proof of an
RRFlow implementation. No upstream code, dependency, resource benchmark or
new tracing authority is introduced.

The design review traces an ordinary edit followed by an explicit replan, an
intervening editor between check and publication, a superseded worker, and an
uncertain external effect. It also identifies a potential tension between the
D-03 zero-content-read unchanged-rerun objective and detecting a same-metadata
content edit: no resolution or weakening of either owner is claimed. A
qualified source-change mechanism or explicit evidence limitation still needs
to be demonstrated. The design requires an independent effect/artifact/audit
oracle instead of comparing a plan with its own generated action list.

## Checks, failures and remaining work

No runtime tests are claimed for this documentation-only package. `python
scripts/knowledge/render_navigation.py --check`, `python
scripts/ci/build_execution_inventory.py --check`, `python
scripts/ci/check_documentation.py`, `python scripts/ci/check_workflow.py`, and
`git diff --check` passed after regeneration. The first documentation-policy
check rejected the coordinated proposal's `proposed` status because coordinated
records must declare `active` or `historical`; the status was corrected to
`active proposed` without implying acceptance. Candidate CI and merging remain
pending. A second documentation check found that the architecture parent index
is maintained as a table rather than by the navigation generator; the missing
link was added. A combined final wording/journal patch failed to match one
journal context and was reapplied with the correct context. No current code
proves this plan/replan design, the required mutation
boundary, a per-prompt handoff, memory receipts, or native capacity and soak.
The checked-in [status-read candidate](https://github.com/rrflow/rrflow-development/pull/146)
is still separate; it explicitly reports source verification as unchecked.

Checklist: main/owners and code boundary reviewed; draft PR brief first;
real-world race and restart scenarios specified; primary sources and limits
linked; one engine authority retained; no unsupported atomicity or release
claim; design/diff review, linked journal, generated indexes and candidate CI
remain the handoff. No user authorization is inferred from this proposal.
