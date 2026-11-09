# Configuring a study, and proving its data was captured

**Status:** active implementation evidence; no model has been called, and capture is proven only against a scripted provider
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/study-requirements`
**Owner:** OBJ-10 / OBJ-11 / J-04 / POAM-010 / PR267

## Package

The repository owner asked on 2026-11-03 for the system to be "truly configurable and can have
updated tests and benchmarks brought in". Of the recommended planning pilot, the owner said "we want
ui to configure this and prvide a baseline and have schema that defines minimal requirements that
will ensure that we have this aligned to research and proven to provide and ensure the data is
getting captured". That did not choose a model or a limit, so the pilot still has neither.

- **Starting revision:** `ba16cefdda1fc8b9d8a402b4f5a950af5ded9739`, stacked on #262.
- **Pull request:** [#267](https://github.com/rrflow/rrflow-development/pull/267).
- **Current to target:** a study was a manifest edited by hand, field validation said only that it
  parsed, and data capture could fail silently. A study is now a small config over a task catalog.
  Requirements citing their sources say what any study must do, a check reports each one, a plan is
  refused until the binding ones are met, and an audit reads what the runs actually hold.
- **Unchanged:** the engine, its public operations, Connectome, and `rrflow-eval run`.
- **Stop conditions:** capture needs an engine change; a required field cannot be obtained without
  calling a model to find out; a requirement cannot be stated without inventing a citation. None was
  reached.

## What it found

Reading the run path for this work found two defects that would have changed results:

| Defect | Effect | Now |
|---|---|---|
| `run_one_task` summed spend with `filter_map` over `cost_usd` | A run with no captured cost counted as $0, so if a provider's output changed shape, the total limit stopped limiting | Such a run is charged at its per-run limit |
| A failed MCP server was not noticed | The agent carried on without RRFlow, and the run counted as an unsolved pair against the RRFlow arm | The run is marked `tooling_failed`, stays visible, and counts neither way |

Tests for both were committed failing at `4609f3c5`, and fixed at `b90021d9`.

## The design

| Piece | Owner |
|---|---|
| What a person or a screen edits | `lab-study-config-v1`, for example [`rrflow-planning-pilot.json`](../../../../eval/lab/studies/rrflow-planning-pilot.json) |
| What any study must do, and why | [`study-requirements.json`](../../../../eval/lab/study-requirements.json), 12 requirements, each graded per stage |
| Checks | A closed list in `lab/study.rs`. A requirement cannot name a check outside it, or cite a source outside `sources.json` |
| What a run owes | `CaptureField`, 9 fields frozen into the plan. A finished run lacking one stays visible and does not count |
| Proof | `rrflow-lab tasks-audit`, which reads the runs and trusts nothing about how they were made |

A pilot may run although it cannot answer the question, and its report says so: its verdict on
whether an arm helped is always `not_run`, however large the difference. A confirmatory study must
have enough distinct tasks for exact McNemar to reach its alpha, and must name an audit proving
capture with the same model and tools. Repeating a task does not count as another task.

Retrieval is captured apart from use, following MERIT: how many needed files the tool returned, and
how many of those the plan named. Identities are read only from MCP tools' replies, so a file that
happens to mention one does not count as a retrieval.

## Files read

`docs/objectives/rrflow-1.0-alpha.md` (OBJ-10, OBJ-11), `poam-010.md`, `gate-j.md`, `lab/tasks.rs`,
`lab/provider.rs`, `lab/sources.rs`, `lab/report.rs`, `lab/knowledge.rs`, `src/bin/rrflow_lab.rs`,
`tests/lab_tasks.rs`, `eval/lab/tasks/rrflow-package-planning.json`, `eval/lab/retrieval_signal.py`,
`claude --help`, and Connectome parking lot 007 for the owner's minimal-schema decision.

## Changed paths

Created `lab/study.rs`, `tests/lab_study.rs`, `eval/lab/study-requirements.json`,
`eval/lab/studies/rrflow-planning-pilot.json`, generated `eval/schemas/lab-study-config-v1.json`,
`lab-study-requirements-v1.json`, `lab-study-check-v1.json`, `lab-capture-audit-v1.json`, and this
journal. Changed `lab/mod.rs`, `lab/provider.rs`, `lab/tasks.rs`, `src/bin/rrflow_lab.rs`,
`tests/lab_tasks.rs`, `eval/lab/README.md`, generated `lab-task-manifest-v1.json` and
`lab-task-run-v1.json`, and the generated index.

## Decisions

- **Research:** method only, and no outside code is adapted. Each requirement names its source.
  Requirements with no outside source, such as pinned models and spend, cite the owning objective or
  a lab rule, and say so.
- **Parsing:** `claude --output-format stream-json --verbose` replaces the single JSON result, so tool
  calls, MCP state and the answering model are available. A field that cannot be found stays absent
  and is never guessed.
- **Credentials:** a study holds a variable's name. A check refuses anything else without repeating
  it, so a pasted key does not end up in a report.
- **Trace, resource, debugging:** each run keeps its whole event stream by digest, so a capture gap
  can be diagnosed afterward. The engine is unchanged.

## Oracle and results

- `cargo test --locked -p rrflow-eval`: every suite passes, including `lab_study` with 23 tests and
  `lab_tasks` with 22. Clippy with `-D warnings` and fmt are clean. No test can start a model.
- Tests cover: every checked-in requirement cites something that exists; an unknown source, no
  citation or an unknown check is refused; the recommended pilot is ready and says it cannot answer;
  a confirmatory study with 5 tasks is refused; the least one-sided pairs match the exact test (6 at
  alpha 0.05, 8 at 0.01); aliases are refused; spend must cover the worst case; a key in place of a
  variable name is refused and not repeated; a changed catalog stops the check; a baseline holding the
  capability or a floor with access is not met; each of 7 missing fields fails the audit and stops the
  run counting; a different model answering fails the audit; a failed run owes no data; a proof must
  cover the study naming it; a study goes from config to report through a scripted provider.
- With no model, on the real catalog:

  ```text
  $ rrflow-lab study-plan --config eval/lab/studies/rrflow-planning-pilot.json ...
  this study is not ready. pinned-model: No model is chosen. spend-covers-plan: No spend limit is
  set. credential-by-name: No credential variable is named.

  with claude-sonnet-5-5, $2 per run, $30 total:
  15 runs across 5 tasks, 3 arms and 1 models; at most $30
  exact McNemar needs at least 6 one-sided pairs, so this design cannot reach significance
  ready
  ```

  The plan's `tasks-order` lists 15 runs, and `tasks-audit` over no runs reports `incomplete` and exits 1.
- The design page's checks are a preview written in JavaScript. A throwaway comparison ran them beside
  `rrflow-lab study-check` on 16 configurations: 192 requirement results, no differences in status,
  wording of a met requirement, run count, worst case or pairs needed.

## Checks not run and remaining errors

- **Capture has not been proven on a real run.** Claude Code's event format comes from its
  documentation and a saved single-result shape. No model may be called yet. The first real run per arm
  is the proof, and any mismatch shows as missing data.
- Whether Claude Code connects to `rrflow-mcp` is still unverified. If it does not, those runs now mark
  themselves instead of counting against the arm.
- A floating name is detected textually: an alias containing a digit would pass the check. The audit
  still catches a different model answering.
- The driver that runs a whole study across checkouts and engines is not written.
- Connectome's screen is not built. A design page is up for the owner's review, and design review comes
  before a slice.
- Benchmarks are not yet this configurable: systems and tests repeat across manifests, starting one
  needs a case in `run_system.sh`, and each result format needs code. That is the next package.
- `check_documentation.py` fails on this branch only on POA&M numbering, until #258 lands.
- Candidate CI has not run.

## Status change

None. No roadmap, objective or POA&M status changes.
