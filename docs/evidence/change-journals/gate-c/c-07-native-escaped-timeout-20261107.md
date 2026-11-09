# C-07 native command escaped-pipe timeout correction

**Status:** active evidence; native candidate validation pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-escaped-timeout-20261107`
**Owner:** [C-07](../../../roadmap/rrflow-1.0/gate-c.md#c-07-implementation-packages), [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)

This corrects the same bounded [native command ownership package](c-07-native-command-ownership-20261107.md)
in [PR #297](https://github.com/rrflow/rrflow-development/pull/297), starting
from its pushed source `6fd27c402c8ee0fb1b1e80337cf1fabdf59fb88f`. It changes
only the existing output-close error, capacity wrapper and its tests. The
controlled failure oracle starts a test-owned descendant in a new session,
leaves its stderr pipe open, and times out the parent. On the starting source,
`python3 -m unittest
 test_native_capacity.NativeCapacityCases.test_timeout_with_escaped_pipe_holder_retains_deadline_classification -v`
failed because the native failure incorrectly reported `timed_out=false`.
The test finally terminates the exact escaped test helper; the wrapper does
not claim it owns or kills that escaped process.

The shared runner now retains the timeout flag in the content-free
`CommandOutputNotClosed`; the existing capacity adapter passes it into its
bounded native failure receipt. The comments now distinguish group members
from processes that escape the group. The original failure and native command
output remain failures; no retry, case reduction, product policy or image
cleanup bypass was introduced. The focused WSL test passed after correction.

With no unstaged source changes, staged tree
`bd867c65fbd311477929bd826666d5ae22d07c92` passed
`python3 -m unittest test_native_capacity test_offline_lifecycle
 test_macos_native_failure_preflight` on WSL Ubuntu: 72 tests, no skips.
The same suites passed on Windows: 72 tests, 12 POSIX-only skips. These are
controlled process fixtures, not APFS qualification. Preflight
[42405376563](https://github.com/rrflow/rrflow-development/actions/runs/42405376563)
was launched on the earlier `6fd27c40` revision and cannot be claimed as
evidence for this source. Exact corrected-source macOS preflight, the full
candidate and protected merge remain pending. Source review covered the
three affected Python files and their full diff. No C-07 or POAM status changed.


## Corrected-source macOS preflight

[Run 42406185131](https://github.com/rrflow/rrflow-development/actions/runs/42406185131)
completed successfully for exact source `d0cf12b6077de93c8ad5aeefef880ab4eabc9a94`.
Its native-preflight job `128504277010` recorded 24 successful installed native
case markers and 24 owned-image detachment markers. The real injected native
create-failure capture step passed and retained artifact `13588020289`. The
full downloaded job log SHA-256 was
`059c3cc7960399a49c5b5ba5f5ee2a6a100ae78df6d617d42c5a98baf1f9ffe5`.
This is one hosted macOS preflight, not proof of the earlier Resource busy
cause, candidate or protected merge CI, the native capacity matrix, or C-07
soak qualification. No roadmap or POAM status changed. The recorded source
revision precedes this evidence-only update; the required candidate CI must
run against the final submitted head.
