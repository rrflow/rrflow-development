# C-07: installed backup-lane characterization

**Status:** active characterization evidence; no qualification or release gate passed  
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-backup-lane-installed-20261107`  
**Owner:** C-07g, [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md), [draft PR #294](https://github.com/rrflow/rrflow-development/pull/294)

## Frozen executable and commands

Starting implementation candidate `399acfd2da14b1b556b9b2db693e1743ea174046` on Windows. The actual installed debug executable SHA-256 was `7dd21bbe27d3be8a0c6a97a541fe31bebb3a5d22b92cc5349b4411587e9af022`. Both runs were one-client, 5 requests/sec, eight synthetic records, 24 edges and 16 8-dimensional vectors. Each declared two forced reopens, 3-second warmup, 5-second measurement, 3-second burst, 5-second drain and 5-second soak. These are **short debug characterizations**, with native allocation enforcement unavailable, not the 100,000-record ARM workload, optimized performance evidence, a 24-hour soak, or C-07 qualification.

The operator-local preparation script `D:/localdev/devlabs/rrflow/target/prepare_c07_backup_lane_399acfd2.py` created separate fresh project/configuration/campaign directories. In the candidate worktree, with `CARGO_TARGET_DIR=C:/Users/jessa/AppData/Local/Temp/rrflow-c07-observer-7e60a4ac`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, and the respective absolute `RRFLOW_SUSTAINED_CAMPAIGN`, `RRFLOW_SUSTAINED_CONFIGURATION` and fresh `RRFLOW_SUSTAINED_EVIDENCE` paths, each run executed `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::installed_sustained_capacity -- --exact --ignored --nocapture`. The explicit native allocation receipt was absent. Both commands exited 0. Each evidence directory was then separately checked with `cargo run --offline --locked -p rrflow-eval -- storage campaign verify <evidence-directory>`; both verifiers exited 0 and reported completed with `qualification_passed: false`.

## Results

| Campaign | Selected lanes | Backup starts / verified receipts | Result SHA-256 | Progress SHA-256 |
|---|---|---:|---|---|
| `058cd4227ca896aa582ce035fdad2730b6f42d0cddda3982b708933a121c0c44` | warm | 0 / 0 | `6994471ec33f636fd23d95c938d2c52bb04b4725a847e38dede3437214bf7957` | `725209d5abae00a682fff6506755a0f5a59338cd0d1142886a582864996433fc` |
| `a17c28e22876520432b9f5c46a65b2cea2418657d378f3aa4ec4ad9bc52be402` | warm, backup-under-write | 1 / 1 | `d2a85192b2e76af422ddde2483d831b9f15754a4b82e12fdff49d0c2cc00422a` | `608921ffe9d64f8fa7402e1e4f559d4e396f302d997396c4414b9f330d175c7b` |

The selected case's archive receipt marked application complete and the independent verifier accepted the backup lane, including the overlapping acknowledged write and archive catalogue verification. The unselected case recorded no backup start or completion. Both had five completed segments and reported zero attempt failures and zero latency-budget overruns in these small local tests. Both also executed two forced reopens despite not listing that lane: the campaign explicitly supplied `forced_reopens: 2`, and the current verifier counts the schedule's reopens independently of `required_lanes`. No broader optional-reopen behavior is inferred or changed.

The full raw inputs, progress, result, driver and verifier output remain under `D:/localdev/devlabs/rrflow/target/c07-backup-no-lane-399acfd2/` and `D:/localdev/devlabs/rrflow/target/c07-backup-selected-399acfd2/`; these local paths are not portable artifact storage. The previously failed [100,000-record ARM result](c-07-arm100k-unrequested-backup-20261107.md) is unchanged. Its two runtime-conflict rejections and six latency overruns remain unresolved.

## Review and remaining evidence

This record adds evidence only. Reviewed the candidate scheduling diff, `sustained/management.rs`, `sustained/lifecycle.rs`, the campaign verifier's backup and forced-reopen validation, and the retained results. No upstream code adapted. [done] selected/unselected installed execution and independent verification; [open] complete candidate review and current-main reconciliation, candidate and protected merge-group CI, native capacity matrix, recovery and 24-hour soak; [open] real-project source visibility and ARM conflict/latency investigation. No roadmap or POA&M status changed and no C-07 or alpha completion is claimed.
