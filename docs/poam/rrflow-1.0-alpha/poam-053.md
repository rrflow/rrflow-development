# POAM-053 — engram rename candidate rejects a previously installed project

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-053`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This is a candidate-only regression observed in [PR #290](https://github.com/rrflow/rrflow-development/pull/290), not a finding that public main already has this defect. The parent ledger owns priority and lifecycle status.

## Observed deficiency

On Windows, the pre-rename engine at `f45305394dbe391786c7d618645a5fc0b1b2bbee` installed and served a new temporary project. The locally built renamed candidate rejected that project's locator before startup: the stored `estate_id` field was unknown to its `engram_id` decoder. The [repair journal](../../evidence/change-journals/gate-h/engram-connectome-repair-2026-11-07.md) records executable digests, source limitations and the retained probe path. The test did not reach executable identity, subsequent digest validation, stored keys or recovery; their compatibility is unknown.

## Impact

Landing the rename without resolving this boundary could make existing installed projects unavailable. Renaming fields in a generated fixture or making a test-only projection does not repair installed data. User projects must not be deleted or silently rewritten to make the candidate pass.

## Owning gates and closure evidence

D-01 installation and recovery, A-07 public contracts, H-04 client integration and the installed restart/replay evidence in the roadmap. Review the full persisted identity, digest, key, permissions and executable-binding changes; implement one authorized canonical convergence path that preserves project-owned data and remains safe across interruption; prove real pre-rename installation, readback, interruption, restart and post-change reopen on supported platforms. Verify clients and generated contracts against the accepted semantics. Candidate and merge-group CI remain required.
