# C-07 installed stale-snapshot conflict oracle

**Status:** active controlled correctness characterization; capacity failure root cause remains unproven
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-installed-conflict-oracle`
**Owner:** OBJ-02 / C-07 / POAM-035 / [PR214](https://github.com/rrflow/rrflow-development/pull/214)

Starting revision `f65e0a9e0de509865c13957fc3a903e31c9ff63b` includes the [failed 4k characterization](c-07-write-conflict-characterization.md). The PR brief was extended before the test edit. That run recorded three runtime-conflict write rejections, but its client intervals did not identify which internal conflict branch fired or establish transaction read-stamp timing. The target was a controlled installed-engine oracle for one possible mechanism, without changing engine behavior, thresholds, retry policy, or the campaign.

The new explicitly ignored `installed_disjoint_writes_from_one_stamp_conflict_without_partial_effect` in `crates/adapters/rrflow-cli/tests/support/sustained.rs` opens two real installed-engine transactions at the same cursor. It commits the first synthetic record with edges and vectors, asserts the second disjoint write returns the exact stale-cursor conflict, aborts it, checks the rejected record is absent and the accepted record remains, explicitly submits the second from a fresh transaction, verifies both after a kill and reopen, and checks idempotent replay has the original receipt. This proves that behavior under the controlled schedule; it does not prove the earlier campaign's conflicts followed that schedule, establish a performance fix, or establish qualification. The source file tested had SHA-256 `527b1bdf067b4e3cc2fd275c543f03fc4bc1fbf100802757c7742c9e05ffe144`.

Reviewed the changed test and diff, the surrounding installed-estate and workload helpers, `rrd-store` semantic commit and runtime conflict mapping, and the existing store snapshot concurrency test. No implementation was moved or deleted. The prior journal records the empirical characterization and source-level alternatives; no upstream implementation was adapted. Existing synthetic identifiers and bounded test requests are used; no production tracing, raw server detail retention, or resource budget was changed.

The first focused test reached the installed API but failed before the conflict because the new fixture omitted the required deadline. The fixture was corrected to the bounded `RequestOptions::new` shape already used by the workload. With `CARGO_NET_OFFLINE=true`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_BUILD_JOBS=2`, and `CARGO_TARGET_DIR=D:/localdev/devlabs/rrflow/target/c07-integrated-profile/target`:

- `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::installed_disjoint_writes_from_one_stamp_conflict_without_partial_effect -- --exact --ignored --nocapture`: 1 passed, 0 failed.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen -- --test-threads=2`: 28 passed, 0 failed, 14 ignored; the controlled test was separately invoked above.
- `cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed before this record.

Checklist: the PR brief preceded the scoped source edit; the failure was preserved and corrected; the changed source and diff were reviewed; no source-adaptation, deletion, or authority change occurred. Navigation and inventory checks are recorded in the associated commit verification. This test is not admitted to candidate CI in this change and has only been exercised on this Windows development host. Integrated candidate and protected merge-group CI, a proven causal repair and repeat measurement, 100k capacity, required native matrix, and the 24-hour soak remain open. No roadmap or POA&M status changed. No merge is claimed.

The first navigation regeneration rejected the Windows PowerShell UTF-8 BOM as a missing title; the BOM was removed and regeneration was rerun.
The first `python scripts/ci/build_execution_inventory.py --check` reported expected drift after the new journal; the local generated inventory was refreshed before rechecking.
The first staged `git diff --check` flagged two Markdown line-break spaces; the command chain committed despite that failure. The spaces were removed in the following local commit and the staged check was repeated.
