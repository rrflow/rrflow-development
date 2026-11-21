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
