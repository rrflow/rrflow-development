# F-06: rrflowCLD publication waits at the transaction gate

**Status:** active implementation evidence; F-06 remains open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-f/f-06-cld-transaction-gate-20261111`

**Owner:** F-06 in [Gate F](../../../roadmap/rrflow-1.0/gate-f.md); [tiered persistence](../../../reference/storage/tiered-persistence.md)

**Starting revision:** `main` at `d9995852`.

**Current behavior.** #329 turned rrflowCLD on by default. Its background publisher commits to the control journal in five places without taking the engine's transaction gate, the in-process serializer every other writer uses. A foreground commit that also appends to the control journal can then meet it as a physical write conflict. `RuntimeRepository` reports that conflict as a runtime conflict at an unchanged cursor, so clients receive a retryable 409 such as `runtime commit conflict: expected cursor 30, actual cursor 30`. Candidate CI on #335 failed this way in `installed_source_read_choice_is_explicit_and_reopens` on Linux ARM64, in a task assessment commit.

**Target behavior.** Every cold-history control commit waits at the gate. The gate is held only for the commit, never while a Parquet file is encoded.

**Changed paths.** `crates/authority/rrd-engine/src/engine/cold_history.rs` (`commit_cold_control`, `commit_cold_control_batch`, and the five call sites), this record.

**Failure oracle.** `publication_waits_for_a_foreground_transaction` holds a foreground permit and runs publication on another thread. On `main` it failed: the control journal advanced while the permit was held. With the change it passes, and publication proceeds once the permit is dropped.

**Acceptance.** On Windows: `cargo test --locked -p rrd-engine --lib` 455 passed; `cargo clippy --locked -p rrd-engine --all-targets -- -D warnings` clean; `cargo fmt --check` clean.

**Not established.** The CI failure was not reproduced locally, so the link from this race to that failure rests on the error's shape and the code path, not on a captured interleaving. #333's earlier queue failure in `selected_inventory_completes_and_replans_across_primary_process_restarts` may share the cause and is unconfirmed. When the gate is saturated, publication now records a storage failure instead of retrying quietly.

**Research decision:** none. **Roadmap/POA&M change:** none.
