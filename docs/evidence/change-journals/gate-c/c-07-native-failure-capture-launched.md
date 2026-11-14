# C-07 native failure-capture experiment launch

**Status:** active experiment; outcome and native artifact readback pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-failure-capture-launched`
**Owner:** OBJ-02 / C-07 / POAM-035 / [PR204](https://github.com/rrflow/rrflow-development/pull/204)

The [bound experiment](c-07-native-failure-capture-experiment.md) was dispatched as [run 40941503762](https://github.com/rrflow/rrflow-development/actions/runs/40941503762) on exact revision `7b26adcb6f7fc415babb3713aef1a0f9ab5c15b9`, tree `a24cfac1d74459885816848372f15d4561b7c39a`. GitHub reported that exact `workflow_dispatch` revision and queued status before restoration. This experiment injects an invalid filesystem to exercise genuine native error handling and artifact retention; it is not a capacity case or a reproduction of the prior EBUSY.

After confirming the pinned run identity, the canonical benchmark workflow was restored from parent `a300ab4ca35c94121c60d927b10e8c8ccb18f103`. `git diff` against that parent's benchmark file was empty, and both file blobs are `aadc8ec6b246c3ceff1d6fc5674f74b8f26c1fb1`. This is an ordinary successor commit, not a rewrite. The run was still queued when this record was written, so neither its outcome nor artifact has been verified. Inspect the actual run and downloaded artifact before making a native-capture claim; retain a separate terminal evidence record. PR194's original failure, candidate and merge-group gates, C-07 native matrix and 24-hour soak remain open. No status authority changed.
