# Recovered backup-scheduling stash integration

**Status:** active integration; merge and validation pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/backup-stash-integration-20261121`
**Owner:** [Gate C](../../../roadmap/rrflow-1.0/gate-c.md), C-07 and [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)

## Change brief

Baseline ffb4c4b1779212aa8fa80e2d5908f3685f98d4ab. Integrate recovered stash
a7f8457c7f790e0d0431c2f75488983c864a9903, including the four formerly untracked
files in third parent 5888eb031df3cd577440ea89cf8aa470664d4964. Use ordinary
merge history and retain the behavior at the current qualification boundary;
do not recreate the removed rrflow-eval crate. This is the owner's requested
source integration, not a new comparison-only inventory.

Source map: CLI tests/support/sustained scheduler, lifecycle and management;
qualification-storage sustained, progress and sustained_backup modules/tests;
existing backup journal, two generated schemas, C-07 and POAM-035 evidence.
The third parent contains management.rs, sustained_backup.rs, its tests and
the source journal. Read the complete source journal and tracked path list
before opening the PR; read affected source while resolving each conflict.

Preserve one owned backup task, original deadlines, cancellation, nonblocking
foreground work and truthful cross-segment completion/overlap evidence. Keep
current module splits, stronger later checks and all recovered assertions.
First oracles: qualification backup histories and CLI management tests. Then
owning qualification suite and installed_reopen target, formatting, generated
projections, documentation, candidate CI and merge queue. Stop on lost effects,
weakened evidence or unknown ownership. No benchmark/throughput claim, external
research or new runtime architecture is needed. No release gate is closed.

The stash, its untracked tree and the verified recovery bundle are readable.
No source is deleted before integration; exact resolutions and results follow.

## Integration and evidence

[PR461](https://github.com/rrflow/rrflow-development/pull/461) retains the stash
and its third parent through ordinary merge 3c62907c. The source tracked-tree
merge and four formerly untracked files are accounted for at these boundaries:

| Recovered source | Current destination |
|---|---|
| CLI management.rs | Same file; owned JoinSet, original deadline, nonblocking polling, abort-on-drop and three tests retained |
| rrflow-eval storage/sustained_backup.rs | qualification/rrflow-qualification-storage/src/sustained_backup.rs; bounded tracker and deadline/overlap/completion checks retained |
| rrflow-eval storage/sustained_backup_tests.rs | qualification-storage/src/sustained_backup_tests.rs; all five source test bodies and their assertions retained |
| c-07g-backup-scheduling.md | Existing Gate C journal retained; no duplicate history record or qualification claim |

Resolved the old inline scheduler into the existing campaign/engram/point_run
modules. Kept the sample gate, reopened-corpus progress checks, separate progress
test module and later failure records in the roadmap/POAM. The obsolete incoming
rrflow-eval storage/sustained.rs was removed from the merge tree; its evidence,
validation, writer and summary responsibilities remain in qualification-storage.
The two generated schemas merge without change. No application-source delta
remains against the baseline: this closes the recovered Git integration and
checks the retained behavior. It does not add a newly working backup subsystem.

Read in full: all four third-parent files, current management helper, backup
tracker and backup test file. Read affected scheduler/progress conflict regions
and mapped call sites; no exhaustive review of unrelated code is claimed.

Windows, CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target, CARGO_BUILD_JOBS=4:

- cargo test --locked -p rrflow-qualification-storage --lib backup: six passed.
- cargo test --locked -p rrflow-cli --test installed_reopen management: three passed.
- cargo test --locked -p rrflow-qualification-storage: 54 passed, zero failed.
- cargo test --locked -p rrflow-cli --test installed_reopen: 39 passed, 14 existing opt-in tests ignored.

Logs: D:/.usr/Temp/rrflow-project-policy-20261121/backup-stash-{focused,management,suite,reopen}.log.
No failing assertion was removed or relaxed. Hosted candidate and queue checks
remain required. No new capacity, endurance or performance run was performed;
C-07 and POAM-035 remain open. Other historical work is not marked complete.

Final local checks: documentation policy, cargo fmt --all --check and git diff --check passed.
