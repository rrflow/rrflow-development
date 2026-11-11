# POAM-074 — the macOS capacity rig fails when a new disk image is busy

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-074`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-11, candidate run 42954943269 failed `installed domain crash (macos-latest)` on a pull request that changes no engine code: `hdiutil: create failed - Resource busy`. Six capacity cases had already passed in that job and the previous image reported as detached.

The first version of this record blamed `scripts/ci/run_filesystem_containment.py`. That was wrong. The create is in `scripts/ci/run_native_capacity.py`, which since 2026-11-08 retries a busy create up to five times and removes a leftover image between attempts.

A second occurrence, run 43017327376 attempt 1 on PR #335, shows why that retry never ran. Case 21 failed 4 seconds after case 20's image detached, so the backoff of 1, 2, 4 and 8 seconds was never used. Its retained receipt has `stderr_open: true`, and the runner killed an orphaned `diskimages-helper` when the job ended. hdiutil starts that helper outside the command's owned process group, and the helper inherits hdiutil's stderr. After the busy create it kept that pipe open. The retry guard refused any create whose output was still open, because a live holder might still be using the image. So the one failure the retry was written for was the one it refused.

Why the image service is busy seconds after `hdiutil info` stops listing the previous image is inferred, not reproduced: teardown is likely still running.

## Remediation

Two changes, one to prevent the busy create and one to recover from it. See the [change journal](../../evidence/change-journals/gate-c/c-07-hdiutil-held-helper-20261111.md).

- **Prevent.** After each detach, the rig waits for the image's whole-disk device node to disappear before the next case creates an image. It logs the wait as `capacity_settle` and fails naming the device if the node remains after 30 seconds.
- **Recover.** The open-output record keeps its capture threads, so a caller can observe the moment the last holder closes the pipe. A busy create with held output detaches the leftover image, then retries only after the held output closes, waiting at most 20 seconds per attempt. If the holder never lets go, the original failure is raised, no retry runs, and the receipt records `holder_outlived_release`. Every retry logs one `capacity_create_retry` line. A timed-out create is still never retried.

## Impact

A required candidate check can fail for a reason unrelated to the change under test, which blocks the merge queue and teaches people to rerun failures.

## Owning gates

C-07 owns the native capacity campaign.

Roadmap owner: [Gate C](../../roadmap/rrflow-1.0/gate-c.md).

## Closure evidence

Closed when image creation tolerates a busy image service with a bounded, attributed retry, or the cause is shown to be elsewhere, and repeated macOS runs pass.
