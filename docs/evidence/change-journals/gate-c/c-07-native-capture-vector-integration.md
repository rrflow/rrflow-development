# Native capture diagnostic with vector candidate

**Status:** active immutable change-journal evidence; C-07 and POAM-035 remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-capture-vector-integration`
**Owner:** PR #215 source-bound macOS diagnostic integration.

## Evidence record

```text
gate/package and alpha outcome: C-07 / J-02 native diagnostic prerequisite; no gate closure.
starting revision/tree and pull request: PR #215 head 760f1705e41db00422e39f3bbfd67968e9e3cd0c normally merged PR #190 candidate b2f209712cb7c62f019e0c44d7fac3dcdad4d9e6; merge preview and merge reported no conflicts.
change brief: retain bounded command-output failure classification and the full vector candidate; test the existing APFS preflight on their combined source to reduce the prior source difference. No native case, timeout, gate or product behavior was deliberately changed.
files and symbols reviewed: prior #215 brief and journals; #190 CI job 124175892535 log; run_offline_lifecycle.run and CommandOutputNotClosed; run_native_capacity.retain_native_failure and tests; c07-native-preflight workflow; #190 integration journal and combined merge diff.
implementation/evidence paths: ordinary merge only plus this linked journal and generated index. All underlying implementation stays in its canonical location; no implementation is removed or replaced.
research/trace: existing bounded typed command category and stream state diagnose the same generic pipe failure; no secret output or new dependency. No root cause is claimed.
first observation: #190 candidate 41088508920 failed repeated APFS with command output did not close. Isolated #215 preflight 41090939263 completed all 24 cases and detached all images, so the original failure did not reproduce and no failure artifact identified a command.
local results: python scripts/ci/test_offline_lifecycle.py passed 18 and skipped 1 POSIX-only case on Windows; python scripts/ci/test_native_capacity.py passed 32; python scripts/ci/check_workflow.py passed; git diff --check passed. Regenerate and check documentation, navigation and inventory for this record.
surfaced failures and stop: absence of reproduction does not clear #190 or prove the cause. The separate Windows concurrent object-publication failure and its crash-durability question remain in PR #219 / POAM-047. Keep this PR draft.
checks not run: targeted combined-source macOS preflight, candidate CI, protected merge-group, native capacity/soak qualification and alpha acceptance. The targeted preflight is to be launched after the exact integrated revision is committed and pushed.
remaining errors and owner: #215 capture characterization, #219 object-publication repair, POAM-035 and POAM-047 remain open.
review: merge preview was clean; the diagnostic and vector parent revisions and their tests were preserved. Change checklist: brief before integration, exact source identities, focused and policy tests, linked journal; no status change.
pull request and merge evidence: https://github.com/rrflow/rrflow-development/pull/215; draft and dependent on #190; no candidate or merge-group success on the integrated head.
```

## Links

- [Native pipe diagnostic](c-07-native-output-capture-evidence.md)
- [E-04c integration](../gate-e/e-04c-current-main-integration.md)
