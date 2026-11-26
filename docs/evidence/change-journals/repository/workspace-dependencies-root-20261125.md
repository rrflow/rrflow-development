# First-party dependencies are declared once, in the root manifest

**Status:** active landing evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/workspace-dependencies-root-20261125`
**Owner:** [Code structure and conventions](../../../architecture/code-structure-and-conventions.md), section 6.12 rule 4; kernel package K13.

## Change brief

Baseline `main` after pull request 516. Current behavior: 84 dependency lines in member manifests named a sibling crate by relative `path`; 18 used `workspace = true`; the root manifest declared only `rrflow-kernel`. Moving or renaming a crate therefore meant editing every consumer's path. Target: the root `[workspace.dependencies]` declares every first-party crate that another crate depends on (25), and every member names it with `workspace = true`. Scope: the root manifest, 26 member manifests, one new test in `workspace_architecture/dependencies.rs`, one paragraph of the structure record, this journal. No source file of any crate changes. Failure oracle: the resolved dependency graph or `Cargo.lock` differs, or the new test passes on a direct path. Stop if either happens.

## What changed

- Each line `name = { path = "...", rest }` for a workspace crate became `name = { workspace = true, rest }`, keeping `optional` and `features`.
- `rrd-engine` is declared in the root with `default-features = false`, because `rrflow-edge` uses it without defaults and a member cannot switch defaults off for an inherited dependency. The four members that used the default engine (`rrd-server`, `rrd-client` in its tests, `rrflow-cli`, `rrflow-mcp`) now name `features = ["default"]`.
- New test `first_party_dependencies_are_declared_once_in_the_root_manifest`: no member manifest gives a path for a workspace crate, and the root declares each depended-on crate exactly once at its member directory.

## Verification (Windows, local)

- `cargo metadata --locked --all-features` before and after: the resolve graph (every package's enabled features and dependency edges) is equal. The only declared difference is the four `rrd-engine` lines above. `Cargo.lock` is unchanged.
- `cargo check --workspace --all-targets --locked`: clean.
- `cargo test -p rrd-engine --locked --test workspace_architecture`: 35 passed (34 before, plus the new test).
- The new test was run against a manifest edited back to a direct path and failed, naming `rrd-vector line 15`; the edit was then undone.
- `cargo clippy -p rrd-engine --locked --test workspace_architecture -- -D warnings` and `cargo fmt -p rrd-engine --check`: clean.
- `test_select_ci_lanes.py`, `test_selected_tests.py`, `check_selected_tests.py`, `check_code_shape.py`, `check_documentation.py`: OK.
- Not run: `cargo test --workspace`, the test-name list comparison of section 6.10, Linux, macOS. No Rust source outside the one test file changed and the resolved graph is equal; the candidate gate and the nightly matrix run the rest.

No objective, roadmap or POA&M status changes.
