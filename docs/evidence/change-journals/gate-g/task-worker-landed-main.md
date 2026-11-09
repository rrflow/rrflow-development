# Worker candidate reconciliation with landed main

**Status:** active source composition; candidate CI and dependency landing pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-landed-main`
**Owner:** G-01b / G-02a / POAM-009; PR229 driver: active SWAG session

## Parents and exact effect

Merge public development main `906bec82c605ed095a2dbf9b0217cd89bd41c05f`
into worker proof head `14b43c93a29557bad47dc8494d001db33cbdb302` through an
ordinary non-fast-forward merge. PR229's description received the bounded brief
before mutation. The worker head contains the
[installed configured-worker evidence](task-worker-installed-live.md); that
trial remains bound to engine `c75c1896` and its separately hashed runner.

Main includes protected landings of PR228 at
`c2a43120375ae9d9d318d48533a42dc05ebe36da` and PR224 at the main revision above.
Their required runs `41216269606` and `41216747041` each passed all 23 jobs.
Those results do not qualify this worker candidate.

Read-only `git merge-tree` and the actual `git merge --no-ff --no-commit`
both composed without conflicts. Before this journal/navigation, the staged
diff contains exactly the Gate D candidate-admission journal and its parent
index: 59 added lines. No production code, fixture, protocol, build dependency
or workflow changes relative to the worker head. `git ls-files -u` is empty.
Preserve prior journals and source histories; no force push or main push.

## Observation and review

GitHub initially reported PR229 conflicting with a base observation of
`f256c8e1`, while direct main-ref lookup and `git fetch development main`
reported `906bec82`. Both observations are retained; no cause is asserted.
The local composition establishes that these exact parents merge cleanly,
not that the whole dependency chain is landed or release-ready.

Reviewed applicable root instructions, CI owner, installed worker record,
both parent revisions, full incoming journal/diff, generated index entry and
unmerged-file state. This is source/evidence reconciliation, so no external
research, tracing, performance claim, implementation relocation or new runtime
authority is needed. Scope is the two incoming documentation files, this new
journal and its regenerated Gate G index. No Rust or provider retest is required
for unchanged production code; this merge does not relabel the old artifact.

Validation uses `python scripts/knowledge/render_navigation.py`,
`python scripts/ci/build_execution_inventory.py` and its `--check`,
`python scripts/ci/check_documentation.py`, and `git diff --check` / staged
whitespace checks. Required candidate/protected CI remains a later gate.
All listed local checks passed: navigation 849 nodes / 3,081 edges, inventory
2,173 paths, and documentation 850 statuses / 848 coordinates. Exact command
exits are retained in SWAG's
`.swag/evidence/20261031/worker-installed-inputs/main-composition-validation.json`.
No roadmap or POAM status closes. PR222's native-client dependency still needs
ordinary composition, including preservation of both installed-runner modes;
the full UI/project-effect/RROFLOW/learning objective remains incomplete.
