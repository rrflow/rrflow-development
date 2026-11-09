# C-07: bind client and engine begin diagnostics

**Status:** active draft prerequisite; no C-07 or alpha gate passed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-client-begin-correlation-20261107`
**Owner:** C-07g, OBJ-10/11, [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md), [PR #295](https://github.com/rrflow/rrflow-development/pull/295).

Starting draft revision `6e2cafefc740cd244f43becba5aa1e29ae0b0e76`,
tree `4fe3e50ac517b85d2819409a85f67aa2175f1352`. The installed workload
used `<key>-begin` as its begin request ID, but its existing slow-write trace
printed the SHA-256 of `<key>`. The engine timing event prints the digest of
the actual begin request ID, so those two fields were different and a direct
digest join could not attribute an observed slow client begin to that event.

The PR brief was amended before implementation. The existing workload now
constructs the begin request ID once, passes those exact bytes to both the
client options and its bounded slow-write formatter, and appends a separately
named `begin_request_sha256` while retaining the old digest and fields.
Both error and success paths use the same identity. The installed restart test
checks the engine events against the digest of the actual client request
options. Neither raw identifier nor credential is printed. The digest may
still be guessable and must be treated as scoped diagnostic data.

Changed `crates/adapters/rrflow-cli/tests/support/sustained/workload.rs` and
`crates/adapters/rrflow-cli/tests/installed_reopen.rs`; created this record
and regenerated the Gate C index. Read both changed files in full, the existing
client `RequestOptions` definition and the complete diff. No upstream source
or new runtime authority was introduced; the code and observed retained
campaign are the research basis.

On Windows, using `CARGO_TARGET_DIR=D:\localdev\devlabs\rrflow\target\c07-diagnostic-trace-build`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, and
`CARGO_PROFILE_TEST_DEBUG=0`:

- `cargo fmt --all` passed.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen`
  passed **39**, failed **0**, ignored **14**. The existing trace privacy test
  checks the begin-request digest against constructed client options; the
  real installed server and restart test checks the emitted events against
  the request ID actually sent.
- `cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings`
  passed.
- `python scripts/knowledge/render_navigation.py`, its `--check` mode,
  `python scripts/ci/check_documentation.py`, and `git diff --check` passed.

The executable used in the installed test had SHA-256
`8ea85a3d72317fe8108b31fee3c3efd4aabf49ffa93fa9c4e6376e9b03155709`;
the local test executable had SHA-256
`dc2b35caa80f74b5d87ef19608ebf30b4494d5944bb33640bc4c1ef68059a375`.
These are local working-tree builds, not candidate or release artifacts.

The sustained driver still does not enable the bounded collector; this
identity repair therefore does not yet diagnose the ARM run. Its original
latency overruns, conflict cause, C-07 capacity and soak, the project snapshot
defect, and all candidate and merge-group checks remain open. No benchmark
policy, runtime state, retry behavior, roadmap or POA&M status changed.
Change checklist: owning failure and driver named; brief amended before
implementation; actual client identity and server event checked; changed files
and diff read; linked journal and generated navigation record the result.
