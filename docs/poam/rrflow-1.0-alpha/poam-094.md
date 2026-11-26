# POAM-094 — a same-size rewrite inside one timestamp step is not seen as drift

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-094`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Read on 2026-11-25 at `7f16ed4be`. Inventory decides whether a Git input changed by comparing two `SourceObservation` values (`crates/storage/rrflow-lsm/src/filesystem/source.rs`): native object identity, kind, byte length, modified time, changed time, mode, owner, group and link count. Content is not compared. `GitInputs::read_optional_with_after_open`, `revalidate` and `revalidate_selected` in `crates/authority/rrd-engine/src/engine/installation/inventory/git_inputs.rs` all use that comparison.

A rewrite that keeps the file's length and lands in the same timestamp value as the previous write produces an equal observation. Measured on a Windows 11 host whose file timestamps step about every 1 ms: a temporary probe drove `read_optional_after_observation` 400 times with a `.gitignore` rewritten to the same length immediately before and during the observation, and 400 times with a rewrite of a different length. The same-length rewrite was accepted without an error 144 times; the different-length rewrite was refused every time. The probe was not committed because its result depends on timing.

This turned `main` red on 2026-11-25 (nightly run 45549680197, `engram process (windows-latest)`): `selected_git_read_observation_drift_is_retryable` rewrote `.gitignore` twice with ten-byte contents and the second rewrite was not seen ([journal](../../evidence/change-journals/repository/git-input-drift-test-20261125.md)). The test now changes the length on each rewrite.

Two windows differ in consequence:

- **Between the first observation and the read.** The bytes that are read are the bytes whose digest is recorded, so the recorded identity matches the content used. No stale content is admitted.
- **After the read.** `revalidate` and `revalidate_selected` compare observations only. A same-length rewrite in the same timestamp value after the read is accepted as unchanged, and the walk continues on content that no longer matches the recorded digest. This follows from the same comparison; it was not separately measured.

## Impact

A Git input (`.gitignore`, `.git/info/exclude`, `.git/config`, the index) changed in that way during an inventory walk can leave the published inventory bound to ignore rules that are no longer on disk. The window is one timestamp step wide and needs an equal-length rewrite. Other source files observed through `SourceObservation` share the comparison and were not examined here.

## Owning gates

D-02 owns attunement inventory and its source binding.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when revalidation of a Git input compares the content digest already recorded for it, or treats an observation whose modified time is not older than the comparison clock by more than the host's timestamp step as requiring a content comparison, and a deterministic test rewrites an input to the same length without advancing its timestamps and is refused.
