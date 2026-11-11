# POAM-074 — the macOS capacity rig fails when a new disk image is busy

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-074`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-11, candidate run 42954943269 failed `installed domain crash (macos-latest)` on a pull request that changes no engine code: `hdiutil: create failed - Resource busy`. Six capacity cases had already passed in that job and the previous image reported as detached. `scripts/ci/run_filesystem_containment.py` retries `hdiutil detach` but calls `hdiutil create` once. This is one observed occurrence.

Later on 2026-11-11 it recurred in the merge queue for PR #349, which also changes no engine code: run 43083711389, job 130742748923, `hdiutil: create failed - Resource busy`. By then `scripts/ci/run_native_capacity.py` already retried a finished create with backoff and removed a leftover image between attempts ([the retry journal](../../evidence/change-journals/gate-c/c-07-hdiutil-busy-retry-20261109.md)); that retry did not recover the case. `hdiutil create` attaches, formats and detaches the new image itself, so every case ran one more attach cycle than it needed.

## Remediation in progress

[The template journal](../../evidence/change-journals/gate-c/c-07-hdiutil-template-20261111.md): `hdiutil create` now runs once per suite, for a pristine template the rig never attaches. Each case copies the template, then attaches, verifies and detaches the copy as before, so it still starts from a new, never-written APFS filesystem. The suite removes the template at the end. This removes the per-case create. It does not prove the busy failure cannot hit the one remaining create, which keeps its bounded retry.

## Impact

A required candidate check can fail for a reason unrelated to the change under test, which blocks the merge queue and teaches people to rerun failures.

## Owning gates

C-07 owns the native capacity campaign.

Roadmap owner: [Gate C](../../roadmap/rrflow-1.0/gate-c.md).

## Closure evidence

Closed when image creation tolerates a busy image service with a bounded, attributed retry, or the cause is shown to be elsewhere, and repeated macOS runs pass.
