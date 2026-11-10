# D-03: characterize interrupted refresh in the public data snapshot

**Status:** active draft; real-engine oracle fails and production repair remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-snapshot-consumer-20261107`
**Owner:** OBJ-04/07/14, D-03, POAM-006; [draft PR #252](https://github.com/rrflow/rrflow-development/pull/252)

## Boundary and brief

The PR brief was amended before repository mutation. Starting draft `1938977e761eef7797b9169a0350e75c2e70bc73` was clean. PR #292 had merged as public main `d21a1556657fb17e783a4396368250459e03a2e8`; `git merge-tree --write-tree HEAD refs/remotes/development/main` returned conflict-free tree `e4b00a41f87c40cc084ba8ac212322e201f19f3e`, and ordinary merge `55d42b829a30f32be3d4b6caf3ae2c022413dc33` produced that tree. The existing 2,048-file interrupted-refresh regression remains intact; its separate public all-model snapshot observations were added to the same fixture and compared before and after reopen. The unrelated newer claim is independently checked through the existing rrflowQL assertions, since the public data snapshot does not expose claims as entries.

Read the product/alpha and project-indexing owner sections, the interrupted-refresh POAM entry, the change-authoring procedure and the prior reconciliation journal. Source ranges examined were the regression function in `inventory_delta.rs` lines 1–495, `engine/data.rs` lines 1–210, `engine/query.rs` lines 1–130, all 153 lines of `rrd-query/src/pipeline.rs`, `rrd-store/src/repository/runtime.rs` lines 1–370, `repository/control.rs` lines 205–320, `control.rs` lines 1–190, `access/semantic_commit.rs` lines 1–180, `access/versioned_read.rs` lines 260–460, `rrd-core/src/runtime.rs` lines 180–480, the source-generation declaration locations in `authenticated_log.rs`, `rrd-contract/src/lib.rs` lines 6190–6220 and 6460–6560, and `engine/model/raise.rs` lines 1–90. The formatted test file's SHA-256 was `2c80f1f3f953df6c5a9f60f5fe690a538194340a14b60e01be634a9460813971`; `data.rs` `0a4911e149efe67e100904bcd8ae1de5a70025900245787de964a0e75fb279ec`; `pipeline.rs` `b6712388a21493f46d46b237dd1e9355998e736dd2339954e46b1bb27300046e`; repository `runtime.rs` `36f5ff664e08c6351542dbbb44bc791290cd638aa2c95dfda4264562d57b25c0`; core `runtime.rs` `5fd43333347cf2492a567bbde1ca71907b8090f6ae26968eb7b00b8e80baecfc`; `control.rs` `1b3c405761fe94c844cbe82c9a796ef0f000cd7a400278b744b60c6ef21f1fa2`. This is a boundary review, not an exhaustive codebase review; the remainder was not reviewed this turn. No source or algorithm was copied from another project. [FoundationDB's developer guide](https://apple.github.io/foundationdb/developer-guide.html) describes staging separate versions and atomically switching a reference; this requires every reader to resolve the chosen version and does not by itself authenticate RRFlow's historical selection. [PostgreSQL's isolation documentation](https://www.postgresql.org/docs/18/transaction-iso.html) describes statement snapshots; individually committed staging batches cannot be treated as one atomic transaction by analogy.

## Failing oracle

With `CARGO_TARGET_DIR=C:/Users/jessa/AppData/Local/Temp/rrflow-c07-observer-7e60a4ac`, `CARGO_PROFILE_TEST_DEBUG=0` and `CARGO_INCREMENTAL=0`, `cargo test --offline --locked -p rrd-engine --lib interrupted_refresh_does_not_expose_partial_facts_as_completed_snapshot -- --nocapture` compiled in 1m33s and failed with exit 101 after 44.32s execution. The all-model snapshot root was `01c184f32c27df510f2672c38c122fb74269c07bca827747e181fb2be709c38b`; the completed root was `e896e3bdaa7f3222929ee84335b22a8050d7c78cc7185ef91765d55119e6d8ca`. The before/after-reopen snapshot values matched each other, the completed head remained unchanged, and the existing unrelated latest-claim, pinned-old-cursor, recovery, completion and replay assertions executed before this failure. The original rrflowQL root assertion remains after the new assertion and therefore did not execute in this run; its earlier independent failure remains recorded in the prior journal. Raw local log SHA-256 `5c351af8852ce2feca343f16de4c1d9ad0df6a9d85ced7683b953de1b6865a19` is retained at `C:/Users/jessa/AppData/Local/Temp/rrflow-d03-snapshot-consumer-20261107.log`. The test uses generated fixture data, not a user's project.

Initial `cargo fmt --all -- --check` identified formatting in the new closure; `rustfmt --edition 2021` on the changed test formatted it. The full engine suite, installed lifecycle driver, production repair, candidate CI and merge-group CI were not run. The current `ReadStamp` binds a runtime cursor and accumulator root; the control journal is a separate chain, and the draft generation history has no production caller or proven binding to arbitrary historical read stamps. A solution must select completed generation and membership under the canonical authority at each requested stamp, preserve unrelated current writes and authorized history, bound selection/membership and all consumers, and prove restart/replay, deletion and no-head behavior. The existing test rules out fixing only rrflowQL or globally rewinding all reads.

## Checklist and next action

[done] Scope/brief before edits; actual main reconciled; second real public consumer exercised before/after reopen; original failing query assertion preserved; exact failure and limits recorded. [open] Complete a code-grounded production design for generation selection, authenticated completeness and stamped reader evidence; amend the PR before changing production; implement and test it across query, full snapshot, context, retrieval and indexes; rerun this unchanged oracle and the installed project driver; review diff and pass package, candidate and protected merge-group checks. Keep this PR draft. No alpha, C-07, roadmap or POA&M completion status changed.

## Current-main integration and Windows red baseline

The PR brief was updated before merging draft head
`fc543813a7ffd19566a4a3ba8aa38eec262abf73` with public main
`838c85120621d5d9dde989d7677cc6ec89d20f03`. The read-only merge preview and
the staged, uncommitted ordinary merge both produced tree
`df30a7320838237e254262d7b147b40b1c2e426e`; no conflicts were reported.
This is a local integration preview, not a merge or implementation of the
visibility repair. The separate unmerged F-06 draft overlaps the runtime
repository and source-read owner and remains a dependency to reconcile.

On Windows x86-64, with the Cargo target directed to the existing C: task
build directory, `cargo test --offline --locked -p rrd-engine --lib
engine::tests::attunement::inventory_delta::interrupted_refresh_does_not_expose_partial_facts_as_completed_snapshot
-- --exact --nocapture` compiled in 45.95 seconds and failed in 55.84 seconds
with exit 101. The observed incomplete-refresh root was
`fc849184d0bb173a33b35cf9212d11b222020be2d080507b1380adae43c06e25`;
the expected completed root was
`aebb9936f51ff215d6a24f6865bd38dc74ee994f2291baecaf05d15476d9b9ca`.
The fixture reached its interrupted/reopened snapshot, unrelated HEAD claim,
resumed successor, final reopen, and idempotent replay checks before failing
the data-snapshot equality assertion. The subsequent original rrflowQL root
assertion did not execute in this run. The retained local log is
`D:/.usr/Temp/rrflow-d03-df30a732-windows-20261107.log`, SHA-256
`c3d0924f7cc67f79b1c3e77839eea889ef65972499fc91004f83ea581f923dd0`.
`git diff --check 838c85120621d5d9dde989d7677cc6ec89d20f03 --` passed
before this journal addition. This test does not establish the installed
parser-to-query journey, native capacity, soak, candidate CI, or alpha.

Inspection of the merged source confirms that group commits write semantic
records before the final control-head transition, and that the stamp's
accumulator authenticates the global runtime log prefix rather than a proven
latest source-generation selection index. The existing corruption
characterization remains relevant: inclusion of a returned selection event
does not prove a newer one was not omitted. The draft stays open; no public
generation resolver, visibility fix, or gate status is claimed here. The
remaining full candidate review and production integration are not complete.
