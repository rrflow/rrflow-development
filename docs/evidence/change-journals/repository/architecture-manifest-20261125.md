# One architecture manifest and one checker

**Status:** active landing evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/architecture-manifest-20261125`
**Owner:** [Code structure and conventions](../../../architecture/code-structure-and-conventions.md), sections 6.8 package 2, 6.11 and 6.12; kernel package K13.

## Change brief

Baseline `aa8bdc907` (`main` after pull request 517). Current behavior: where a crate sits and what it may depend on was written in several places. The `workspace_architecture` test named 63 crate folders by literal path and kept two hand-written lists of the components below the engine; `check_code_shape.py` held its own limits and the digest owner's path; no check knew the tiers of section 6.2, the naming rule of 6.1 or the standalone rules of 6.12. Target: `crates/architecture.toml` lists every crate's tier, folder, name and standing once; `scripts/ci/check_architecture.py` fails the candidate gate on a difference; the two existing checks read the manifest or find crates by package name. Scope: the manifest, the checker and its test, `check_code_shape.py`, the `workspace_architecture` test files, one `toml` dev-dependency of `rrd-engine`, two lines in each of two CI jobs, the structure record, the session standard's check table, this journal. No crate source changes. Failure oracle: the checker passes on a tier, name, folder or standalone violation; or a test of `workspace_architecture` passes without checking anything because its list came back empty. Stop if a rule cannot be made a hard error without first changing code.

Research decision: none needed; the rules are the plan's. Trace and debugging decision: none; no runtime path changes.

## What changed

- `crates/architecture.toml`: 32 crates in six numbered tiers plus evaluation, the code-shape limits, the naming rule, the banned module names and the digest owner. A crate not yet at its target records `target_name` or `target_folder`: 12 renames, 17 moves. `rrd-maintenance` is marked for removal. `rrflow-cld` records one standalone exception, its dev-dependency on `rrd-store`.
- `scripts/ci/check_architecture.py`: the rules listed at the top of the manifest, as hard errors. A target record that no longer applies is itself an error, so the pending list can only shrink.
- `scripts/ci/test_architecture.py`: 14 tests, one or more per rule, each asserting the rule fails on its violation, and one asserting the recorded manifest matches this workspace.
- `check_code_shape.py` reads its three limits and the digest owner from the manifest.
- `workspace_architecture`: the 63 literal crate paths became `metadata.in_crate("<package>", "<path>")`, which finds the crate's folder from Cargo. The two lists of components below the engine are read from the manifest (every crate in a tier below `rrd-engine` that is not a daemon crate). The derived list adds `rrd-maintenance` to the earlier hand-written one. New test `components_below_the_engine_come_from_the_architecture_manifest` guards against an empty or wrong list.
- Both CI jobs that run the code-shape check now run the architecture test and checker after it.

## Measured while writing the manifest

- No crate has a normal or build dependency on a higher tier today, and no same-tier dependency points downward in the list, so the tier rule landed with no exception.
- No source module is named `utils`, `common`, `helpers` or `misc`, so that rule landed with no exception.
- The 6.12 table was wrong in two cells: `rrflow-arrow` depends on `rrflow-kernel`, and `rrflow-cld` depends on `rrflow-kernel` as well as `rrflow-arrow`. Both are standalone crates, so rules 1 and 2 still hold; the table is corrected.

## Verification (Windows, local)

- `python scripts/ci/check_architecture.py`: `OK: 32 crates; pending: 12 renames, 17 moves, 1 removals, 1 standalone exceptions`.
- `python scripts/ci/test_architecture.py`: 14 passed.
- Against the real workspace with the manifest changed in memory: `rrd-store` set to tier 0 fails with "depends on rrd-lsm in the higher tier 1"; removing the `rrflow-cld` exception fails with "is standalone but has a dev dependency on rrd-store"; removing `rrd-query`'s `target_name` fails with "rename it or record `target_name`".
- `python scripts/ci/check_code_shape.py` and `test_code_shape.py` (9 tests): OK, the same 2 oversize files and 51 pattern sites as before.
- `cargo test -p rrd-engine --locked --test workspace_architecture`: 36 passed (35 before, plus the new test). `Cargo.lock` is unchanged.
- `cargo clippy -p rrd-engine --locked --test workspace_architecture -- -D warnings` and `cargo fmt --all`: clean.
- `check_workflow.py`, `test_ci_lanes_policy.py`, `check_selected_tests.py`: OK. `select_ci_lanes.py` on this change selects the scripts and engine lanes.
- Rule 3 of 6.12, each standalone crate's tests run alone with `cargo test -p <crate> --locked`: `rrflow-kernel` 103 passed, `rrflow-arrow` 2 passed, `rrflow-cld` 12 passed, `rrd-lsm` 276 passed; none failed. The `rrflow-cld` run builds `rrd-store` for its one excepted test, so it does not yet prove rule 3 for that crate.
- Not run: `cargo test --workspace`, Linux, macOS. The manual nightly on `aa8bdc907` (run 45750468323) was still running when this was written.

## Not done in this package

The structure record, section 6.11, lists each rule that is not yet enforced and the package that adds it. The largest is the 391 file paths in `scripts/ci/build_execution_inventory.py`, which every crate move still edits.

No objective, roadmap or POA&M status changes.
