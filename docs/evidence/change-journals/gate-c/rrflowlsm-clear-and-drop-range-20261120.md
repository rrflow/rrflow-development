# rrflowLSM clears, and drops whole segments by key range

**Status:** active change-journal evidence; two Fjall capabilities added as manifest operations, with no on-disk format change

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/rrflowlsm-clear-and-drop-range-20261120`

**Owner:** [rrflowLSM against Fjall](../../../reference/storage/rrflowlsm-fjall-parity.md), step 4 of its order

**Starting revision:** main `c158a0667`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage; [kernel contract](../../../roadmap/kernel-contract.md) K10. Owner direction of 2026-11-19: rrflowLSM has all of Fjall's capabilities first.
- **Current behaviour.** The only way to remove data was one tombstone per key.
- **Target behaviour.** `Database::clear` removes every key in one manifest publication. `Database::drop_range` drops every segment whose keys all lie in a range.
- **Scope.** New `crates/persistence/rrd-lsm/src/database/drop.rs` and `tests/clear_and_drop_range.rs`.
- **Failure oracle.** `tests/clear_and_drop_range.rs`, six tests.
- **Research decision.** Fjall `3.1.8` and `lsm-tree 3.1.8` were read before designing. This corrected the plan: an earlier note said these operations need new log record kinds and range tombstones in segments. They do not. Fjall's `clear` installs a tree version with an empty memtable and no tables; its tree's `drop_range` "drops tables that are fully contained in a given range"; neither writes a range tombstone.
- **Stop conditions.** Stop if a reopen replays a log frame from before a clear.

## What each does

- **`clear`** follows the flush's publication order: sync the log, create the successor log at the next sequence, publish a manifest that lists no segment, then swap in an empty memtable. Either manifest is recoverable at every boundary. Sequences continue; nothing is reset.
- **`drop_range`** publishes a manifest without the segments wholly inside the range. A segment that only overlaps is kept whole, and the memtable is untouched.
- **Neither is a versioned delete.** A snapshot taken before the call reads the data as gone afterwards, so both refuse while a transaction is open (`Error::ActiveTransactions`). A live read stream keeps the segments it holds. After a range drop, an older version of a dropped key that lives in a kept segment is visible again; a test states this.
- **Space** is reclaimed by the next collection, unless a named checkpoint pins the earlier manifest; a test checks both.

## Evidence

Windows 11, one run each.

| Command | Result |
|---|---|
| `cargo test -p rrd-lsm --test clear_and_drop_range --locked` | 6 passed |
| `cargo test -p rrd-lsm --locked` | 272 passed |
| `cargo test -p rrd-store --all-features --locked` | 288 passed |
| `cargo clippy -p rrd-lsm -p rrd-store --all-targets --all-features --locked -- -D warnings` | clean |

## Not proven

- No failure was injected between the successor log and the manifest publication of a clear; the order is the flush's, whose boundaries are tested.
- Not offered through the `rrd-store` port, and not compared with Fjall: Fjall clears one keyspace and rrflowLSM has one tree.
- Weak delete is not built.
- Linux and macOS are for the candidate gate.

## Roadmap and POA&M

No status change.
