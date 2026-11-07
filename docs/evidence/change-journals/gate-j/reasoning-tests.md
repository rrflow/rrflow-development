# Testing whether RRFlow helps an agent plan

**Status:** active implementation evidence; no model has been called, and the only result is a retrieval measure
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

So `rrflow-attuned` is declared `unimplemented` and stays a visible gap. The owner approved
building an arm over text a harness imports through public transaction operations, labelled as
measuring BM25 and graph retrieval and not attunement, parsing or embeddings. That arm,
`rrflow-harness-import`, is now available, because its path is proven below.

## A first signal, with no model

`eval/lab/retrieval_signal.py` installs a fresh engine for each task, imports the repository's
text at the task's starting revision, and sends the task's text to the real `rrflow-mcp` process
over the protocol an agent uses. It scores the files returned against the files the change touched,
beside textbook BM25 over the identical chunks. Thirty tasks, one engine and client build each:

| Mean share of needed files found | Top 10 files | Top 50 files |
|---|---|---|
| `rrflow_context` | 0.284 | 0.484 |
| Plain BM25 over the same chunks | 0.279 | 0.454 |
| Picking at random | 0.008 | 0.041 |

Report SHA-256 `fea62018be15eac2c6976f29cacb2c7bf70893254da456586e87e05b05669fdb`, kept with the
session's working files. Its engine carries #259's fix and its client #263's.

What this shows and does not show:

- The path works, and the tool finds about half of what a change needs within 50 files, far above
  chance. It does not show that an agent would use the result or plan better with it.
- RRFlow and plain BM25 are close. In the top 50, RRFlow was higher on 9 tasks, BM25 on 5, and 16
  tied, which does not separate them. That is expected, because the engine is using BM25 here and
  no graph relations were imported, so nothing beyond BM25 had anything to work with.
- Both are weak on some tasks: each found none of the needed files in its top 50 on 2 tasks.
- The query is the brief's text. An agent would write its own query.
- Files a change created did not exist yet, so only files present at the starting revision score.

Things it found about the tool itself:

- A real client refused every real answer until #263, because floats changed on the wire. That is a
  defect, found here and fixed in its own package.
- The tool's schema offers 512 items and 786,432 output bytes, while an installed engine refuses
  more than 128 and 524,288. A caller following the schema is refused.
- A call for 128 items returns about 320 KB, which is roughly 80,000 tokens if it reaches a model
  unchanged. Every call was truncated at the item limit. The arm's preface asks for few items, and
  whether agents listen is something a pilot has to show. This is the cost leanrea's study warns of.
- A query took about 2 seconds over about 10,000 chunks on a laptop, because BM25 is rebuilt from
  the snapshot for each request. This is rough, and it would grow with a project.

## Files read

`docs/objectives/rrflow-1.0-alpha.md` (OBJ-10 to OBJ-14), `poam-001.md`, `poam-010.md`,
`gate-j.md`, `engine-data-flow.md` (project evidence and reasoning efficiency),
`rrflow-eval/src/main.rs`, `lab/report.rs`, `engine/attunement.rs`, `inventory.rs`,
`engine/context.rs`, `rrflow-mcp/src/main.rs`, `gate-d/project-map.md`. Outside: rea-lab's README
and study 3 preregistration, leanrea's README, MERIT's abstract.

## Changed paths

Created `lab/tasks.rs`, `lab/provider.rs`, `lab/knowledge.rs`, `tests/lab_tasks.rs`,
`eval/lab/extract_package_tasks.py`, `eval/lab/retrieval_signal.py`,
`eval/lab/tasks/rrflow-package-planning.json`, the generated `eval/schemas/lab-task-*-v1.json` and
this journal. Changed `lab/mod.rs`, `lab/rrflow.rs`, `src/main.rs`, `src/bin/rrflow_lab.rs`,
`tests/lab_contract.rs`, `eval/lab/README.md` and the generated indexes.

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

- `cargo test --locked -p rrflow-eval --test lab_tasks`: 22 passed. Scoring on known plans,
  including a padded plan, directories and an empty plan, none of which can solve a task. Exact
  McNemar against hand-computed values. Wilson against the published formula. Nothing runs without
  models and a spend limit, and a run that could pass the total is refused, with the provider not
  called in either case. The floor arm is sent to an empty directory. A prompt holding its answer's
  path or file name is refused. A changed design or a declared gap cannot use a run. A reply with
  no plan, a failed run and a timed-out run stay visible and unsolved. A dry run through a scripted
  provider reaches a valid report. An arm with an MCP tool must be given its server, and no other
  arm may be. Chunking never loses text and splits a long line on a character boundary. Binary files,
  lock files and a test's own answers are never imported. No test can start a model.
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
- **The measuring path found a real defect, and stopped.** As the brief's stop condition says, the
  float failure needed an engine-side change, so it went to its own package, #263.
- **One import failed and could not be explained.** The run stopped at `pr-63` while other builds
  were using the machine, and the driver crashed before keeping the error. The driver now records a
  failed import in the report, and the task imported on the next try. No later task failed.

## Checks not run and remaining errors

- No model has been called, so whether these tasks discriminate between arms is unknown. rea-lab
  found most pairs agreed, so a pilot should come before freezing.
- The manifest has no models and no spend limit, so `tasks-run` refuses and the run order is empty.
- `ClaudeCode` has been tested on its arguments and on parsing a saved result, but it has not
  started a real process. Whether Claude Code connects to `rrflow-mcp` is unverified: the driver
  speaks the same handshake, but it is not Claude Code.
- The driver that would run a real study across checkouts and engines is not written. It waits on
  the owner's choice of models and spend limit.
- The real diff is one valid plan, so file scores under-credit others. Grading does not check that
  a plan would work.
- REA and leanrea are declared not applicable to these tasks. A known-answer suite that suits them
  is not imported.
- Candidate CI has not run.

## Status change

None. No roadmap, objective or POA&M status changes.
