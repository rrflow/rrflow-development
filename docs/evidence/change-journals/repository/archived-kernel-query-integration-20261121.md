# Archived kernel and query integration

**Status:** active integration evidence; qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-kernel-query-integration-20261121`
**Owner:** Gate C kernel storage and Gate E E-02 source recovery

## Before-edit brief

Recover three original kernel/query sources through normal merges.

Driver: ongoing original-source integration, following PR483. Baseline main f734c29cf3d3402b83a5194f1ac1a2e5d9762c7b; join PR483 green main before readiness. Owning outcomes: kernel K13 structure, Gate C authenticated storage and Gate E E-02 index reads. No performance/qualification status change.

Sources: d0c52ea783f09ed56ce7569b817d601c522cbd64 (nine LSM database responsibility modules), 38d55aa8f8366df898d483081c6b562a69aef93b (kernel SHA-256 wrapper and independent vectors), 07dba294a74ebf2e96c9d72386c948bdce038aaf (streaming index decoder).

Current and target: current canonical modules already contain these behaviors. Merge original ancestry, preserve current point/range/metadata reads, measuring-transaction refusal, durability/flush/snapshot semantics, SHA-256 byte identity and bounded index row decoding. Resolve only reviewed conflicts; carry any missing assertion before removing a superseded source. No second runtime, compatibility path or benchmark execution.

Read: source streaming decoder patch, current execute/read_path.rs decoder and index-catalogue journal; full current digest.rs and original digest patch; original/current LSM split journal, current reads/transactions additions. Inspect remaining conflict regions, source modules and tests before resolving. Exact scope: rrd-core digest/Cargo lock; rrd-lsm database modules; rrd-query execute owners/Cargo; associated existing evidence records and this journal/index. Do not restore old monoliths or stale status.

Smallest oracles: core digest tests; query index_catalogue; owning rrd-lsm and rrd-query suites; architecture, docs/navigation and diff checks. Dedicated target D:/.usr/Temp/rrflow-root-target, four jobs. No throughput/comparison runs or production/provider access. No new research or instrumentation needed for recovery; actual failures go in journal. Stop on unmapped source behavior or failed check, repair before readiness. Candidate and queue must pass; automatic branch deletion after merge.

## Original source joins

| Source | Merge commit | Resolution |
| --- | --- | --- |
| d0c52ea783f09ed56ce7569b817d601c522cbd64 | 4be8bc5ac | Keep existing nine database responsibility modules and later additions |
| 38d55aa8f8366df898d483081c6b562a69aef93b | cdc08e930 | Clean merge, no remaining source delta; SHA-256 wrapper and independent tests already present |
| 07dba294a74ebf2e96c9d72386c948bdce038aaf | 4cd826483 | Keep streaming decoder in execute/read_path.rs and current root exports |

The LSM source split's compaction, failpoints, flush, gc, open, options and
snapshot modules are byte-identical to their current owners. The existing
reads owner adds contains/size/prefix/first/last/reverse/empty operations and
selects the winning version before loading its value. Current transactions
also refuse measuring transactions before commit. Current database.rs imports
SegmentReverseCursor and the later drop module; neither is removed. The original
split journal already exists and is retained, not duplicated. Read the complete
source/current differences for the three conflicting Rust files, the original
get/batch-read and transaction-admission sections, and the source/current journal.
This is a conflict-resolution review, not a claim to have reread every unchanged
LSM function.

The digest source's two parents contain no untracked third-parent tree. Its
lazy locked sha2 wrapper, length-overflow refusal, independent padding/chunk
vectors and cloned-stream oracle are already in digest.rs, read in full.
The source dependency and kernel boundary patch was read; no dependency or
lockfile change remains after the clean merge. Manual throughput stays ignored.

The complete original streaming-decoder patch and current read_path.rs
285-560 region were read. The current owner validates the envelope, coordinates,
digest, row count, kind-specific artifact and maintenance evidence while
materializing only an eligible unfiltered/unjoined prefix. Remaining rows are
counted without full row decoding. This remains a serialized snapshot read
optimization, as docs/reference/query/index-catalogue.md explains; it does not
establish native incremental indexes or Gate F streaming closure. Current
root execute.rs imports/exports the split modules instead of restoring the
incoming monolith. The source's acceptance placeholder is superseded by the
existing record's actual acceptance text, which is retained. POAM-040 merged
without a net change.

No implementation/test/fixture delta remains against the starting green main.
Only this linked evidence and its index are new. One guessed metadata test path
and one PowerShell wildcard path were absent; repository file/search results
resolved the actual source locations. No test failure was hidden by those lookups.

## Verification

Test tree 4cd826483, target D:/.usr/Temp/rrflow-root-target, four jobs.
Core digest filter: six passed, one manual throughput test ignored.
Query index_catalogue: eight passed, including real rrflowKV reopen and
corrupt-artifact rejection. `cargo test -p rrd-lsm --locked` completed successfully, including native
crash/reopen, transaction conflict and measuring-transaction denial, projected
read ownership, compaction, reverse iteration and WAL suites. Its two ignored
unit tests are the explicitly spawned child fixture and manual encoder profile.
`cargo test -p rrd-query --locked` passed all selected targets (87 tests),
including bitemporal query/reopen, index lifecycle, graph traversal and live-query
checks. `cargo test -p rrd-engine --test workspace_architecture --locked`: 35 passed.
Navigation, documentation policy and diff whitespace checks passed. No benchmark/comparison, full workspace lint, non-Windows runtime
or installed UI acceptance is claimed. No roadmap/POA&M closure is declared.
