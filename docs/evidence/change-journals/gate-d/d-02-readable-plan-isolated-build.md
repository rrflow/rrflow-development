# Isolated-build verification for readable attunement plans

**Status:** active draft evidence; D-02, OBJ-14, POAM-039 and C-07 remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-readable-plan-isolated-build`
**Owner:** PR #225, dependent on #221.

This supplements the [initial record](d-02-readable-attunement-plan.md) and
[integration record](d-02-readable-plan-offline-integration.md). Its tested
source was local commit `a6d08a33b0986a7fb8ab1e9e057701623a6796d9`, tree
`38a5e18cf4f74f71a9a61fae0e8a2e06f36975b7`. The starting main revision and
change brief remain those recorded in the initial record. No source behavior
or roadmap status changed in this evidence increment.

The earlier Windows checks used a Cargo target shared across divergent
worktrees. Switching the contract exporter between the v2 #221 tree and the v3
#225 tree produced the same v3 OpenAPI digest in that shared target, while a
fresh isolated #221 build produced the different v2 digest
`17370b53bc27e385e37eb049099846e35ece2ee22956a8ad94030100f60999c0`.
The exact Cargo invalidation mechanism was not established. Treat the prior
shared-target results as provisional rather than exact-source evidence.
The [Cargo build-cache documentation](https://doc.rust-lang.org/cargo/reference/build-cache.html)
describes workspace-local build artifacts and compiler-wrapper caching.

On Windows, with `CARGO_TARGET_DIR` set to a fresh directory dedicated to this
worktree, `cargo test --offline --locked -p rrd-contract --test
attunement_contract` passed 20 tests; `cargo test --offline --locked -p
rrd-engine --lib publicly_selected_plan_replans_after_progress_and_reopens`
passed one; `cargo test --offline --locked -p rrd-engine --lib` passed 258 with
four ignored; and `cargo test --offline --locked -p rrflow-cli --test
installed_lifecycle` passed seven. `cargo build --offline --locked -p
rrflow-cli` passed. The engine test binary SHA-256 was
`500338a90eaf44f01f45dc50c68e125e2ccd764b0c5f10d781233dee97796489` and the
installed executable SHA-256 was
`a72b6698fb96558fd679f194501b96fd8a7538adc749b890a39fec5d70c9fa8f`.

Using that installed executable in `RRFLOW_TEST_BINARY`, `uv --directory
sdks/python run --frozen --offline pytest -o addopts= --color=no -q
tests/test_installed_attunement.py` passed two real HTTP tests, including
description readback following replan and restart. With the same isolated
target, `python scripts/ci/check_generated_surfaces.py --check` passed: 70 HTTP
operations and seven signal projections, OpenAPI SHA-256
`040ebab504a951e3781467fc7b3b0309e2d9998b4515c61b233f09c407cc1a2d`.
Navigation regeneration and `render_navigation.py --check` passed. A first
documentation check used the nonexistent `check_docs_policy.py` path and
failed before executing any checks; the repository-owned command is
`python scripts/ci/check_documentation.py`.
Its first execution then rejected this journal's status because it did not
declare `active` or `historical`; the status was corrected to active draft.
The next check found the generated index stale after that correction, requiring
another navigation regeneration. Regeneration followed by the documentation
policy check passed: 819 document statuses and 817 classified coordinates.

The earlier full engine suite's Windows object-store access failure remains
recorded in the initial journal. A passing isolated run does not establish its
cause or prove the separate #219 candidate fixes it. No performance, capacity,
soak, candidate CI or merge-group claim follows. The #221 dependency, complete
changed-file reread, final candidate validation and the product gaps listed in
the initial journal remain outstanding. Existing digest-bound receipts and
readback supply this package's trace; no source contents, private reasoning or
new external capability are logged. The checked source, test and reference
paths and full-file review limitations are those listed in the prior records;
this increment creates only this journal and its mechanically regenerated index.
