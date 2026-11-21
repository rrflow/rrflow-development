# Recovered parser stashes integration

**Status:** active integration; candidate and queue pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/parser-stashes-integration-20261121`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), D-04 and [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md)

Baseline 15ed675d403b30eabfe69882bda39e1a0ce34c6c. Integrate the three
successive recovered D-04 stashes through ordinary merges, oldest first:
a105804f84e80858b86050009dfbcbd606f34894,
c5491c0613d419439ad21ca91856fdb9c1f1337e and
c53cf21040ec500c280b5d0e476311ec6da31117. Include the last stash's formerly
untracked worker source and two tests from 9aa899a75f89e226745a52e9844ec51572fb28e2.
The first two third-parent trees contain no files.

The recovered code implements source-bound parsing, policy admission, anchored
reads and worker framing. Current main has later publication, native process
isolation, module splits, generated contracts and a partial-header repair.
Retain those later capabilities while integrating each recovered behavior.
Do not restore the old incomplete Parse dispatcher, old Estate identity, stale
SDK digests or a single-request worker over the current bounded batch protocol.

Scope: the stash-touched parser, worker, installation, attunement, contract,
CLI tests and their current split modules; generated projections only when
required by actual owning-source changes; this journal and generated index.
Map removed inline code to its current modules while resolving conflicts.
Read the complete recovered D-04 journal, current worker and Gate D owner
before implementation; read source and tests at each affected boundary.

Oracles: source digest/range and limit assertions; malformed/partial worker
frames; committed Inventory source identity after reopen; real installed
lifecycle Parse/checkpoint test. Run focused worker/parser tests, attunement
and contract suites, engine attunement tests and installed lifecycle, followed
by formatting, documentation and required candidate/queue checks. Stop on a
lost assertion, weakened admission/isolation, unknown canonical destination or
red gate. Preserve exact failures and correct them in this package.

No new provider, upstream adaptation, external research, benchmark or runtime
trace is needed for source integration. This is not a claim of completed D-04,
whole-project prompting or alpha readiness. Record source deltas and actual
results below; other recovered packages remain outstanding.

## Resolution evidence

PR463 merges a105804f as a1b864b1, c5491c06 as 0883814a and c53cf210 as
a7c93595. All three original commits and their index/untracked parents remain
reachable. Their application behavior was already carried forward in main;
after resolving the module splits no application-source delta remains. This
adds the missing ordinary integration history, not a new parsing capability.

| Recovered boundary | Canonical destination and retained behavior |
|---|---|
| parse_source and SyntaxNode proposals | parse.rs plus facts.rs: same locked Rust/Python/Go grammars, source digest, exact symbol ranges, error counts, cancellation and limits; current fact/chunk output retained |
| Inline SourceParsePolicy | rrd-contract/src/attunement/source_parse.rs and phase plan: ordered bindings, safe wire integers, per-file/aggregate/commit bounds and policy-bound plan digest |
| Inline install policy admission | installation/profile.rs, plan.rs and validate.rs: preview and sealed-preview admission retained |
| Inline selected-source reader | installation/inventory/source.rs and tests.rs: authenticated inventory entry, anchored file identity, charged reads and disposable output retained |
| Inline engine policy/reopen tests | engine/tests/attunement/parse_policy.rs: policy readback, invalid checkpoint/source rejection before reads, rrflowQL traversal and altered-source rejection retained |
| installed_lifecycle.rs | tests/installed_lifecycle/replan.rs and main.rs: actual installed process must checkpoint Parse after restart |
| workspace_architecture.rs | tests/workspace_architecture/dependencies.rs and main.rs: pure parser dependency boundary and pinned grammars retained |
| attunement_contract.rs | tests/attunement_contract/operations.rs and main.rs: both recovered policy validation/schema tests retained |
| Three formerly untracked worker files | Same paths: current v2 source-fact protocol, bounded response validation, batch processing and partial-header repair retained; original four library and two executable test behaviors retained |

The three removed old inline test paths only existed in the incoming merge
index; their current directory/main.rs targets remain. An automatically merged
duplicate load_parse_inventory re-export was removed because the current
attunement module already exports it. Retained current lockfile, catalogue,
SDK projections and later documentation/POAM evidence rather than restoring
old digests or claiming old draft results as current results.

Read all three formerly untracked files, current worker and both worker tests,
current facts implementation, selected-source reader, contract parse policy,
engine parse-policy tests and parser tests. Read incoming changed source hunks,
current profile admission and installed checkpoint call site. The complete
recovered D-04 journal was read; unrelated source was not exhaustively reviewed.

Focused tests and owning suites are in progress. No test assertion was removed
or relaxed. Candidate CI and merge queue remain required.
