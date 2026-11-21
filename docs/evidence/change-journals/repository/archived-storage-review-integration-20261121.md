# Archived storage review integration

**Status:** active integration evidence; no storage or journey qualification
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-storage-review-integration-20261121`
**Owner:** C-07 recovery and existing POAM-029/030/081 findings

## Before-edit brief

Baseline ed7f4933642984d44fc8fc1f8e04d9417fb1005c. Integrate the four archived
c07-independent-review stash objects as ordinary source merges:
1f1e30997f8f167c813435325099e9d0e34386f1,
db90541b2ba824ed6bc3465e801c5e0a2342b64f,
45eeca3fc2c43fa872f5d6599181d8387bc4ccc3,
c84cd701278e289d9678255bc6199fdc726d87cf.
Their saved work extends shared immutable manifest generations and records an
independent review; all have two parents, with no untracked third-parent tree.

Read original REVIEW.md, the complete source/test changes and current
Database manifest, database submodules, reader/views and reader_lifetime_tests.
PR460 already integrated the original manifest stash; keep the current split
implementation, its point-index publication and newer pin accounting. Preserve
both recovered manifest lifetime/admission oracles and their assertions.
Retain the original review in a linked historical record, with a current note
mapping its findings to present owners; do not publish it as a new root authority
or claim its historical failing characterizations prove fixes.

First checks: the two manifest tests and projected lifetime suite; same-tree
full LSM evidence (276 passed/3 ignored in PR460) is reusable only if behavior
is unchanged. Then documentation, architecture and required candidate/queue.
No new benchmark/research or instrumentation is required for history integration.
Stop on lost lifetime assertion, changed admission, untracked finding or red CI.
Other archived stashes remain; C-07 and native lifetime qualification stay open.


## Resolutions and validation

Source merges: 9539334d, df800ca2, 03e2494e and b3fa4b6a. The four saved
copies name identical reviewed blobs for REVIEW.md and the three source/test
paths; each object was merged separately. Current Database and split
open/flush/snapshot/compaction/transactions modules retain Arc ownership and
point-index publication. reader/views retains pointer-fast equality plus
content validation, with newer per-segment and memtable pin accounting.
reader_lifetime_tests retains both original oracles with the current admission
arguments; no assertion or application path was removed.

The [original review](independent-storage-review-20261026.md) is preserved below
its historical status and current-owner note, rather than adding REVIEW.md as
a root authority. POAM-029 owns audited diagnostic conflicts; POAM-030 contains
later audit-tail correction evidence; POAM-081 owns task/flight pre-page scans.
Current Parse execution supersedes the historical unavailable-worker finding;
that does not establish the complete requested reasoning journey. Context
planner convergence and C-07 qualification remain open in their existing owners.

Exact local checks: shared manifest lifetime 1 passed; manifest admission
1 passed; projected_lifetime 13 passed; workspace_architecture 35 passed.
All Cargo subprocess exits were 0. Filtered package targets selecting zero
additional tests are not counted. Same-tree full LSM evidence remains PR460's
276 passed/3 ignored; it was not repeated because implementation is unchanged.
Navigation and documentation checks passed after adding the required
Superseded by link to the historical record; the first documentation run failed
for that missing link. `git diff --check` passed. Candidate and queue remain
required. No new benchmark, gate closure or product-completeness claim.
