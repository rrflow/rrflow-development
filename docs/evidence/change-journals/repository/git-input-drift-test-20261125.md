# The Git input drift test no longer depends on the file clock

**Status:** active repair evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/git-input-drift-test-20261125`
**Owner:** [CI and runner operations](../../../operations/ci.md) and [Gate D, D-02](../../../roadmap/rrflow-1.0/gate-d.md); alarm [issue 505](https://github.com/rrflow/rrflow-development/issues/505); deficiency [POAM-094](../../../poam/rrflow-1.0-alpha/poam-094.md).

## Change brief

Baseline `7f16ed4be`. The nightly full matrix on `68e34e83c` (run 45549680197) failed only in `engram process (windows-latest)`: `engine::installation::inventory::git_inputs::tests::selected_git_read_observation_drift_is_retryable` panicked at its second `unwrap_err()` with `Ok(Some("alpha.bak\n"))`. The same job passed in the previous nightly, and the one commit between the two runs changes a Linux-only test file.

Current behavior: the test rewrites `.gitignore` during an observation and expects a drift error. Both rewrites are ten bytes, the same length as the original. Target behavior: the test fails only when the engine maps an observed drift to the wrong error. Scope: the test's two rewrite contents, a new POA&M entry and this journal. No engine code changes. Failure oracle: the test, run repeatedly, and the inventory unit suite. Stop if the repair would need to remove an assertion or change what the engine detects.

Files read: `git_inputs.rs`, `rrd-lsm/src/filesystem/source.rs`, the failed job log.

## Findings

The engine compares file metadata, not content. A rewrite of equal length in the same timestamp value is invisible to it.

- A script wrote a ten-byte file twice in succession 2,000 times on this Windows host; size and modified time were identical after 1,846 of them. The host's file timestamps step about every 1 ms.
- The unchanged test passed 150 of 150 local runs. The work between its two rewrites takes longer than 1 ms here, which fits a hosted runner with a coarser step failing where this host does not; the runner's step was not measured.
- A temporary probe in the test module called `read_optional_after_observation` 400 times with an equal-length rewrite and 400 times with a longer one. The equal-length rewrite was accepted 144 times; the longer one was refused 400 times. The probe was removed because its result depends on timing.

The failure is therefore the test's equal-length contents meeting a real limit of the detection. The limit is recorded as POAM-094 and is not repaired here: the owner ruled on 2026-11-24 that no new engine code is written until the existing code is aligned.

## Change

The first rewrite is now eight bytes (`alpha.t\n`) and the second ten (`alpha.bak\n`), so each rewrite changes the length the engine observes. Both assertions are unchanged.

## Verification

On Windows at the baseline plus this change:

- The repaired test, run 150 times from the built test executable: 150 passed.
- `engine::installation::inventory` unit tests: 20 passed.
- `cargo fmt -p rrd-engine -- --check`: clean.
- `python scripts/ci/check_documentation.py`: passed.
- Not run: the hosted Windows lane (the nightly is its proof), the full engine suite, Linux or macOS.

POA&M: POAM-094 added, open. No objective or roadmap status changes.
