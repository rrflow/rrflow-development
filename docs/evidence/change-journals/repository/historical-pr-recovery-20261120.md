# Historical PR and stash recovery, with lossless Connectome templates

**Status:** active recovery evidence; historical behavior reconciliation remains incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/historical-pr-recovery-20261120`
**Owner:** [POAM-011](../../../poam/rrflow-1.0-alpha/poam-011.md) and [Gate H](../../../roadmap/rrflow-1.0/gate-h.md); repository integration follows the [session standard](../../../operations/agent-sessions.md)

## Package brief

Starting root revision: `e708980de22028d09fcd12c7410bf560c79d3596`.
Starting Connectome revision: `e780304b39afe8becee4beca63c1f8566d0ba5fd`.
Driver: the owner's ongoing historical-code recovery session.
Parent [PR444](https://github.com/rrflow/rrflow-development/pull/444) depends on
[Connectome PR109](https://github.com/rrflow/connectome-development/pull/109).
Both briefs were opened before implementation in their repositories.

The previous package established the condition of current development branches,
not that every historical PR merged or every historical behavior survived.
This package makes that distinction inspectable, preserves recoverable objects
and integrates the concrete template loss repaired in PR109. Scope: the child
gitlink, this journal and its inventory, nearest generated index, and POAM-011
evidence link. No engine implementation or status is changed by this parent.

The failure oracle is the recovered template equality assertion against the
current parser, followed by real browser upload/download and invalid-batch
rejection. The old code dropped opaque authored fields and unresolved links,
changed timestamps and silently skipped/coerced invalid content. PR109 replaces
that parser at `templates/template-interchange.ts`, preserves the latest-state
and stale-workbench checks in `lanes/workbench-state.ts`, and reports unresolved
links as unavailable. Unknown effects remain rejected. Its linked child journal
records the source map, recovered tests and exact acceptance commands.

Stop conditions: absent source, unexplained behavior differences, loss of
authored content, weakened authorization, or missing required candidate/queue
checks. An unexplained difference stays unresolved rather than being merged
blindly or called absorbed. No official-repository ref is changed here.

## What the historical records actually establish

The [old absorption record](archive-absorption-20261114.md), read at its opening,
stash/re-audit section and final reconciliation, explicitly limits its line
review: for refs below 97% line presence, it read the five files with most missing
lines; lines beyond those files were not read. The record separately used
ancestor, patch-equivalent, empty, carried and superseded dispositions. Those
are not interchangeable with a tested no-loss behavioral integration.

The paginated GitHub closed-PR inventory at the starting revisions contains:

| Repository | Closed PRs | Merged | Closed without merge | Unmerged heads already ancestors of main | Unmerged heads outside main |
|---|---:|---:|---:|---:|---:|
| rrflow-development | 413 | 364 | 49 | 30 | 19 |
| connectome-development | 94 | 91 | 3 | 0 | 3 |

All 52 closed-unmerged head objects are present locally. The exact numbers,
titles, head revisions and ancestry results are in the
[inventory](historical-pr-recovery-20261120.json). These are mechanical facts,
not a count of missing features. The current development remotes having only
main before this package did not prove this historical work was complete.

Individual dispositions newly examined in this package:

| Historical input | Current evidence and remaining limit |
|---|---|
| Root PR300, busy macOS image creation | Its create/retry/owned-leftover cleanup intent is in `scripts/ci/run_native_capacity.py::darwin_template`, with bounded retries and stricter timeout/held-output handling. The old per-case create is replaced by one pristine template and a fresh copy per case. This is a deliberate replacement, not a missing retry to cherry-pick. |
| Root PR344, escaped output holder | Read its added retry/wait code, current `CommandOutputNotClosed::wait_closed`, current template retry, tests and both owning journals. The holder-release check, no-retry failure, retained image and diagnostic flag are carried. `wait_for_device_release` and its four tests were deliberately removed when create stopped following each case's detach; the existing [replacement journal](../gate-c/c-07-hdiutil-held-output-retry-20261114.md) records that reason and limits. No new real macOS busy-create observation is claimed. |
| Root PR286, flush-stall record | Read its complete 31-line addition and current journal. The record is present; its original POAM-056 number collided and the deficiency was restored as [POAM-087](../../../poam/rrflow-1.0-alpha/poam-087.md). Recording the stall does not repair storage maintenance. |
| Root PR234, PR192, PR135 | Their exact heads are empty opening commits (`git show --stat` and GitHub file lists). No implementation is supplied by those commits. Their product/evidence intent is not thereby complete. |
| Child PR42 | Empty opening commit. The current child ledger maps assessment UI work from that worktree to PR82; this package does not newly qualify the entire assessment journey. |
| Child PR48 | Deliberate failing merge-alarm test, titled "Do not merge". It must not be merged as product implementation merely to change its closed status. |
| Child `fa8d769c85d9c165d07dcb76b43f50594a9aeba1` | Actual lost template behavior reproduced and repaired in PR109. This is not a review of all 337 files in child PR10. |
| Root stash `a2004fd90840c8fd7061420e8b33f4b8bfed1a40` | Read every tracked delta: shared `Arc<Manifest>`, admission identity/content checks and two lifetime tests. Current destinations are `rrd-lsm/src/database.rs`, `database/transactions.rs`, `segment/reader/views.rs` and `segment/reader_lifetime_tests.rs`. Both original behavioral tests remain and pass on Windows: the old view survives publication/database drop, shared descriptors are not copied, and denied admission releases ownership. No throughput claim. |

Root PR315's cold-source counters and streaming index decoder were located at
their split current destinations (`runtime/coldlight.rs`, diagnostic storage,
vector search operations/results, contract diagnostic sections and
`rrd-query/src/execute/read_path.rs`). The relevant historical patch was read,
but the complete current paths and every generated surface are not newly
qualified here. Counter presence does not prove columnar cold history or recall.

Unreviewed remainder: full behavior reconciliation of root PR365, 364, 316, 315,
274, 231, 220, 197, 189, 151, 148, 25 and 24; the remaining implementation in
child PR10; the 30 root heads already in main beyond their ancestry fact; and
the recovered stashes other than the manifest-sharing behavior above. The
existing records give candidate destinations, not a blanket new review claim.
The next recovery action is to compare the recovered D-04 parser and worker
stashes, including their formerly untracked files, against the current parser,
installation and native worker boundaries before selecting a missing behavior.

That next comparison has now exposed a specific framing regression. Read the
complete recovered `c53cf210...:crates/compute/rrd-attunement/src/parse.rs`,
the recovered worker/test files from its third parent, current `parse.rs`,
`worker.rs`, `tests/worker.rs`, the CLI process tests and `facts.rs` lines 1-182.
The draft's source identity, digest and resource checks remain in the current
parser/facts boundary; complete source-to-durable-record reconciliation is still
pending. However, the batch worker treats every `UnexpectedEof` while reading
the four-byte next header as clean shutdown. A one-, two- or three-byte partial
header therefore succeeds, whereas the recovered single-request worker rejects
it. Reproduced with the actual debug executable SHA-256
`bf7b95e06ba92eea62f6a97ffb1e021c6f466b4be54420eb902a882421be0a4d`:
empty input and `00`, `0000`, `000000` all exit 0; `ffffffff` exits 1. Output
stays empty. `parser-prefix-before.json` retains the probe. This is a newly
identified bounded repair for the existing D-04 worker, not fixed by PR444.

## Recovery and integrity evidence

The original recorded files `D:/localdev/devlabs/rrflow-stashes-20261115.bundle`
and `D:/localdev/devlabs/rrflow-archive-20261114.bundle` are absent at those paths.
A bounded search of devlabs, the session temporary tree and Codex storage did
not find them; access-denied temporary test fixtures prevent an exhaustive
machine-wide search claim. A similarly named bundle found elsewhere in scratch
held another project's commits and was not used as RRFlow evidence.

With commit-graph acceleration disabled, root and child
`git fsck --unreachable --no-reflogs --connectivity-only` pass. The root's derived
commit graph references five absent commit IDs and makes the ordinary command
fail; no source ref or history was rewritten to hide that metadata problem.
The complete stderr and both results are retained in session scratch.

Recovered 266 root and 34 child unreachable commit tips. All eight named root
stash work commits are present, including third-parent untracked trees. The
child also retains stash work commits `3e644ed080e068b4756f94e5dd47444c8a7e88db`
and `8cf93f88632928a9b0a101d01ec472d2c71f4339`. A bundle for each repository
contains all existing refs plus the recovered tips supplied through `--stdin`.
No new archive, stash or recovery refs were created.

Both bundles pass `git bundle verify`. Each was independently unpacked with
`git index-pack --stdin --strict` into a separate object directory with no
alternates, then every recovered tip was read there with `git cat-file`.
The first verifier harness used a buffered header reader and produced a pack
signature error; switching that reader to unbuffered I/O made the unchanged
strict verifier pass. This was a harness offset error, not accepted corruption.

| Bundle in session scratch | Bytes | SHA-256 |
|---|---:|---|
| `rrflow-recovered-objects-20261120.bundle` | 113143419 | `7e23b1cc9d9c902c0809e763e3e96a2c6db92395eb5c279c101fd97494cbbaf9` |
| `connectome-recovered-objects-20261120.bundle` | 174078029 | `fb9b239df56cd9752ed6b1898151d934b996f15367e2120812412b888e3e3b6a` |

The large artifacts described as archive-only in the old record are also
recoverable from the independent root pack: blob
`89aa8000fdb9263a12ad0a2d3636cbda976399e8` is the 53,708,641-byte
`connectome-ready.json`; blob `42640d0a193e3390fa590debd9abed098f3eec7e` is the
176,054,272-byte runner shared by four historical paths. They were not executed,
republished or treated as current acceptance evidence. This corrects a possible
loss inference from the missing bundle paths; it does not reconstruct every
deleted branch name or prove every historical object was inventoried.

## Acceptance and change checklist

Scratch root: `D:/.usr/Temp/rrflow-selective-recall-audit-20261120`. Recovery
bundles, isolated validation packs, PR metadata, file lists, blob comparisons and
logs remain there. No original cache, user process, history or old branch was
deleted by this package. Research: local history is the source; no third-party
adaptation. Trace: before/after test output and object verification, no runtime
telemetry. Resources: existing Node and Cargo caches, four Cargo jobs, root
target at the scratch root's `target` directory.

- Child PR109: `pnpm check`, types, build, two focused browser cases and all
  36 smoke cases passed. Its new journal records exact commands and limits.
- `python -m unittest test_native_capacity test_offline_lifecycle` in
  `scripts/ci`: 78 ran, 65 passed and 13 skipped on Windows, 91.351 seconds.
  Log: `historical-capacity-reconciliation-tests.log`. No live macOS run.
- `cargo test --locked -p rrd-lsm --lib manifest`: the two recovered
  manifest-sharing tests passed, 60 filtered out, 0.05 seconds test time.
  Log: `stash-manifest-sharing-tests.log`. No code change or benchmark.
- Root documentation/navigation/gitlink and exact candidate/merge-group results
  are recorded after they run. Initial `check_documentation.py` passed (1,282
  statuses, 1,278 coordinates); navigation passed (21 indexes, 1,279 nodes,
  4,495 edges); execution inventory regenerated 3,688 records. A green empty
  opening commit is not repair proof.
- `python -m unittest discover -s scripts/dev -p test_connectome_dev.py`:
  ten development-launcher checks passed. Final documentation/navigation and
  `git diff --check` passed after the merged child pin was selected.

Connectome PR109 merged as `26d1f83cf9fb10118f6e93dee70750f30ca7db3c`.
Merge-group run `44827483107` passed all six jobs, including native Ubuntu,
macOS and Windows, browser smoke, real RRD process integration and `ci-gate`.
The source tip `ba0e0d05b3b9438a96f4a35d3bb99adc4b98d083` has the identical
tree. Its verified `connectome-merged-109.bundle` precedes local branch deletion;
fetch/prune confirms the server removed the remote branch. PR444 pins the merge
revision. The separate candidate run `44827444215` still had one Ubuntu job
running at this observation; its final conclusion is not asserted here.
Parser framing repair is opened separately as
[PR445](https://github.com/rrflow/rrflow-development/pull/445), from main.

The before-edit brief binds current/target behavior, exact scope, owner, oracle,
research/resource decisions and stop conditions. The source map and explicit
unreviewed remainder above satisfy this package's review boundary, not the
owner's whole-repository recovery request. Changed evidence is reread and the
complete diff reviewed before commit. POAM-011 remains open; no objective,
roadmap, capability-parity or historical-absorption completion status changes.
