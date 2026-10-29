# C-07 read-admission labels: integrate reopen-window candidate

**Status:** active integration evidence; dependency and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-read-admission-labels-main-refresh`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR153](https://github.com/rrflow/rrflow-development/pull/153)

[PR153](https://github.com/rrflow/rrflow-development/pull/153) bound this dependency refresh before the merge. Starting candidate `33d26f13f35b7406c8242a8550f048822f9890bc`, tree `0613c6698c2576b0e0f612d7debe04fd94abbca4`, ordinarily merges refreshed PR152 head `f679e33f7ac89f16a86c04af0abc31b21629accb`, which includes public development main `68c06059fbba600af5f06a6de30667c07e35ae6b`. Driver is this working session. The generated execution inventory was the sole conflict and is regenerated. The closed error labels, original hashed diagnostic, independent bounded post-reopen verification and resource limits remain unchanged; no arbitrary error text is emitted. No source is removed or moved. This refresh changes this record, generated navigation/inventory and the merge from the stated dependency.

Read README, C-07/POAM-035 owners, original [read-admission evidence](c-07g-read-admission-labels.md), dependency PR152 refresh, candidate brief, full conflicting inventory and affected classifier/tests. No upstream code or new diagnostic authority is adapted. The retained native failure's cause is unknown; the label is not a repair or qualifying evidence.

On Windows with `CARGO_TARGET_DIR=D:/localdev/devlabs/rrflow/target/c07-independent-review/target`, `cargo test --offline --locked -p rrflow-cli --test installed_reopen read_admission_uses_real_store_error_labels_without_private_details` passed, and `cargo test --offline --locked -p rrflow-cli --test installed_reopen --quiet` passed 15 with seven long/native tests ignored. `cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings` and `cargo fmt --all -- --check` passed. Navigation/inventory, documentation, workflow and staged diff are checked before commit; inventory is repeated after commit. PR152 landing and fresh exact-candidate CI are required. No new 100,000-record readback, capacity matrix or soak was run; C-07 and POAM-035 remain open.

Checklist: PR records owner, starting revision, dependency, scoped merge, oracle and stop conditions; generated conflict and actual checks are recorded; review, dependency, candidate CI and merge remain pending.
