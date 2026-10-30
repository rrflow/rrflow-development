# C-07 actual scheduled segment and inventory integration

**Status:** active draft; native interruption and qualification remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-held-segment-integration`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR171](https://github.com/rrflow/rrflow-development/pull/171)

Starting PR171 head `5068968447728955b9214f5baf55e2c748ea9622`; integrated
[PR193](https://github.com/rrflow/rrflow-development/pull/193) candidate
`bdb954e48d2eb751b4839ed6efa761cc1e294071`, itself containing public main
`e73ba9287dafba5d8fb8fb6a990fd145ffd01ab3`. The PR description bound the
dependency integration and the held-segment oracle before source edits.
PR193 has not landed, and this package stays draft until it does and the
original native-interruption stop conditions are met.

The only merge conflicts were the generated Gate C journal index and the old
tracked inventory projection. The latter retains the retirement descriptor;
the former is regenerated from all linked records. The current inventory is
generated under ignored `target`. Preserve all sampler scheduling, sample
format, thresholds, public engine behavior, landed source and tests, and
the revision-bound inventory policy. The prior loaded trace journal remains
unchanged. The direct new source is an additional case within the existing
real installed `installed_capacity_workload_smoke`: hold its test-only async
sample gate in a disposable actual warmup segment, require a later offered
operation's real acknowledgement while the gate remains held, release it,
and compare the complete returned attempts and real resource samples to the
checkpointed progress journal. The existing independently read back,
interrupted, killed/reopened and replayed case remains. The exact new oracle
does not hold or isolate synchronous host acquisition. It cannot establish
bounded native cancellation or identify the original 100k slow RPC.

Reviewed the earlier loaded diagnostic and sampler evidence, actual segment,
sample gate and installed child-process helper, existing trace/reopen and
interruption smoke, native allocation membership implementation, the CI and
inventory conflict and staged source diff. Existing Tokio process and
blocking-task documentation was researched in the earlier linked record;
no third-party implementation is adapted. The test's evidence is real
installed operations and progress readback, not a benchmark or capacity
qualification.

The first actual installed held-segment run did observe a later ACK while the
gate was held, but timed out after its first release: a subsequent sample
entered the same closed test gate. The retained local log
`target/c07-held-segment-integrated.log` has SHA-256
`dc8fda2392015870a664bd6b62ac185bd2f9d25c246a58d41205083e36da0576`.
Before correction the PR brief included the test-only gate helper. The focused
new gate test first failed to compile because `open` was absent; it passed
after adding a one-way open state and a retained `notify_one` permit for the
single admitted waiter. The existing cancellation guard remains. A subsequent
real smoke passed; after adding bounded summary output the final real smoke
passed in 14.88 seconds and reported `acknowledged_sequence=2`, six attempts,
eight resource observations and matching progress checkpoint. It also passed
the existing interrupted ACK, restart and exact replay assertions. The final
local log `target/c07-held-segment-integrated-final.log` has SHA-256
`223f6ff0cdc924b099d2c4199b625318fbe7589298667dc426d36d9ad57b99c8`.
An attempted invocation with an invalid working-directory spelling was
rejected before starting a process; the correct invocation is the result above.

The real test used `cargo test --offline --locked -p rrflow-cli --test
installed_reopen sustained::installed_capacity_workload_smoke -- --exact
--ignored --nocapture` on native Windows x64 with isolated
`CARGO_TARGET_DIR=target/c07-loaded-trace-6bf`, `CARGO_BUILD_JOBS=2`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, and
`CARGO_INCREMENTAL=0`. At that run the modified `sustained.rs` SHA-256 was
`95e7d739b9fcc628bfc0ce9094e5efa2c745c08998e2671978880285ab2b03ef`,
`sample_task.rs` SHA-256 was
`affa75470389c0ac1e37b4f6454a6f5328f67f102603ccbc81e7d8110a502495`,
and their unstaged patch SHA-256 was
`e245ae03df6e55395250a05ff3f5cdd7ce59b28b8f0ae251f6a74b4061ffb7d8`.
The installed test harness SHA-256 was
`5f8b54ebab9a662e477473b7b5ea99901986ed47aa90749a543cd91c47f70f36` and
primary executable SHA-256
`cad3b469136e03b08736224dc00c1d525260fbfc3381ea3decf605174af99ea2`.
These are source plus uncommitted merge identities, not a signed release.

`cargo test --offline --locked -p rrflow-cli` passed 54 tests, with ten
ignored cases; the separately invoked real held-segment smoke is described
above. The prior idle trace/reopen test, selected with `--exact --ignored
--nocapture`, passed against the same integrated source. Ten inventory tests,
11 candidate-admission tests, `python scripts/ci/check_workflow.py`,
`python scripts/ci/check_documentation.py`, generated navigation/inventory
checks and `git diff --cached --check` passed. The separately selected real
`sustained::installed_sampler_trace_during_write_load` completed in 114.74
seconds: 4,000 acknowledged writes, 32 correlated diagnostic samples, 32
overlapping client intervals and no recorded failures, plus its four
representative pre/post-reopen readbacks and exact replays. Client RPC
median/p95/max was 9.282/14.128/14.424 ms, maximum service time 6.135 ms,
mutex wait 0.592 ms, assembly 3.953 ms, memtable profile 3.630 ms and host
acquisition 47.670 ms; up to 40 segments were observed. The local log
`target/c07-integrated-loaded-trace.log` SHA-256 is
`3e02b64f6413f8496e21b40d5e98d691c1207c3386f93127eb6104a3548249a7`.
This uncontrolled small workload neither reproduced the earlier approximately
1.269-second RPC with approximately 1,980 segments nor proves concurrent
storage-lock contention or its cause. `cargo clippy --offline --locked -p
rrflow-cli --test installed_reopen -- -D warnings` and `cargo fmt --all --
--check` passed. Final generated checks follow the journal update. Hosted
candidate and merge-group results, a representative native 100k attribution,
native sampling interruption, full capacity matrix and soak remain unrun.
No roadmap, objective or POA&M status changes.
