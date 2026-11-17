# Joined checkout baseline (direct-main plan superseded)

**Status:** active evidence; its direct-main delivery plan is superseded by the pull-request-per-package process of 2026-11-16 (PR #398); no release or absorption claim
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/joined-checkout-baseline-20261116`
**Owner:** J-01/J-02 repository prerequisite, H-06 Connectome; POAM-027

> **Superseded 2026-11-17.** This journal was written while a second session
> held the owner's earlier "commit to main" instruction. The owner rejected that
> rule on 2026-11-16 and [AGENTS.md](../../../../AGENTS.md) now requires one
> branch and one pull request per package, merged through the queue. Read the
> "direct commits and pushes to main" parts below as history, not as the plan.
> The Connectome source inventory, the `corepack pnpm check` result and the
> release-workflow defect remain valid evidence at the stated revisions.

## Change brief

Starting engine revision: `e94a060090d06abc6b6698170d1ded11f5440d43` on
`main`, clean, tracking public `development/main`. Connectome is clean but
detached at `a7af4e0cf630e9f785658ce073b6b16cdecea876`; its remote main is
`1f930fd8f1fd42ee98655ce348b52a53a0fcbbfc`. The engine session check reports
success without inspecting the submodule. The change-authoring and CI records
still prescribe the PR workflow superseded by the owner's one-branch rule.

Target: make the existing session check report the complete source checkout,
including initialized submodules and live upstream state; align operational
instructions with direct commits and pushes to main; establish the exact
Connectome source and executable baseline without discarding existing work.
This supports OBJ-01/OBJ-09, not their completion.

Scope: session check and its regressions, owning operational instructions,
POAM-027, this journal and generated navigation; Connectome integration and
build only after inspecting its actual divergence. No official promotion,
version change, release-status change, force push or history rewrite.

First oracle: the old session check returned exit 0 and "OK" while Connectome
was detached and held numerous side branches/worktrees. Regressions must
reject detached/uninitialized/dirty submodules and stale remote observations.
Run the focused Python suite, documentation/navigation checks, then the
development gate for the exact pushed engine revision. Connectome needs its
frontend checks/build and real native-engine driver before any usability claim.

Research/source adaptation: none; this repair is governed by local source,
the explicit owner rule and live GitHub API observations. Trace/debug decision:
retain Git identities, command exits and build/test output; no runtime tracing
or performance claim. Stop destructive cleanup unless each old source has a
verified bundle and a content/behavior disposition. Stop integration on source
divergence rather than replacing the current workbench with an older tree.

## Initial evidence

- Engine CI run `44004208956` passed for the exact starting revision.
- GitHub `rules/branches/main` returned deletion and non-fast-forward rules
  only. It does not currently require PRs or the merge queue.
- No open engine issues were returned by the open-issues API request.
- Connectome CI run `43643963565` passed for remote main, not the detached
  workbench revision. These are distinct candidates.
- `origin/main..HEAD` in Connectome contains `a7af4e0c`, the file-based routing
  and original-workbench restoration. `HEAD..origin/main` contains the merge
  commit `1f930fd8`. Its second-parent tree has no diff from the merge tree.
  A checkout of remote main alone would remove the unpublished workbench work.
  This is source inventory, not proof of absorption or rendered behavior.

## Files reviewed

Owners: AGENTS.md, README.md, docs/operations/agent-sessions.md,
docs/roadmap/rrflow-1.0-execution/change-authoring.md,
docs/guides/getting-started.md, docs/reference/client/connectome.md,
docs/operations/ci.md, docs/roadmap/rrflow-1.0.md,
docs/poam/rrflow-1.0-alpha.md and poam-027.md. Implementation:
scripts/ci/check_session_end.py, scripts/ci/test_session_end.py,
.github/workflows/ci.yml, .gitmodules. Connectome: README.md,
docs/native-workbench.md, package.json, scripts/check-menu.mjs,
tests/visual/capture-native.mjs, .github/workflows/draft-release.yml.
The Connectome CI workflow has only been read in part so far.

## Results and remaining work

Before the first implementation patch, another session switched the shared
checkout from main to `process/restore-pr-flow`, replaced the session check
and its tests with PR-required behavior, and modified AGENTS.md and the
operational owners. The patch's context check failed, so no implementation
change was applied. That session then committed and pushed `e1a6cec34`.
This contradicts this session's explicit main-only user instructions. The
conflicting files were not overwritten, the branch was not switched back,
and the user was asked to stop the concurrent editor or clarify its authority.
This journal is the only tracked-source addition authored by this session;
it remains uncommitted rather than being committed to the prohibited branch.
The generated index is already modified by the other session and was left alone.

Independent validation: `corepack pnpm check` in Connectome at `a7af4e0c`
exited 0 on Windows. It checked 622 files, passed the menu floor (27 project
views including all 17 original views), checked 83 API examples, and passed
the value/query, disclosure, graph, installation facts, guides, dependency and
version checks. pnpm synchronized the existing node_modules to the lockfile.
It warned that the available Node v24.21.0 is below the declared >=24.22.0;
there is no supported-toolchain qualification claim. Connectome's source
worktree remained clean. No native binary or installer was built or launched.

Additional source-reviewed release defect: Connectome's
`.github/workflows/draft-release.yml` invokes tauri-action without staging
the engine or selecting `src-tauri/tauri.bundle.conf.json`. The default
`tauri.conf.json` has `externalBin: []`. `src-tauri/build.rs` returns without
embedding a sidecar manifest when it is absent. Thus this workflow does not
assemble the documented engine-bearing desktop distribution. That conclusion
is from source review, not an executed release. The release workflow also
needs target-matched engine staging for its macOS cross-target matrix.
H-06/Gate J/POAM-013 own the eventual distribution proof.

Implementation, regression tests, navigation update, final diff review, push
coordinates and exact-candidate CI evidence remain pending. No old Connectome
work was absorbed or removed. No installed release, full project journey,
hosted sync, sequencer scalability, nest isolation or adaptive personalization
is proved. No roadmap or POA&M status was changed. No processes remain from
this session's completed frontend check. Next action: resolve the concurrent
PR-policy editor, then carry the existing Workbench source onto main without
losing its contents and qualify the engine-bearing desktop build.
