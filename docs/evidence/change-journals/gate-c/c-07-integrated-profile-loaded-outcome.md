# C-07 integrated profile: installed loaded characterization

**Status:** active bounded characterization; capacity and soak remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-integrated-profile-loaded-outcome`
**Owner:** OBJ-02 / C-07 / POAM-035 / [PR207](https://github.com/rrflow/rrflow-development/pull/207)

The existing ignored installed load/trace/restart test was run on clean revision `15a0ff90d0a09e9951c42663c87e097fea3eec80`, tree `e40f95e57341a08efa27d3bcf7c665f0b8e0a600`, containing the integrated native sampler and [constant-time profile repair](c-07-integrated-memtable-profile.md). Driver: this working session. Before launch the PR brief required the installed path, retained timings, exact source identity, and no capacity claim. This record changes evidence and its generated index only; there is no runtime or gate-status change.

Windows x64 unisolated developer host; `CARGO_BUILD_JOBS=2`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`. `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::installed_sampler_trace_during_write_load -- --exact --ignored --nocapture` passed one test in 137.87 seconds after a 5m38s first build. The repository Cargo.lock SHA-256 was `56af26e058ab3d82945d1b2a81940eadf511e34dada913e09622cfe8d151c961`; modified memtable source SHA-256 `c0fc9e678c44f985a5c5d2e0f337b5732336410a162331016c799566729e83bf`; test harness SHA-256 `f0a3cb208f69bace13c190a8b1d7096a3e33d781561b1f86c595a4397bbb9bd5`; `CARGO_BIN_EXE_rrflow` staged executable SHA-256 `0d2a86a975644d80208c7f42433bd583f7263d2982059cd4979bbe5d8dc49093`. This is not a signed release.

The actual test acknowledged 4,000 synthetic writes and obtained 32 correlated diagnostic samples; all 32 client write/sample intervals overlapped. Four representative readbacks succeeded before restart and again after restart, followed by four idempotent replays. The test reaped both owned processes. Overlapping client intervals do not prove simultaneous mutex contention. The raw local UTF-16 log `target/c07-loaded-profile-15a0ff90.log`, SHA-256 `811d25b6078cb6fe69bbe1a712b867c826847f664df42844ff2e9c4748345c47`, and locally parsed summary `target/c07-loaded-profile-15a0ff90-summary.json`, SHA-256 `9c2fe07fc60394f1ff1ee93b589a1e1fc0096885367dc3af29f7bb893b3bc197`, remain ignored local artifacts, not CI or qualification artifacts. The parser requires exactly 32 samples and the successful test summary; it reports nearest-rank p95 and the mean of the two middle values as median.

| Component | Median | p95 | Maximum |
| --- | ---: | ---: | ---: |
| Client diagnostic RPC | 8.646 ms | 10.401 ms | 11.588 ms |
| rrflowKV mutex wait | 0.138 ms | 0.453 ms | 0.953 ms |
| In-lock diagnostic assembly | 0.347 ms | 0.406 ms | 0.415 ms |
| Memtable profile | 0.00045 ms | 0.0007 ms | 0.0007 ms |
| Server service | 2.196 ms | 2.850 ms | 2.967 ms |

At most 56,377 keys, 75,972 versions and 40 segments were observed; maxima need not be from the same sample. Unlike the retained failed full-corpus observation, this run does not contain roughly 1,980 segments or its workload, isolation and native resource constraints. It does not reproduce or explain that run's 1.269-second client-observed diagnostics call. The prior [loaded characterization](c-07g-loaded-diagnostic-trace.md) used similar synthetic test shape on a different revision and execution, but host scheduling and build differences prevent attributing all observed differences solely to this change. No installed throughput, C-07 capacity or 24-hour soak claim follows from these numbers.

Checklist: source/tree and installed binary identities recorded; unchanged test and prior evidence inspected; no external adaptation or new research needed; opt-in existing trace measures the real storage boundary; exact command, output and limitations retained; journal and nearest navigation index created. The original compile/fixture, simultaneous Windows relink and navigation issues are recorded in the preceding journal. The candidate remains draft; PR194's macOS image-creation failure, dependency review, candidate and merge-group CI, full native matrix, 100,000-record admission and actual 24-hour soak remain open. A later evidence-only commit does not change the tested revision. No objective, roadmap or POAM status changed.
