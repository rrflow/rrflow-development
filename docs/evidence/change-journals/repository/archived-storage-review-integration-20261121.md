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
