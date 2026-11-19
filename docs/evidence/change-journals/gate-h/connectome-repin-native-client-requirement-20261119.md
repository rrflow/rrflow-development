# Connectome pin: the native client requirement and its first package

**Status:** active submodule pin update; no H-06 qualification claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-repin-native-client-requirement-20261119`

**Owner:** H-06 Connectome Workbench; checkout boundary in `docs/reference/client/connectome.md#repository-checkout`

**Starting revision:** main `f6de9515d`, which pins `connectome/` at `2b8deade`.

**Change.** The gitlink moves to Connectome main `fc4d04fe48e35d39a6c4641658afbf26769c11df`, the merge of rrflow/connectome-development#100, which follows #99.

- **#99** makes the intent of the last two old Connectome branches a requirement on Connectome's `main`: Connectome runs entirely on RRFlow's own client, with every inherited capability equal or better. A check measures what is left from the tree and refuses anything new built on the inherited client. Both branches were then deleted.
- **#100** is the first package against it. The native client now carries a query's typed parameters to the engine (it sent none), and the Query lab gains variables, inference, completion, a pre-run check, formatting and run-selection.

**Evidence, from the Connectome repository.** Both pull requests passed every job on the candidate and in the merge queue: frontend build and smoke, the native crate on Linux, macOS and Windows, and the real-engine integration job against the engine at `cc5ebe0fd`. The requirement's check reports, at this pin: 4 inherited dependencies, 55 files reaching the inherited client, 289 of 348 inherited capabilities not yet native, 66 visual items not yet shown.

**Scope.** The `connectome` gitlink and this record. The bound engine revision is unchanged, so `docs/reference/client/connectome.md` is unchanged.

**Not proven.** The real-engine suite ran locally on Windows (29 passed) and in CI on Linux only. Live queries still send no parameters.

**Acceptance.** The candidate run on this branch and `git submodule status` resolving the new revision.

**Roadmap/POA&M change:** none.
