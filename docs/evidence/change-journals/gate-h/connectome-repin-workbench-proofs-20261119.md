# Connectome pin: the old workbench checks, proven on today's components

**Status:** active submodule pin update; no H-06 qualification claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-repin-workbench-proofs-20261119`

**Owner:** H-06 Connectome Workbench; checkout boundary in `docs/reference/client/connectome.md#repository-checkout`

**Starting revision:** main `161b11c2a`, which pins `connectome/` at `c3451dcc`.

**Change.** The gitlink moves to Connectome main `e4ae118a20f71f4ac353ca53063a0d3b3cf2c1c5`, the merge of rrflow/connectome-development#105, which follows #104. Both add browser tests and ledger rows and change no product code. They prove on today's components what the workbench's two earlier checks drove through a harness that was not carried: template export and import; every guided step, skill lens and routine step opening a view the project has; and a scripted event authored, approved, run and recorded as a receipt.

With these, Connectome's absorption ledger accounts for everything those two checks asserted: two rules had been lost and are restored (#102 and #103, already pinned), and the rest is proven.

**Evidence, from the Connectome repository.** Both pull requests passed every job on the candidate and in the merge queue: frontend build and smoke, the native crate on Linux, macOS and Windows, and the real-engine integration job against the engine at `cc5ebe0fd`.

**Scope.** The `connectome` gitlink and this record. The bound engine revision is unchanged, so `docs/reference/client/connectome.md` is unchanged.

**Not proven.** That a scripted event stops when the workbench changes under it is held by a unit check in Connectome, not by a browser test.

**Acceptance.** The candidate run on this branch and `git submodule status` resolving the new revision.

**Roadmap/POA&M change:** none.
