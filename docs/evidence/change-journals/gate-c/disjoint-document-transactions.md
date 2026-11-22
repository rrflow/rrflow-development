# Independent document writes at one read stamp

**Status:** active implementation evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/disjoint-document-transactions`
**Owner:** [Gate C, C-02](../../../roadmap/rrflow-1.0/gate-c.md) and [POAM-071](../../../poam/rrflow-1.0-alpha/poam-071.md), OBJ-02.

## Change brief

Baseline `b08542f7ec32f942321b27877473b3bc9bf01edf`. Two installed, authenticated sessions began at cursor 7. The commit for document `left` advanced it to 8. The commit for `right` received HTTP 409. The public API, embedded store contention test and installed CLI campaign all reach the same semantic commit planner. The current store test even requires one disjoint writer to fail.

TX retains the begin stamp, operation identity and prepared intent. DB/KV validates the written identities and publishes the next contiguous log positions in one physical transaction. Record puts with unbounded valid-to and no governed receipts are the initial scope. Preserve conflicts on overlapping identity, including effective-time corrections and retirements, and preserve schema/catalogue constraints. All other operations retain their existing admission semantics. Preserve the original audit read across export, verify, restore and exact replay. QL, MX, LSM and CLD require no new engine or format implementation.

Read the actual transaction entry points, stamped commit preparation, version addressing, archive reconstruction and both existing concurrency tests. The source map is `RrdEngine::commit_transaction`, `prepare_semantic_commit`, `RuntimeRepository::commit_at_read`, `RuntimeCommitPager::next_commit`, `ReplayState::accept`, the store snapshot suite and the installed sustained campaign. These owners carry the replacement behavior.

Run the existing threaded MX/KV case against the new oracle, the full snapshot and archive suites, and the installed process campaign for distinct and overlapping writers after reopen. Check exact idempotent replay and original read stamps. Record the tested target and commands below. Stop on weakened constraints, ambiguous publication, altered commit identities, lost historical reads or failed recovery. No external research or benchmark comparison is needed to establish this correctness property.
