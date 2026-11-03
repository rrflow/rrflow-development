# One file per system and per test

**Status:** active implementation evidence; run through the script on a laptop, and not yet on `clyffy-01`
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-catalog`
**Owner:** OBJ-10 / OBJ-11 / Gate J / PR268

## Package

The repository owner asked on 2026-11-03 for the system to be "truly configurable and can have updated
tests and benchmarks brought in".

- **Starting revision:** `a532c672ea2842447e78cc8fe5d6227e82ec3730`, stacked on #260.
- **Pull request:** [#268](https://github.com/rrflow/rrflow-development/pull/268).
- **Current to target:** 4 manifests repeated 11 to 13 full system bindings each, 11 of the 13 systems
  byte-identical everywhere they appeared, and `run_system.sh` held a `case` per system to start it and
  another to drive it under VectorDBBench. A system is now one file in `eval/lab/systems/` and a test
  one file in `eval/lab/tests/`. The manifests are generated from them, and the script has no case
  per system.
- **Unchanged:** every binding's digest and every test's digest, so runs already made still combine.
  The engine, Connectome and report format.
- **Stop conditions:** a binding's digest would change; a system's start cannot be described without
  shell logic. Neither was reached. RRFlow's own install path stays in the script as one launch kind,
  because it is a sequence of engine commands and not one start.

## What moved

A test may set only a system's index, search settings and availability. Across the 4 tests, the only
override needed is `lance-ivf-rq`'s partition count per corpus, plus its ladder and
`custom-reference`'s availability on some tests. A pin has one owner, so moving to a new release means
editing one file.

## Corrected while moving

- SurrealDB's password was passed as `-e SURREAL_PASS=value`, which put it on the host's process list.
  It now reaches the container by name from the environment.
- A declared gap such as `rrflow-routed` used to start an engine it could not use and then fail. It now
  says it is a declared gap and exits cleanly.
- Milvus's readiness wait used to fall through silently when it ran out, leaving the run to fail later.
  It now stops there and says which address did not answer.

## Files read

`docs/objectives/rrflow-1.0-alpha.md` (OBJ-10, OBJ-11), `gate-j.md`, `lab/manifest.rs`, `lab/driver.rs`,
`src/bin/rrflow_lab.rs`, `eval/lab/run_system.sh`, `eval/lab/probe.sh`, `eval/lab/rrflow_settings.py`, and
all 4 manifests.

## Changed paths

Created `lab/catalog.rs`, `tests/lab_catalog.rs`, `eval/lab/systems/*.json` (13),
`eval/lab/tests/*.json` (4), generated `eval/schemas/lab-system-v1.json` and `lab-test-v1.json`, and this
journal. Changed `lab/mod.rs`, `src/bin/rrflow_lab.rs`, `tests/lab_contract.rs`, `eval/lab/run_system.sh`,
`eval/lab/README.md`, regenerated `eval/lab/vector-*.json`, and the generated index.

## Decisions

- **No shell parsing of values.** The script reads arguments as NUL-separated words from `rrflow-lab
  launch-args`, with no `eval`.
- **Research:** none needed, and no outside code.
- **Trace, resource, debugging:** the script still keeps server logs and host samples per run.

## Oracle and results

- `cargo test --locked -p rrflow-eval`: every suite passes, including `lab_catalog` with 8 tests. Clippy
  with `-D warnings` and fmt are clean.
- The test that resolved each test file against the manifests **as they were before regenerating** passed,
  including every `binding_sha256`, so moving to files changed nothing a report binds to.
- Each container's generated arguments are compared with the arguments the script used, written out in
  the test. A floating image, a port beyond loopback, a password in a file, a readiness check elsewhere,
  a runnable system with no way to start, an unknown field, a test changing a pin, a test naming a
  system with no file, and a system listed twice are all refused.
- A new custom system added only as a file resolves, keeps the test's identity, and gets its arguments.
- Through the rewritten script on a laptop with Docker, with `vector-small` unless noted:

  | System | Kind | Result |
  |---|---|---|
  | `qdrant` | Container | Completed 20 cases with no errors, recall 1.0, request floor 1.14 ms at 8 clients |
  | `custom-reference` | Process | Completed on a 1,000-point copy of the test. On the full test the pure-Python search outran a 7-minute limit, and the process was still cleaned up |
  | `rrflow-routed` | Declared gap | Said so and exited 0, starting nothing |
  | `nobody` | No file | Refused, naming the file to add |

  These prove the start path and not any speed: the laptop was busy.

## Checks not run and remaining errors

- **Not run on `clyffy-01`,** which is waiting on a login approval. Milvus, SurrealDB, Lance and RRFlow's
  own path have not gone through the rewritten script, and they must before this leaves draft.
- Result formats from other harnesses still need code. That is the next package.
- `check_documentation.py` fails on this branch only on POA&M numbering, until #258 lands.
- Candidate CI has not run.

## Status change

None. No roadmap, objective or POA&M status changes.
