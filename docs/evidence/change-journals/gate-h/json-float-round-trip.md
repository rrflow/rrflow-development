# Keeping floats exact on the JSON wire

**Status:** active immutable change-journal evidence; no roadmap status changed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/json-float-round-trip`
**Owner:** OBJ-09 / H-01 / PR263; Gate H retains completion authority

## Package

- **Advances:** OBJ-09 equivalent client operation and H-01 context assembly.
- **Starting revision:** `ed3027b79e41dfcdf5b27e57f07346f11f6da347`.
- **Pull request:** [#263](https://github.com/rrflow/rrflow-development/pull/263), found by the
  retrieval test in draft #262, under the owner's 2026-11-03 direction to get RRFlow "tuned and
  competing".
- **Current to target:** the engine counts a context packet's bytes and computes its digest, then
  sends it as JSON, and a client re-encodes it and refuses it if either differs. `serde_json`'s
  default parser is fast rather than exact, so on one million deterministic values between 0 and
  40, 110,241 came back changed and 48,601 came back a different length. `rrflow_context` through
  the daemon path then failed with `context packet output_bytes differs from its encoded items` on
  each real task tried. The workspace now enables `float_roundtrip`, so every float keeps its value.
- **Unchanged:** how a packet is counted and digested, the check itself, the contract, and
  `Cargo.lock`.
- **Stop conditions:** the dependency set changing, a passing test failing, or a packet still
  failing after the change. None was reached.

## Files read

`rrd-contract/src/memory_context.rs` (`ContextPacket::validate`, `context_packet_sha256`),
`rrd-engine/src/engine/context.rs` (how `output_bytes` is counted),
`rrflow-mcp/src/authority.rs` (the daemon path), `rrd-client/tests/real_server.rs` (the existing
context test, which searches one word in one record).

## Changed paths

Changed `Cargo.toml` (the workspace `serde_json` dependency) and
`crates/authority/rrd-engine/src/engine/tests/context.rs`. Created
`crates/transport/rrd-contract/tests/json_float_round_trip.rs` and this journal, and changed the
generated gate H index.

## Decisions

- **Keep the check.** Removing it would have made the error disappear. It was correctly reporting
  that the wire changed the data.
- **Research:** `float_roundtrip` is `serde_json`'s documented feature for this. No code is adapted.
- **Trace, resource, debugging:** no new trace.

## Oracle and results

Both new test targets fail at `ce5df779`, before the change, and pass after it:

- `every_float_keeps_its_exact_value_through_json` failed with `15.269460205859353 came back as
  15.269460205859351`, and so did the same check through a decoded value.
- `a_packet_over_varied_text_validates_after_crossing_the_wire` failed with exactly the error from
  the real run: `context packet output_bytes differs from its encoded items`.

After the change:

- `cargo test --locked -p rrd-contract --no-fail-fast`: 167 passed.
- `cargo test --locked -p rrd-client --no-fail-fast`: 29 passed.
- `cargo test --locked -p rrflow-mcp --no-fail-fast`: 6 passed.
- `cargo test --locked -p rrd-engine --lib context`: 7 passed; `--lib digest`: 3 passed.
- Clippy with `-D warnings` on `rrd-contract`, `rrd-client` and `rrflow-mcp`: clean. fmt is clean.
- `Cargo.lock` is unchanged.

Real path: `eval/lab/retrieval_signal.py` from #262 sent 30 real tasks through a real `rrflow-mcp`
process built from this change, against an engine built without it, on this repository's text. All
30 returned a validated packet of 128 items. The same run with the unchanged client failed on both
tasks tried with the error above.

## Checks not run and remaining errors

- The other language SDKs were not checked. Their own JSON parsers and number formats are separate
  from this, and if one validates a packet's count or digest it could fail the same way.
- Parsing speed was not measured. Exact parsing costs something on every float on every JSON path
  in the workspace.
- `rrd-engine` was not run in full here, and candidate CI has not run.

## Review

- **Full-file reread and diff review:** complete.
- **Change checklist:** complete. The brief came before implementation, and the failing tests were
  committed before the fix.

## Status change

None. Pull request, CI run and merge evidence are recorded on
[#263](https://github.com/rrflow/rrflow-development/pull/263).
