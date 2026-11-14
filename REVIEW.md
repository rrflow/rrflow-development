# Independent review: C-07 and the requested planning journey

This is a local review aid, not an accepted evidence journal, a roadmap owner,
or a claim that the repository has been exhaustively audited.

## Baseline and scope

Public development main was `04fae27665c3081ef6bc193d54d8c1c9c8d3f2e0`,
tree `0effb6f859ddd0a99e24efd6016f80e06405cd40`. The clean D-02 worktree
has the same tree and was used for mainline tests. Draft PR 142's published
head `989fb87557e6c237f920e49a3e1e5d1971489f03` contains only its opening
commit. Its unstaged implementation was preserved in stash commit
`a2004fd90840c8fd7061420e8b33f4b8bfed1a40`.

For review only, the stash patch was applied to a separate detached worktree
at the main revision. Six conflicts in `database.rs` were resolved by
retaining the current point-index initialization/publication and using the
proposed shared manifest ownership. The original branch and stash were not
modified. This combined working tree is not a committed or CI-qualified PR
candidate. It has three staged source/test files and this untracked review.

Inspected: `task.rs`, `flight.rs`, the D-02 attunement control path, the audit
read and its reproducer, diagnostic conflict documentation and reproducer,
projected-read capture and manifest registry/release paths, the point index,
the entire stash diff and its new tests, the current roadmap and relevant
POA&M and prior review. This is not a whole-engine file-by-file review.

## Findings, in priority order

1. **The requested workflow is not implemented.** `engine/task.rs` records
   open/closed tasks and explicitly says a task orchestrates nothing.
   `engine/flight.rs` commits a prepared packet and `handoff_ready` event;
   the observe provider runs no external model. There is no demonstrated
   dynamic implementation plan, approved-plan execution, drift detection,
   minimal-question selector or complete memory-creation activity view. The
   absence is an implementation gap, not a passing or failing end-to-end test.
2. **Project ingestion cannot yet run in the reviewed worker.** The embedded
   attunement controller durably records a Connect checkpoint but its next
   unavailable phase is explicitly failed. Its header states that it adds no
   public transport, automatic background activation, source scanning or
   external capability. This is a deliberate partial implementation. A job
   record is not evidence that files were indexed or memories created.
3. **Audit pages can reject valid history.** `rrd-security::audit_since`
   marks the tail reached when the last fetched page is short, even when it
   stopped consuming that page at the requested audit-record limit. On a valid
   14-entry fixture, six records succeed while five return an invalid-chain
   error. Existing POAM-030 owns this. The characterization test below passes
   precisely because it asserts the defect; it does not mean the bug is fixed.
4. **Diagnostic visibility can fail during audited traffic.** The snapshot
   retries three times when its control-journal stamp changes, including due
   to unrelated audited reads. A deterministic test forces a read in each
   window and confirms a conflict while the runtime cursor is unchanged.
   Existing POAM-029 owns this; its passing characterization asserts failure.
5. **Pagination and read-view growth need measured follow-up.** Task and flight
   lists request the whole respective record family before applying the cursor
   and page size. Thus those page parameters do not bound prior family scanning;
   `max_storage_keys` bounds it by rejection, not by cursor-selective access.
   Separately, projected-read setup walks all segment descriptors and charges
   the full manifest's segment bytes per view, including unrelated ranges.
   The default physical pin ceiling is 4 GiB. The draft shared-`Arc` change
   removes a manifest copy; it does not remove those loops or change that
   conservative admission. These are source-level scaling risks, not a new
   installed benchmark or a demonstrated cause of the failed corpus run.
6. **Context access has known estate-size work.** Context assembly obtains a
   data snapshot and rebuilds a BM25 artifact; the vector search read helper
   requests vectors with no kind filter. The prior engine review maps these
   to E/H owners. The observed code is not a quality or throughput benchmark.

## Validation performed

- Main-equivalent tree: `cargo test --offline --locked -p rrd-security --test
  security_authority a_record_limit_met_inside_the_final_partial_journal_page_fails_the_audit_read`
  passed 1 characterization that asserts the known defect.
- Main-equivalent tree: `cargo test --offline --locked -p rrd-engine --lib
  an_audited_read_inside_every_assembly_window_exhausts_the_diagnostic_read`
  passed 1 characterization that asserts the known defect.
- Main-equivalent tree: `cargo test --offline --locked -p rrd-engine --lib
  engine::tests::attunement::` passed 8, failed 0, ignored 1 child fixture.
- Combined review copy: focused lifetime tests passed 5; `--test
  projected_lifetime` passed 6; `cargo test --offline --locked -p rrd-lsm`
  completed successfully, with 2 ignored library fixtures; and
  `cargo clippy --offline --locked -p rrd-lsm --all-targets -- -D warnings`
  passed. The first attempted filter `reader_lifetime_tests` selected zero
  tests and is not counted as evidence; the corrected filter selected 5.
- `git diff --cached --check` passed after correcting a review-copy line-ending
  error. No performance comparison, candidate CI, installed crash matrix,
  native three-platform capacity matrix, full corpus, or 24-hour soak ran.

The previous turn separately ran the real installed-engine HTTP database test
on the same tree and obtained 1 pass; it does not prove the full product flow.

## Implementation implications

Use one engine-owned, versioned effect and handoff contract with source/plan
identity, explicit blockers, authorization, budgets, progress, checkpoints,
verification and receipts. Record observable decisions and effects, not hidden
reasoning. Derive the user-facing memory activity from committed canonical
records and receipts, with explicit missing/failed/uncertain states. Questions
need an independently testable blocking criterion; nonblocking work must have
authorized boundaries and a durable cancellation/restart path. This is a
proposed direction, not an implementation or a claim of proven model quality.

The C-07 draft requires a real branch integration, its linked package journal,
exact-candidate CI and measured native before/after evidence. Do not conclude
that the `Arc` change repairs corpus timing from these component tests. Full
C-07 qualification and all downstream user-journey acceptance remain open.
