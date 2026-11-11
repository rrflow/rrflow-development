# D-02 P3: Parse executor

**Status:** active implementation series; D-02, OBJ-07 and POAM-059 remain open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-p3-parse-executor-20261111`

**Owner:** OBJ-07 / OBJ-14; [Gate D project-workflow packages](../../../roadmap/rrflow-1.0/gate-d.md#project-workflow-build-packages); package P3 of the [D-02 evidence-driven attunement plan](../../../roadmap/d02-evidence-driven-attunement-engineering-plan.md)

**Starting revision:** main `b28c5719`.

**Current behavior.** `rrd-attunement` parses one file into a flat list of named nodes (`parse_source`, `ParseProposal`) and the worker protocol carries that list. The Windows and Linux supervisors bound a child process, but nothing launches the parser through them. `EXECUTABLE_PHASES` is Connect and Inventory, so a derived plan never selects Parse.

**Target behavior.** Every Rust, Python and Go file an Inventory recorded is parsed by the primary binary's worker under native supervision. Definitions and references from each grammar's tag query, top-level spans and AST chunks are published as `source-symbol`, `source-span` and `source-chunk` records, with a per-file `source-parse` state, in the transaction that carries the job's checkpoint or lease renewal. A restart resumes from the last commit, an unchanged file is not parsed again, and a deleted file's facts are retired.

**Series.** The package lands as pull requests of at most 400 changed lines, each leaving main green: the parse step as its own function; symbol facts; native supervision for the worker; spans and chunks; worker protocol v2; the parsed-source records; the executor; enabling Parse with the oracles and the real run. Each adds its section below.

## The parse step as its own function

- **Change.** `parse::parse_tree` validates the request and returns the tree, grammar revision, start time and deadline; `parse_source` runs its unchanged walk on that tree. The facts extraction that follows reuses one bounded parse. No behavior change.
- **Oracle.** Characterization only: `tests/parse.rs` (7) and `tests/worker.rs` (6) pass unchanged.
- **Commands.** `cargo test --locked -p rrd-attunement`: 49 passed, 0 failed. `cargo clippy --locked -p rrd-attunement --all-targets -- -D warnings`: clean.
- **Not run.** Linux and macOS; candidate CI is the oracle there.
