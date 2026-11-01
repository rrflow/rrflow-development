# C-07 native harness real-error repair

**Status:** active repair candidate; the cause of `Resource busy`, native macOS evidence and candidate CI remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-harness-real-errors`
**Owner:** C-07 / POAM-035 / [PR244](https://github.com/rrflow/rrflow-development/pull/244)

This package restores the real error. It does not fix why `hdiutil create` returns `Resource busy`, and `pipeline / installed domain crash (macos-latest)` can still fail.

## Evidence record

```text
gate/package and alpha outcome: OBJ-02 prerequisite under C-07 and POAM-035; the owner decided on 2026-11-01 to keep full CI and fix flaky tests test-first. No gate closure.
starting revision/tree and pull request: main c9241d09beabc3472e204e943797803e50189ea7, tree 13167551a552073a866f1a630d25d957f7e7da6c; PR244, driven by a Claude Code working session.
change brief: a non-zero owned native command whose output a descendant holds must surface its real status and text as NativeCommandFailure instead of losing them; a successful group command's still-finishing descendant must not be killed merely because the direct child exited. Unchanged: every hdiutil argument, the 25-second fill budget, retries (none), step and command deadlines, the 2-second output-close bound, closed metadata for product commands, and no kill of a helper a non-group command does not own.
files and symbols read in full: AGENTS.md; change-authoring.md; POAM-035 matches for this failure; c-07-native-output-capture-evidence.md; c-07-native-create-diagnostics.md; PR215 brief; scripts/ci/run_native_capacity.py; run_filesystem_containment.py; macos_native_failure_preflight.py and its test; test_offline_lifecycle.py; test_native_capacity.py to its first 240 lines; run_offline_lifecycle.py run, CommandOutputNotClosed and its isolation caller; run_sustained_capacity.py caller; check_workflow.py capacity policy; docs/operations/ci.md capacity sections; retained job logs.
implementation/evidence paths: changed scripts/ci/run_offline_lifecycle.py, scripts/ci/run_native_capacity.py, scripts/ci/test_offline_lifecycle.py, scripts/ci/test_native_capacity.py, docs/operations/ci.md, docs/poam/rrflow-1.0-alpha/poam-035.md; created this journal; regenerated docs/evidence/change-journals/gate-c/README.md and the execution inventory. Nothing moved or deleted.
research decision: no external code adapted. Python's BufferedReader.read1 is standard library behavior, measured here on both platforms. Darwin returning EPERM for a zombie-only process group is implementation knowledge, handled in the wait but not observed in CI or exercised here.
trace/resource/debug decision: no new trace. The create receipt gains two closed booleans. Added wait is at most 2 seconds, within the command's own deadline, and only when a clean group command leaves a live descendant.
first-failure oracle and result: commit e61ff02c added the tests before any harness change; they failed against the unchanged harness on Windows and Ubuntu WSL. See the failing output below.
acceptance commands and exact results: see Results below.
surfaced failures and corrections: see Surfaced failures below.
checks not run and reason: real macOS hdiutil, APFS or any hosted macOS job, because none is available here, so neither the macOS helper's behavior nor Darwin's process-group result is exercised; c07-native-preflight.yml, not authorized for this package; candidate and protected CI, pending ready state; scripts/ci/test_linux_native_sample.py, which needs a delegated --cgroup-parent and a Cargo build; scripts/ci/test_native_sample_worker.py, whose Windows release build was ended by a 280-second limit with `qualification command failed with exit status 143` before any test ran.
remaining known errors and owner: why hdiutil create returns Resource busy is unknown (POAM-035). A helper left by a failed non-group hdiutil create is still not cleaned up by the harness; PR215 records why that ownership is not established. The separate fill-budget class is untouched (POAM-035).
full-file reread and diff review: reread the complete diff; existing test lines are unchanged against main, so no assertion was weakened.
change checklist: brief in the pull request before the harness change; failing tests committed first; focused suites, then policy and navigation checks; linked journal. Owner records carry no status change.
roadmap/POA&M status change: none. POAM-035 gains one evidence note and stays open.
pull request, CI run, and merge evidence: https://github.com/rrflow/rrflow-development/pull/244; no candidate CI result is claimed by this record.
```

## Diagnosis checked against code and logs

| Claim | Result | Basis |
| --- | --- | --- |
| A held stream makes `run` raise before `native_command` sees the exit status | Verified | `run` raised at the reader check, ahead of `native_command`'s status test |
| The exit status is lost and `retain_native_failure` never fires | True only before #215 | #215 keeps status and stream flags in closed metadata and writes a receipt; text and the status in the log reason were still lost |
| The stderr text had been captured and was merely dropped | Refuted | A held stream's reader had captured nothing; see the reader failure below |
| A successful group command's descendant is killed | Verified | `run` signalled the group on every path; a 300 ms descendant was killed on WSL |
| Pipe failures are `hdiutil create` exiting non-zero | Not proven | No log records the command or status. No detach marker follows the failing case, which only narrows it to `create` or `attach` |
| `-verbose` moved one failure into the other | Hypothesis | All 10 retained `Resource busy` jobs ran heads without `5ab909b0`; all 5 pipe jobs ran heads with it |
| The timings fit one failure | Consistent only | Last detach to failure: pipe jobs 3.15 to 3.37 s; `Resource busy` jobs 1.15 to 1.30 s in 6 of 10, up to 3.29 s. One held stream adds the 2 s bound; two would add 4 s |
| A `diskimages-help` orphan accompanies the failure | Correlation only | 14 of those 15 jobs (all 5 pipe, 9 of 10 busy); none of 7 other retained macOS jobs |

The kill itself dates from `4e6042da` and enforces that no owned descendant outlives `run`: `docs/operations/ci.md` says an owned process group bounds cleanup, and `run_native_capacity.py` relies on owned process cleanup before detaching a volume. A survivor could keep an image busy or its output open, so success still ends with the kill. It now comes after a bounded wait rather than instead of one, so the requirement and the cleanup do not conflict. A descendant that outlives the 2-second bound is still killed, which is the one behavior a reviewer may want to revisit.

## Failing output before the harness change

Harness identical to `c9241d09`, tests at `e61ff02c`. Ubuntu WSL, Python 3.14.4:

```text
ERROR: test_failed_child_with_held_output_keeps_stderr_out_of_closed_metadata
    self.assertEqual(error.captured.returncode, 3)
