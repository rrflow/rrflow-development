# POAM-069 — a first install is bounded by a fixed 30-second inventory limit

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-069`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-11, on a fresh clone of this repository (2,756 files, 42,149,935 bytes), `install plan` with a release binary on an otherwise idle Windows machine took 20.8 s and 16.0 s cold and 3.7 s warm, against the bundled profile's 30,000 ms `elapsed_ms`. A debug binary while compilers were running failed with `DeadlineExceeded` at 30.2 s. The same profile admits 1 GiB of content, which this limit makes unreachable. The cold cost has not been attributed to a component; the filesystem cache and antivirus scanning are unmeasured assumptions.

## Impact

At the measured cold rate a project of about 80 MB is refused on its first install while it would pass once warm, so the outcome depends on cache state rather than on the project.

## Owning gates

D-01 owns installed planning and project admission.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when the limit is derived from the admitted work or planning is resumable, a project at the content limit plans on a cold cache, and the cold cost is attributed by measurement.
