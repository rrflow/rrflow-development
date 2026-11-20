# rrflowLSM range scans seek to the range and stop at its end

**Status:** active change-journal evidence; a measured improvement that does not close the gap to Fjall

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/rrflowlsm-range-scan-seek-20261120`

**Owner:** [rrflowLSM against Fjall](../../../reference/storage/rrflowlsm-fjall-parity.md), step 1 of its order; continues [the first disk-backed comparison](rrflowkv-vs-fjall-comparison-20261120.md)

**Starting revision:** main `ce10aedcb`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage; [kernel contract](../../../roadmap/kernel-contract.md) K10.
- **Current behaviour.** A bounded range read over flushed data opened each overlapping row group at row 0, examined every row to the end of the group, and went on testing later group descriptors. A 16-row read examined 1,519 versions.
- **Target behaviour.** The scan opens a row group at the start of the next range, jumps the rows between two ranges, and ends at the end of the last range without loading a later group.
- **Scope.** `crates/persistence/rrd-lsm/src/segment/reader/cursors.rs`, the import in `reader.rs`, the removal of `ProjectedReadRange::contains` (no caller left), and `tests/range_scan_seek.rs`.
- **Failure oracle.** `tests/range_scan_seek.rs`: three reads whose examined-version count must stay within the rows returned plus one per range. Committed failing first (`1,519` examined).
- **Trace and research decisions.** No profile was taken. The comparison was run before and after.
- **Stop conditions.** Stop if any read returns different rows; the hybrid-segment model tests are the check.

## Why it is correct

Projected read ranges are validated as sorted and disjoint, and a segment's rows and row groups are in key order. So a range the cursor has passed is never needed again, a row below the next range's start can be skipped to that start by binary search over the group's key column, and a row or a group at or after the last range's end ends the scan for that segment.

## Result

`cargo test -p rrd-lsm --test range_scan_seek`: a 16-row read examines 17 versions (was 1,519); two 4-row ranges in one group examine 10; an 8-row read at the start selects one row group and examines 9.

The comparison, same machine and method as the first run (WSL2 Linux, ext4, 24 CPUs, Fjall `3.1.8`, seven trials, three for `large`). Ratios are rrflowKV over Fjall, before and after this change. The workload's read is one range scan plus one point get for each returned row, so only part of it is the scan.

| Profile | Reads a second after maintenance, rrflowKV | Read rate after maintenance, ratio | Read p95 after maintenance, ratio |
|---|---|---|---|
| small-batch | 28,582 to 33,236 | 0.71 to 0.79 | 1.40 to 1.25 |
| standard | 16,569 to 17,649 | 0.75 to 0.79 | 1.31 to 1.25 |
| read-heavy | 8,588 to 8,920 | 0.76 to 0.79 | 1.36 to 1.32 |
| sustained | 8,212 to 8,443 | 0.80 to 0.82 | 1.23 to 1.16 |
| extended | 7,414 to 7,653 | 0.81 to 0.85 | 1.26 to 1.21 |
| large | 3,661 to 3,896 | 0.64 to 0.70 | 1.59 to 1.44 |

- rrflowKV reads 3 to 16 percent more a second after maintenance than before the change.
- **It is still behind Fjall on every profile**, by 15 to 30 percent. The scan was not the larger share of the loss.
- Peak memory at 1,000,000 claims is unchanged at 1.42 times Fjall's.
- Write rate, recovery and bytes moved within run-to-run noise.

## Evidence

| Command | Result |
|---|---|
| `cargo test -p rrd-lsm --locked` (Windows) | 257 passed |
| `cargo test -p rrd-store --all-features --locked` (Windows) | 287 passed |
| `cargo clippy -p rrd-lsm -p rrd-store --all-targets --all-features --locked -- -D warnings` | clean |
| `storage_comparison` on six profiles, `TMPDIR` on ext4 (Linux) | both engines verified in every trial |

Raw output: `eval/results/2026-11-20-rrflowkv-vs-fjall-scan-seek-<profile>.json`.

## Not proven

- One noisy machine; differences of a few percent are within noise.
- No profile was taken. What remains is expected in the per-row point gets and the page-cache lock taken for each lookup, which is the next package; that expectation is from reading the code.
- Linux and macOS test runs are for the candidate gate.

## Roadmap and POA&M

No status change.
