# Streaming index-artifact decoder in the query executor

**Status:** active build-and-test absorption; no gate acceptance claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-02-streaming-index-artifact-decoder-20261110`

**Owner:** E-02 native index reader; `docs/reference/query/index-catalogue.md`

**Starting revision:** main `a8551dc9`. Sibling of `e-04-rrflow-cld-triggers-20261110.md`; both were first opened together as #315 and split to stay under the candidate line limit.

**Current behavior.** The executor decoded a whole index artifact into memory before reading rows from it.

**Target behavior.** `decode_index_artifact_for_read` and `decode_index_rows` read the artifact through a streaming `serde_json` visitor over its raw value, charged to the same query memory budget, with `validate_count_analytics` unchanged in meaning. Lifted from the owner's 2026-11-08 editor commits, which cannot be pushed as committed. Also carries the POAM-040 paragraph #231 had that main lacks.

**Changed paths.** `crates/compute/rrd-query/{Cargo.toml (serde_json raw_value), src/execute.rs}`, `docs/reference/query/index-catalogue.md`, `docs/poam/rrflow-1.0-alpha/poam-040.md`, this record.

**Acceptance.** On Windows at this package off main `a8551dc9`: `cargo test --locked -p rrd-query` passes (the golden plan fixture is unchanged: the decoder alters no planned shape); `cargo clippy --locked -p rrd-query --all-targets -- -D warnings` and `cargo fmt --check` clean; `render_navigation.py` and `check_documentation.py` OK.

**Research decision:** none. **Checks not run:** Linux/macOS, candidate CI. **Roadmap/POA&M change:** none.