AttributeError: 'CommandOutputNotClosed' object has no attribute 'captured'

FAIL: test_successful_group_command_lets_its_descendant_finish [holding stdout]
    self.assertEqual((result.returncode, result.stdout), (0, stdout))
AssertionError: Tuples differ: (0, '') != (0, 'finished\n')

FAIL: test_successful_group_command_lets_its_descendant_finish [detached]
    self.assertTrue(marker.exists(), "descendant was killed because its parent exited")
AssertionError: False is not true : descendant was killed because its parent exited

Ran 26 tests in 6.778s
FAILED (failures=2, errors=1)

ERROR: test_failed_create_with_held_output_retains_real_status_and_stderr (three subtests: stdout, stderr, both)
  File "scripts/ci/run_native_capacity.py", line 214, in volume
    native_command("hdiutil", "create", "-verbose", "-size", f"{size}m", "-fs", "APFS", "-type", "UDIF",
  File "scripts/ci/run_native_capacity.py", line 152, in native_command
    result = run(*arguments, check=False, **kwargs)
  File "scripts/ci/run_offline_lifecycle.py", line 348, in run
    raise CommandOutputNotClosed(category, child.returncode, *open_streams)
run_offline_lifecycle.CommandOutputNotClosed: command output did not close

ERROR: test_failed_attach_with_held_output_reports_status_and_keeps_closed_receipt
run_offline_lifecycle.CommandOutputNotClosed: command output did not close

ERROR: test_native_deadline_with_held_output_stays_bounded_and_reports_status
run_offline_lifecycle.CommandOutputNotClosed: command output did not close

ERROR: test_successful_native_command_with_held_output_remains_a_closed_failure
AttributeError: 'CommandOutputNotClosed' object has no attribute 'captured'

Ran 36 tests in 17.233s
FAILED (errors=6)
```

Windows, Python 3.11.9, gave the same native-capacity result (`Ran 36 tests in 17.584s`, `FAILED (errors=6)`) and `test_failed_child_with_held_output_keeps_stderr_out_of_closed_metadata` failing alone among offline tests (`Ran 26 tests in 2.561s`, `FAILED (errors=1, skipped=6)`), because the process-group cases are POSIX-only. The characterization tests for no descendant, a descendant outliving the bound, the deadline cap, and immediate kill after an exit status, a deadline or an output limit passed against the unchanged harness on WSL.

## Before and after at `native_command`

A real child exits with `hdiutil: create failed - Resource busy` while a descendant holds the named stream. WSL results; Windows differs only by process start time and carriage returns.

```text
                       before                                             after
no descendant          0.02s NativeCommandFailure rc=1, text              0.02s unchanged
holds stdout           2.02s CommandOutputNotClosed, no text              2.02s NativeCommandFailure rc=1, text, open=(True,False)
holds stderr           2.03s CommandOutputNotClosed, no text              2.02s NativeCommandFailure rc=1, text, open=(False,True)
holds both             4.02s CommandOutputNotClosed, no text              4.02s NativeCommandFailure rc=1, text, open=(True,True)
exit 0, holds stdout   2.02s CommandOutputNotClosed rc=0                  2.03s unchanged
group success, 300 ms descendant   0.01s, descendant killed               0.30s, descendant finished
group success, 30 s descendant     0.01s, killed                          2.02s, killed
```

The output-close bound is unchanged. The log reason now reads `native hdiutil failed (1): hdiutil: create failed - Resource busy; preceding failure: command output did not close`, so both symptoms appear on one line.

## Surfaced failures

**A held stream captured no text.** With only the status and type repaired, the stderr and both-held subtests still failed:

```text
AssertionError: 'Resource busy' not found in 'native hdiutil failed (1): '
```

`capture` called `BufferedReader.read(65536)`, which blocks until the buffer fills or EOF, so bytes already written to a held pipe never reached the destination. A direct probe on Windows and WSL reported `BufferedReader.read: reader_alive=True captured=b''` and `BufferedReader.read1: ... captured=b'hdiutil: create failed - Resource busy...'`. The reader now uses `read1`, and the tests also require text written to a held stdout. The brief was amended before the harness was pushed.

**My first wait raced an output-limit failure.** Repeating the new tests under load gave 23 passing and 1 failing suite run:

```text
FAIL: test_failed_group_command_loses_its_descendant_at_once [output limit]
AssertionError: True is not false : failed command's descendant was allowed to finish
```

The child could exit before the reader saw the overflow, and the wait checked for an overflow only when it started. The wait now ends as soon as the output failure is set, with a deterministic test. Two test deadlines that depended on child start time were widened from 2 and 3 seconds to 5.

**Also corrected:** a bulk `sed` meant for the new tests removed `create=True` from an existing `geteuid` patch. It was restored before any commit carrying it, and the diff against main shows no existing test line changed.

## Results

Final harness and tests, Windows Python 3.11.9 and Ubuntu WSL Python 3.14.4:

```text
py -3 scripts/ci/test_offline_lifecycle.py        Ran 27 tests, OK (skipped=7, POSIX process-group and inherited-pipe cases)
py -3 scripts/ci/test_native_capacity.py          Ran 36 tests, OK
python3 scripts/ci/test_offline_lifecycle.py      Ran 27 tests, OK (WSL)
python3 scripts/ci/test_native_capacity.py        Ran 36 tests, OK (WSL)
test_macos_native_failure_preflight.py            Ran 3 tests, OK (both)
test_sustained_launch.py                          Ran 3 tests, OK (both)
test_collect_storage_acceptance.py                Ran 5 tests, OK (both)
test_inspect_candidate_runs.py                    Ran 9 tests, OK (both)
render_navigation.py --check                      navigation: OK: 18 generated indexes, 917 nodes, 3287 edges, 0 index files updated
build_execution_inventory.py --check              execution-inventory: OK: 2303 current, generated, and planned path records
check_documentation.py                            documentation-policy: OK
check_workflow.py                                 CI policy OK: 5 workflows, 9 substantive jobs
check_version.py                                  version-policy: OK: RRFlow release train is 1.0.0
git diff --check                                  clean
```

Against the baseline at `c9241d09` (20 and 32 tests on each platform), every existing test still passes and no existing test line changed.

The new real-process tests were repeated to look for timing flakes. After the wait correction, WSL passed 48 of 48 suite runs (12 rounds of 4 concurrent runs with a CPU spinner on each of 24 cores), and Windows passed 12 of 12 runs of the cross-platform cases (4 rounds of 3). Nine deliberate mutations of the new behavior were each caught by a named test, including removing the output-failure stop, giving failed exits the wait, never killing on success, ignoring the command deadline, restoring `read()` and rendering captured text in the error message. No holder process remained after the suites.

## Links

- [C-07 ordered plan](../../../roadmap/rrflow-1.0/gate-c-07.md)
- [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)
- [Native output-capture failure evidence](c-07-native-output-capture-evidence.md)
- [Native image creation diagnostics](c-07-native-create-diagnostics.md)
