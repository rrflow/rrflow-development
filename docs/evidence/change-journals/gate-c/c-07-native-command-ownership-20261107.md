# C-07 native capacity command ownership investigation

**Status:** active evidence; repair and native validation pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-command-ownership-20261107`
**Owner:** [C-07](../../../roadmap/rrflow-1.0/gate-c.md#c-07-implementation-packages), [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)

## Baseline and evidence

Started at public main `838c85120621d5d9dde989d7677cc6ec89d20f03`. The
reviewed main blobs were `5612e482f046bf981f8f731d2a66f3e14c268c61`
(`scripts/ci/run_native_capacity.py`),
`a98544f0a8a1a15e3e88ece6e49f3ec176740fd4`
(`scripts/ci/run_offline_lifecycle.py`) and
`8315f8b72db268a0f979505e68b46dc76abc5fd2`
(`scripts/ci/test_native_capacity.py`). Reviewed the full capacity runner,
the process runner and native detach implementation, relevant native-capacity
and process tests, README, C-07 gate, POAM-035 and CI admission.

Candidate [run 42400526130](https://github.com/rrflow/rrflow-development/actions/runs/42400526130)
at PR #294 head `3891f32ffd64ff3c7f231a0ab81cc3cc96eda7a3` failed on
macOS owned-image create at existing-mode case 24. Its retained diagnostic
artifact ID `13586041057` recorded native return 1, `Resource busy`,
`stdout_open=false`, `stderr_open=true` and cleanup state unknown. An
independent preflight [42402399533](https://github.com/rrflow/rrflow-development/actions/runs/42402399533)
passed 24 APFS cases; it did not explain the failure. Protected merge-group
[run 42403138769](https://github.com/rrflow/rrflow-development/actions/runs/42403138769)
at group commit `f4224661a1206795e7539a3ab775940ebb49293b` then failed on
another macOS runner at existing-mode case 23, again with return 1,
`Resource busy`, `stdout_open=false`, `stderr_open=true` and cleanup state
unknown. Its diagnostic artifact is `13586613978`; the locally retained raw
job log SHA-256 is `731e30e575b23bf09e52b28cd7e144244ebe1a313898afba85dd7ad2e1db9d79`.
Neither observation identifies the stderr holder or proves what caused the
native image failure. The [GitHub runner-image issue](https://github.com/actions/runner-images/issues/7522)
describes the same 49168 symptom elsewhere, not its cause here.

## Reproduced process ownership gap

`native_command` calls the common runner without `process_group=True`; it
reaps the immediate child but does not own its descendants. The detach helper
already uses a separate owned process group. In WSL Ubuntu a controlled
`held_output_command` was substituted for `hdiutil create`: its parent
returned 1, its descendant kept stderr open, `native_command` returned
`NativeCommandFailure` with `stderr_open=true`, and `os.kill(pid, 0)` showed
the test-owned descendant still alive. The probe then terminated that exact
descendant. This is a Linux synthetic process-ownership result, not an APFS
qualification or evidence that the actual macOS pipe holder was a descendant.

## Change brief and checks

Prerequisite for C-07 and candidate CI: admit native capacity commands under
an owned POSIX process group, terminate that group on failure or
deadline and reap the direct child while preserving bounded output, native failure status and exact
image identity checks. A successful parent and legitimate descendant must
have the existing bounded completion interval. No native case is skipped,
no product state or error is hidden, and no retry is accepted as a proof.
Scope: the capacity wrapper and its tests, the shared runner only if a real
boundary defect requires it, plus this journal and generated navigation.

The failing oracle is a test-owned stderr holder that outlives a failed
command; after repair, that holder must no longer be running and the native
error must be retained. An orphaned zombie awaits its operating-system reaper. Cover successful and failed parents, timeout, both output pipes,
retained diagnostics and owned cleanup. Run focused Python tests on Windows
and Linux, a real macOS preflight with the exact candidate and the existing
24-case repeated native suite, then required candidate and merge-group CI.
A passing Linux model is not native macOS evidence. Stop if process groups
cannot safely bind only owned children, native failure or cleanup becomes
uncertain, or an implementation requires changing the capacity oracle.

Before implementation, the PR description must bind this brief and actual
source scope. Record commands, results, full-file/diff review and unrun
checks below. No roadmap status changed. C-07 and POAM-035 remain open.

## Implementation and local validation

The red WSL test `python3 -m unittest
 test_native_capacity.NativeCapacityCases.test_failed_native_command_does_not_leave_its_stderr_holder_running -v`
failed on main because the test-owned holder remained running. The capacity
wrapper now opts POSIX native commands into the shared runner's owned process
group and strict completion: a successful parent cannot hide an unfinished
owned descendant, failure and deadline terminate the group, and a pipe holder
that escapes the group remains an explicit output-close failure and is not
killed as an owned child. Deadline output capture is separately opt-in and
retains a typed timeout with bounded native output; other callers keep the
previous timeout exception and completion callback behavior. Failed attach
receipts retain bounded closed metadata without copying its native output.
No image identity, native case, capacity result or retry policy changed.

The first combined Linux run had three failures and two errors: its existing
fixture expected held pipes after group cleanup and expected the old timeout
shape; failed attach diagnostics also needed a closed receipt after the
owned pipe closed. Those were resolved by preserving the native failure and
timeout classification and testing the changed cleanup outcome, not by
accepting failed native cases. Final source tree
`ee89f27bc87e862f36c6af0b53d05c9f62e5a6e4` passed
`python3 -m unittest test_native_capacity test_offline_lifecycle
 test_macos_native_failure_preflight` in WSL Ubuntu: 71 tests, no skips.
The same command under Windows `python` passed 71 tests with 11 POSIX tests
skipped. Tests include a silent surviving group member, a descendant finishing
within the bound, a timed-out process and an escaped stderr holder. They are
controlled subprocess fixtures, not APFS results. Real macOS preflight,
repeated native cases, candidate CI and protected merge remain unrun for this
source. Process groups cannot contain a descendant that deliberately leaves
the group; the wrapper reports unclosed output and does not kill that process.
The source review and final PR checks must be completed before readiness.

The existing [native harness error record](c-07-native-harness-real-errors.md)
already reports many earlier macOS failures and a correlation with hosted-runner
cleanup of `diskimages-helper`; correlation did not identify the pipe holder
or prove the cause of `Resource busy`. The two newer runs add observations, not
a root-cause verdict.

Full-file reread covered the three changed Python files, this journal, the
POAM-035 record and the generated Gate C journal index; the complete diff was
reviewed and the generated index checked against its renderer. The local
Windows and WSL tests above do not exercise macOS `hdiutil`, and no native
preflight, candidate or merge-group result exists for this changed source yet.

The first `python scripts/ci/check_documentation.py` attempt failed because
the evidence status lacked the required active/historical marker and its
coordinate omitted the canonical data path. Those two fields were corrected;
the navigation index must be regenerated and the check rerun.

The corrected documentation policy passed; `git diff --cached --check` then
found trailing metadata whitespace, which was removed before the final check.
