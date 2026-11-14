# C-07 macOS native image creation diagnostics

**Status:** active draft diagnostic candidate; native validation and failure cause pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-create-diagnostics`
**Owner:** OBJ-02 / C-07 / POAM-035; [PR204](https://github.com/rrflow/rrflow-development/pull/204)

## Bound change and observed failure

Driver: this session. Public main `44cbfa7346fe458d84daead188d33cbac7392efd` was ordinarily merged with the failed standalone candidate [PR194](https://github.com/rrflow/rrflow-development/pull/194) head `a385b484529c8eefd6c957342e90d0a6f7c2441b`, producing starting candidate `0c5ec82d00124b6d047f8d1fbf9bd9c9eaaaed19`. The PR brief preceded the first repository edit. This candidate depends on PR194 and remains draft.

PR194 [candidate run 40920440107](https://github.com/rrflow/rrflow-development/actions/runs/40920440107) failed its macOS installed-domain-crash job in the repeated APFS capacity step: preceding fixtures printed detached, then `hdiutil create` reported `Resource busy`. The OS command's exit was 1; no detailed cause was reported. The temporary fixture was said to be retained on the runner, but GitHub decommissions hosted VMs after jobs, and this step uploaded no fixture or diagnostics. Do not infer whether this is RRFlow, macOS, runner contention, or an unsafe cleanup defect from that observation.

The targeted change adds `-verbose` to the owned APFS creation command; retains bounded head and tail of this controlled native command's output with mode, point, case ordinal, harness digest, fixture basename and CI run coordinates; and attempts a macOS-only upload of the resulting diagnostic JSON even after step failure. Cleanup state is explicitly `unknown`; neither disk images nor arbitrary product output is uploaded. Native failures still stop the case loop and fail the job; cleanup failures remain failures, no automatic retry is introduced, and all existing capacities, native assertions, repetition counts and candidate gate remain intact. A runner loss or artifact failure can still prevent retention.

## Review, research and traceability

Reviewed README and the C-07 / POAM-035 / CI owners, change-authoring procedure, `scripts/ci/run_native_capacity.py`, `run_filesystem_containment.py`, `run_offline_lifecycle.py`, `test_native_capacity.py`, `.github/workflows/ci-reusable.yml`, `scripts/ci/check_workflow.py`, candidate-admission tests, and the failed run log. The platform manual for [`hdiutil`](https://manp.gs/mac/1/hdiutil) describes verbose failure diagnostics and EBUSY as failure to obtain needed exclusive access; it does not diagnose this case. GitHub's [hosted runner documentation](https://docs.github.com/en/actions/how-tos/manage-runners/github-hosted-runners/use-github-hosted-runners) describes VM decommissioning, and its [artifact documentation](https://docs.github.com/en/actions/tutorials/store-and-share-data) describes later retrieval. No upstream source was adapted.

The diagnostic captures native-create output only; it never logs arbitrary test-harness output into the artifact. The bounded test oracle injects a controlled failure, checks case identity and truncation, verifies that later cases do not run, checks unrelated product error text is not retained, rejects a diagnostic directory outside runner temp, and ensures a diagnostic-write failure cannot replace the original error. A chained cleanup failure retains the preceding native error while leaving cleanup state unknown. CI policy tests reject lost diagnostic wiring, broadened artifact paths and missing APFS repetitions. Actual macOS native behavior remains a required later observation, not a mock-derived claim.

Changed paths: `scripts/ci/run_native_capacity.py`, `scripts/ci/test_native_capacity.py`, `.github/workflows/ci-reusable.yml`, `scripts/ci/check_workflow.py`, `docs/operations/ci.md`, `docs/poam/rrflow-1.0-alpha/poam-035.md`, this journal and generated Gate C navigation. No implementation was deleted or moved; this package changes diagnostic evidence, not the engine or product lifecycle.

## Local validation and remaining work

`python scripts/ci/test_native_capacity.py`: 29 tests passed, including simulated native-create failure, one verbose create attempt, case identity, bounded output, unrelated-output exclusion, invalid diagnostic directory, preserved original error, chained cleanup and policy mutations. `python scripts/ci/test_candidate_admission.py`: 11 tests passed. `python scripts/ci/check_workflow.py`: passed with all nine substantive jobs and required gate. `python -m py_compile scripts/ci/run_native_capacity.py scripts/ci/test_native_capacity.py scripts/ci/check_workflow.py`, navigation generation/check and documentation policy passed. These Windows local tests do not run `hdiutil`, mount APFS, or prove the actual cause. The first `rg` attempt used a Windows-incompatible wildcard and failed; rerunning against the directory with a glob worked. No native macOS preflight, candidate CI or merge-group CI has run for this draft.

The exact failure, its parent candidate and the previously observed separate metadata-fill timeout remain retained as their own CI evidence. Do not call diagnostics a fixture repair or update C-07, POAM-035 or alpha to complete. Next: run a bounded macOS native preflight against the exact candidate, inspect any retained diagnostic and cleanup outcome, fix the demonstrated underlying boundary, then run the full candidate and protected merge group on the final integrated revision. C-07 native capacity matrix and actual soak remain open.

## Change checklist

- [x] Bound the observed failure, owner, dependency, exact scope and no-waiver stop conditions in the PR before edits.
- [x] Inspected runner ownership, cleanup, CI gate and existing tests; retained the failure rather than claiming a diagnosis.
- [x] Tested bounded failure evidence, safe artifact scope, preserved original errors and mandatory CI wiring locally.
- [x] Run documentation and navigation checks; review the authored diff. Stage the final record and generate/check the Git-backed inventory before commit.
- [ ] Obtain native macOS evidence, fix any proven failure, integrate the dependency and pass candidate and protected merge-group CI. Roadmap and POAM status remain open.
