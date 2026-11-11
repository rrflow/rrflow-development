# C-07: one `hdiutil create` per suite, a fresh copy per case

**Status:** active change-journal evidence; a CI harness repair, not capacity or release evidence

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-hdiutil-template-20261111`

**Owner:** C-07 candidate CI, [POAM-074](../../../poam/rrflow-1.0-alpha/poam-074.md); follows [the busy-create retry](c-07-hdiutil-busy-retry-20261109.md)

**Starting revision:** `development/main` at `abc29393`.

**Owner direction, 2026-11-11.** Fix the cause of the macOS failure blocking #349 rather than re-running it. The owner approved this approach: stop creating a disk image per case.

## Current behavior

`volume()` in `scripts/ci/run_native_capacity.py` runs `hdiutil create` for every capacity case. `hdiutil create` attaches, formats and detaches the new image internally. On macOS runners that cycle intermittently ends `create failed - Resource busy`. That happened in candidate run 42954943269 and again in the merge queue for #349 (run 43083711389, job 130742748923), after the bounded retry from #301 was already in place. Neither pull request changes engine code.

## Target behavior

- `darwin_template()` runs `hdiutil create` once per suite and keeps the same bounded, attributed retry and fixture-owned leftover cleanup. It produces `capacity-template.dmg`, which the rig never attaches.
- Each case copies the template to `capacity.dmg`, then attaches, verifies, fills and detaches that copy exactly as before. Every case therefore still starts from a new, never-written APFS filesystem.
- `main()` removes the template after a complete run, after the same fixture-ownership checks as the other owned files. A failed run leaves it in place for review, like the other owned artifacts.
- Linux and Windows are unchanged.

## Files read in full

- `scripts/ci/run_native_capacity.py`
- `scripts/ci/test_native_capacity.py`
- `scripts/ci/run_filesystem_containment.py`
- the `run` and `require` parts of `scripts/ci/run_offline_lifecycle.py`
- POAM-074
- the busy-create retry journal

## Changed paths

- `scripts/ci/run_native_capacity.py`: `DARWIN_TEMPLATE`, `darwin_template()`, the darwin branch of `volume()`, and template removal in `main()`.
- `scripts/ci/test_native_capacity.py`:
  - New: `test_macos_cases_create_one_template_and_copy_it_for_each_case`. Over two cases plus the per-case cleanup it asserts exactly one create, aimed at the template, that each attach reads an unwritten copy, and that the template survives each case's cleanup.
  - The leftover-image and until-success tests now target the template, and the fake create writes the image it reports.
- POAM-074 and this record.

## Commands and results

- `python -m pytest -q scripts/ci/test_native_capacity.py`: 41 passed, 6 skipped. The skipped tests need POSIX process groups or symlinks; this host is Windows.
- Two failures were fixed during authoring, and both were test fakes, not runner code:
  - The faked `os.stat` also answered pathlib's `exists()` and `is_dir()` during cleanup. It now applies only while a volume is attached.
  - The fake detach left the case's owner marker in the mount directory. A real detach takes it away, and the fake now does too.

- `gh workflow run c07-native-preflight.yml --ref agent/macos-capacity-template-20261111 -f point=all -f mode=both -f repetitions=3`, run 43105134014 on `macos-latest` at `c2cc5baa`: the job succeeded.
  - All 24 APFS cases passed, each followed by a detached cleanup, in one fixture, so every case attached a copy of one template.
  - No `Resource busy` appears in the log.
  - The suite reported `native_capacity_suite` status `passed`.

## Not established

- One macOS run is not closure. The busy failure was intermittent, and POAM-074 needs repeated passes, including in candidate CI.
- This removes the per-case create. It does not prove the busy failure cannot hit the one remaining create. Closure still needs repeated macOS runs, per POAM-074.
- The extra cost per case is copying a 256 MB read/write image; it has not been measured on a runner.

**Research decision:** none. **Trace/debug decision:** the existing attributed native-command failure output is unchanged. **Roadmap/POA&M change:** POAM-074 records the recurrence and this remediation; its status is unchanged.
