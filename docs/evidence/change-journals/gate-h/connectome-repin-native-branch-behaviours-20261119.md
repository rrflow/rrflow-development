# Connectome pin: three behaviours the native-client branch had and main lacked

**Status:** active submodule pin update; no H-06 qualification claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-repin-native-branch-behaviours-20261119`

**Owner:** H-06 Connectome Workbench; checkout boundary in `docs/reference/client/connectome.md#repository-checkout`

**Starting revision:** main `332b4b095`, which pins `connectome/` at `d6f6c352`.

**Change.** The gitlink moves to Connectome main `20431e2e60ad7eb9cf5b9ea61e9b8512a481e2a1`, the merge of rrflow/connectome-development#101. That change re-read the native-client branch (Connectome pull request 10) by what its tests demand, after an earlier reading by code comparison had called it absorbed. It carries three behaviours `main` lacked: code blocks colour 24 language names where `main` coloured two; a failed engine command keeps its whole error output where `main` kept the last line; and the engine's announced address must be a canonical loopback origin with a valid instance id, where `main` accepted any address beginning with `http://127.0.0.1:`. It also adds the missing test that a refused announcement leaves no engine process behind.

**Evidence, from the Connectome repository.** Pull request 101 passed every job on the candidate and in the merge queue: frontend build and smoke, the native crate on Linux, macOS and Windows, and the real-engine integration job against the engine at `cc5ebe0fd`.

**Scope.** The `connectome` gitlink and this record. The bound engine revision is unchanged, so `docs/reference/client/connectome.md` is unchanged.

**Not proven.** The other Connectome branches that were deleted after a code comparison have not been re-read test by test.

**Acceptance.** The candidate run on this branch and `git submodule status` resolving the new revision.

**Roadmap/POA&M change:** none.
