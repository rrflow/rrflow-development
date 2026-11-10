# C-07 targeted macOS native preflight

**Status:** active implementation-stage evidence; native preflight pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-targeted-macos-preflight`
**Owner:** OBJ-02 / C-07f / POAM-035 diagnostic prerequisite
**Pull request:** [261](https://github.com/rrflow/rrflow-development/pull/261)

## Baseline and failure

Public main `ed3027b79e41dfcdf5b27e57f07346f11f6da347`; unchanged-tree
bootstrap `9efe1643392e10c06070d8d0302c2be5fcddc6ca`, tree
`18fbef0dc4a271f5ce8004cd865b9272b1115a9c`. The PR brief preceded edits.
The pull-request candidate for #256 passed, but its protected merge-group
[run 41600990700](https://github.com/rrflow/rrflow-development/actions/runs/41600990700)
failed on macOS in the installed native capacity fixture before the selected
WAL-pressure test. The initial data fill wrote 257,097,728 bytes in 1,167 ms.
Metadata preparation then reached 758 files in 892 create attempts, 139
settlement barriers, 6,709,782 microseconds creating and 17,120,724
microseconds settling before the existing 25-second bound. This is an observed
fixture timeout; it does not establish acknowledged data loss or a passing
native WAL/recovery case. Diagnostic upload found no matching JSON artifact.

The existing manual preflight defaults to all four points, both install modes
and three repetitions, and therefore takes 24 native cases to reproduce one.
The selected change makes point, mode and repetitions explicit inputs for that
manual characterization while retaining its complete original default. The
required candidate and merge-group workflows, selected native error oracle,
recovery/history checks, cleanup, limits and timeout remain unchanged.

## Implementation and provenance

The manual workflow passes quoted input values to a Python argument-vector
wrapper. The wrapper derives point names from the existing native runner,
allows one to three repetitions and rejects unknown inputs before any child
is started. It prints the selected scope and delegates to the existing runner,
returning its exit status. The finite repetition ceiling preserves the
pre-existing maximum; it is a diagnostic resource bound, not a reasoning
sequence. The workflow-policy check now requires manual-only dispatch,
unchanged default values, the wrapper and its test, input wiring, the original
native-failure capture, exact revision and controlled artifacts. Candidate CI's
independent required native matrix checks were not changed.

Read in full: the existing manual workflow, metadata and APFS fill helpers,
prior C-07f APFS settlement and metadata-payload journals, and the focused
selector and workflow-policy sections. Targeted reads: `capacity.rs` fill,
settlement, native error and recovery flow; `run_native_capacity.py` command
and ownership; current CI and failure log. [Apple's APFS description](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/APFS_Guide/FAQ/FAQ.html)
describes copy-on-write and asynchronous reclamation; it does not predict this
runner's timing. A prior native failure proved that data pressure on one file
alone does not necessarily stop another APFS file, so metadata pressure is
not removed. No upstream code was copied. The existing phase counters and
native diagnostic artifacts remain the trace; no guessed latency claim is made.

## Validation and outstanding work

The initial selector test failed because the new wrapper was absent. After
implementation `python scripts/ci/test_macos_capacity_preflight.py` passed
five tests for exact default, selected cases, invalid values, no invalid-input
dispatch and child failure
propagation. The workflow-policy test initially rejected the new input shape;
the validator and negative tests now validate the manual input contract and
still enforce the unchanged required candidate matrix. `python
scripts/ci/test_native_capacity.py` passed 36 tests and `python
scripts/ci/check_workflow.py` passed. The first focused Ruff run found an
unsorted import and nested context managers; both were corrected and the two
new Python files pass Ruff. A broader Ruff check reports 17 findings in the
two previously existing workflow test/validator files; the same 17 occur in
their unchanged baseline bytes. They were not reformatted as part of this
diagnostic change. Documentation and navigation checks passed.
The existing CI operations record now explains the optional selector and
retains the complete default and non-qualification warning.

The selected native macOS preflight has not yet run. This change is a
diagnostic tool, not a fix for the 25-second failure and not proof of capacity,
recovery, soak, C-07 completion or alpha readiness. A real targeted run and
its exact revision and artifacts are necessary to investigate the fixture;
any later fix still requires the original full candidate and protected
merge-group CI. No product, roadmap or POA&M status changes.

Checklist: owner, exact baseline and PR brief; retained native failure and
oracle; research, resource and trace decisions; no weakening of required
tests; focused and owning test results with first failures; one linked
journal; generated navigation and complete diff review before publication.
Stop for any change that bypasses required CI, suppresses errors, touches an
unowned volume or substitutes a preflight for qualification.

## 2026-11-09 main reconciliation

Merged `development/main` (243 commits ahead of this branch's base) by an
ordinary merge; the only conflict was the generated Gate C index, regenerated.
`test_macos_native_failure_preflight.py`, `test_native_capacity.py`,
`check_workflow.py` and documentation policy pass on the merged tree.
