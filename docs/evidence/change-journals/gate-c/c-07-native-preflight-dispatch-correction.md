# C-07 macOS preflight dispatch correction

**Status:** active experimental correction; native outcome and restoration pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-preflight-dispatch-correction`
**Owner:** OBJ-02 / C-07 / POAM-035; [PR204](https://github.com/rrflow/rrflow-development/pull/204)

The [experimental preflight](c-07-native-preflight-experiment.md) was pushed as `6053112df6bdb10ce7b25dbd7a5aac102b84ae72`. `gh workflow run rrd-lsm-benchmark.yml --repo rrflow/rrflow-development --ref codex/c07-ci-native-diagnostics` failed with HTTP 422 before creating a run: at line 17 GitHub rejected `runner.temp` in a job-level environment expression. No native test ran and this is not capacity evidence.

The correction moves that one environment binding to the native preflight step, where the runner context is available. The workflow remains temporary and must be restored byte-for-byte to `5ab909b025d19a204914e26419ef49a40cd16852` after dispatch. The original benchmark workflow, native test script, required repetitions and main candidate gate are not changed by this correction. The PR brief already bound the experimental workflow and restoration. Source reviewed: temporary workflow, the installed-crash job's step-level environment bindings, and the GitHub dispatch error. No external algorithm, runtime trace or resource claim is introduced.

The failed dispatch is the first-failure oracle. Stage and check the corrected workflow, regenerate navigation and inventory, then dispatch its exact pushed revision. A separate later record must report the actual run and restoration. No C-07, POAM or alpha status changes.

- [x] Retain the failed dispatch and exact error.
- [x] Correct the runner-context placement without changing the preflight or required tests.
- [ ] Dispatch the corrected experimental revision, inspect native results, and restore the original benchmark workflow before readiness.
