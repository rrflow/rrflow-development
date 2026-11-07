# C-07: ARM 100,000-record campaign exposed unrequested backup

**Status:** active characterization evidence; C-07 remains open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-arm100k-unrequested-backup-20261107`

**Owner:** C-07g, OBJ-10/11, [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)

## Frozen run and observed failure

Public source commit `40870e5c5f8f79dfebc7126bf721458ccf789634`, tree `fd041325a1861ad3c7180541c0e563a02ab6b31c`, is the tested #292 candidate tree on public main `d21a1556657fb17e783a4396368250459e03a2e8`. On clyffy-01 Linux ARM64, the installed release executable SHA-256 was `b885d2071f99c590a27921c88c710df301729fa5a3617b5ef3da90a5f5b8eabc` and the installed test harness SHA-256 was `348cc9aa59e7e6346f4ec441778f2ef494b6d17f7ad99df2eb419f5f4e9f0a24`. The launcher enforced an aggregate 2 GiB cgroup-v2 ceiling and four CPU equivalents. Host RAM was approximately 121.6 GiB, not the qualifying 8 GiB desktop envelope.

This explicitly short **characterization**, not a qualification or 24-hour soak, used 100,000 synthetic records, 500,000 edges and 100,000 384-dimensional vectors. It acknowledged and verified the corpus, reopened twice, and the retained progress recorded 100,000 post-reopen verifications. The warmup receipt records 50 attempts, two failures and six latency-budget overruns. A 20-second measured period started an unrequested backup, and the installed driver exited 101 with `backup interference deadline`; launcher exited 1 with `cleanup_confirmed: true`. The run did not establish backup completion or a valid measured segment. No cause of backup duration or the warmup failures is inferred here.

Campaign digest `3812ff433946c1b2152c962d8bfe79fdd34a0138b11176c78267a270e07b2a0b`; retained artifact SHA-256: progress `9104329c95c4a7b513b68a6c877ce09afe6b5212c0640358a1ed707b078db575`, result `f5208022d3194e3b9f18aa06fa2d8ee7da81a2e94dc0128986dd9dec84f7f043`, launch `7f5e570e44411f84f6565e24f65e999b628fe392c16f7ec07066cc2bd2b15241`, and launcher log `881d53dff39607e36f703a3e87161a378003a471a7e51d5652be0db9550adb47`. The complete retained evidence is at `D:/localdev/devlabs/rrflow/target/c07-arm100k-failure-evidence-20261107/` and remains on the run-owned server directory `/tmp/rrflow-joined-arm-20261107.5Wra1fil/`. These local/server locations are not a portable artifact guarantee.

Independent command `/tmp/rrflow-joined-arm-20261107.5Wra1fil/build/debug/rrflow-eval storage campaign verify /tmp/rrflow-joined-arm-20261107.5Wra1fil/characterization-100000/evidence` verified the retained structure, printed the failed result and exited 1 with `campaign did not complete; failure evidence retained`. The verifier executable SHA-256 was `69f24af789050b12c6aeedd0d7f1c8c0e373f087384210fb2bca935856f09f89`. `qualification_passed` is false.

## Source finding and next repair

The frozen campaign requested `warm` and `forced_reopens` only. `crates/adapters/rrflow-cli/tests/support/sustained.rs::run_estate` checks the measured duration for backup only when `BackupUnderWrite` is requested, but its measured scheduling loop starts backup whenever sequence is 14 regardless of the lane. `management.rs::ManagementTask::start` uses the campaign's 30-second operation deadline for both create and verification; `lifecycle.rs::backup_started` invokes the installed backup and archive verification. This establishes unrequested execution and the observed timeout, not whether a requested full-size backup meets a separately chosen operating objective. A correction must not weaken required backup qualification or relabel this failed run as passing.

Source inspected: `README.md` opening and C-07 owner; `docs/roadmap/rrflow-1.0/gate-c.md` C-07; `docs/roadmap/rrflow-1.0-execution/change-authoring.md`; `sustained.rs` measured loop and admission; `sustained/management.rs` and `sustained/lifecycle.rs` relevant routines; `rrflow-eval/src/storage/sustained.rs` campaign lane validation and `sustained_backup.rs` tracker. This is not a whole-file or repository review. No third-party code was adapted. Progress events, the original deadline and the retained failure serve as trace/debug evidence; do not add sampled telemetry as a substitute.

This evidence-only record changes no engine or harness code. Next: prepare the reviewable PR brief, implement lane-aware scheduling, test both selected and unselected backup, rerun the installed journey on a new frozen revision and preserve every failure. The 100,000-record run is not rerun automatically. C-07, POAM-035 and alpha status remain unchanged; no CI or merge is claimed.
