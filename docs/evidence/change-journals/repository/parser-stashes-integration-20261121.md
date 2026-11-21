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
