# C-07 temporary macOS capacity preflight

**Status:** active experimental preflight; outcome and restoration pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-preflight-experiment`
**Owner:** OBJ-02 / C-07 / POAM-035; [PR204](https://github.com/rrflow/rrflow-development/pull/204)

The PR brief bound this experiment before editing. Driver: this session. The source parent `5ab909b025d19a204914e26419ef49a40cd16852` contains the diagnostic candidate and the ordinary benchmark workflow. A registered GitHub workflow is required for manual dispatch, so this experimental revision temporarily replaces only `.github/workflows/rrd-lsm-benchmark.yml` with a manual macOS preflight: exact recursive checkout, pinned Rust, focused harness tests and the same three fresh/existing APFS capacity repetitions. It retains controlled diagnostic JSON on failure. The source parent remains the canonical destination for the original scheduled and manual benchmark, and the workflow must be restored byte-for-byte by an ordinary commit before the PR can be ready or merged. No engine, fill assertion, limit, test corpus or candidate gate changes.

The benchmark workflow and its benchmark matrix, candidate workflow, capacity runner and its tests were inspected. Traceability: the temporary preflight does not replace the benchmark capability; it isolates the real macOS failure at the already-owned C-07 runner. Restoration is proved by byte comparison against the parent, not by ancestry or compilation. The retained run's exact experimental commit/tree, harness and diagnostics are evidence only for that source; the source after workflow restoration requires its own full candidate and protected merge-group checks. No retry, skip, force push, or claim that this preflight is candidate CI is allowed.

The local focused runner suite passed 29 tests before the experiment. GitHub-hosted macOS is needed because this Windows host has no `hdiutil` or APFS. The bounded preflight runs the repeated native cases without waiting for unrelated CI partitions. Stop on source mismatch, unsafe cleanup, unexpected failure or inability to restore the benchmark workflow. A later separate evidence record will state the actual run, outcome, exact restoration and any remaining failures; this record does not claim a result or change any roadmap or POAM status.

- [x] Bind the experiment, owner, source, safe temporary scope and exact restoration in the PR before implementation.
- [x] Preserve the original benchmark workflow in the identified parent and keep the existing native tests and limits.
- [ ] Dispatch the exact experimental revision, retain and inspect its result and diagnostics.
- [ ] Restore the benchmark workflow byte-for-byte, record the outcome separately, and keep full candidate and protected merge-group gates mandatory.
