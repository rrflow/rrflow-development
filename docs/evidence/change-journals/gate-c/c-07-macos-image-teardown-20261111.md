# C-07: wait for macOS image teardown before the next capacity case

**Status:** active implementation evidence; POAM-074 stays open until macOS runs pass

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-macos-image-teardown-20261111`

**Owner:** [POAM-074](../../../poam/rrflow-1.0-alpha/poam-074.md)

**Starting revision:** `main` at `55d6c89f`.

**Owner direction, 2026-11-11.** Treat the actual reason for a CI failure; do not re-run it away.

**Current behavior.** Each native capacity case creates `capacity.dmg`, attaches it, and detaches it once `hdiutil info` no longer lists it. The next case's create can fail with "Resource busy" and leave a helper holding the rig's output, which the rig correctly treats as unretryable.

**Target behavior.** After each detach, the rig waits for the image's whole-disk device node to disappear, records the wait, and fails naming the device if it remains after 30 seconds.

**Changed paths.** `scripts/ci/run_native_capacity.py`, `scripts/ci/test_native_capacity.py`, POAM-074, this record.

**Failure oracle.** Four `DeviceRelease` tests: an absent node returns at once, a lingering node is waited for, a node that never goes fails naming it, and an unowned identity is refused.

**Acceptance.** On Windows: `python scripts/ci/test_native_capacity.py` ran 50 tests and passed with 5 skipped; `python scripts/ci/check_workflow.py` reported policy OK.

**Not established.** The cause is inferred from two CI logs, not reproduced on a Mac. Only macOS candidate runs can show whether teardown was the race.

**Research decision:** none. **Roadmap/POA&M change:** POAM-074 gains the second occurrence and the change; it stays open.
