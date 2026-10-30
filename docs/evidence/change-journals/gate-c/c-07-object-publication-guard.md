# Concurrent object publication guard

**Status:** active immutable change-journal evidence; POAM-047 remains open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-object-publication-guard`
**Owner:** PR #219 follow-up for in-flight publication ordering.

## Evidence record

```text
gate/package and alpha outcome: OBJ-02, C-07/J-02 and E-04c prerequisite; no gate or release status change.
starting revision and pull request: PR #219 draft 8d0316bf8acdc7ff5c12b6e60fbc89cd0cc78eab, itself based on public main 6fe8b7590d913beda9b009551e5d2d2644e406d9. The PR brief recorded the lock design before these source edits.
change brief: a verified winner can become visible while its Windows rename call is still in flight and the directory-sync helper is a no-op -> serialize cooperating publish, explicit quarantine and reclamation by the canonical object's existing SHA-256-prefix shard using anchored OS-backed file locks. Hold the guard until publication calls return, verify a collision winner, retain exclusive rename and existing native crash hooks. Derive the lock shard from ObjectReference::canonical_key; at most 256 lock files are possible. Lock metadata is not an object or a new authority. No public wire contract or product version changes.
files and symbols reviewed: LocalObjectStore::publish_stage, publish_absent_stage, quarantine, reclaim_orphans, put/stream paths, inventory, ObjectReference::canonical_key, AnchoredPath OpenOptions/rename/sync and Windows native rename; existing object tests and the E-04c racing-build test; multi-model-object-contract reference; original PR #219 journal.
implementation/evidence paths: rrd-store/src/object.rs adds a scoped File::lock guard for publication, quarantine and reclaim, payload-free wait-start and acquisition traces, a same-shard waiting oracle and a child-process death/reacquisition oracle. rrd-store/tests/object_publication_trace.rs validates the observable trace separately. Existing thread/process race, corruption and exclusive-publication tests remain. The object reference documents unbounded lock wait and outstanding qualification. This linked journal and generated index record the follow-up.
research: Rust File::lock uses OS file locks and releases the lock when its handle closes (https://doc.rust-lang.org/std/fs/struct.File.html#method.lock); Microsoft documents MOVEFILE_WRITE_THROUGH completion behavior (https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw). No upstream source was copied. These APIs and process-kill tests do not prove power-loss behavior for every device/filesystem.
trace/resource decision: one lock per canonical shard, created on demand; trace records wait start and elapsed acquisition microseconds with no digest or payload. Wait is currently blocking while a live holder stalls; no configurable wait deadline or cancellation claim. No throughput or tail-latency improvement is claimed.
first failure: the prior journal records the synchronized Windows access-denied failure and the tentative collision repair. The same-shard ordering was unproven on Windows because directory sync is a no-op. New tests hold the guard while a writer waits, then release it, and kill a separate lock-holder process before successful reacquisition.
local results: cargo test --offline --locked -p rrd-store passed, including the 200-iteration same-content race and separate-process identical/distinct publication test. cargo test --offline --locked -p rrd-store --lib --quiet: 87 passed, 3 ignored (including child fixtures). cargo test --offline --locked -p rrd-store --test object_publication_trace: 1 passed. cargo clippy --offline --locked -p rrd-store -p rrd-lsm --all-targets -- -D warnings and cargo check --offline --locked -p rrd-store --features crash-test passed. cargo fmt --all -- --check and git diff --check passed.
surfaced failures and corrections: an initial trace-capture unit test passed alone but failed while run with other tests, with an empty capture; its exact cause was not established. The trace check was moved to its own integration-test binary and passed; no runtime error was ignored. Earlier Linux ARM evidence covers prior head 8d0316bf only, not these lock changes.
checks not run: Linux/macOS execution of the lock follow-up, Windows power-loss qualification, real E-04c racing engine test with this exact store revision, contention/resource qualification, candidate CI and protected merge-group. Prior server or CI results do not qualify the changed head.
remaining errors: POAM-047 remains open for native durability, wait cancellation/budget, concurrent corruption/reclamation and performance evidence. The separate macOS APFS capture failure and C-07 capacity/soak remain open.
review and checklist: canonical shard mapping, guard ownership and non-reentrant quarantine, atomic no-overwrite, native hook preservation and failure paths reviewed. Brief before implementation, focused and package tests, immutable journal and index; no completion claim.
pull request and merge evidence: https://github.com/rrflow/rrflow-development/pull/219; draft; candidate and protected merge pending.
```

## Links

- [Initial reproduction](c-07-concurrent-object-publication.md)
- [POAM-047](../../../poam/rrflow-1.0-alpha/poam-047.md)
