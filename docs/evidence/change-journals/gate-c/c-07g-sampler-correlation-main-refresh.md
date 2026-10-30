# C-07 sampler correlation and main refresh

**Status:** active implementation evidence; integrated server trace and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-sampler-correlation-main-refresh`
**Owner:** OBJ-02 / C-07g / POAM-035 and [PR171](https://github.com/rrflow/rrflow-development/pull/171)

## Brief and integration

PR171's description carried this scope before edits. Driver: this Codex session. Starting sampler head `c5b58a3c6f8e02b1d020aa9a68ecf1af0f457509`, public development main `89ec21eaa7fc129e6c532a3bb9f0ce1b57a9f3f9`. The sampler worktree was clean. The ordinary merge of main into the sampler branch conflicted only in generated `docs/evidence/change-journals/gate-c/README.md` and `docs/roadmap/rrflow-1.0-file-plan.jsonl`; both are regenerated from the merged source and linked journals. All other merged main files remain intact. The existing asynchronous acquisition, bounded native sample phases, resource-window validation, interruption and original failed campaign remain preserved at the same C-07 driver boundary. No implementation was removed or replaced.

Previously every sampler storage request used `capacity-resource`, preventing an individual slow sample from being joined to a server trace. The driver now assigns each request an atomic, checked monotonically increasing ID within the harness process. The existing bounded slow-sample record reports only its SHA-256 digest, matching the digest used by the separate [server trace candidate PR178](https://github.com/rrflow/rrflow-development/pull/178). An ID is unique only within one harness process; combine the process/campaign identity and retained artifact context for later runs. The slow-record limit and threshold remain unchanged, and failed/canceled RPCs are not claimed to produce a completed slow record. No raw request ID or credential is added to this trace. No provider, engine lifecycle, resource deadline, workload, campaign schema or gate status changed.

## Tests and limits

On Windows 11 x64, with `CARGO_TARGET_DIR=D:/localdev/devlabs/rrflow/target/c07-sampler-correlation-build`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_PROFILE_DEV_DEBUG=0`, and `CARGO_BUILD_JOBS=3`:

- `cargo test -p rrflow-cli --test installed_reopen sample_trace:: --locked -- --nocapture` passed four tests, including 800 distinct concurrently generated IDs, checked counter exhaustion, existing trace output bounds, and an independently computed fixed SHA-256 vector.
- `cargo test -p rrflow-cli --test installed_reopen sustained::installed_capacity_workload_smoke --locked -- --exact --ignored --nocapture` passed against the real installed engine. It recorded eight fixture records, 24 edges, 16 vectors, 12 replayed transactions and one acknowledged write in the interrupted segment; the sample task was canceled, progress reread, and the acknowledged write verified and replayed after the owned process reopened. Build metadata called itself `unqualified`. This is a small correctness smoke, not a C-07 capacity or soak run.
- `cargo test -p rrflow-cli --all-targets --locked` passed with its explicitly ignored tests left ignored. `cargo clippy -p rrflow-cli --test installed_reopen --locked -- -D warnings` passed. Navigation, documentation, workflow, inventory, formatting and diff checks run before commit.

The earlier `c5b58a3c` Windows characterization remains a failed 2-second sampling-gap result; its raw artifact cannot acquire new server events retroactively. No loaded one-to-one client/server correlation has been run, and PR178 is not integrated here merely because its digest convention matches. Actual source/head/configuration/binary/harness identity, trace sink behavior, resource limits and interference must be frozen and retained for a later bounded integrated characterization. Native sampling cancellation, the full capacity/fault matrix and actual 24-hour soak remain open. Candidate CI and the merge queue remain required; no objective, roadmap or POAM lifecycle status was changed.

- [x] Brief, merge source and effect/traceability scope recorded before changes; original C-07 evidence preserved.
- [x] Real installed interruption/restart/replay smoke, focused bounded-ID tests, owning CLI suite and strict target lint.
- [ ] Combined server/driver trace, fresh exact-head CI, native operating point and full C-07 acceptance.
