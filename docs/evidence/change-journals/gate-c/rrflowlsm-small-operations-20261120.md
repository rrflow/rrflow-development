# rrflowLSM gains contains, size-of, prefix scan, first row, is-empty, take and update

**Status:** active change-journal evidence; six Fjall capabilities added in `rrd-lsm`, not yet offered through the `rrd-store` port

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/rrflowlsm-small-operations-20261120`

**Owner:** [rrflowLSM against Fjall](../../../reference/storage/rrflowlsm-fjall-parity.md), step 2 of its order

**Starting revision:** main `9babe6d5f`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage; [kernel contract](../../../roadmap/kernel-contract.md) K10. Owner direction of 2026-11-19: rrflowLSM has all of Fjall's capabilities first.
- **Current behaviour.** rrflowLSM had no way to ask whether a key holds a value, or how long the value is, without reading the value; no prefix scan below `rrd-store`; no first row or emptiness test; no take or update on a transaction.
- **Target behaviour.** `Database` offers `contains_key`, `size_of`, `scan_prefix`, `first_key_value` and `is_empty`. `Transaction` offers `contains_key`, `size_of`, `take`, `fetch_update` and `update_fetch`, each through its own pending writes.
- **Scope.** `crates/persistence/rrd-lsm/src/database/reads.rs`, `transaction.rs`, `segment.rs`, `segment/read.rs`, and `tests/small_operations.rs`.
- **Failure oracle.** `tests/small_operations.rs`, six tests.
- **Research decision.** Semantics follow Fjall `3.1.8`: `fetch_update` returns the previous value and `update_fetch` the new one; `None` from the closure removes the key.
- **Stop conditions.** Stop if `get` changes behaviour; its selection was moved into one shared function and the existing suites check it.

## What each operation reads

- `contains_key` on a flushed key loads the key spine (three pages) and the validity page. `size_of` adds the value offsets. Neither loads value data. The test opens a database cold and counts page loads: 4, then 1 more, then 1 more for `get`.
- `first_key_value` and `is_empty` open a projected read over every key with one row per batch and take the first batch, so they stop at the first visible row and skip tombstones.
- `scan_prefix` scans to the smallest key above the prefix; a prefix of only `0xff` bytes, or an empty one, scans to the end.
- `take` writes nothing when the key holds no value. An update whose closure returns `None` for an absent key writes nothing.

## Row-group size experiment, not carried

Recorded in the parity record: with the default row-group target at 8 KiB and at 4 KiB instead of 64 KiB, rrflowKV read slower at one million claims and used more memory and bytes. The default is unchanged.

## Evidence

Windows 11, one run each.

| Command | Result |
|---|---|
| `cargo test -p rrd-lsm --test small_operations --locked` | 6 passed |
| `cargo test -p rrd-lsm --locked` | 264 passed |
| `cargo test -p rrd-store --all-features --locked` | 287 passed |
| `cargo clippy -p rrd-lsm -p rrd-store --all-targets --all-features --locked -- -D warnings` | clean |

## Not proven

- The operations are on `rrd-lsm`. The `rrd-store` transaction port and the Fjall baseline behind it do not offer them yet, so no differential against Fjall runs for them.
- The last key and a length wait for reverse iteration.
- Linux and macOS are for the candidate gate.

## Roadmap and POA&M

No status change.
