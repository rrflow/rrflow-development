# Contains, size-of, reverse scan and take through the store port, compared with Fjall

**Status:** active change-journal evidence; four operations agree line for line in rrflowMX, rrflowKV and Fjall, on fresh and on maintained data

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/store-port-fjall-operations-20261120`

**Owner:** [rrflowLSM against Fjall](../../../reference/storage/rrflowlsm-fjall-parity.md); continues [the small operations](rrflowlsm-small-operations-20261120.md) and [reverse iteration](rrflowlsm-reverse-iteration-20261120.md)

**Starting revision:** main `576763980`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage; [kernel contract](../../../roadmap/kernel-contract.md) K10. Owner direction of 2026-11-19: rrflowLSM is proven against Fjall, not only given its operations.
- **Current behaviour.** The operations added to `rrd-lsm` on 2026-11-20 were checked against a model written for them. The store's transaction port did not offer them, so the Fjall baseline could not be asked the same questions.
- **Target behaviour.** `StorageTransaction` offers `contains_key`, `size_of`, `scan_reverse` and `take`. rrflowKV answers through `rrd-lsm`; the Fjall baseline answers with Fjall's own `contains_key`, `size_of` and reverse range; rrflowMX is the reference.
- **Scope.** `crates/persistence/rrd-store/src/transaction.rs`, the five implementors of the port, and `tests/fjall_baseline.rs`.
- **Failure oracle.** `tests/fjall_baseline.rs`: the transaction transcript gains presence, length, reverse scans and take; a new transcript reads 500 keys, with deletions and rewrites, after each engine's own flush and compaction. Every line must match in all three engines.
- **Stop conditions.** Stop if an engine must special-case an answer to match.

## Result

`cargo test -p rrd-store --all-features --test fjall_baseline`: 5 passed. No engine needed a change to agree: rrflowKV's answers on memtable data and on flushed and compacted data matched Fjall's and the reference on the first run.

## Evidence

Windows 11, one run each.

| Command | Result |
|---|---|
| `cargo build --workspace --all-features --all-targets --locked` | built |
| `cargo test -p rrd-store --all-features --locked` | 288 passed |
| `cargo clippy -p rrd-lsm -p rrd-store --all-targets --all-features --locked -- -D warnings` | clean |

## Not proven

- `fetch_update`, `update_fetch`, prefix scan, the first and last key and is-empty are not on the port; they take a closure or have no caller above `rrd-lsm` yet.
- The comparison is of answers, not of speed. The reverse scan has not been measured against Fjall's.
- Linux and macOS are for the candidate gate.

## Roadmap and POA&M

No status change.
