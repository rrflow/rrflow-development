# C-07 candidate consolidation and merged branch cleanup

**Status:** active integration evidence; candidate CI and capacity acceptance remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-pr-consolidation`
**Owner:** OBJ-02 / C-07 / POAM-035 / PR139

The owner requested consolidation into a real baseline rather than accumulating
PRs and branches. PR139's pre-effect brief recorded this bounded administrative
package and its source/test preservation map. Driver: current Codex session.
Starting candidate `09756751cec9342b94f34199a196a6e1c63b5c01`; public main
`9a441efd5cbef924fc01f6cc47b6e6c5c844d4e6`. No runtime, tests, public API,
configuration, acceptance limits or frozen native candidate change here.

## Implementation preservation map

| Superseded package | Canonical destination in PR139 | Replacement evidence |
|---|---|---|
| PR134 `4e2dbaffad29039d742f6b151f7636378ce3e7fe` | Existing `rrd-store/src/access/{mod,proof_nodes,versioned_read}.rs`; `rrd-store/tests/direct_read_paths.rs`; `rrd-engine/tests/runtime_query_trace.rs` | All five files compared byte-for-byte with that revision. All 425 immutable journals in its source tree are also preserved byte-for-byte. |
| PR141 `3b28e5b75db461e4bddb8ee88fb0d5da4a56d66f` | Existing `rrd-lsm/src/database.rs`, private `database/point_index.rs`, `tests/point_read_selection.rs` | All three files and all 421 immutable journals in that tree compared byte-for-byte. The linked implementation journal owns 229 passing storage tests and strict lint; the integrated installed smoke passed at frozen `7c329d49`. |

Both package heads are ancestors of PR139, but ancestry is only an additional
check: source, tests and immutable evidence were compared directly. No source
is moved or discarded. The [indexed integration](c-07g-indexed-native-candidate.md)
and [native launch](c-07g-indexed-native-launch.md) records retain exact behavior,
commands, failures and workload limits. PR134/141 can now close as superseded
into this published destination. Such closure is not a claim of landing on
main; their remote refs remain until main contains their work. PR139 is the
single C-07 implementation candidate and must pass full candidate CI.

## Merged branch cleanup

The development destination was resolved as
`https://github.com/rrflow/rrflow-development.git`. Tooling selected 97 remote
`agent/` refs whose exact tips matched merged PR heads, were reachable from
the fetched main above and had no open PR. Immediately before deletion,
`git ls-remote --heads development` verified every remote tip still matched.
One ordinary `git push development --delete` removed exactly those 97 refs,
exit zero. No force option, history rewrite, official-repository push, worktree
removal or estate deletion occurred. All commits remain reachable from main.

The exact remote/ref/revision/main/merged-PR receipt is retained locally in
`target/merged-branch-cleanup-plan.json`, SHA-256
`1fa04fb2a444c41f7781add44e70cb1f767ec03e0a95f43b1ce3f2144996e70c`;
command output is `target/merged-branch-cleanup-result.log`. Source comparison
results are `target/c07-pr-consolidation-provenance.json`. These mechanical
records avoid a duplicate manually maintained branch inventory.

## Review, checks and remaining work

PR140 head `b623bd441d5b83abefcae845f872688767eeab39` passed all candidate
checks including `pipeline / ci-gate` in run 40326533968 and was admitted to
the normal merge queue with its exact expected head. Merge-queue qualification
is still required; no bypass or completed merge is claimed here.

Read/review scope: README, CI operation contract, traceability procedure,
source PR briefs/journals, complete changed-path comparisons and branch/PR
state. Research: no algorithm change; repository and GitHub evidence suffice.
Resource/debug decision: preserve the running native estate, original limits
and source `7c329d49`; no extra runtime collection. No new Cargo execution for
this evidence-only package. Repository documentation/navigation/inventory,
workflow, formatting and whitespace checks passed before candidate push.

Checklist: pre-effect brief, exact source/test destinations, independent file
comparison, preserved failures/evidence, safely bounded ref deletion, recorded
commands, full diff review, generated projections and explicit noncompletion.
Stop on a changed/unmerged tip, lost source/test/evidence or a weakened gate.
C-07 and POAM-035 remain open; no alpha, full-corpus or 24-hour result is inferred.
