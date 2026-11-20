# Recovered scheduling, conflict policy and UI stash reconciliation

**Status:** active recovery evidence; remaining historical behavior reconciliation is incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/recovered-scheduling-and-ui-20261120`
**Owner:** [POAM-011](../../../poam/rrflow-1.0-alpha/poam-011.md), [C-07](../../../roadmap/rrflow-1.0/gate-c-07.md) and [Gate H](../../../roadmap/rrflow-1.0/gate-h.md)

## Brief and scope

[PR446](https://github.com/rrflow/rrflow-development/pull/446) was opened with
its brief before this record was authored. Starting main:
`561ca59b782d0b2960f5a0daf9505683e299fd50`; Connectome:
`26d1f83cf9fb10118f6e93dee70750f30ca7db3c`. Driver: the owner's ongoing
historical-code recovery session. This extends the
[initial inventory and recovery receipt](historical-pr-recovery-20261120.md).

Current to target: items previously inventoried without a complete behavior
comparison now have explicit source maps, reviewed differences and checks.
This package changes this journal, its generated index and the POAM-011 link.
It adds no engine capability, changes no release status, and rewrites no Git
history. Parser framing repair remains the separate PR445. Unexplained losses,
absent source, weakened constraints or failed checks stop the dependent claim;
they are not resolved by merging an old tree wholesale.

## Backup scheduling stash

Input `a7f8457c7f790e0d0431c2f75488983c864a9903`: read all tracked deltas
and all four third-parent files. Current implementation uses the same owned
task and streaming evidence boundary; it does not need another implementation.

| Recovered behavior | Current destination and review |
|---|---|
| One task, original absolute deadline, nonblocking collection, errors and cancellation on owner drop | Complete `crates/adapters/rrflow-cli/tests/support/sustained/management.rs`; identical after the existing estate-to-engram naming change. Its three tests retain these oracles. |
| Start before launch; carry a backup across segments; settle only at the final boundary; keep session alive | `sustained/point_run.rs` lines 305-383 and 448-509, return field at 579; `sustained/engram.rs` lines 288-361; parent `sustained.rs` task member. Launch/checkpoint, late completion poll and session-rotation guard remain. |
| Verify the actual saved archive and bind its original start | `sustained/lifecycle.rs::backup` and `backup_started`, lines 1-65, retain create and verifying-list calls and receipt equality. |
| Cross-segment start, acknowledged-write overlap, bounded state, deadline and completion validation | Complete `crates/evaluation/qualification/rrflow-qualification-storage/src/sustained_backup.rs` and `sustained_backup_tests.rs`, identical to the recovered files after the naming change. Six histories retain missing/duplicate/forged/expired/unsettled rejection. |
| Copy-before-publication and offline chain reconstruction | `sustained/writer.rs` lines 110-254 and `sustained/summary.rs` lines 84-112 and 270-390. The tracker advances only after accepted publication; the verifier reconstructs it and rejects unfinished completed campaigns. |
| Raw progress and accepted-start equality | `progress.rs` lines 375-407 and 550-628; its recovered exact-start test is retained in `progress/tests.rs`. |
| Generated evidence shape | The `backup_started_ns` property in `eval/schemas/storage-segment-v1.json` and `backup_started` variant in `storage-progress-event-v1.json` are structurally identical to the stash additions. |
| Foreground work while backup receipt is pending, then restart/readback | `sustained/capacity_workload.rs` opening and lines 460-560 retain the recovered real-process smoke assertions. The held receipt is test coordination; archive creation/verification are real engine operations. |

The complete recovered `c-07g-backup-scheduling.md` and the tracked C-07/POAM-035
additions were read. Their qualification limits remain applicable: integrated
growth, profile repetitions and the 24-hour soak are not established by these
checks. Source ranges above are the reviewed boundaries, not a claim that every
line of each large current module was reread.

Checks ran on root `84f31dc6d9ff6935ba14024f5721fdbbb50185c4`, the parser
repair candidate. The backup implementation is unchanged from this package's
main baseline. With four Cargo jobs and the checkout's existing scratch target:

- `cargo test --locked -p rrflow-qualification-storage --lib backup`: six passed.
- `cargo test --locked -p rrflow-cli --test installed_reopen management::tests`:
  three passed.
- `cargo test --locked -p rrflow-qualification-storage`: 54 passed, zero failed
  or ignored (43 library and 11 integration checks).
- `cargo test --locked -p rrflow-cli --test installed_reopen installed_capacity_workload_smoke -- --ignored --nocapture`:
  one passed in 27.15 seconds after a 49.05-second debug build. The receipt
  reports eight records, 24 edges, 16 vectors, 12 replayed transactions and
  `management_followup_verified: true`. Forced process termination, reopen and
  independent readback passed. `capacity_qualification` remains false.

The installed binary SHA-256 is
`72d96b1843bb3d198c2bbac40fb556e21ac08e0a2abadadcdbe4d30c0a4cab97`.
Logs: `stash-backup-tests.log`, `stash-backup-management-tests.log`,
`stash-backup-suite.log` and `stash-backup-installed-smoke.log` in the scratch
directory named below. These are Windows observations, not cross-platform
capacity or competitor measurements.

## PR220 and the two child stashes

Root [PR220](https://github.com/rrflow/rrflow-development/pull/220), head
`5697b208b5269f76817d437cb219f79b2434c500`, was closed without a merge flag.
All 334 added lines remain at their current paths. More substantively, its
`scripts/ci/check_documentation.py`, `inspect_merge_conflicts.py` and
`test_merge_conflict_policy.py` are exact blob matches. Read the full historical
patch, including CI wiring, all added owner prose and the complete 29-line
journal. The changed policy functions also compare identically by AST source.
The policy allows sparse ordered POAM IDs while retaining historical first-parent
IDs; merge inspection changes neither worktree nor refs and refuses configured
external merge drivers. The existing ten tests pass, including malformed and
pathless conflicts, shallow-history refusal and nonmutation. Its code is carried
and these behaviors are verified; no old branch needs to replace current CI.

Child `8cf93f88632928a9b0a101d01ec472d2c71f4339` contains two tracked deltas.
Read both in full and the entire current `src/features/runtime/live/use-live-changes.ts`.
That hook is an exact blob match (`7fc9ee7d898c3f6b42b8531647a2b1a3ed14aa26`).
On subscription it invalidates cached graph data when the stream head exceeds
the cached cursor, as well as on reconnection/replacement. The recovered browser
oracle remains in `tests/smoke/connectome.spec.ts:92`; it passed in the unchanged
PR109 source tree's 36-case smoke run (`template-smoke-full.log`). This browser
test uses the native bridge fixture; it does not newly prove real stream replay.

Child `3e644ed080e068b4756f94e5dd47444c8a7e88db` contains two tracked deltas
and has no third parent. Read both in full. Plain installation facts and the
exact expandable plan remain in `components/ActionBar/installation-facts.ts`
and `installation-summary.tsx`, both read completely. `projects.tsx`, also
read completely, now scopes errors/actions to each project; `project-row.tsx`
lines 165-235 display the summary and explicit install action. The old
"Use Existing project" error recovery button is replaced by directory-based
mode selection before planning, not silently lost: `add-project.tsx` lines
1-130, native `rrd/engram/mode.rs` in full, and `engram.rs` mode/plan binding
were inspected. Empty folders select fresh, nonempty folders select existing;
the returned mode and identity are frozen into the reviewed request. Explicit
review still precedes installation. The installation-facts script remains in
`pnpm check` and passed in `template-check.log`; no new manual native-dialog
run or whole UI-parity qualification is claimed here.

## Repository integrity correction

After PR444, preserved `.git/objects/info/commit-graphs` under
`commit-graphs-before-rebuild` in scratch. The first write with
`core.commitGraph=false` did not rebuild anything and ordinary fsck still
failed. After verifying the resolved metadata path, moved only the stale
`commit-graph-chain` to scratch and ran
`git -c core.commitGraph=true commit-graph write --reachable`.
Ordinary `git fsck --unreachable --no-reflogs --connectivity-only` now exits 0;
log `root-fsck-after-graph-rebuild.log`. No commit, source ref or history was
rewritten. The independent recovery bundles in the prior receipt are retained.
This corrects the derived-cache problem, not historical coverage.

## Checklist and remaining work

Scratch: `D:/.usr/Temp/rrflow-selective-recall-audit-20261120`. Research and
upstream adaptation are unnecessary for this local-history comparison. Trace
consists of preserved test output, object identities and the installed receipt;
no new runtime instrumentation. Reused existing build outputs, with four Cargo
jobs. No user process, source branch or unreconciled historical object is deleted
by this package. The PR brief precedes edits; the maps above bind current/target
behavior, owner, oracle, scope and stop conditions. Full changed-record reread,
diff review and generated/documentation checks precede ready publication.

Local record checks passed: documentation policy (1,283 statuses, 1,279
coordinates), navigation (21 indexes, 1,280 nodes, 4,501 edges), execution
inventory (3,689 records), and `git diff --check`. PR220's ten policy checks
were rerun for a retained log (`pr220-policy-tests.log`), passing in 4.135
seconds. Candidate and merge-queue checks remain required after publication.

This removes backup scheduling, PR220 and the two child stash deltas from the
previous record's unreviewed list at the exact scope stated. The remaining
historical PRs, parser-to-durable-record behavior, clock installation stashes
and the rest of child PR10 still need reconciliation. Clock comparison so far
read the recovered/current clock module and historical apply/recovery deltas;
it is incomplete and is not an absorption verdict. No roadmap, POAM lifecycle,
native capability-parity or visual-parity status is changed. Next: continue
those source comparisons and repair any concrete losses through bounded PRs.
