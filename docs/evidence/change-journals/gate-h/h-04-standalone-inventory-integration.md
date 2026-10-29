# Standalone rrflowDB journey: inventory candidate integration

**Status:** active draft integration; dependency and hosted native CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-standalone-inventory-integration`
**Owner:** OBJ-01 / OBJ-04 / H-04 / D-01 / POAM-006 / [PR194](https://github.com/rrflow/rrflow-development/pull/194)

Starting PR194 head `b8d85d471b288b9b6b7bc32a187c9151ebdf6799`;
dependency [PR193](https://github.com/rrflow/rrflow-development/pull/193)
candidate `bdb954e48d2eb751b4839ed6efa761cc1e294071`, containing public main
`e73ba9287dafba5d8fb8fb6a990fd145ffd01ab3`. The PR194 brief was extended
before integrating the candidate. PR193 must land before this draft becomes
ready; its candidate result cannot be transferred to this integrated revision.

The normal merge produced only the two predicted generated conflicts: the
Gate H journal index and old tracked inventory JSONL. Keep the inventory
retirement descriptor, regenerate the index from both sets of linked journals,
and regenerate current Git-derived inventory under ignored `target`. The
existing native standalone, inventory artifact and candidate policy checks
remain together; no runtime, security, threshold, existing test, source or
historical evidence was intentionally removed. All other source and SDK
changes came from the dependency merge. Direct integration changes are the
generated conflict resolutions and this successor journal. Read the existing
standalone journal, complete process runner and combined workflow/policy diff;
the first journal and the PR brief identify prior full reads, failure oracle,
native admission and decision boundaries. No third-party implementation was
adapted. Bounded executable hashes and independent values are the existing
diagnostic choices; no performance claim is made.

On clean original head `b8d85d471b288b9b6b7bc32a187c9151ebdf6799`, an
isolated WSL2 Linux x86_64 build used pinned Rust 1.99.1 after verifying
`rustup-init` against the upstream SHA-256 file. With an isolated target,
`CARGO_BUILD_JOBS=3`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0`,
`cargo build --locked -p rrflow-cli --bin rrflow -p rrd-client --example
rrflowdb` passed. `python3 scripts/ci/test_rrflowdb_standalone.py` passed six
tests, `python3 scripts/ci/check_workflow.py` passed, and the real standalone
runner passed with primary SHA-256
`2fea7364fff39ffb05a6f54979c707f3b848aa5877efc6c2703a48965f0c3e19`,
example SHA-256
`38b218c6a50f5b8da3b0e570b76a482567fd0f539aae000aee38dfcd2a915147`,
and reopened cursor 7. This is Linux under WSL characterization of the old
head, not hosted Linux CI or proof for this new integrated source.

On the merged working tree, `python scripts/ci/test_rrflowdb_standalone.py`
passed six tests, `python scripts/ci/test_execution_inventory.py` passed ten,
`python scripts/ci/test_candidate_admission.py` passed 11,
`python scripts/ci/check_workflow.py` passed, and `git diff --cached --check`
passed. A new Windows-interpreter policy test first failed: the new CI step
used `python3` even on Windows. The existing native jobs use `python` there,
and an older [GitHub runner-images report](https://github.com/actions/runner-images/issues/2461)
documents why `python3` cannot be assumed on all Windows images; that report
does not prove the current hosted image's PATH. The PR brief was extended
before changing the shared step to select `python` on Windows and `python3`
elsewhere. `python scripts/ci/test_rrflowdb_standalone.py` then passed seven
tests, including rejecting the incorrect Windows interpreter, the ten
inventory tests and `python scripts/ci/check_workflow.py` passed again.
`python scripts/ci/check_documentation.py`,
`python scripts/knowledge/render_navigation.py --check` and
`python scripts/ci/build_execution_inventory.py --check` passed after the
initial integration and again after recording the integrated native result.
For the integrated Windows x64 working tree, an isolated
`CARGO_TARGET_DIR=target/verify-standalone-194-integrated`,
`CARGO_BUILD_JOBS=2`, `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_INCREMENTAL=0`
ran `cargo build --offline --locked -p rrflow-cli --bin rrflow -p rrd-client
--example rrflowdb`: passed in 4m03s. The staged working tree was
`8ae721ae407806052ab1d96eddd7ed32c329da8b` before recording this result;
`git diff bdb954e48d2eb751b4839ed6efa761cc1e294071 -- crates Cargo.lock
Cargo.toml rust-toolchain.toml` was empty. Thus the build used the integrated
candidate's Rust sources, but this was an uncommitted merge working tree, not
a clean final-candidate build. Subsequent edits only record this evidence and
refresh generated navigation/inventory.

The actual Windows Git Bash branch from the corrected workflow selected
`python` and ran `scripts/ci/run_rrflowdb_standalone.py` against these newly
built binaries. It passed native owned process install, authenticated
initialization, task and relationship at equal stamps, update, abort,
wrong-state denial, offline verify, same-instance reopen and cursor 7. Primary
SHA-256 `d5fc391c6a091ab57b3cd7f8a8da81133cc0b0a7722c54ce3ac4ec81c902313f`;
example SHA-256
`55e0be6f213bce4abb9feb08e6ea8a507c30fb536a6db2c7f9ab4396a092ebc6`.
Local receipt `target/verify-standalone-194-integrated-ci-result.json` has
SHA-256 `c12e5387b03a2b69bee8820d3be1ecfad81240f4e2a5ed46d675026997be5fca`;
the local shell driver has SHA-256
`3c09eea5f5918a5b475fefb15afea90de5b57938394c7465b655702f082d082e`.
Both are local characterization artifacts, not hosted qualification receipts.
On the same isolated target, `cargo test --offline --locked -p rrd-client
--test rrflowdb_local` passed four real-server integration tests, including
relations/atomicity/temporal read/reopen, stored attunement read and cancel
replay, and foreign/truncated Parquet rejection. No product-wide conclusion
is inferred from those tests.

No C-07, H-04, D-01, POA&M or alpha
completion status changes; protected CI and the native matrix remain pending.
