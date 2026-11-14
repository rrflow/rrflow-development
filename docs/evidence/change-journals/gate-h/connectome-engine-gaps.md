# Connectome's engine items checked against the source

**Status:** active implementation evidence; documentation only, no engine capability advanced
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-engine-gaps`
**Owner:** OBJ-11 / H-05 / G-02a / POAM-050 to POAM-052 / PR258

## Package

Connectome parking lot 007 listed seven engine items its restored views need, and
the repository owner decided on 2026-11-03 that each view is built on what the
engine serves, with the rest raised as engine work. The owner's addition to
decision 7 made an eighth: configurable search. That list says each item is
checked before it enters the POA&M. This package is that check.

- **Starting revision:** `8f3cbfbe8a52bc2c25e19ec9436ba464d1861515`.
- **Pull request:** [#258](https://github.com/rrflow/rrflow-development/pull/258).
- **Current to target:** three verified gaps were not recorded anywhere; they now
  are. Nothing else changes, including source, gate status and schedule.
- **Unchanged:** every operation, contract and roadmap status.
- **Stop condition:** drop a row if the current source serves or already tracks it.

## Result of the check

| Item | Finding | Disposition |
|---|---|---|
| Listing a task's attempts | `task-attempt-read` needs an id. Attempts sit under a control-store point key, so rrflowQL cannot reach them. | New, [POAM-050](../../../poam/rrflow-1.0-alpha/poam-050.md) |
| Reasoning stage records | `governed-reasoning-runner` is unavailable. | Tracked, Gate G (G-02a) |
| Flight provider arms | `PromptFlightProvider` has only `observe`. | Tracked, [POAM-032](../../../poam/rrflow-1.0-alpha/poam-032.md) |
| A public trace operation | None in the catalogue. | Tracked, H-05 |
| A public source-route operation | None in the catalogue. | Tracked, POAM-032 |
| A link between a change and its audit record | `RuntimeChangeSnapshot` and `AuditRecordSnapshot` share no field. | New, [POAM-051](../../../poam/rrflow-1.0-alpha/poam-051.md) |
| A bounded transaction preview | Previews rebuild the scope. | Tracked, [POAM-043](../../../poam/rrflow-1.0-alpha/poam-043.md) |
| Controls over context ranking | `AssembleContext` takes only seeds, valid time and budgets. | New, [POAM-052](../../../poam/rrflow-1.0-alpha/poam-052.md) |

The ledger must run without gaps, so these take the next free numbers. Draft #253 had
reserved POAM-050 in its branch, and renumbers its own records after these.

## Files read

`docs/poam/rrflow-1.0-alpha.md`, `poam-032.md` (its opening), `poam-049.md`,
`docs/roadmap/rrflow-1.0/gate-g.md` and `gate-h.md` (their matching rows),
`crates/transport/rrd-contract/src/operation_reference.rs` (the operation list and
the task and context entries), `lib.rs` (`RuntimeChangeSnapshot`,
`AuditRecordSnapshot`), `memory_context.rs` (`AssembleContext`), `flight.rs`,
`crates/authority/rrd-engine/src/engine/reasoning_budget/public.rs`
(`read_task_attempt`), and the `[recall]` section of
`crates/authority/rrd-engine/assets/install/default-estate-configuration-v3.toml`.

These were targeted reads, not an audit of those files.

## Changed paths

Created `docs/poam/rrflow-1.0-alpha/poam-050.md`, `poam-051.md`, `poam-052.md` and
this journal. Changed the ledger `docs/poam/rrflow-1.0-alpha.md` and the generated
navigation indexes.

## Decisions

- **Research:** none. The evidence is this repository's own contract and source.
- **Trace, resource, debugging:** not applicable to a documentation change.
- **Priority:** High, matching comparable open records. No gap blocks default
  readiness, and the owner can change that in the ledger.

## Oracle and results

There is no behavior to test. The check is reading the current source, summarised
above, and the repository's documentation checks over the new records. Commands
and results are in the pull request.

## Checks not run and remaining errors

No client was run against these gaps; each record says it is a source finding. No
design for closing them exists yet. Connectome's views use what the engine serves
until one lands.

## Status change

Three new Open rows in the POA&M. No roadmap or objective status changes.

## Public-main reconciliation, 2026-11-06

The draft head `5f1c6cbd9165a6ab07dab0298be3bba6e79816b7` was
merged in an isolated worktree with public main
`1fd7f163de3448c694e09c916b328540303eab1c`; the merge had no conflicts.
The updated review read the three detail records, the task/context operations
in `operation_reference.rs`, `reasoning_budget/public.rs` and
`reasoning_budget/persistence.rs`, `RuntimeChangeSnapshot` and
`AuditRecordSnapshot` in `rrd-contract/src/lib.rs`, `AssembleContext` in
`memory_context.rs`, rank-fusion references in `engine/context.rs`, and the
POA&M detail records matched by an attempted duplicate search. The three
public-contract gaps remain evident in those sources. The gap search did not
prove that no differently worded duplicate exists. This was source review, not a
running-client test; no capability or gate status is newly claimed.

`rg` using a Windows literal wildcard as a path failed with an invalid path;
repeating the search against the directory using `-g` succeeded. This
reconciliation changes this journal beyond the original package; navigation regeneration made no changes. Candidate CI has not yet run for the refreshed head.

Local checks: `python -X utf8 scripts/knowledge/render_navigation.py` passed
(18 indexes, no files changed); `python -X utf8 scripts/ci/check_documentation.py`
passed; `python -X utf8 scripts/knowledge/test_navigation.py` passed all six
tests. The first `git diff --check` found a stray carriage return introduced by
the journal append; the line ending was corrected and the check rerun.
