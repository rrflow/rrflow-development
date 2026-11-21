# Recovered storage and diagnostic branch integration

**Status:** active integration evidence; no storage qualification claim
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/storage-history-integration-20261121`
**Owner:** storage source recovery under POAM-011 and Gates C, E and F

## Before-edit brief

Baseline fec02e3778ee5100c4b9975c56b03abd0c7b450a. Integrate recovered
backup scheduling 2665f99110b0407eb3f38c573185de91e835ca61, run progress
65385004d8f9d01401606a3918fe45025c1bfb74 and PR135
bec6e0d8685d37ae8848f5624d370433af48976b, PR286
6795422b7b9ed1187af60f707576473f99866084, PR315
d26ad75bed82fe529f804b5b0fee3c19f50b6c73 and PR316
5741a7660e244e8bd71425c029d93a41f0420a35 through ordinary merges.

Scope: recovered storage/maintenance evidence, vector cold-trigger counters,
query execution observations and their assertions, prior consolidation's HNSW
oracles and journals. Read source changes and current owners before conflict
resolution. Existing rrflowCLD cold-history authority is distinct from old
cold-vector trigger observations; do not regress it into a diagnostic-only tier.
Retain native failure records, not old completion claims or generated identities.

First checks: affected vector-trigger/query and HNSW read-path tests, then
owning engine/query suites and storage qualification where application changes
require it. This session's same-tree LSM/backup/qualification evidence may be
reused with explicit identity limits. Documentation/workflow/format checks and
required candidate plus queue remain mandatory. No new research, benchmark,
cloud operation or profiler needed for history integration. Stop on lost
behavior/assertion, misleading diagnostics or red CI. No no-loss parity,
performance, soak or alpha completion claim; other archives remain pending.

## Resolutions and traceability

Ordinary source merges: backup scheduling a21c1755, run-progress 8affe664,
flush-stall record b424ff94, CLD/index decoder 940e8409, consolidation
82b459a9. PR135 was already an ancestor after run-progress; no duplicate merge
was created. All six source tips are ancestors of this candidate. There is no
application delta against fec02e37 from these source merges.

Read the incoming storage evidence, coldlight module, diagnostic projection,
vector search calls, index decoder, contract/test additions, consolidation
journal and conflict hunks. Read current coldlight, diagnostic/storage.rs,
vector/search/operations.rs, execute/read_path.rs, contract diagnostic sections
and validation, public_contract/vectors_and_audit.rs and HNSW read-path tests.
The cold-source counters remain process-local observations; they do not replace
rrflowCLD's current cold-history authority. Search keeps engine observation time
rather than the earlier request valid-time coordinate. The bounded decoder
retains strict envelope/version/source/maintenance validation and counts the
whole array while only materializing the admitted result prefix.

The current split modules already contain the recovered implementation. The
old inline execute, diagnostic, vector-search and contract files are not
reintroduced over them. The old public_contract.rs was removed from the merge;
its added optional-counter fixture field remains in the current split test.
Current generated contracts and SDK identities remain bound to today's owner.
Current CLD journal correctly records the later split package; it is retained
instead of restoring the earlier combined-package account.

Both recovered HNSW oracles remain in hnsw_read_path.rs: distinct source
addresses cannot collide, and an emptied generation cannot answer for another
collection. Their only conflict was the old estate helper name; current engram
is retained with all assertions. Source-bound design retains current immutable
job identity and naming. The flush-stall record retains POAM-087, where the
missing deficiency was actually restored; old POAM-056 now names another gap.
Current native failure evidence and later gate results are not overwritten by
older progress descriptions. Existing consolidation/parser/snapshot journals
remain, without adopting their broad absorption wording as current proof.

## Verification

Focused and owning-suite results are being recorded below. Existing same-tree
storage evidence from PR460/461 is reusable only for unchanged application
behavior: LSM 276 passed/3 ignored, backup 6, management 3, qualification 54,
installed reopen 39 passed/14 ignored. No new performance or parity conclusion
follows from those tests or these source merges. Other archive work remains.

The focused coldlight test passed (1), HNSW read-path suite passed (14), engine
diagnostic suite passed (9), and the complete default-feature rrd-query suite
passed (87 tests across 13 suites; doc tests empty). Initial PowerShell stream
redirection reported wrapper exit 1 despite all test results being green; each
command was rerun through Python subprocess with stdout/stderr captured and
explicit Cargo exits, all 0. The repeated run resolves the wrapper ambiguity;
it did not change tests or code. Optional feature matrices were not run locally.
`python scripts/knowledge/render_navigation.py`,
`python scripts/ci/check_documentation.py`, `cargo fmt --check` and
`git diff --check` passed. Candidate and merge-queue gates remain required.
No roadmap/POA&M status was advanced.
