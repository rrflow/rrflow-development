# Connectome pin: Workbench lanes and honest dataset checklist

**Status:** active submodule pin update; no H-06 qualification claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-repin-workbench-lanes-20261110`

**Owner:** H-06 Connectome Workbench; checkout boundary in `docs/reference/client/connectome.md#repository-checkout`

**Starting revision:** main `036d36cc`, which pins `connectome/` at `38142b77`.

**Change.** The gitlink moves to Connectome main `19ee075cbd7d5ef5062f94fecab9b28a07fc69c2`, the merge of rrflow/connectome-development#45: the Workbench lanes (dataset authoring, MCPL learning, linked paths and cloud docs) mounted beside the prompt workbench, the native path probe and write commands restored, and the generated execution checklist carrying a typed status per check instead of hard-coded passes. Evidence for that change is the H-06 journal in the Connectome repository (`docs/evidence/change-journals/gate-h/h-06-connectome-workbench-recovery-20261108.md` there): full check, build, native clippy and tests, and 13 of 13 smoke cases including the new lane case; its candidate run 42735405297 passed.

**Scope.** The `connectome` gitlink and this record. No engine, adapter, installation or default-readiness change: the optional UI stays an optional public-contract integration.

**Acceptance.** The candidate run on this branch (full lane, because the gitlink changed) and `git submodule status` resolving the new revision from a clean clone.

**Roadmap/POA&M change:** none.
