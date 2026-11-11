# F-06: rrflowCLD on by default, and a policy that cannot stall itself

**Status:** active implementation evidence; F-06 remains open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-f/f-06-rrflow-cld-default-on-20261111`

**Owner:** OBJ-02 and F-06 in [Gate F](../../../roadmap/rrflow-1.0/gate-f.md); [tiered persistence](../../../reference/storage/tiered-persistence.md)

**Starting revision:** `agent/p1-capability-plans-20261111` at `b2cd651e`, because the installation identity changes in both packages.

**Owner decisions, 2026-11-11.** rrflowKV's LSM is a custom AI-optimized Fjall and is the hot tier. rrflowCLD is the same idea for Parquet and is the cold tier. rrflowCLD is the canonical copy once compaction moves data into it, because a derived copy can never free LSM space. The `rrflow_cld` diagnostic counters and `coldlight.rs` are vector trigger observations and are not rrflowCLD.

**Current behavior.** The Parquet tier existed and worked but was off unless an operator wrote a policy into the install configuration. Run with a policy of 8 MiB input, 8 MiB output and 64 MiB memory on a clone of this repository, it published 6,181 of 6,183 changes and then stopped permanently. Change 6182 is one 1,193,279-byte `source-tree-change-set` record. The encoder's admission estimate for it is about 72 MB, which that budget refuses, and each retry fails the same way with only `resource_limit` recorded. Validation accepted that policy.

**Target behavior.** A policy whose working-memory budget cannot encode one full input page is refused when it is validated, using the same formula the encoder admits with. The bundled configuration turns rrflowCLD on with an input bound matching the largest native transaction.

**Changed paths.** `crates/transport/rrd-contract/src/deployment.rs` (`ColdHistoryPolicy::minimum_working_memory_bytes` and the validation rule), `crates/compute/rrd-query/src/parquet.rs` and `lib.rs` (`parquet_working_set_reservation`, now the encoder's one formula), `crates/authority/rrd-engine/assets/install/default-estate-configuration-v3.toml`, `crates/authority/rrd-engine/src/engine/cold_history.rs` (parity test), `engine/tests/installation_bootstrap.rs`, `crates/transport/rrd-contract/tests/deployment_configuration_contract.rs`, `crates/transport/rrd-contract/src/cold_history.rs` (a 4 KiB fixture budget the rule now refuses), `fixtures/rrd-sdk-conformance-v1.json` (the installation identity), `docs/reference/storage/tiered-persistence.md`, this record.

**Failure oracles.** The defect was observed first on the real run above. `rrflow_cld_policy_that_cannot_encode_its_own_input_is_refused` and `bundled_configuration_enables_rrflow_cld` were written before the change and failed to compile against the old code, which is a weaker red than a failing assertion. `policy_minimum_equals_the_encoder_reservation_for_a_full_page` ties the contract's formula to the encoder's.

**Acceptance.** On Windows, tests: `rrd-engine --lib`, `rrd-contract` (200 passed after the fixture correction), `rrd-query`, `rrflow-cli --test installed_lifecycle`, `rrflow-eval --test storage_contract`, and `rrd-server --test http_process -- installed_attunement_replan` all passed. `cargo clippy --locked --workspace --all-targets -- -D warnings` clean. Generated surfaces verified.

**Real run on this repository.** Release binary from this tree, a default install with no configuration file on a fresh clone of main at `2d2fba8`. After attunement the head cursor was 6183 and rrflowCLD reported `through_cursor` 6183, 14 generations and no failure. On disk there were 14 Parquet files totalling 2,330,398 bytes under the storage root's `cold-history` directory. pyarrow, which shares no code with the engine, read 6,183 rows, and each file's SHA-256 equalled its file name. Generation 1 read back through `cold-history-read`.

**Not established.**

- rrflowCLD is still a derived copy. LSM compaction does not write it, no LSM space is reclaimed, ordinary queries and context assembly do not read it, and backups omit it.
- The file format is the stock `parquet` crate writing generic change rows. Nothing about it is custom or AI-specific yet, and what that must mean has not been specified.
- A change whose escaped row form exceeds the input bound still stops the tier at that change, with only the failure class recorded.
- Reserving 32 times the input is the encoder's existing estimate; it has not been measured against real peak memory.
- No Linux or macOS run, no candidate CI, no sustained run.

**Research decision:** none. **Roadmap/POA&M change:** none; F-06 stays open.
