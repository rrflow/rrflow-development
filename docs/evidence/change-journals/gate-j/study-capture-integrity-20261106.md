# Study capture and planning integrity integration

**Status:** active draft integration evidence; provider provenance unqualified
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/study-capture-integrity-20261106`
**Owner:** OBJ-10, OBJ-11, J-04 and POAM-010; PR 267

Starting study head `d276f9f489e8407d5d6a991d470fbc9db76e72fb` was merged
with PR 262 head `a7587fed7d7cf4f279bee90ffe06f0cfacfae8b4` in an isolated
worktree. The tested source/index tree before this journal was
`5b3e05f1eab39059e304e7f5e082f67659df599a`. The PR brief records the
current/target behavior, bounds and dependency order. This increment does not
invoke a model, train, publish project data or claim answer quality.

## Change and findings

The merge had conflicts in `lab/tasks.rs` and `tests/lab_tasks.rs`. The
resolution keeps study capture, tool failure and spend accounting while using
the prerequisite's historical-file-match names, scheduled-pair completeness
and correlated-repetition refusal. A system reports partial if any scheduled
run lacks a usable reply and required capture. Schemas were generated from the
combined types. The checked-in pilot's catalog digest was refreshed after the
prerequisite changed the canonical manifest, and its floor test uses the
corrected floor-match verdict identifier.

The study checker previously returned `can_reach_significance` for repeated
runs even though the joined report cannot perform a clustered significance
test. The new test first failed on that claim, then passed after the checker
reported the design unqualified. Pilot behavior remains descriptive. This is a
measurement integrity repair, not a substitute for an executable task oracle.

The existing `a_capture_proof_has_to_cover_the_confirmatory_study_it_is_named_by`
test constructs a capture audit using a scripted provider and has the check
accept that audit. Neither a matching digest nor the audit structure proves a
real provider executed. The user-facing check text now says provenance is not
authenticated, and POAM-010 records this open draft deficiency. Until a real
execution receipt is bound and tested against substitution and forgery, do not
use this draft to authorize a confirmatory claim; this change does not create
such an attestation mechanism.

## Evidence and checklist

- Inspected the complete PR brief, the change-authoring procedure, POAM-010,
  the relevant study/capture and reporting functions, conflict hunks, and
  affected tests; the earlier package journals describe the underlying study
  implementation. This is not a claim of whole-repository line review.
- Initial combined focused `cargo test --offline --locked -p rrflow-eval --test lab_tasks --test lab_study` produced 21 study passes and two failures: the superseded `no-leak` verdict name and a changed catalog digest. Correcting those produced 23 study and 23 task passes. A new repeated-task oracle then failed as expected before the checker repair.
- `cargo run --offline --locked -p rrflow-eval --bin rrflow-lab -- schemas --out eval/schemas` regenerated projections. The full `cargo test --offline --locked -p rrflow-eval` passed after the significance repair, including 24 study and 23 planning tests; `cargo clippy --offline --locked -p rrflow-eval --all-targets -- -D warnings` passed after the subsequent audit wording change, followed by 24 passing study tests. The later focused missing-capture test passed and verifies a mixed-coverage arm is partial.
- The source/test/POAM/pilot/requirements/schema paths are enumerated by `git diff --cached --name-only a7587fed7d7cf4f279bee90ffe06f0cfacfae8b4`; this journal and its generated index are also added. No implementation paths were deleted or moved in this increment.
- Research decision: retain the existing preregistered paired comparison rationale, but do not extrapolate independent-pair statistics to correlated repeated tasks. No upstream source was copied. Trace/resource decision: retain existing capture and max-per-run accounting; missing cost cannot be silently treated as zero, and incomplete tool/capture observations cannot become evidence of an incorrect plan.
- Not run: an actual provider, real agent patch/test oracle, RRFlow-assisted answer comparison, training, candidate CI, or merge-group CI. Existing raw scripted event streams do not attest their origin. Current workflow is not evidence of alpha or competitiveness.
- Checklist: owner and dependency recorded; exact conflicts and tests reconciled; failures and limitations retained; projections regenerated; no status advanced. The PR remains draft pending full diff review, authenticated capture, executable oracle, prerequisite merges and protected CI. The newly verified capture-provenance deficiency is in POAM-010.
