# Linux supervisor tests run at the product's 512 MiB limit and fail on a dead child

**Status:** active repair evidence for the last open failure of the 2026-11-23 nightly
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/supervisor-test-limit-512-20261124`
**Owner:** [CI and runner operations](../../../operations/ci.md) and [Gate C, C-07](../../../roadmap/rrflow-1.0/gate-c.md); alarm [issue 505](https://github.com/rrflow/rrflow-development/issues/505). Follows the [nightly matrix repair](nightly-matrix-repair-20261124.md).

## Change brief

Baseline `4b4befbab`. The owner decided on 2026-11-24: "let errors surface and raise to 512". Scope is one file, `crates/authority/rrd-engine/src/engine/attunement/linux_supervisor/tests.rs`. No engine, supervisor or policy code changes. Stop if the change would need a product limit to move or a test to be skipped.

Files read: `linux_supervisor.rs`, `linux_supervisor/tests.rs`, `attunement/parse/step.rs`, `rrd-contract/src/attunement/derivation.rs`.

## Findings and changes

**The tests ran below the product default.** Fixture children were limited to 256 MiB of address space. The default parse-worker limit in `rrd-contract` (`derivation.rs`) is 512 MiB. With both cold codecs linked the test executable is 527,579,248 bytes and a child cannot start under 256 MiB. Every fixture child now runs under one constant, `CHILD_ADDRESS_LIMIT`, of 512 MiB, and the limit fixture still proves that a 1 GiB mapping is refused.

**Two tests passed on a dead child.** `native_linux_supervisor_drop_terminates_owned_child` only waited for the child to be gone, and the nested fixture behind `native_linux_supervisor_kills_child_when_supervisor_process_dies` reported its worker without checking it was alive. A worker that crashed at start satisfied both, which is why the nightly showed three failures and not five. Both now assert that the child is still running before the termination under test, through one helper, `assert_running`.

## Verification

All in WSL on this revision, `cargo test -p rrd-engine --lib --all-features --locked --no-fail-fast -- engine::attunement::linux_supervisor`, debug information off, `libclang` from the PyPI wheel:

- At 512 MiB: 7 passed, 0 failed, 3 ignored fixtures.
- Probe with the constant temporarily set to 256 MiB: 2 passed, 5 failed. The two previously silent tests failed with `the child exited while starting` and `child supervisor did not report its worker`. The constant was restored to 512 MiB afterwards.
- `rustfmt --check` on the file: clean.
- Not run: the default-feature Linux suite, clippy on Linux, any hosted job, a release build. The nightly `optional feature (rrd-engine)` lane is the first hosted run of this change.

## Remaining risk

A release engine built with a cold codec was not measured. A deployment that configures the parse worker below the default, or an executable that grows past 512 MiB of mapped image, would fail to start its worker; the engine reports that as `parse worker did not start`. No POA&M entry is added because no product failure was observed.

No objective, roadmap or POA&M status changes.
