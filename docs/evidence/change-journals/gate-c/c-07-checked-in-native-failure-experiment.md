# C-07 checked-in native failure helper experiment

**Status:** active experiment; launch, result and restoration pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-checked-in-native-failure-experiment`
**Owner:** OBJ-02 / C-07 / POAM-035 / [PR204](https://github.com/rrflow/rrflow-development/pull/204)

Driver: this working session. Parent `529475acfed09d376c84dc9696e11b6faa046de7` contains the checked-in manual helper, modeled tests, permanently separated artifact paths and the original benchmark workflow. The PR brief authorized testing that actual helper on hosted macOS before editing this temporary workflow. Because the new permanent workflow is not yet on default main, temporarily replace the already registered benchmark with a manually dispatched exact-revision macOS job that runs the checked-in modeled tests and helper and uploads only its injected JSON receipt. Restore the benchmark byte-for-byte from the parent in an ordinary successor commit after confirming the run's pinned head. No required CI gate, capacity case, retry or product state is changed.

The earlier [inline experiment](c-07-native-failure-capture-outcome.md) established genuine macOS failure and uploaded receipt, but cannot prove this newly checked-in helper. This run is an injected invalid-filesystem failure; it cannot establish EBUSY cause or native capacity. Stop on unexpected image creation, cleanup uncertainty, missing or malformed artifact, or failure to restore the benchmark. Local helper tests, native runner tests, policy, documentation, navigation and inventory checks passed at the parent; temporary workflow YAML parsed. The native result, artifact readback and restoration will be recorded separately. No roadmap, objective or POAM status changed.
