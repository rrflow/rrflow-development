# Archived native preflight integration

**Status:** active integration evidence; native capacity qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-native-preflight-integration-20261121`
**Owner:** C-07 bounded storage diagnostics and native capacity; alpha recovery prerequisite

## Before-edit brief

Baseline 3ef272c48b68de9b5f92ef4c2f7f6d2428e347cd. Ordinary-merge the two saved
h04-c07-merge-preflight sources, 5936e895eaca3194dea09192eb12347d9250f0d2 and
e8d5ac07dab50ae580090832749b0654141e1352. Both have two parents and identical
tree 0a9cf6a33441bfc9594049495464b9638c41c1c0. Preserve current native-worker,
sampler, bounded storage-cursor and shared-memtable behavior in their canonical
modules. Source history is not yet joined; no new throughput capability claimed.

Read the source diff and current implementation/test owners before resolving:
rrd-engine engine/storage_cursor and tests, rrd-contract storage_cursor and
public_contract/storage_cursor, rrd-store rrflow_kv/inspection_cursor and its
storage_inspection/direct_read_paths tests, rrd-lsm memtable, CLI sustained
campaign/reader/native_worker/sample_task/sample_trace, qualification-storage
sustained, native capacity supervisor and selected_tests. Existing archive
ledger names these destinations; it is only a navigation aid. The current
five-attempt finished-busy retry and held-output policy supersede the old
single-attempt fixture. Current selected-case preflight supersedes the old
manual pairs. Do not restore old generated SDK digests or the retired eval
crate boundary. No unrelated source or submodule pin is in scope.

Oracle: original cursor range/kind and lifetime/refusal tests must remain;
run focused cursor/store/LSM tests, relevant Python native sampler/supervision
suites and architecture, then candidate and merge queue. Read complete changed
behavior and assertions, not only names. Reuse unchanged full-suite evidence
only with explicit identity and scope. No external research or new performance
trace is needed for this integration; native long-run capacity and real-world
reasoning optimization remain unqualified. Stop on unmapped source, discarded
assertion, new platform behavior or red CI. Driver: this session; PR472 is an
independent history package. Continue remaining archives after this lands.
