# C-07 macOS mount fixture pending-file correction

**Status:** active implementation candidate; native macOS and exact candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07f-macos-containment-pending`
**Owner:** OBJ-02 / C-07f / D-01 / POAM-035 / [PR170](https://github.com/rrflow/rrflow-development/pull/170)

Driver: this session. Starting candidate `4e3d25e2bea6ae96a932317aa336e60af0e86990`. Before editing, PR170 recorded that the real `mounted_volume_crossing_is_denied_before_any_effect` creates a host-side `project/pending` file and that the initial cleanup would reject it. The linked prior journal retains the original native EBUSY failure and first fix unchanged.

The new `test_known_native_pending_file_is_cleaned_after_mountpoint` first failed with `unexpected project entries; retained`. Cleanup now allows only that named regular, non-symlink file and unlinks it only after non-recursive mountpoint removal succeeds. A busy mount retains the pending file and image; a nonregular pending entry is rejected and retained. The mounted directory contents are never enumerated or recursively removed. The existing test, image detachment, all resource bounds and gate policies are unchanged. No native macOS success is implied.

`python scripts/ci/test_native_capacity.py` passed 23 tests, including nine mount-fixture cases. `python scripts/ci/test_offline_lifecycle.py` passed 17 and `python -m py_compile scripts/ci/run_filesystem_containment.py scripts/ci/test_native_capacity.py` passed. The real macOS adversary and exact full candidate CI remain required. Read in full: native mounted-volume test's relevant function, the two changed Python files' relevant functions and prior journal; owner and repository procedure as recorded in the parent. No upstream source was copied; no new research algorithm or native trace was introduced. The failure and its correction are retained. Local checks ran on the host of the active native campaign, so that campaign's timing is not isolated.

Documentation, navigation, workflow, staged diff and inventory checks are required before push. No objective, roadmap or POAM completion status changes. Checklist: scope and oracle recorded before edit; failed first test retained; exact known path guarded and mount protected; helper suites passed; generated indexes and diff checked; native macOS, exact CI and merge remain pending.
