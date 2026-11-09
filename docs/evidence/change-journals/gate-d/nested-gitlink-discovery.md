# Read-only tracked Git submodule discovery

**Status:** active implementation evidence; selection and candidate qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/nested-gitlink-discovery`
**Owner:** OBJ-07 / OBJ-14, D-01 / D-03 / H-06 and POAM-006
**Pull request:** [233](https://github.com/rrflow/rrflow-development/pull/233); depends on [232](https://github.com/rrflow/rrflow-development/pull/232)

## Bound change and real-project result

Starting development revision `5b4039fe930332895c1b24138891d39f600030c7`,
tree `3c8e935c29e879bccef6162e257e3ebdfaffc049`. PR233's change brief was
published before code edits, following empty anchor
`b54d55f7859dbe966929ee243a71ecc33b61249b`. The anchor push resolved to
`https://github.com/rrflow/rrflow-development.git`, remote `development`,
`refs/heads/codex/nested-gitlink-discovery`, that exact revision. No official
repository was a destination.

The baseline actual executable rejected `install discover` as an unrecognized
subcommand; the real Connectome preview independently rejected its external
Gitfile. The existing source walker already recognizes tracked gitlinks but
records them as excluded in the committed inventory. The new read-only engine
operation and `rrflow install discover` project that existing detection into a
sorted candidate list. The response names the bundled profile and observed Git
input digests, reports whether an index was present and checksum verified,
recommends separate projects, and states `selection_available: false`. Coverage
is explicitly `present_tracked_gitlink_directories_only`; it does not claim
uninitialized gitlinks, arbitrary nested repositories, a complete source
snapshot or an authorized installation plan. It does not open child Gitfiles.

The first real-project green run used independently copied Windows executable
`target/real-project-build/rrflow-discovery-candidate.exe`, SHA-256
`b6e4a2a20ff8a82582de1311b0d4dfc24b577d013ba628b90f4d2e50bed1b140`, built
from source tree `d5945998f771d2a32acf479c28277b788e763fae` over the starting
revision. It ran `rrflow-discovery-candidate.exe --json install discover
--project D:/localdev/devlabs/rrflow` twice. The actual RRFlow checkout was
`f256c8e1337d85fa01f2a0c61e895a4d213163fa`; its tracked Connectome gitlink and
child checkout were both `38142b77110a8f1db77e306e7d1ad375fd918267`. Both runs
returned candidate `connectome`, `git_index_verified: true`, the separate
recommendation and `selection_available: false`, with the same Git-input
SHA-256 `11b1476866393cb55fdd04e4c2c24cb2502a32c1b89b4995c793db652f304fdd`.
The second raw JSON is `target/real-project-build/rrflow-discovery-real-result.json`,
SHA-256 `8815d00448cdceaa99a3f84f284f55f8247fa221d9676abcaef609bf53f7ca0f`.
Git status was clean before and after for both checkouts; status alone does not
prove ignored files unchanged. The isolated CLI oracle separately compares all
files in its small project and verifies no `.rrflow` was created. No actual
project plan was applied and no service, model or provider was started.

A later read-only inspection narrowed the separate root-install failure recorded
in PR232: the existing `.rrflow` directory has an unprotected inherited Windows
DACL, and baseline `EstateFilesystem::admit` checks it first. The locator
`.rrflow/config.toml` is absent, while `.rrflow/rrd/CURRENT` exists. Even after
private admission, `reject_existing_estate` refuses an unrecognized `.rrflow`
tree. No secret or database contents were read, no ACL changed, and no state
was removed. These observations do not authorize adoption or deletion; a
separate reviewed resolution must preserve the existing state.

The real-process binary predates a later test-only fixture hardening and new
CLI test; those edits did not change the production implementation. The source
and test tree after those changes is `4da5324bf039e763722c46686fef996dbce1d63f`.
Neither tree nor this local receipt is a candidate CI result.

## Implementation, research and traceability

