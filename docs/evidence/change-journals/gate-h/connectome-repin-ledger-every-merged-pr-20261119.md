# Connectome pin: its ledger of every merged pull request

**Status:** active submodule pin update; no H-06 qualification claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-repin-ledger-every-merged-pr-20261119`

**Owner:** H-06 Connectome Workbench; checkout boundary in `docs/reference/client/connectome.md#repository-checkout`

**Starting revision:** main `d709abfa4`, which pins `connectome/` at `e4ae118a`.

**Change.** The gitlink moves to Connectome main `eb31a718f73c3588fbdb210567d3d4aa518ae6de`, the merge of rrflow/connectome-development#106. That change adds one section to Connectome's absorption ledger: each of its 99 merged pull requests taken in turn by the tests it added. 272 tests were added, 261 are on its `main` by exact name, and the other 11 were each read: renamed with the engram name, removed with the instance limits, or dropped on purpose with a recorded reason. No behaviour was found lost in a merged pull request. No code changes.

**Evidence, from the Connectome repository.** Pull request 106 passed every job on the candidate and in the merge queue.

**Scope.** The `connectome` gitlink and this record. The bound engine revision is unchanged, so `docs/reference/client/connectome.md` is unchanged.

**Not proven.** A behaviour a pull request shipped without a test cannot be found this way.

**Acceptance.** The candidate run on this branch and `git submodule status` resolving the new revision.

**Roadmap/POA&M change:** none.
