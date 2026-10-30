# C-07 macOS preflight launched and benchmark restored

**Status:** active preflight; native outcome pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-preflight-launched`
**Owner:** OBJ-02 / C-07 / POAM-035; [PR204](https://github.com/rrflow/rrflow-development/pull/204)

The first dispatch error is retained in the [correction record](c-07-native-preflight-dispatch-correction.md). The corrected experiment `5e8f6ed293158b071f9d4d922ebf70020ce080bf` was dispatched with `gh workflow run rrd-lsm-benchmark.yml --repo rrflow/rrflow-development --ref codex/c07-ci-native-diagnostics`, creating [run 40933586936](https://github.com/rrflow/rrflow-development/actions/runs/40933586936). GitHub reports `workflow_dispatch` and that exact head SHA; the native job was in progress when recorded. No result, native capacity success or root cause is claimed here.

After confirming the run's immutable head, restored `.github/workflows/rrd-lsm-benchmark.yml` with `git restore --source=5ab909b025d19a204914e26419ef49a40cd16852 -- .github/workflows/rrd-lsm-benchmark.yml`. `git diff 5ab909b025d19a204914e26419ef49a40cd16852 -- .github/workflows/rrd-lsm-benchmark.yml` returned no diff, proving the normal benchmark workflow's bytes match its original canonical parent. The temporary workflow never touched main or a merge gate. The PR remains draft; final native outcome, diagnostic artifact, dependency integration and full candidate/merge-group CI require separate evidence. No roadmap or POAM status changes.

- [x] Dispatch the exact experimental revision and verify its run identity.
- [x] Restore and compare the original benchmark workflow before PR readiness.
- [ ] Inspect the eventual native result and any diagnostic artifact; record the outcome in a new linked journal.
- [ ] Resolve PR194, reproduce and repair any proven issue, and pass full candidate and protected merge-group CI.
