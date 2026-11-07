# Joined delivery and source-review workflow correction

**Status:** active draft workflow correction; source review and installed journey incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-joined-delivery-workflow-review`
**Owner:** repository change-authoring procedure and [PR #243](https://github.com/rrflow/rrflow-development/pull/243)

Starting design candidate `a5a8929e749ee667e672f771f20d1a6f58ba519c`; public main `3a72a6b4adc43c842f67a59f4fc91d725eac7580`, tree `68edd7b9519fa363faca0220875e2a3bd2bac596`; separately reviewed parser draft `fa2409867fd906a85f5b98b2e512107dd2044c08`. The PR brief was extended before editing the existing authoring procedure and PR template. This record does not claim those drafts are integrated or accepted.

The observed delivery failure was selecting disconnected native process prerequisites and reporting their component evidence while the owner's installed source-to-index request remained unmet. The prior procedure required an individual package's smallest oracle but did not explicitly require each prerequisite to stay tied to the joined installed driver and its next integration boundary. The authoring procedure and PR template now ask for that driver, current failure, integrated candidate and next dependency; the [indexing plan](../../../roadmap/rrflow-1.0/project-indexing-execution-plan.md) names the existing installed test and distinguishes its older failing revision from the current draft. Full candidate and merge-group gates, safety, independent readback and roadmap ownership are unchanged.

A read-only Git-object inventory of the exact public-main tree was written locally to ignored `target/rrflow-review-inventory-3a72a6b4adc4.jsonl`: 2,107 UTF-8 text files / 589,387 lines, nine binary files and one gitlink. Every entry is `not_attested`; this is a baseline, not evidence that any main line was reviewed. The gitlink pins another tree whose contents are not included. The workspace reported 204 registered worktrees, three prunable registrations and 15 open PRs at inspection; this is evidence of fragmentation, not permission to delete anything or a claim about each branch's content.

The following complete files were examined in parser candidate `fa2409867fd906a85f5b98b2e512107dd2044c08`; these objects do not occur in the main baseline inventory:

| File | Git blob | Lines |
|---|---|---:|
| `crates/authority/rrd-engine/src/engine/attunement/windows_supervisor.rs` | `5c16f7618b25fb1b312eb2efd63b5c1dcf71fa92` | 260 |
| `crates/authority/rrd-engine/src/engine/attunement/windows_supervisor/tests.rs` | `c3b2268438fce562c8f258652fb785ae7cb8a06f` | 266 |
| `crates/authority/rrd-engine/src/engine/attunement/linux_supervisor.rs` | `c58bbbf4f5057eb1f952059116435119c124ba19` | 134 |
| `crates/authority/rrd-engine/src/engine/attunement/linux_supervisor/tests.rs` | `2178ade289ce7d08a88de69ccc9b267ea08112d9` | 210 |

The implementations are explicitly disconnected from the installed Parse dispatcher and still lack executable-identity admission, bounded source pipes, full cancellation/reaping and publication; Linux's per-child keeper is not a resource-bounded scheduler. Both process-death tests also exposed a fixture race: they can read a created PID marker before it has been fully written. [PR #241](https://github.com/rrflow/rrflow-development/pull/241) owns the test-only correction and native reruns. These findings do not imply an independent full security review or any indexing proof.

The local `rrflow-joined-delivery` skill was created as a discoverable routing aid to the repository's existing owners; it is not a repository runtime, hook, lifecycle or second status authority. Its metadata validator passed; that validation checks structure, not future agent behavior. No product code, roadmap completion status, candidate CI or native qualification is claimed by this workflow correction. The full main source review, joined installed indexing driver and C-07 evidence remain incomplete.

Reviewed: README, current objective, change-authoring procedure, implementation navigation, indexing design, PR template, skill-creator instructions, current PR state and the four complete candidate files above. Source and PR state were inspected before the procedure edit; no implementation was deleted, moved or rewritten in this documentation package. `python scripts/ci/check_documentation.py`, `python scripts/knowledge/render_navigation.py --check`, `python scripts/ci/check_workflow.py`, and `git diff --cached --check` passed. Exact-candidate and merge-group CI remain required.
