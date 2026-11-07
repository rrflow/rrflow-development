# C-07: honor the requested backup lane

**Status:** active draft; installed lane runs and candidate CI pending

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-backup-lane-admission-20261107`

**Owner:** C-07g, OBJ-10/11, [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md), [draft PR #294](https://github.com/rrflow/rrflow-development/pull/294)

## Change and traceability

Starting main `d21a1556657fb17e783a4396368250459e03a2e8`; the immutable [ARM failure record](c-07-arm100k-unrequested-backup-20261107.md) was committed as `07c82fe5e475412ef8ab4f801171476338cf9ddd`. The PR brief preceded implementation. In `crates/adapters/rrflow-cli/tests/support/sustained.rs`, the existing measured-sequence backup branch now requires the campaign's `BackupUnderWrite` lane. The same original first mixed-write point, deadline, task, progress, verification and qualification requirements remain; no engine behavior or evidence threshold changed. The selection test covers omitted and selected lane, phases, neighboring sequences and confirms the selected point is a mixed write. This is a driver correction, not a completed capacity measurement.

Read README identity/C-07 sections, Gate C C-07, change-authoring procedure, the measured scheduling and admission routines in `sustained.rs`, `sustained/management.rs`, `sustained/lifecycle.rs`, `rrflow-eval/src/storage/sustained.rs` validation and `sustained_backup.rs` tracker, and the final diff. Full-file review outside these specified ranges is not claimed. No third-party source was adapted. Existing durable progress and failure outputs remain the trace and the original resource limits remain in force.

## Checks and outstanding work

Windows local `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::backup_is_due_only_for_the_selected_lane_and_first_measured_mixed_write -- --exact --nocapture` passed one test. An earlier attempt omitted the module name and ran **zero** tests; it was corrected and is not claimed as coverage. The full `cargo test --offline --locked -p rrflow-cli --test installed_reopen` exited 0: 33 passed, 14 ignored; the log SHA-256 is `71affb6d46dc32aceca7dd714ce11abc34217e87d88cd64e7fca91a9b7f80498`. `cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check` passed. Cargo used `C:/Users/jessa/AppData/Local/Temp/rrflow-c07-observer-7e60a4ac` with incremental compilation off.

The retained failed ARM warmup ledger identifies its two rejected writes as `api-conflict-storage-runtime-conflict`; six latency overruns were also recorded. These are separate from the unrequested backup timeout and are not resolved or explained by this patch.

[done] Prior PR brief, frozen failure and source mapping; [done] lane admission and unit/owning-target checks; [open] actual installed selected and unselected backup campaigns, independent result readback, complete review and candidate/merge-group CI; [open] native capacity, crash matrix and 24-hour soak. No status change, merge or C-07 completion is claimed. The full 100,000-record run remains failed and has not been retried.
