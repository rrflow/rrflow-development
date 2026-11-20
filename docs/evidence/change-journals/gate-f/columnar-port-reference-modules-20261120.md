# A columnar port, with Parquet and Vortex as reference modules behind it

**Status:** active change-journal evidence; the port and two plug-in modules exist and agree; the cold tier does not use them yet

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-f/columnar-port-reference-modules-20261120`

**Owner:** [reference modules and the reasoning fabric](../../../roadmap/reference-modules-and-reasoning-fabric-owner-direction-20261120.md), step 2 of its order; [POAM-090](../../../poam/rrflow-1.0-alpha/poam-090.md)

**Starting revision:** main `d7e1d6381`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage; F-06; [kernel contract](../../../roadmap/kernel-contract.md) K10. Owner direction of 2026-11-20: Parquet, Vortex and Fjall are plug-and-play modules for testing, reverse engineering and learning.
- **Current behaviour.** Fjall was the only reference module. Parquet was reachable only as the product's own writer in `rrd-query`. Nothing named Vortex. No boundary existed that a columnar format could sit behind.
- **Target behaviour.** `rrd-store` has a columnar port: one typed table and one contract (`encode`, `decode`, `decode_columns`). Parquet and Vortex sit behind it as optional modules, answer one transcript identically, and run in one comparison program. An architecture test keeps both out of every other package's build.
- **Scope.** `crates/persistence/rrd-store/src/columnar.rs`, `columnar_reference.rs` and its two modules, `tests/columnar_reference.rs`, `examples/columnar_comparison.rs`, `Cargo.toml`, `Cargo.lock`, and the reference-module rule in `rrd-engine/tests/workspace_architecture/storage.rs`.
- **Failure oracle.** `tests/columnar_reference.rs`: each module must answer exactly what the table holds, whole and for three column selections, at 0, 1, 2, 257 and 5,000 rows, with absent values, empty strings, non-ASCII text and both ends of the 64-bit range; each must refuse an unknown column, a truncated artifact and the other module's artifact.
- **Research decision.** `vortex` `0.93.0` was built in a scratch project first: Apache-2.0, Rust 1.95, compatible with the workspace's Arrow `59.2.0`, with a blocking write and an in-memory open and scan. It adds about 250 packages to the lock file, all behind the optional feature.
- **Stop conditions.** Stop if a product package gains Vortex, or if adding it moves a version the product pins.

## What the test found while it was written

- Vortex returned a constant text column in another Arrow representation than the one written. The port's reader now brings each column to the representation the port names before reading it, for every module.
- The Parquet module passed the unprojected schema to a projected read. It now reads with the schema of the columns it asked for.

## Comparison

Windows 11, x86_64, release build, in memory, seven trials, median. One run. The table has ten columns shaped like a cold-history page. Parquet uses Zstandard level 3, the cold tier's setting; Vortex uses its default compressor and was built without Zstandard, so the settings are not equalised.

| Rows | Module | Encoded bytes | Share of raw value bytes | Encode | Decode all | Decode 2 of 10 columns |
|---|---|---|---|---|---|---|
| 10,000 | Parquet | 265,816 | 11.1% | 13.0 ms | 7.4 ms | 1.5 ms |
| 10,000 | Vortex | 388,460 | 16.3% | 64.1 ms | 5.3 ms | 1.3 ms |
| 100,000 | Parquet | 2,145,399 | 9.0% | 89.7 ms | 66.0 ms | 14.6 ms |
| 100,000 | Vortex | 3,773,084 | 15.8% | 248.1 ms | 49.7 ms | 8.1 ms |
| 1,000,000 | Parquet | 16,864,102 | 7.1% | 851.9 ms | 701.5 ms | 131.4 ms |
| 1,000,000 | Vortex | 37,526,388 | 15.7% | 1,941.7 ms | 482.2 ms | 61.3 ms |

- Vortex decodes faster: 1.3 to 1.5 times for the whole table and up to 2.1 times for two columns at a million rows.
- Parquet is smaller, 1.5 to 2.2 times, and encodes 2.3 to 4.9 times faster.
- Every time includes conversion between the port's table and Arrow, which both modules pay alike.

Raw output: `eval/results/2026-11-20-columnar-reference-modules-<rows>.json`.

## Evidence

Windows 11, one run each.

| Command | Result |
|---|---|
| `cargo test -p rrd-store --features parquet-reference,vortex-reference --test columnar_reference` | 3 passed |
| `cargo test -p rrd-store --all-features --locked` | 292 passed |
| `cargo test -p rrd-store --locked` (no module) | 284 passed |
| `cargo test -p rrd-engine --test workspace_architecture --locked` | 35 passed |
| `cargo clippy -p rrd-store -p rrd-engine --all-targets --all-features --locked -- -D warnings` | clean |
| `cargo tree -p rrd-engine -e normal --locked`, searched for `vortex` | no match |
| `python scripts/ci/check_workflow.py` | CI policy OK |

## Not proven

- Linux and macOS. The local Linux run did not happen: WSL stopped answering. The candidate gate's all-features job builds and tests the modules on Linux.
- There is no native rrflowCLD arm, and the cold tier still writes Parquet through `rrd-query`. POAM-090 stays open.
- The comparison is in memory, with unequal compression settings and no file I/O, on one machine.
- The port has two column types, text and unsigned 64-bit, which is what a cold-history page stores.

## Roadmap and POA&M

No status change. POAM-090 gains a note.
