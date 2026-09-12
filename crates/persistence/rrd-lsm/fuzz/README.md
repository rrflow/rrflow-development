# rrflowKV fuzz qualification

This nested, development-only Cargo project exercises the public `rrd-lsm`
surface without joining RRFlow's product workspace or changing its dependency
closure. It requires a Rust nightly toolchain and `cargo-fuzz`; neither is a
runtime or distribution dependency.

List and compile both targets:

```text
cargo +nightly fuzz list --fuzz-dir crates/persistence/rrd-lsm/fuzz
cargo +nightly fuzz check --fuzz-dir crates/persistence/rrd-lsm/fuzz
```

Run bounded reproducible qualification from the repository root:

```text
cargo +nightly fuzz run --fuzz-dir crates/persistence/rrd-lsm/fuzz projected-read-state-machine crates/persistence/rrd-lsm/fuzz/corpus/projected_read_state_machine -- -runs=512 -max_len=256 -timeout=10
cargo +nightly fuzz run --fuzz-dir crates/persistence/rrd-lsm/fuzz segment-v4-open crates/persistence/rrd-lsm/fuzz/corpus/segment_v4_open -- -runs=4096 -max_len=128 -timeout=10
```

The state-machine target reuses the stable suite's independent MVCC oracle.
Every panic includes a replay coordinate derived from the input. The segment
target mutates the checked-in authenticated v4 fixture and accepts only a
fully usable segment or a typed `rrd-lsm` error.

Generated corpus growth, minimized crash artifacts, coverage, and target
output remain ignored. The named seed files are reviewed and tracked. A crash
artifact must be reproduced with the command printed by libFuzzer before any
production repair is planned; it must never be deleted or treated as a clean
run. Target compilation, an empty corpus, zero executions, or one bounded run
does not prove absence of defects or complete C-06/C-07 qualification.
