# Testing whether RRFlow helps an agent plan

**Status:** active implementation evidence; no model has been called and no result exists
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/reasoning-tests`
**Owner:** OBJ-10 / OBJ-11 / J-04 / POAM-010 / PR262

## Package

The repository owner asked on 2026-11-03 whether RRFlow shows accuracy or quality for
planning. The answer was no: nothing measures it. The owner then said "go ahead and get it
100% setup for our testing" and "get RRFLOW tuned and competing".

- **Starting revision:** `9c14e2e2a29b0bd141b6cc70594628d9c117f971`, stacked on #260.
- **Pull request:** [#262](https://github.com/rrflow/rrflow-development/pull/262).
- **Current to target:** the only task evidence was eight trials looking up one integer on
  synthetic repositories. `rrflow-lab` now has a frozen, paired test of real planning work,
  graded mechanically, that writes the lab's ordinary report.
- **Unchanged:** the engine, its public operations and Connectome. `rrflow-eval run` behaves
  as before, and its usage parser moved unchanged to `lab/provider.rs`.
- **Stop conditions:** measuring needs an engine change; a task cannot be stated without
  leaking its answer; a provider cannot run without network or write access beyond what its
  arm declares. None was reached.

## What RRFlow can serve today

This decided the arms. A read of the code found, and direct checks of each point confirmed,
that RRFlow cannot return a repository's content to an agent:

- Attunement phases past connect fail with `phase-unavailable`
  (`rrd-engine/src/engine/attunement.rs`).
- Inventory stores paths, identities, kinds and digests, but no text
  (`rrd-attunement/src/inventory.rs`).
- The project map is "not yet indexed as a recall record" (`gate-d/project-map.md`).
- POAM-001 records that the knowledge package is not yet imported through persistent storage.
- `rrflow_context`, the only MCP tool, works over whatever text is present. Nothing has yet
  been shown to import a repository's text for it.

So `rrflow-attuned` is declared `unimplemented` and stays a visible gap. A harness could import
chunked text through public transaction operations, which would measure BM25 and graph
retrieval but not attunement, parsing or embeddings. That arm is not built yet, and it is
declared as a gap until its path is proven on a real engine.

## Files read

`docs/objectives/rrflow-1.0-alpha.md` (OBJ-10 to OBJ-14), `poam-001.md`, `poam-010.md`,
`gate-j.md`, `engine-data-flow.md` (project evidence and reasoning efficiency),
`rrflow-eval/src/main.rs`, `lab/report.rs`, `engine/attunement.rs`, `inventory.rs`,
`engine/context.rs`, `rrflow-mcp/src/main.rs`, `gate-d/project-map.md`. Outside: rea-lab's README
and study 3 preregistration, leanrea's README, MERIT's abstract.

## Changed paths

Created `lab/tasks.rs`, `lab/provider.rs`, `tests/lab_tasks.rs`,
`eval/lab/extract_package_tasks.py`, `eval/lab/tasks/rrflow-package-planning.json`, the generated
`eval/schemas/lab-task-*-v1.json` and this journal. Changed `lab/mod.rs`, `src/main.rs`,
`src/bin/rrflow_lab.rs`, `tests/lab_contract.rs`, `eval/lab/README.md` and the generated indexes.

## Decisions

- **Research:** method only, and no outside code is adapted. Preregistration, paired exact
  McNemar and Wilson intervals follow rea-lab. The floor arm follows MERIT's leak-verified floor
  and rea-lab's tasks newer than a model's training data. Grading what the agent wrote follows
  MERIT's finding that agents acted on a correctly retrieved value only 55% of the time. Counting
  tokens follows leanrea.
- **Grading:** mechanical, with no model as judge. A plan is solved at file recall of at least
  0.6 and precision of at least 0.5, frozen before any run. These are a judgement, open to review.
- **Isolation:** `claude --bare` keeps a person's memory, hooks and settings out, and it needs
  `ANTHROPIC_API_KEY`, so a real run also depends on that choice.
- **Trace, resource, debugging:** each run keeps its reply by digest, tokens, time, cost and
  exit status. The engine is unchanged.

## Oracle and results

- `cargo test --locked -p rrflow-eval --test lab_tasks`: 18 passed. Scoring on known plans,
  including a padded plan, directories and an empty plan, none of which can solve a task. Exact
  McNemar against hand-computed values. Wilson against the published formula. Nothing runs without
  models and a spend limit, and a run that could pass the total is refused, with the provider not
  called in either case. The floor arm is sent to an empty directory. A prompt holding its answer's
  path or file name is refused. A changed design or a declared gap cannot use a run. A reply with
  no plan, a failed run and a timed-out run stay visible and unsolved. A dry run through a scripted
  provider reaches a valid report. No test can start a model.
- `extract_package_tasks.py` over this repository: 30 tasks, all valid, median answer of 9.5
  files. It skipped 68 pull requests with no behavior section, 67 outside 2 to 25 answer files and
  4 too short.

## Surfaced failures and corrections

- **The first extraction leaked answers that no check could see.** Reading the tasks showed
  briefs running on into an "Exact scope" list of crates, with the paths already removed, which
  nearly states the answer. Extraction now stops before such a list. This is why the floor arm
  exists, since a mechanical check catches only exact paths and file names.
- **Owning records were found in 3 of 30 tasks,** because older briefs format that line
  differently. They now come from everything ahead of the behavior section, and all 30 have them.
- **A brief still names symbols.** Nine prompts mention a crate by name, and some name a type or
  function. A person asking for this work would too, so this stays, and the floor arm measures it.

## Checks not run and remaining errors

- No model has been called, so whether these tasks discriminate between arms is unknown. rea-lab
  found most pairs agreed, so a pilot should come before freezing.
- The manifest has no models and no spend limit, so `tasks-run` refuses and the run order is empty.
- `ClaudeCode` has been tested on its arguments and on parsing a saved result, but it has not
  started a real process.
- The real diff is one valid plan, so file scores under-credit others. Grading does not check that
  a plan would work.
- `rrflow-harness-import` is not built. REA and leanrea are declared not applicable to these tasks.
  A known-answer suite that suits them is not imported.
- Candidate CI has not run.

## Status change

None. No roadmap, objective or POA&M status changes.
