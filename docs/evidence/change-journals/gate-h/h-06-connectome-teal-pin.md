# H-06: pin Connectome after the guided window and the new palette

**Status:** active source integration evidence; parent candidate pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-06-connectome-teal-pin`
**Owner:** H-06 project UI integration prerequisite; POAM-011

**Outcome and package.** OBJ-09 and the H-06 UI-pin prerequisite. On 2026-11-03 the repository owner
asked for Connectome's palette to be merged, and the owner's standing order for Connectome is land,
move the pin, then design.

**Starting revision and pull request.** `bf0b86a4`, this repository's `main`. Pull request:
[#269](https://github.com/rrflow/rrflow-development/pull/269).

**Change brief.** The gitlink moves from `c1cbff20`, Connectome `main` after #27, to
`95ba9975c506886f79984ad956e5dba557369f93`, Connectome `main` after #38. It stays bound to engine/SDK
`15f36a8758e32b33c930a20da836def3e5cc3207`. `.gitmodules`, every engine crate and default engine
readiness are unchanged. Stop conditions: a gitlink other than the landed `main`, a change outside this
scope, or a candidate CI failure. None was reached.

**What the pin brings.** Connectome #31 to #34 (sprint 016, steps 1 to 4: preserving the Workbench
reference, one window, three inspector depths, and guides), #36 (the plan checked against the code), and
#38 (dark teal and burnt orange, by token value with no change to any view or control).

**Files read in full.** `docs/evidence/change-journals/gate-h/h-06-connectome-chain-pin.md`,
`docs/poam/rrflow-1.0-alpha/poam-037.md`, and the pin statements in
`docs/reference/client/connectome.md`.

**Paths changed.** The `connectome` gitlink, `docs/reference/client/connectome.md`, `poam-037.md`, this
journal and the generated `gate-h` index.

**Research decision.** None needed. This package moves an optional source pin and records evidence.

**Trace, resource and debug decision.** Not applicable, because no engine code changes.

**Oracle and result.** `git ls-files --stage connectome` shows mode 160000 at
`95ba9975c506886f79984ad956e5dba557369f93`. That revision's tree equals PR #38's tested head, so what
landed is what was checked.

**Connectome evidence, from #38.** `pnpm check`, `tsc` and 4 smoke tests pass. `capture-native.mjs` at
1366x768 dark and 1440x900 light, on an engine at `15f36a87`: 56 captures each with no failed step,
reviewed by eye on about 15 screens and not all 112. Its candidate CI passed on every platform, and it
landed through the merge queue.

**Acceptance commands and results.** Recorded in the pull request, as they depend on this commit.

**Surfaced failures and corrections.** The first attempt at #38's captures failed twice, both plainly
reported by the application: installation refuses a hard-linked executable, which cargo's output is, and
a newer engine failed the public-contract check. A copied engine at the pinned revision passed.

**Checks not run.** No engine or SDK test, because neither changed. The UI is not qualified by this pin.

**Remaining known errors.** Connectome still binds engine `15f36a87`, while this repository's `main` has
moved on, so a later engine needs its own pin move and contract check. POAM-037's knowledge-tree editor
is still absent. 8 of 26 planned views are in Connectome.

**Change checklist.** Scope kept. Gitlink verified. Generated index regenerated.

**Status change.** None. No roadmap, objective or POA&M lifecycle status changes.

**Pull request, CI and merge.** Recorded on the pull request.
