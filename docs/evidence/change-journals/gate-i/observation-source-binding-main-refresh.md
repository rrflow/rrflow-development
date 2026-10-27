# Observation-source binding: integrate the provider boundary

**Status:** active integration evidence; dependency and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-i/observation-source-binding-main-refresh`
**Owner:** OBJ-08/11 / I-01/I-04 prerequisite / POAM-008 / [PR163](https://github.com/rrflow/rrflow-development/pull/163)

[PR163](https://github.com/rrflow/rrflow-development/pull/163) bound this dependency refresh before the merge. Starting head `a4b2e79d1c917d8f7fac58930e5c73482a462da5` ordinarily merges PR161 candidate `f3b6f160d10a57e41309deb5f36f7761083143d7`, which includes merged public development main `68c06059fbba600af5f06a6de30667c07e35ae6b`. Driver is this working session. The only conflicts were the generated Gate I journal index and execution inventory; both are regenerated from the combined tree. No implementation is removed or moved. The source declaration remains inactive, with exact approval, engine authentication, authorization, revision/replay and host-clock behavior retained. The host clock remains unverified. The PR161 dependency is not yet merged, and its old queued head was removed from the queue after GitHub rejected a push to that queued branch; fresh CI is required for its updated head.

Read README, Gate I and POAM-008, the source-binding journal and PR brief, PR161's refresh, combined source/test changes and the generated conflict boundaries. No upstream adaptation or new resource mechanism is involved. With the isolated `target/rroflow-source-binding-build` and offline locked dependencies, `cargo test --offline --locked -p rrd-engine --lib observation_source` passed two installed-estate tests and `cargo fmt --all -- --check` passed. Navigation/inventory, documentation, workflow, staged diff and post-commit inventory checks are required on this exact merge before push. The earlier full contract and engine suites remain recorded against the original implementation candidate; they do not replace exact refreshed candidate CI.

No provider stream, event admission, cross-session event deduplication, queue/backpressure, automatic relationship generation or remote delivery was implemented or proven in this refresh. No native C-07 capacity or soak ran. PR161 landing, full candidate CI, review and merge remain pending; no roadmap or POA&M status changes.

Checklist: the PR names owner, baseline, dependency, scope and stop conditions; the generated conflicts, retained implementation and exact focused oracle are recorded; candidate CI and merge remain mandatory.
