# POAM-070 — a Git worktree or linked checkout cannot be installed

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-070`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-11 `install plan` on a Git linked worktree failed with `inventory Git input: external or non-directory Git metadata requires admission`, because its `.git` entry is a file. There is no admission path for it.

## Impact

Linked worktrees are how parallel work on one repository is normally done, and the same `.git` file form is used by submodule checkouts, so those projects cannot use the product path.

## Owning gates

D-01 owns installed planning and project admission.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when a worktree and a submodule checkout either install through an explicit admitted path or are refused with an error naming the supported alternative, with a test for each.
