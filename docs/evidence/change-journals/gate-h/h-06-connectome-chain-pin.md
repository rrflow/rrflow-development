# H-06: pin Connectome after the #24–#27 landing chain

**Status:** active source integration evidence; parent candidate pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-06-connectome-chain-pin`
**Owner:** H-06 project UI integration prerequisite; POAM-011

**Outcome and package.** OBJ-09 and the H-06 UI-pin prerequisite. On
2026-11-01 the repository owner chose an order: land Connectome PRs 24, 25,
26 and 27 in turn, move RRFlow's pin, and only then design and restore views.

**Starting revision.** `development/main`
`15f36a8758e32b33c930a20da836def3e5cc3207`, tree
`a674bdf4a428b4b0af0d4ec8ed921aee7f52c9b0`, on branch
`agent/connectome-pin-2026-11-01`, in
[PR #239](https://github.com/rrflow/rrflow-development/pull/239).

**Change brief.**

- **Current:** the `connectome` gitlink is
  `38142b77110a8f1db77e306e7d1ad375fd918267` (Connectome PR13), bound to
  engine/SDK `4860a253`.
- **Target:** Connectome `main` `c1cbff203ecee89013878609d11aed04e89d0e71`,
  after PR 27. It is bound to engine/SDK
  `15f36a8758e32b33c930a20da836def3e5cc3207`, the RRFlow `main` this package
  starts from. That revision carries E-04c (#190), E-04d (#191) and POAM-045
  (#211).
- **Exact scope:**
  - the gitlink;
  - the [client reference](../../../reference/client/connectome.md)'s
    statements about the pinned workbench;
  - one sentence of POAM-037 evidence;
  - this journal, its generated index and the execution inventory.
- **Unchanged:** `.gitmodules`, every engine crate and the default engine
  readiness. No UI source is copied into RRFlow.
- **Stop conditions:**
  - a dirty or unreviewed child checkout;
  - a gitlink other than the landed `main`;
  - a change outside the scope above.

**The four Connectome changes in this landing chain.** Each landed through
Connectome's merge queue after its candidate CI passed:

| PR | Change | Merged as |
|---|---|---|
| [24](https://github.com/rrflow/connectome-development/pull/24) | Each project's instance lifecycle (Sprint 009 F4a) | `999c144a` |
| [25](https://github.com/rrflow/connectome-development/pull/25) | Search generations and an exact-versus-TurboQuant comparison in the Vectors lens (Sprint 009 T1) | `d90c947d` |
| [26](https://github.com/rrflow/connectome-development/pull/26) | Native task and controller-proposal workspace (sprints 013 and 014) | `423a13cb` |
| [27](https://github.com/rrflow/connectome-development/pull/27) | Endpoint credential mapping at project start (sprint 015) | `c1cbff20` |

Relative to the old pin, the gitlink also takes up Connectome PRs 9 and 14 to
23, which had already landed on Connectome `main`:

- the theme and the merge-queue gate;
- RRFlow isolation and the visual baseline;
- the API reference and the re-authoring plan;
- governed effects, live results, the value model and the rrflowQL language.

`git log --merges --first-parent 38142b77..c1cbff20` lists all fourteen.

**Child evidence.** Each of the four PRs records its integration in its own
journal:

- Connectome's Sprint 009 journal, step T1;
- sprints 014 and 015, "Integration onto main".

Every merged head carried the same engine pin, `15f36a87`, and each passed:

- `pnpm check`;
- `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`;
- the native unit tests;
- every real-engine process test against that engine;
- native and inherited before/after captures in both themes, with every
  changed screen explained.

**Files read.**

- RRFlow: `AGENTS.md`; the root README's checkout warp point; the sections of
  the [Connectome client reference](../../../reference/client/connectome.md)
  that name the pin; POAM-037; the earlier
  [project workspaces pin receipt](h-06-project-workspaces-pin.md); and the
  linked-journal template.
- Connectome: its `.agents` sprint journals and its capture harness.

**Changed paths.**

- the `connectome` gitlink;
- `docs/reference/client/connectome.md`;
- `docs/poam/rrflow-1.0-alpha/poam-037.md`;
- this journal and the generated `gate-h` index.

The execution inventory is generated under `target/` and is not committed.
Its `connectome` record now names the new gitlink.

**Decisions.** No research, adaptation, trace or resource change. This
package moves an optional source pin and records evidence.

**Oracle and results.**

- `git ls-files --stage connectome` shows mode 160000 at
  `c1cbff203ecee89013878609d11aed04e89d0e71`.
- The child checkout is clean at that revision. Its tree,
  `94d9e3dc201f5b27a690450911aed1a15cf90d31`, equals the tree of PR 27's
  tested head `722f9fac`.
- `py -3 scripts/knowledge/render_navigation.py`, then `--check`: 18
  generated indexes, 865 nodes, 3156 edges. The first run updated the
  `gate-h` index.
- `py -3 scripts/ci/build_execution_inventory.py`, then `--check`: 2206
  current, generated and planned path records. The `connectome` record
  carries `git_revision` `c1cbff20…`.
- `py -3 scripts/ci/check_documentation.py`: knowledge ownership, 866
  document statuses, 864 classified coordinates, parent indexes and
  local links.
- `py -3 scripts/ci/check_workflow.py`: 5 workflows, 9 substantive jobs.
- `py -3 scripts/check_version.py`: the release train is 1.0.0.
- `git diff --cached --check`: clean.

**Failures, checks not run and remaining errors.**

- The first inventory run did not see this journal, because it was not yet
  tracked. The run was repeated after `git add`.
- No RRFlow engine, UI build or browser suite was rerun for this pin. The
  child PRs ran them, and the parent candidate pipeline owns this PR.
- macOS and Linux desktop runs belong to Connectome's CI.

**Diff review.** The whole authored diff and the gitlink were reviewed. The
generated index and inventory are projections, not lifecycle authority.

**Status.** No roadmap gate, POA&M lifecycle status, product version or
official promotion changes. The pin is source association; it does not
qualify the UI or H-06.

**PR, CI and merge.**
[PR #239](https://github.com/rrflow/rrflow-development/pull/239) is opened,
not enqueued. Its candidate CI and merge are pending.
