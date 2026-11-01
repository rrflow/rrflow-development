# C-07 selected-segment native outcome and resource-sample gap

**Status:** active retained failed native characterization; qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-selected-segment-native-outcome`
**Owner:** OBJ-02 / C-07g,h / POAM-035 / [PR169](https://github.com/rrflow/rrflow-development/pull/169)

Driver: this session. Before repository edits PR169 recorded this terminal-outcome scope and stop conditions. Its immutable [launch record](c-07g-selected-segment-native-launch.md) identifies tested source `c02f33c8254d5bbac95344a8dd6de5bf71c8cc96`, tree `7abd791b81482cab9150c92b8839e017cd6ec511`, executable `f950ab3d223dbe6f137b74a7a25f05bbfa37ee3261b259b196a651ccb78d5257`, harness `0b5b4aff3ff5dbd035de2a9ed59e9d72d53a65344ccba225009d52f2ae1613d8` and campaign `67b400736f9951e5da38db468a670ca9582a1b06a868f7b79d08234aa460e12e`. The same Windows Job 2-GiB committed-memory and four-CPU-equivalent bounds, original corpus, deadlines and warm-only ten-minute soak field apply. Other host build and evidence-check activity was observed and documented in the launch/integration records; timing is not isolated.

The run acknowledged and independently verified 100000 initial records, completed two forced reopens, and independently verified 100000 consecutive post-reopen records. It advanced past the prior 47240-record post-reopen failure. The first workload segment recorded 50 attempts: 42 acknowledged and eight rejected. The result hashes do not establish each rejection's cause. Twelve retained resource samples all report PID 77216; one consecutive sample interval was 2252002100 ns, exceeding the frozen 2000000000 ns maximum. The evaluator rejected the segment with `resource series has a gap or invalid process identity`; the identical recorded PIDs and observed excessive interval identify a gap but not why sampling was late. No measured segment was accepted, qualification is false, the launcher reports failure and owned-process cleanup confirmed. The earlier failure remains retained separately.

| Local artifact | SHA-256 |
|---|---|
| `evidence/progress.jsonl` (80452851 bytes) | `73f80504545f2028383e4d975df5a766795a13e1e2eb50af1e887be1cd05e9c7` |
| `evidence/result.json` | `35d5434df7ae60bb3554ec5ba1537168d4f11a26fee33292f8f3144bd8a0ddcd` |
| `evidence/launch.json` | `5a60da49f4e9d7b4b82d84046b41c071d8dd8895220ef33e924245f87c17d6a4` |
| `launcher.stderr.log` | `0a568559421fcae73f7ab36fda36b158c41b4668feaece906c2cbff5d9b0e9bd` |

Artifacts remain under `D:/localdev/devlabs/rrflow/target/c07-windows-selected-segments-c02f33c8`; hashes identify local files, not publicly uploaded artifacts. A read-only Python scan counted event kinds and outcomes, checked all post-reopen ordinals consecutively from zero, and computed the maximum adjacent resource-sample gap from recorded elapsed times. The result and launch JSON plus the original stderr supplied the terminal outcome. No sample or event was invented, no original resource threshold was raised, and no cause is assigned to host activity.

Read in full: this run's `result.json`, `launch.json`, stderr, PR169 brief and the user-provided `AGENTS.md`; relevant launcher and evaluator validation, owner, previous launch journal and progress records were inspected. No source algorithm, upstream code or new runtime trace was introduced. The specific sampling gap, workload rejections, full native matrix, actual 24-hour soak and exact candidate CI remain open work. This record adds verified failure detail to POAM-035 without changing its status or any roadmap/objective status. `python scripts/ci/check_documentation.py`, `python scripts/ci/check_workflow.py`, `python scripts/knowledge/render_navigation.py --check`, staged `git diff --check` and `python scripts/ci/build_execution_inventory.py --check` passed. The first documentation check rejected the missing active status marker; correcting the status and regenerating navigation resolved it.

Checklist: exact source and failure bound before record edits; raw artifacts and SHA-256 retained; full corpus and partial workload separated; observed gap distinguished from cause; all rejections and nonqualification disclosed; generated navigation/inventory and full diff reviewed; candidate CI and merge pending.
