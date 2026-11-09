# C-07 evidence index repair after merge integration

**Status:** active repository-navigation repair; no C-07 performance or release gate passed

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-generated-index-repair-20261109`

**Owner:** repository navigation integrity; [PR #270 integrated verification](https://github.com/rrflow/rrflow-development/pull/270)

## Change brief

**Outcome and owning records.** Preserve discoverability of the C-07 transaction-begin diagnostic in the generated Gate C evidence index. Gate C acceptance remains owned by `docs/roadmap/rrflow-1.0/gate-c.md`; this repair does not change that status.

**Starting revision.** Public development `main` at `ae01ce3287bf64a067c9074f73b2da73541d400d`.

**Current behavior and failure oracle.** Commit `4dbd6a90d56273bf4c49f2c0233901bbac9e031e` added `c-07-arm-begin-phase-evidence-20261107.md` and its generated Gate C index row. The later merge commit `8ca6cbce2707da62ddde956ebeb66392a405712d` integrated main and omitted that row while retaining the record. The protected merge-group for PR #270 then failed `Enforce documentation authority and local links` with: `c-07-arm-begin-phase-evidence-20261107.md: owner index docs/evidence/change-journals/gate-c/README.md does not link the record`.

**Exact scope.** Regenerate the marked Gate C index using `scripts/knowledge/render_navigation.py`; add this journal record; regenerate the nearest index again so this record itself is discoverable. No engine, Connectome, CI, roadmap, POA&M, or historical evidence content is changed.

**Acceptance commands.** `python3 scripts/knowledge/render_navigation.py`; `python3 scripts/knowledge/render_navigation.py --check`; `python3 scripts/ci/check_documentation.py`; `git diff --check`.

**Research, trace, and resource decisions.** No external research or runtime tracing is applicable. The existing repository generator is the owner of the marked index projection. This repair adds no runtime work or resource cost.

**Stop conditions.** Stop if the generator changes anything beyond its marked navigation regions, if unrelated indexes drift, or if documentation policy still reports an orphaned record.

## Execution record

**Files read.** `README.md`; `docs/roadmap/rrflow-1.0-execution-map.md`; `scripts/knowledge/render_navigation.py`; `docs/evidence/change-journals/gate-c/README.md`; `docs/evidence/change-journals/gate-c/c-07-arm-begin-phase-evidence-20261107.md`; the relevant portions of commits `4dbd6a90` and `8ca6cbce`.

**Observed pre-change check.** `python scripts/knowledge/render_navigation.py --check` reported only `docs/evidence/change-journals/gate-c/README.md: generated index is stale`.

**Changed paths and behavior.** `docs/evidence/change-journals/gate-c/README.md` now includes the two existing coordinated records that were missing from its generated region (`c-07-arm-begin-phase-evidence-20261107.md` and `c-07-bounded-transaction-trace-capture-20261107.md`) plus this journal. No source or record content changed.

**Commands and results.** The `python3` Windows app-execution alias resolved to the Microsoft Store prompt, so it was not usable. `python --version` identified the installed Python 3.11 interpreter, used for the repository's specified scripts. `python scripts/knowledge/render_navigation.py` passed and updated only the Gate C README; `python scripts/knowledge/render_navigation.py --check` passed (18 generated indexes, 1,026 nodes, 3,598 edges, 0 stale indexes); `python scripts/ci/check_documentation.py` passed (1,027 document statuses, 1,025 classified coordinates, parent indexes and local links); `git diff --check` passed.

**Checks not run.** Engine, SDK, native capacity, and soak suites are outside this documentation-only package and were not run.

**Remaining errors.** Candidate CI has not run on this repair. PR #270's earlier integrated run `42562405311` was dequeued and cancellation requested after its first verify job exposed the stale index; its overall workflow was still running when last observed. That run does not verify this repair.

**Roadmap status.** Unchanged. C-07 and the alpha remain open.
