# Connectome pin: the last old branch's remainder

**Status:** active submodule pin update; no H-06 qualification claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-repin-native-client-remainder-20261118`

**Owner:** H-06 Connectome Workbench; checkout boundary in `docs/reference/client/connectome.md#repository-checkout`

**Starting revision:** main `794cf52d5`, which pins `connectome/` at `9f663705`.

**Change.** The gitlink moves to Connectome main `2b8deadec54c3312f4e0ee343474bd918fcaf8b0`, the merge of rrflow/connectome-development#97. That change read `agent/rrflow-native-connectome`, the last old Connectome branch with unabsorbed work, symbol by symbol against `main`. It carries the query result as a table keyed by identity, the dependency rules `main` lacked, a real-engine test that another instance's name and a forged credential are refused, and the branch's design record as evidence. It fixes the rrflowQL language so completions registered on it reach the editor. Its ledger records what was deliberately not carried and three decisions left to the owner.

**Evidence, from the Connectome repository.** Pull request 97's candidate run passed every job: frontend build and smoke, the native crate on Linux, macOS and Windows, and the real-engine integration job (28 tests against the engine at `cc5ebe0fd`).

**Scope.** The `connectome` gitlink and this record. The bound engine revision is unchanged, so `docs/reference/client/connectome.md` is unchanged.

**Not proven.** Locally, one of five runs of the real-engine suite failed the two-project isolation test at a session connect; the message was not captured and it did not recur. Connectome's CI does not run that suite on Windows, macOS or Linux ARM64.

**Acceptance.** The candidate run on this branch and `git submodule status` resolving the new revision.

**Roadmap/POA&M change:** none.
