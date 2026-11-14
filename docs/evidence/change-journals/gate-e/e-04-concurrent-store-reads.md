# Concurrent store reads

**Status:** active immutable change-journal evidence; E-04 and OBJ-13 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-concurrent-store-reads`
**Owner:** evidence that rrflowKV reads no longer wait for one another

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: OBJ-13 and E-04, vector search within bounded resources, and OBJ-10 through the lab's measurement. Owner direction of 2026-11-04: "how do we get this moving forward we are dog shit".
starting revision/tree and pull request: development/main at 12039148; pull request #281.
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the description of pull request #281. RrflowKvStore held Mutex<Database>, and get, scan and begin_transaction took it although they need only &Database, so every read waited for every other for its whole duration. It now holds RwLock<Database>. Reads and read-only inspection take the shared side. Anything needing &mut Database takes the exclusive side. Commit behavior, durability, formats and public operations are unchanged.
files and symbols read in full: rrd-store/src/rrflow_kv.rs lock users and tests; rrd-lsm/src/transaction.rs get and scan signatures; rrd-lsm/src/database.rs begin_transaction and its &mut self methods. The lab's profile and measurements on clyffy-01.
implementation/evidence paths created, changed, moved, or deleted: changed crates/persistence/rrd-store/src/rrflow_kv.rs (RrflowKvStore.database, lock, read_lock, StorageTransaction::get and scan, begin_transaction, manifest, the inspection read, try_maintenance_step, tests). Created this journal and generated the gate-e index.
research decision and primary-source/adaptation record: none needed. It uses the standard library's RwLock, and its comment records why a stream of readers cannot starve a commit on Linux, where a waiting writer blocks new readers. That reading of the futex implementation was not separately tested here, beyond the overlap test finishing its 200 commits with four readers running.
trace/resource/debug decision (add/preserve/not applicable, with reason): preserved. begin_mutex_wait_ns and commit_mutex_wait_ns keep measuring the same waits, now on the lock's shared and exclusive sides.
first-failure or characterization oracle and result: a perf profile and the lab on clyffy-01 showed rrflow-hnsw at ef-256 doing 21.9 queries per second with one client and 21.6 with eight, with threads about 25% busy on about 2 of 10 cores. 2483f92b committed four tests. a_read_does_not_wait_for_another_read and a_new_transaction_does_not_wait_for_a_read failed with their stated messages after waiting ten seconds. a_commit_still_excludes_reads and overlapping_reads_never_see_half_of_a_commit passed, and they hold what must not change.
acceptance commands and exact results: at c914da26 on Windows: cargo test --locked -p rrd-store --no-fail-fast, 259 passed, 0 failed, 2 ignored. The same with --features crash-test, 259 passed, 0 failed, 2 ignored. cargo test --locked -p rrd-lsm --no-fail-fast, 246 passed, 0 failed, 3 ignored. cargo test --locked -p rrd-engine --no-fail-fast, 402 passed, 0 failed, 4 ignored. cargo test --locked -p rrd-server --no-fail-fast, 34 passed. cargo clippy --locked -p rrd-store --all-targets with -D warnings, clean. cargo fmt --all --check, clean. check_version.py passed. On clyffy-01, vector-small, 20,000 points, comparing measurement tree f3ddda18 with 515b60b3, which adds only this package. rrflow-hnsw at ef-32, recall 0.9782: 50.6 to 49.4 queries per second with one client, as expected, and 88.7 to 166.9 with eight. At ef-256: 21.6 to 57.1 with eight, so it is no longer flat. rrflow-turboquant at rerank-400 with eight clients: 17.1 to 35.9. These are one run each on one host, and no repeat was made.
surfaced failures and corrections: my serial model of about 5 ms plus 0.16 ms per candidate predicted eight-client speed before this change closely, but scaling after it is 3.4 times at ef-32, not close to eight, so this lock was not the only limit. What remains has not been identified, and contention on the shared page cache's mutex or the allocator are candidates only. A first test design shared a transaction between threads, which the compiler refused because a boxed StorageTransaction is not Sync, so each transaction now stays on one thread.
checks not run and reason: candidate CI, because it skips drafts. Linux for these suites on this commit, beyond the real runs. macOS. The rrd-lsm feature lanes, because rrd-lsm did not change.
remaining known errors and owner: scaling remains below the comparators', as above. Reads now run concurrently against shared interior state, so a logic error assuming one reader at a time would surface as a race, which the overlap test exercises and cannot exclude. Database compiles as Sync. try_maintenance_step now fails to get the exclusive side while any read is active, as it failed while any read held the mutex, and sustained reads could defer idle maintenance, which has not been measured.
full-file reread and diff review: complete for the changed code, the tests and this record.
change checklist: complete. Brief before implementation; failing tests committed first; no format change; no public change; no parallel path; no runtime fetch; no provider state.
roadmap/POA&M status change, or explicit none: none.
pull request, CI run, and merge evidence: recorded on pull request #281.
```

## Links

- [Pull request #281](https://github.com/rrflow/rrflow-development/pull/281)
- [Gate E](../../../roadmap/rrflow-1.0/gate-e.md)
