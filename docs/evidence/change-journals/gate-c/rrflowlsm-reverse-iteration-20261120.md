# rrflowLSM iterates in reverse

**Status:** active change-journal evidence; reverse iteration and the last key added in `rrd-lsm`, checked against a model

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/rrflowlsm-reverse-iteration-20261120`

**Owner:** [rrflowLSM against Fjall](../../../reference/storage/rrflowlsm-fjall-parity.md), step 3 of its order

**Starting revision:** main `22227a263`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage; [kernel contract](../../../roadmap/kernel-contract.md) K10. Owner direction of 2026-11-19: rrflowLSM has all of Fjall's capabilities first.
- **Current behaviour.** Every cursor walked forward. The greatest key, or the last rows of a range, could only be found by reading the whole range.
- **Target behaviour.** `Database::scan_reverse` returns at most `limit` rows of a range from the greatest key down, `Database::last_key_value` the greatest visible row, and `Transaction::scan_reverse` the same through the transaction's pending writes.
- **Scope.** New `crates/persistence/rrd-lsm/src/segment/reverse.rs`; `database/reads.rs`, `memtable.rs`, `transaction.rs`; new `tests/reverse_iteration.rs`.
- **Failure oracle.** `tests/reverse_iteration.rs`: an independent model of every version of every key, compared on six ranges, three limits and every snapshot taken, in the memtable, after each flush and after compaction, for three seeds and three row-group sizes.
- **Stop conditions.** Any difference from the model, or from the forward scan read backward.

## How it works

- Each segment gets a downward cursor. It opens the last row group that starts below the end of the range, seeks to the end inside it, and walks rows down, loading one key spine at a time and no value page. For each key it keeps the newest version at or below the read sequence, and it yields tombstones so a merge cannot resurrect an older value.
- The memtable is walked by moving an upper bound down past each key it yields.
- The database takes the greatest key among the heads. The memtable's version wins; among segments the highest sequence wins and the first segment wins a tie, as in `get`. Only the winner's value is read.
- A transaction fetches `limit` plus its pending keys in the range, because each pending key hides at most one stored row, then merges.

## Evidence

Windows 11, one run each.

| Command | Result |
|---|---|
| `cargo test -p rrd-lsm --test reverse_iteration --locked` | 2 passed |
| `cargo test -p rrd-lsm --locked` | 266 passed |
| `cargo test -p rrd-store --all-features --locked` | 287 passed |
| `cargo clippy -p rrd-lsm -p rrd-store --all-targets --all-features --locked -- -D warnings` | clean |

## Not proven

- The reverse walk loads its key spines as ordinary cache accesses and is not metered by the projected-read budgets. A caller bounds it with `limit`.
- It is on `rrd-lsm`; the `rrd-store` port does not offer it yet, and it has not been measured against Fjall.
- Linux and macOS are for the candidate gate.

## Roadmap and POA&M

No status change.
