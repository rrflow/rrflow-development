# Code-shape ratchet

**Status:** active change evidence

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/code-shape-ratchet-20261113`

**Owner:** rules R1, R2, R3, R5, R6 and R7 of the code structure and conventions standard (kernel package K13, first step)

**Starting revision:** `development/main` at `501a16bc`, after #378.

**Owner direction, 2026-11-13:**
- "holy shit you have to get serious patterns folders if necessary colocated and consistent naming and get these properly fully broken down this is a nightmare".
- "get to work".

**Change:**
- `scripts/ci/check_code_shape.py` classifies every tracked Rust file under `crates/`:
  - `lib.rs` and `main.rs` at most 200 lines;
  - test, bench and example files at most 1,500;
  - all other source at most 800.
- In non-test files it also counts `mod.rs` files, `#[path]` attributes, `Result<_, String>`, `Box<dyn Error>`, allowed `clippy::too_many_arguments`, and `Sha256::new`/`Sha256::digest` outside `rrd-core/src/digest.rs`.
- `code_shape_baseline.json` records what existed at the start:
  - 103 oversize files;
  - `mod_rs`: 10 files;
  - `path_attribute`: 23 sites in 17 files;
  - `string_error`: 70 in 25;
  - `boxed_error`: 89 in 21;
  - `too_many_arguments_allow`: 125 in 57;
  - `direct_sha256`: 41 in 25.
- The check fails when:
  - a new file breaks a rule;
  - a listed file grows or a count rises;
  - an improvement has not been locked in with `--tighten`, which only lowers numbers.
- The check runs in `verify` and in `repository-invariants`, so every candidate runs it.

**Commands:**
- `python3 scripts/ci/test_code_shape.py`: 8 tests OK.
- `python3 scripts/ci/check_code_shape.py`: OK.
- `python3 scripts/ci/check_workflow.py`: OK.
- `python3 scripts/ci/test_ci_lanes_policy.py`: OK.
- `python3 scripts/ci/check_documentation.py`: OK.

**Not done here:**
- No file is split yet. The splits follow one per pull request, starting with `rrd-contract/src/lib.rs`.
- Connectome gets the same check in its own repository.
