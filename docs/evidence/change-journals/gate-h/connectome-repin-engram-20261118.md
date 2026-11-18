# Connectome pin: the UI built against the renamed engine contract

**Status:** active submodule pin update; no H-06 qualification claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-repin-engram-20261118`

**Owner:** H-06 Connectome Workbench; checkout boundary in `docs/reference/client/connectome.md#repository-checkout`

**Starting revision:** main `cc5ebe0fd`, which pins `connectome/` at `a7af4e0c`.

**Current behaviour.** The pinned UI was 65 commits behind Connectome `main` and bound engine `b099b5bc`, from before the engine renamed estate to engram (POAM-078, 2026-11-15). Step 13 of [the rename review](../repository/engram-rename-review-20261114.md) was never done: "the pinned UI builds against the old revision and cannot talk to a renamed engine". So this checkout's engine and its pinned UI were not one system.

**Change.** The gitlink moves to Connectome main `9f6637056936147153e528da47eebf26372567e5`, the merge of rrflow/connectome-development#96. That revision binds engine `cc5ebe0fd` for the Rust client and the TypeScript SDK and says engram throughout. It also carries the absorbed work landed since the last pin: the Workbench runtime actions and guided routes (#85), per-view instance lifecycle (#90), the capture scripts and guide wording (#91), the extracted Workbench state (#92) and the inherited capability parity record (#95). `docs/reference/client/connectome.md` names the bound engine revision.

**Evidence, from the Connectome repository.** Pull request 96's candidate run passed every job, including `RRFlow SDK and RRD process integration`, which builds the engine at `cc5ebe0fd` and runs the 27 real-engine tests on Linux x86_64. Locally on Windows the same 27 tests passed against the sidecar staged from `cc5ebe0fd`, and the three engine fixtures they rewrite differ from the previous pin only in the revision and the two renamed names.

**Scope.** The `connectome` gitlink, one sentence in the Connectome reference, and this record. No engine, adapter, installation or default-readiness change.

**Not proven.** The real-engine tests do not run on Windows, macOS or Linux ARM64 in Connectome's CI; on those platforms its CI compiles, unit-tests and lints the native crate only.

**Acceptance.** The candidate run on this branch and `git submodule status` resolving the new revision.

**Roadmap/POA&M change:** none.
