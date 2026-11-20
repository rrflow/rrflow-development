# Every pull request closed without merging, read by what it set out to do

**Status:** active change-journal evidence; one lost record restored, nothing else claimed

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/unmerged-pull-requests-by-intent-20261119`

**Owner:** repository absorption; continues [the archive absorption record](archive-absorption-20261114.md)

**Starting revision:** main `3fb03316a`.

## Why

The owner's direction of 2026-11-19: work is absorbed when `main` does what it set out to do, not when its code was compared. The archive record compared code. This reading takes the 49 pull requests on `rrflow/rrflow-development` that were closed without merging and asks, for each, whether its own stated purpose is met on `main`.

## Method

Each pull request's head was fetched from `refs/pull/<n>/head` and tested three ways:

1. Is the head an ancestor of `main`? Then its commits landed by another route.
2. If not, does every commit have a change with the same patch identity on `main` (`git cherry`)?
3. If not, the title and description were read for the purpose, every file it changed was compared with `main`, and the archive record's row for its branch was checked against that purpose.

## Result

| Result | Count | Pull requests |
|---|---|---|
| Head is an ancestor of `main` | 30 | 66, 67, 68, 117, 119 to 125, 132, 133, 134, 136, 137, 139, 141, 198, 199, 201, 203, 219, 225, 260, 262, 267, 268, 272, 277 |
| Every commit's change is on `main` | 7 | 24, 25, 135, 151, 189, 192, 234 |
| Read by purpose | 12 | below |

| Pull request | It set out to | On `main` |
|---|---|---|
| 148, 197 | Design source-bound plans and replay across plan successors | the design is `docs/architecture/source-bound-plan-design.md`; building it is owned by the roadmap, not by these |
| 220 | Reduce recurring conflict churn in pull requests | every added line is on `main` (archive record) |
| 231 | List a project's attunement jobs through the public engine | `rrd-contract/src/attunement/discovery.rs` and `engine/attunement/discovery.rs`, with a wider lineage rule |
| 274 | Run the lab from installed definitions with one command | its one file is identical on `main` |
| **286** | **Record the storage flush stall as a POA&M deficiency** | **was not met.** See below |
| 300, 344 | Stop the macOS capacity tests failing on a busy image create or teardown | `run_native_capacity.py` and `run_offline_lifecycle.py` retry and wait in `main`'s own form (archive record). 344's journal is not on `main`; the archive record holds its findings |
| 315 | Lift the cold-tier trigger observations out of an unsynced editor commit | `engine/diagnostic/storage.rs` |
| 316 | Absorb eight leftover branches that had no pull request | every added line is on `main` (archive record) |
| 364 | Give the lab's shared vocabulary its own crate | `crates/evaluation/rrflow-evaluation-core` |
| 365 | Move the lab's data under one folder | superseded by the owner's "Workbench and Lab are one thing"; all 94 files are on `main` at their present paths (archive record) |

For ten of the twelve the purpose was matched against the archive record's earlier reading and a file comparison made today; the code itself was not re-read line by line.

## The one that was lost

Pull request 286 recorded a measured deficiency: a storage flush or compaction step holds the store's lock exclusively, so searches that take under 12 ms stall for 400 to 960 ms several times in ten seconds. It wrote the deficiency as POAM-056 and a journal.

The journal reached `main` (`gate-e/e-04-flush-stall-record.md`). The POA&M entry did not, and POAM-056 on `main` is a different deficiency. The archive record called the branch "content on `main`" because the journal file exists. So since 2026-11-08 the stall has had no owning record, and a journal on `main` pointed at the wrong entry.

Restored as [POAM-087](../../../poam/rrflow-1.0-alpha/poam-087.md), text unchanged, with a provenance note. The cause was read again on `main`: `try_maintenance_step` in `crates/persistence/rrd-store/src/rrflow_kv/store.rs` still takes the write lock for the whole step. The journal's link and the archive record's row are corrected.

## Deleted refs, read by the tests they added

Done later the same day. 135 commits from deleted refs were still readable locally and are not ancestors of `main`. Every Rust, TypeScript and Python test each one added (against its merge base with `main`) was looked up on `main` by name: 2,156 tests added, 29 distinct names absent. Each of the 29 was then read for what it demands.

| Tests | Count | On `main` |
|---|---|---|
| `retained_estate_*`, `prior_source_cannot_cross_estate_project_or_root`, `inherited_estate_preview_reports_retained_ownership` | 11 | present under the engram name; the preview test is `inherited_engram_preview_retains_the_folder_without_effects` |
| Three endpoint contract tests (`*_projected_*`, `captured_model_choice_*`) | 3 | present with their later names in `rrd-contract/tests/endpoint_{worker,policy,selection}.rs` |
| Four Parse executor tests | 4 | three in later form in `engine/tests/attunement/parse_step.rs`; that a Step replay commits nothing has no Parse-specific test, only the general replay path's |
| `gitignored_folders_are_neither_walked_counted_nor_hashed` | 1 | `installation/inventory/tests.rs` (`git_ignored_locked_file_is_never_opened` and its neighbours) |
| Seven macOS capacity tests | 7 | later form in `scripts/ci/test_native_capacity.py`: the template busy-create pair, the exhausted-retry test, and the device ownership tests |
| `adopted_estate_entries_are_bound_only_when_present` | 1 | removed on purpose with the relocation it tested (see the archive record) |
| `zz_kernel_timing` | 1 | a measurement helper, not a behaviour |
| **Two lab tests of explicit native HNSW parameters** | **2** | **were not met.** `main`'s lab built every RRFlow graph with two constants. Restored in [the lab's native HNSW record](../gate-j/lab-explicit-native-hnsw-20261119.md) |

The first pass of the script over-reported 226 absent names because it mis-matched Python tests; a direct `git grep` of every reported name gave the 29.

## Changed paths

- `docs/poam/rrflow-1.0-alpha/poam-087.md` (new), `docs/poam/rrflow-1.0-alpha.md` and the generated indexes.
- `docs/evidence/change-journals/gate-e/e-04-flush-stall-record.md`: owner line and link.
- `docs/evidence/change-journals/repository/archive-absorption-20261114.md`: the `agent/poam-flush-stalls` row.
- This record.

## Not proven

- The stall's measurements are from 2026-11-08 on a tree with unlanded search packages and were not repeated.
- A head that is an ancestor of `main` proves its commits landed, not that later work kept every effect.
- The draft pull request 412 also uses the numbers POAM-087 and POAM-088; it renumbers when it is resumed.

**Acceptance.** `render_navigation.py`, `check_documentation.py` and the candidate run on this branch.

**Roadmap/POA&M status change:** adds POAM-087 as Open, High, owned by E-04.
