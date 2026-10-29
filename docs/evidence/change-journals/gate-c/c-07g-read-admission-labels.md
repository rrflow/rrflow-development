# C-07g: retain read-admission failure categories

**Status:** active implementation evidence; full native characterization pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-read-admission-labels`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR153](https://github.com/rrflow/rrflow-development/pull/153)

## Baseline and problem

The package started from development main `86adada6dab6411cc58cb9aaa931e35472c0f452`; opening commit `d135bab09081d03d1d98160ca4aa76487e3bdaa4` and its PR brief preceded implementation. It incorporates unmerged PR152 candidate `386beab43eaa650c0c0fa87dce3a2891e54c9baf` through an ordinary merge; PR152, PR142 and PR150 remain dependencies.

The retained 8721f17 Windows characterization failed after initial full-corpus verification and two forced reopens with only a generic storage-admission label. The source maps projected-read, source-page and segment-handle read limits to `StorageAdmission` at query/read boundaries, while the driver's read-pressure labels previously matched only `Storage`. Therefore valid read-admission reasons could be reduced to the generic label. Checkpoint admission also had no specific label. These verified classifier gaps do not identify the retained error, its operation, or its resource.

## Change and evidence

`engine_error_label` shares the existing closed projected-read and source-page labels between `Storage` and `StorageAdmission`, and adds allowlisted segment-handle and checkpoint-entry labels. No arbitrary server substring is returned, no retryability or durable outcome is inferred, and unknown details retain the generic fallback plus their existing digest. Engine behavior and limits are unchanged.

The new focused test constructs actual `rrd_store::Error` values and the engine's `StorageAdmission` wrapper. Before the change it failed with `storage-admission` instead of `read-runs`. Afterward it passed for projected reads, source pages, segment handles and checkpoint entries, verifies the public ResourceExhausted classification and bounds label length, and rejects a secret-suffixed lookalike. The existing projected-read test checks all its listed resource types under both engine wrappers. Commands and results on Windows with `CARGO_TARGET_DIR=D:/localdev/devlabs/rrflow/target/c07-independent-review/target`: `cargo test --offline --locked -p rrflow-cli --test installed_reopen read_admission_uses_real_store_error_labels_without_private_details` passed; `cargo test --offline --locked -p rrflow-cli --test installed_reopen` passed 15 and ignored seven explicitly separate tests. `cargo test --offline --locked -p rrflow-cli --test installed_reopen diagnostic_labels` passed three tests; `cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings`, `cargo fmt --all -- --check`, and `rustfmt --edition 2021 --check crates/adapters/rrflow-cli/tests/support/sustained/workload.rs` passed.

## Checklist and limits

Reviewed relevant portions of README, C-07/POAM-035, retained terminal evidence, `workload.rs` classifier/tests, engine error conversion, store error Display and the complete change diff. No external research or source adaptation: this is a local mismatch between existing typed errors and their evidence projection. Native diagnostics retain a bounded constant label and hash only; previous artifacts remain unchanged. The actual 8721f17 error remains unknown and this repair cannot recover its redacted message. Fresh frozen full-corpus characterization, candidate CI, prerequisite merges, native profile and 24-hour soak remain outstanding. Navigation and inventory checks, `python scripts/ci/check_documentation.py`, `python scripts/ci/check_workflow.py`, and `git diff --check` passed. No roadmap or POA&M status is closed.
