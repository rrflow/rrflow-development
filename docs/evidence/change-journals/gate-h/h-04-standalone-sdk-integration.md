# H-04: integrate standalone rrflowDB and SDK candidate

**Status:** active draft integration; SDK prerequisite and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-standalone-sdk-integration`
**Owner:** OBJ-01 / OBJ-04 / D-01 / H-04 / POAM-006 / [PR194](https://github.com/rrflow/rrflow-development/pull/194)

## Brief and traceability

Starting standalone head `e4f561f7b24f7214303b6867193f8edc9fd4de2b`, tree
`4fafff5602fcc5c8f53fd27cc06b8b38b5e1db2d`. Its brief was extended before
normally merging [PR177](https://github.com/rrflow/rrflow-development/pull/177)
exact head `1ab64897bcee1440159d19846e1bf0a6eb3d02b2`. Both contain
[PR193](https://github.com/rrflow/rrflow-development/pull/193), which passed
its candidate and protected merge-group checks and landed as
`291fee511887932572ecc9935e9786e098727677`. PR177 is now the first landing
dependency, and its own exact candidate and merge group remain required.

The sole textual conflict was the generated Gate H journal index; regenerated
it from all retained records. The workflow auto-merge retains both SDK native
manifest privacy steps and the standalone build/run steps. Existing hard-link
denial, client and runtime behavior, all required jobs and immutable evidence
remain. No product implementation is moved or removed. The complete staged
diff against each parent was reviewed; no upstream source is adapted. The
smallest failure oracle is the native standalone process and independent
expected task/relation readback, together with the Windows DACL tests.

Reviewed README, the D-01/H-04 and POAM-006 owners, current CI and authoring
policy, both preceding standalone and SDK integration journals, standalone
runner/tests, workflow policy and the actual merged conflict and source diff.
No new telemetry or resource threshold is introduced. The prior isolated
Windows build's Rust inputs and standalone script are unchanged in the merge:
`git diff --cached e4f561f7 -- crates Cargo.toml Cargo.lock
 scripts/ci/run_rrflowdb_standalone.py docs/reference/sdk/rrflowdb-local.md`
shows only `crates/transport/rrd-client/examples/sdk_conformance_server.rs`,
which is a different example and is not an input to the standalone executable.
This is source-diff evidence, not a fresh clean-head build.

## Local Windows evidence

`python scripts/ci/test_rrflowdb_standalone.py`: 7 passed;
`python scripts/ci/test_sdk_private_manifest.py`: 4 passed and the POSIX case
skipped; `python scripts/ci/test_candidate_admission.py`: 11 passed;
`python scripts/ci/test_execution_inventory.py`: 10 passed;
`python scripts/ci/check_workflow.py` passed. The original standalone
processes were re-run through `python scripts/ci/run_rrflowdb_standalone.py`
on this merged working tree, using the identical previously built binaries:
primary SHA-256 `d5fc391c6a091ab57b3cd7f8a8da81133cc0b0a7722c54ce3ac4ec81c902313f`,
example `55e0be6f213bce4abb9feb08e6ea8a507c30fb536a6db2c7f9ab4396a092ebc6`.
It passed actual fresh install, authenticated task/relationship read and
mutation, abort, independent wrong-state rejection, offline verification and
same-instance reopen at cursor 7. Local output under
`target/standalone-194-sdk177-real-process.log` SHA-256
`f162adc553518b584ef968009cf497a2e57fe9a1e39144c9e38e6c5bf973fc80`.
The tested binary is the earlier isolated build, not a new clean candidate
artifact, and this is a small synthetic project rather than capacity proof.

The first merge attempt reported the expected generated-index conflict; no
semantic conflict was ignored. Candidate and merge-group CI for this combined
head, cross-platform hosted execution, 100k capacity and soak have not run.
No alpha, roadmap or POAM status changes. Checklist: linked pre-edit brief,
exact dependency, preserved implementation and independent tests, conflict
regeneration, full diff review, local pass/skip and unrun evidence recorded,
linked journal and generated inventory before commit.

`cargo fmt --all -- --check`, documentation policy (738 statuses / 736
classified coordinates), navigation check (18 indexes), inventory check
(1896 records) and staged/unstaged diff whitespace checks passed on the
integrated working tree. No new hosted candidate run was started.
