# POAM-074 — the macOS capacity rig fails when a new disk image is busy

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-074`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-11, candidate run 42954943269 failed `installed domain crash (macos-latest)` on a pull request that changes no engine code: `hdiutil: create failed - Resource busy`. Six capacity cases had already passed in that job and the previous image reported as detached. `scripts/ci/run_filesystem_containment.py` retries `hdiutil detach` but calls `hdiutil create` once. This is one observed occurrence.

A second occurrence, on 2026-11-11 in run 43017327376 (attempt 1, #335), shows why retries did not help. The busy create left its output open (`preceding failure: command output did not close`), so the rig correctly refused to retry over a possibly live helper. Earlier cases in that job had each logged a verified detach.

The rig now waits, after each detach, for the image's whole-disk device node to disappear before the next case creates an image. It records the wait as `capacity_settle`, and fails naming the device if it remains after 30 seconds. This addresses the likeliest cause, teardown still running after `hdiutil info` stops listing the image; macOS runs must confirm it.

## Impact

A required candidate check can fail for a reason unrelated to the change under test, which blocks the merge queue and teaches people to rerun failures.

## Owning gates

C-07 owns the native capacity campaign.

Roadmap owner: [Gate C](../../roadmap/rrflow-1.0/gate-c.md).

## Closure evidence

Closed when image creation tolerates a busy image service with a bounded, attributed retry, or the cause is shown to be elsewhere, and repeated macOS runs pass.