`engine/installation/inventory.rs` reuses `collect_project_entries` and moves
its existing final observation checks into one shared helper. Discovery uses
the bundled policy's entry, metadata, Git input, elapsed and output bounds,
retains the anchored parent, rechecks admitted entries, Git observations and
root identity, and never treats metadata revalidation as an atomic filesystem
snapshot. `inventory/git_inputs.rs` exposes only whether the already verified
index exists. The parent-source digest is informational; apply does not accept
it as a selection or grant. `engine/installation.rs` owns the typed response
and engine operation, its module exports it, and the primary CLI dispatches it.
No storage format, installation-plan format, operation catalogue, runtime Git
subprocess, new dependency, client state or product version changed.

The real Git-index test initializes its own temporary repository using Git
`2.55.0.windows.5`, sanitizes Git directory/index and global configuration
environment, and changes child `.git` and `.gitignore` without changing the
discovery result. A changed gitlink object changes the input digest. The test
also covers sorted results and an untracked nested directory; the no-index case
and output-bound denial are independent. The real-process CLI test uses an
isolated executable and empty PATH. Test-only Git is a declared qualification
input, not an installed-product runtime requirement.

Primary references: [Git submodules](https://git-scm.com/docs/gitsubmodules),
[Git index format](https://git-scm.com/docs/gitformat-index), and
[Git worktrees](https://git-scm.com/docs/git-worktree). No source was copied.
Source identities, bounded redacted errors and existing admission are reused;
no content trace, new signal, performance/RSS or comparative claim is added.
The operation scans admitted parent entries and is not claimed to be constant
work as unrelated files grow.

## Verification and surfaced failures

Windows, `CARGO_TARGET_DIR=target/real-project-build`, two jobs, dev/test debug
information disabled:

- Baseline real CLI red: `install discover` unrecognized.
- `cargo build --locked -p rrflow-cli --bin rrflow`: passed; the full source
  build used for the first real-project green run completed in 3m06s.
- `cargo test --locked -p rrd-engine --lib discovery_ -- --nocapture`: two
  passed, zero failed; test build 6m18s.
- `cargo test --locked -p rrd-engine --lib`: 292 passed, four existing ignored,
  zero failed; 79.72s test execution.
- `cargo test --locked -p rrflow-cli --test installed_lifecycle
  repository_discovery_is_read_only_without_an_installed_engine -- --nocapture`:
  one passed. The complete `installed_lifecycle` target: eight passed, zero
  failed; 27.86s test execution.
- `cargo fmt --all -- --check`: passed after formatting. The initial format
  check failed on the new source layout, and `cargo fmt --all` corrected it.
- `git diff --cached --check`: passed for staged source.
- `cargo clippy --locked -p rrd-engine -p rrflow-cli --all-targets -- -D warnings`:
  passed, 5m51s including dependency checking.
- `python scripts/knowledge/render_navigation.py --check`: passed; 18 indexes,
  848 nodes and 3,080 edges. The nearest index was regenerated.
- `python scripts/ci/build_execution_inventory.py --check`: passed after
  regeneration; 2,165 records.
- `python scripts/ci/check_documentation.py`: passed; 849 statuses and 847
  classified coordinates, including parent indexes and local links.

Candidate CI and PR232 are pending.

## Review, remaining work and checklist

Read README, AGENTS, the change-authoring procedure, instance topology, Gate D,
POAM-006, Connectome contract, installed lifecycle, ADR-0002, Git-input and
installed-trial journals; the current inventory walker, GitInputs, proposal, policy, CLI command and
installed dispatcher; and the installation plan and anchored filesystem paths.
Reviewed all changed Rust files and tests and the full diff. The source map is:
existing Git index facts and anchored walker -> new read-only engine projection
-> primary CLI; the original inventory and apply paths remain canonical and no
module is moved or deleted.

Changed `rrd-engine` installation/inventory/git-inputs and its exports;
`rrflow-cli` command, dispatcher and installed-lifecycle test; the installed
lifecycle command reference; this journal and generated navigation/inventory.
No other source is changed. No status is closed. Full user choice, plan binding and revalidation of selected membership,
Gitfile admission, arbitrary nested-repository discovery, two actual installed
instances, a shared instance, Connectome, cross-estate links, persistent
source-to-graph/vector indexing, hosted allocation and idle/wake remain open.
PR232 supplies the owner decision; this PR remains draft until reconciled with
it and its exact candidate passes the repository CI. No release or production
capability claim follows from this bounded slice.

Checklist: pre-edit PR brief; exact baseline; real red and green; source/owner
mapping; primary research; resource and trace choice; focused and owning tests;
actual project readback; retained limits and failures; no upstream code copied.
Generated indexes, documentation checks, complete diff review and candidate CI
must be completed before ready/merge.

## 2026-11-09 main reconciliation

Starting revision `0505e2e7042cee9f66b64f9f7703d4e4e6fb59e8` on
`codex/nested-gitlink-discovery`, with a clean worktree. The locally available
`development/main` was `a9f1b0fb8d948e113dca347e5eeac10dd224330a`, 421 commits
ahead. This bounded reconciliation retains OBJ-07 / OBJ-14's D-01 / D-03
discovery prerequisite and does not change roadmap or POA&M status. The owner
explicitly requested an ordinary local merge, the five acceptance checks below,
and local commits only; no push or pull-request creation/edit is authorized.
That instruction supersedes the procedure's PR-edit and push steps for this run.

Change brief and traceability: merge main's current implementation and carry
`NestedRepositoryDiscovery`, `RrdEngine::discover_nested_repositories`,
`inventory::discover_gitlinks`, `GitInputs::has_verified_index`, the
`InstallAction::Discover` route, the two engine discovery tests, the CLI
read-only discovery test, this journal and the installed-lifecycle reference
into the same canonical files. Keep main's explicit installation source-read
choice, installed job discovery and adaptive selection records. The failure
oracle is the installation-filtered engine suite followed by the entire CLI
package; no failing test may be deleted. Stop if completing this scope requires
an unauthorized effect or unavailable filesystem/tool access.

`git merge development/main` stopped with four content conflicts:

- `crates/authority/rrd-engine/src/engine/installation.rs`: retained the new
  discovery response alongside main's engram configuration loader and retained
  source-level estate loader alias.
- `crates/authority/rrd-engine/src/engine/mod.rs` and `src/lib.rs`: retained
  discovery exports alongside `InstallationPlanningOptions` and main's
  `ProjectMapSummary` exports.
- `docs/evidence/change-journals/gate-d/README.md`: generated index requires
  regeneration, not a hand merge.

The CLI route, focused tests, inventory walker/Git-input changes and lifecycle
reference merged automatically. The added reference paragraph now says
"engram"; historical receipts and main's retained wire/source identifiers are
preserved. No test expectation or production discovery behavior has been
changed during conflict resolution. No new dependency, source adaptation,
external research, runtime trace or performance claim is needed for this merge.
Compiler/test output supplies the diagnostic evidence.

Reviewed AGENTS, README, the change-authoring procedure, the alpha objective,
Gate D and POAM-006 ownership, the independent-engine integration rules, the
original branch diff and journal, and the relevant installation, inventory,
Git-input, export, CLI dispatch and lifecycle-test sections. These are scoped
reads, not a claim of reviewing all 421 incoming commits. The generated-index
renderer and documentation checker entry points were inspected for validation.

Environment: Windows; Rust `1.99.1-x86_64-pc-windows-msvc`;
`CARGO_TARGET_DIR=./target`; `TEMP` and `TMP` set to this worktree's
`target/tmp` so test/build temporary files remain local. No shared target is
used. Validation and final state will be recorded below.

Surfaced environment failures so far:

- `git restore --source=development/main --worktree --
  docs/evidence/change-journals/gate-d/README.md` and `git add --` the three
  resolved Rust files: denied creating `.git/index.lock`. The session's
  filesystem policy marks this worktree's `.git` read-only.
- `python scripts/knowledge/render_navigation.py`: `python` was not on PATH.
  Retrying with a session alias to the available `python3.15.exe` launcher
  failed: `uv trampoline failed to spawn Python child process`, permission
  denied (OS error 5). Navigation has not been regenerated yet.

Acceptance results, executed in the requested order against the merged working
tree (HEAD is still the starting revision because a commit is blocked):

| Command | Actual result |
|---|---|
| `cargo test --locked -p rrd-engine --lib installation` | Passed: 64 passed, 0 failed, 1 ignored subprocess fixture, 336 filtered out; build 2m14s, tests 2.47s. Both discovery tests and the explicit source-read choice test passed unchanged. |
| `cargo test --locked -p rrflow-cli` | Passed: 71 passed, 0 failed, 15 ignored across all targets; build 2m23s. The installed-lifecycle target passed all 9 tests in 20.75s, including read-only repository discovery and explicit source-read installation/reopen. |
| `cargo clippy --locked -p rrd-engine -p rrflow-cli --all-targets -- -D warnings` | Passed, 51.48s. |
| `cargo fmt --all -- --check` | Initially failed only on wrapping the reconciled `src/lib.rs` exports. `cargo fmt --all` corrected that formatting; the repeated check passed. |
| `python scripts/ci/check_documentation.py` | Attempted with `python` aliased to the available Python 3.15 launcher; interpreter startup denied with OS error 5. The checker did not execute. |

CLI counts by target: binary unit tests 8; backup controller 1; build identity 1;
estate admin 1; installed lifecycle 9; installed quantization 1; installed reopen
39 passed / 14 ignored; lifecycle process 11 passed / 1 ignored. Ignored native
campaign/fixture tests were not enabled. No failing test was removed, no
assertion changed, and no test asserted a replaced main contract in these runs.
The only correction after testing was rustfmt's whitespace-only export layout.

`git diff --check --` the affected engine, CLI, lifecycle reference and journal
paths passed before final journal completion. Reviewed the complete discovery
delta against `development/main`, including the shared revalidation helper,
exports, unchanged focused tests and CLI route. Main's source-read authorization
and adaptive selection implementations were preserved, not replaced by a
compatibility path. No broader engine suite, workspace suite, native campaign,
candidate CI, push or PR action was run; those are outside this local check list.

Final stop: Git index writes and Python execution remain unavailable. An
explicit `git merge --abort` also failed with permission denied creating
`.git/index.lock`; the environment therefore prevents both permitted consistent
end states (commit or clean abort). No commit was created. The merge remains in
progress: the three Rust conflict bodies are resolved but cannot be staged;
the generated Gate D index still has its conflict pending Python regeneration.
Main's staged Connectome gitlink is `4da67e27f52ef20861fd9ee8c8351ce48920f49a`;
its unchanged, clean nested checkout remains at
`38142b77110a8f1db77e306e7d1ad375fd918267`. No submodule checkout was attempted
because its Git metadata is also under the read-only `.git` directory.

The dated journal is also copied to ignored `target/main-reconciliation-journal.md`
for preservation if the merge is later aborted. All writes were confined to
this worktree. No release/objective/roadmap/POA&M status changed; these are local
execution-environment blockers, not a newly demonstrated product deficiency.
The branch is **not ready for candidate CI**. Resume with writable worktree Git
metadata and an executable Python: regenerate navigation, run the documentation
check, stage the resolutions, align the clean submodule checkout to main's pin,
review the final diff, and commit the ordinary merge locally. Do not push or
edit PR233 as part of this task.

### Completion of the 2026-11-09 reconciliation

The merge Codex resolved was completed from a shell with writable Git
metadata: the three Rust resolutions were staged unchanged, the generated
Gate D index was regenerated with `render_navigation.py`, the Connectome
gitlink was aligned to main's pin `4da67e27`, and the stray `Microsoft/`
directory created by the blocked Python launcher was removed. Documentation
policy, navigation check, formatting, the engine installation tests, the
CLI tests and strict Clippy were rerun before the merge commit; results are
in the commit message. Candidate CI remains the acceptance.
