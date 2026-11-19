# Connectome pin: the project manager without instance limits

**Status:** active submodule pin update; no H-06 qualification claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-repin-project-manager-20261119`

**Owner:** H-06 Connectome Workbench; checkout boundary in `docs/reference/client/connectome.md#repository-checkout`

**Starting revision:** main `dbb116377`, which pins `connectome/` at `fc4d04fe`.

**Change.** The gitlink moves to Connectome main `d6f6c3521b396d8616cdf3f0d7f5f4bfc1af79ca`, the merge of rrflow/connectome-development#98. That change removes the agent-written limits of 4 engines, 8 tracked instances, 8 sessions and 64 known projects; gives every project its own start, stop and remove with its own state and error; adds a project by its folder and starts it; removes a project with a stated confirmation; and pauses instance polling while the window is hidden.

**Evidence, from the Connectome repository.** Pull request 98 passed every job on the candidate and in the merge queue: frontend build and smoke, the native crate on Linux, macOS and Windows, and the real-engine integration job against the engine at `cc5ebe0fd`. Locally on Windows: 83 native unit tests, 31 real-engine tests and 29 browser tests passed.

**Scope.** The `connectome` gitlink and this record. The bound engine revision is unchanged, so `docs/reference/client/connectome.md` is unchanged.

**Not proven, and not done.** Connectome still binds the engine at `cc5ebe0fd`, which has no `rrflow instances` or `rrflow stop`. So it cannot yet list an instance it was never told about or stop one it did not start; its own text says so. That needs Connectome's engine pin to move to a revision with rrflow/rrflow-development#412.

**Acceptance.** The candidate run on this branch and `git submodule status` resolving the new revision.

**Roadmap/POA&M change:** none.
