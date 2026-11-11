# C-07: wait for macOS image teardown, and retry a busy create once its helper lets go

**Status:** active remediation evidence; POAM-074 remains open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-hdiutil-held-helper-20261111`

**Owner:** C-07 in [Gate C](../../../roadmap/rrflow-1.0/gate-c.md); [POAM-074](../../../poam/rrflow-1.0-alpha/poam-074.md)

**Starting revision:** `development/main` at `55d6c89f`.

**Current behavior.** `volume()` in `scripts/ci/run_native_capacity.py` retries a busy `hdiutil create`, but only one whose output closed. On macOS runners the busy create leaves `diskimages-helper` holding hdiutil's stderr from outside the owned process group. That makes the failure look unfinished, so the retry refuses it and the job fails on its first attempt.

Run 43017327376 attempt 1 is the evidence. Case 21 failed 4 seconds after the previous detach. Its receipt has `stderr_open: true` and `stdout_open: false`, the native text is `hdiutil: create failed - Resource busy` after `Finishing`, and the runner killed an orphaned `diskimages-helper` when the job ended.

**Target behavior.** Two parts. To prevent the busy create, the rig waits after each detach for the image's whole-disk device node to disappear. It logs the wait as `capacity_settle` and fails naming the device if the node remains after 30 seconds. This part is commit `bcac6931`, written in a parallel P3 session and carried here so one package and one pull request own POAM-074.

To recover when it still happens, a busy create whose output a holder still keeps open is released first: the leftover image is detached with the existing owned-image check. The create is retried only after the held output is seen to close, within `HELPER_EXIT_SECONDS` (20). A holder that outlives that bound gets no retry. The original failure is raised and its receipt records `holder_outlived_release: true`. Timed-out creates are still never retried. Each retry prints `capacity_create_retry` with the attempt, whether a leftover image existed, whether output was held, and how long it took to close.

**Changed paths.**

- `scripts/ci/run_offline_lifecycle.py`: `CommandOutputNotClosed` keeps its capture threads and gains `wait_closed`. Without the threads, `wait_closed` always answers no.
- `scripts/ci/run_native_capacity.py`: `wait_for_device_release` after each macOS detach. `is_busy_darwin_create` and `held_output` replace `is_retryable_darwin_create_failure`. The create retry loop changes, and the receipt gains `holder_outlived_release`.
- The tests in `scripts/ci/test_offline_lifecycle.py` and `scripts/ci/test_native_capacity.py`.
- [POAM-074](../../../poam/rrflow-1.0-alpha/poam-074.md), whose wrong file attribution is corrected.
- This record.

**Failure oracles.**

- `test_held_output_is_reported_closed_only_after_its_holder_exits` runs a real child that exits while a descendant holds stderr. The short wait must answer no while the holder lives and yes once it is ended.
- `test_macos_volume_retries_a_busy_create_after_its_helper_lets_go` checks the order: create, detach the half-made image, then a create that starts from no image after the holder let go. It also checks that one retry line is printed.
- `test_macos_volume_keeps_a_busy_create_whose_helper_never_lets_go` checks that there is no retry and no sleep, that the receipt has `holder_outlived_release`, and that the image is kept.
- Four `DeviceRelease` tests: an absent node returns at once, a lingering node is waited for, a node that never goes fails naming it, and an unowned identity is refused.
- Against main's code the three held-output tests fail. That failure is a missing interface (`readers`, `wait_closed`), which is weaker evidence than a failing assertion. `test_macos_volume_does_not_retry_an_unfinished_busy_create` is unchanged and still passes, because a record without capture threads proves nothing.

**Acceptance.** On Windows, `python -m pytest -q test_native_capacity.py test_offline_lifecycle.py` in `scripts/ci`: 64 passed, 12 skipped. The skipped tests need POSIX process groups.

**macOS preflight, run 43030750207 on `4eaec4d2` (the retry alone).** Passed: 24 real APFS cases, every point in both modes three times. No real case hit a busy create, so the macOS run did not exercise the retry path. The retry lines in that job's log came from the unit-test step, where simulated cases printed them; the tests now capture their own output so a real job's log holds only real retries.

**macOS preflight, run 43032802682 on `70e4170f` (both parts).** Passed: the same 24 real APFS cases. Every `capacity_settle` wait was 0 ms, because the device node was already gone when the detach returned, and no create was busy. Neither preflight reproduced the failure, so neither part has been seen to act on a real busy create.

**Not established.**

- Nothing has run on macOS yet. Whether the real helper exits within 20 seconds of the detach is the open question. Candidate CI and the C-07 macOS preflight with repetitions are the evidence, and their results belong in the pull request.
- Why the image service is busy 4 seconds after a verified detach is not known. Both preflights saw the device node gone at once, which weakens the teardown explanation. The retry is what handles the observed failure; the wait only rules teardown in or out when the next occurrence is logged.

**Research decision:** none. **Trace decision:** a `capacity_create_retry` line on every retry, so the frequency and release time can be counted from CI logs. **Roadmap/POA&M change:** POAM-074's evidence is corrected and a remediation section is added; it stays open until repeated macOS runs pass.
