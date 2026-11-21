# Scoped configuration, memory freshness and vectorization review

**Status:** active documentation and research evidence; runtime journey unqualified
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/memory-freshness-vectorization-20261121`
**Owner:** [knowledge lifecycle](../../../reference/data/knowledge-connectors.md#memory-freshness-and-vectorization), [ADR-0006](../../../decisions/0006-scoped-policies-and-preferences.md), D-05/D-06 and H-05
**Starting revision:** `d5fb183dbae31ccbd2498fd642d15e8ae8a4806e`
**Pull request:** [455](https://github.com/rrflow/rrflow-development/pull/455)

## Brief and before/after oracle

The owner requested another documentation pass using current November 2026
practices to reduce rot and connect vectorization. The baseline had separate
scoped-configuration and knowledge-provenance contracts. It did not explicitly
forbid top-k/summary selection of mandatory policies or join their acceptance
with correction-aware context after a project pivot.

The updated owners distinguish context overload, stale knowledge and stale
projections; require exact applicable policy resolution; bind optional vector
work to source/embedding identities; reject stale worker publication; and
measure current evidence and downstream answers, not retrieval scores alone.
The installed oracle focuses on A, pivots to isolated B, corrects A and user
requirements, revokes a source, then returns and repeats after restart.
Late/backdated updates and independent valid/recorded-time reads are explicit.
This corrects the specification; it does not execute or qualify that journey.

Scope: README; ADR-0006; knowledge-connectors; the existing temporal/lexicon
survey's separately dated follow-up; kernel S9; Gate D acceptance; POAM-052/077;
this journal and its generated parent index. No implementation or test is
removed, rewritten or moved. Existing source/ACL/embedding identity behavior
stays at the same engine boundary. No new runtime, database, scheduler or
planning authority is introduced. Stop on runtime/API changes, policy weakening,
new persistence authority or unsupported performance/completion claims.

## Read coverage

Owner context: AGENTS.md session/process rules; README identity and scoped
configuration; ADR-0006 in full; knowledge-connectors in full; the temporal/
lexicon survey in full; kernel configuration/consolidation and Gate D
configuration sections; POAM-052/077 in full. The parent roadmap/objective and
session/change-authoring procedure were retained from the preceding package.
The research index and rrflowDB capability memory/build-order sections were
consulted for ownership. Changed documents and the complete diff were reread
before commit; unrelated historical claims were not requalified.

Code read at the starting revision: `context/assemble.rs` lines 1-420,
`context/candidates.rs` lines 1-160 under `rrd-engine/src/engine`, and
`rrd-contract/src/memory_context.rs` lines 1-78. The contract has query, valid
time, seeds and bounds; assembly captures one stamp, checks snapshot identity,
uses lexical/compatible vector/graph stages and records selected/skipped paths.
This is source inventory, not proof of a global policy resolver, source-derived
embedding lifecycle or joined rot-prevention behavior. Remaining code and
tests were not exhaustively reviewed. A search named nonexistent
`rrd-contract/src/embedding.rs`; `rg --files` located the actual inference and
vector modules. No absent-file result was counted as a read.

## Research and adoption decisions

Primary URLs, versions/dates, inspected sections, findings and limitations are
in the survey's 2026-11-21 follow-up. Read Chroma Context Rot; LongMemEval v2
and RECON v1 abstracts; StaleBench v1 task/results/limitations; LEDGER v1
mechanisms and limits; Qdrant hybrid and incremental embedding documentation.
The current-date cutoff is November 21, 2026, not an assertion of exhaustive
coverage. No upstream source code was copied or adapted. Recent publications
are evaluation inputs, not RRFlow measurements. The original November 16
search-summary survey remains explicitly limited to its original method.

Local design inferences: resolve mandatory policies exactly outside ANN;
invalidate affected eligibility before rebuilding; preserve independent support;
use versioned incremental embedding work; measure freshness and reader outcomes
with held-out comparisons. The sources do not prove these are implemented here.
Their in-order/single-clock assumptions do not substitute for bitemporal tests.
No runtime profiling or new diagnostic implementation was appropriate for this
documentation package; required metrics are assigned to existing owners.

## Validation and remaining work

`python scripts/knowledge/render_navigation.py` passed: 21 indexes, 1289 nodes,
4569 edges; one parent index updated. `python scripts/ci/check_documentation.py`
passed: knowledge ownership, 1292 document statuses, 1288 classified coordinates,
parent indexes and local links. `git diff --check` and `git diff --cached --check`
passed; the complete changed-document diff and new journal were reviewed.
Full candidate and merge-group evidence attach to PR 455's exact revision. Documentation checks
cannot establish token savings, memory correctness or vector quality. No runtime
tests, installed journey, provider/model calls, UI run or benchmark was run.

POAM-052/077 remain open with additional acceptance; no gate/objective/maturity
status changes. Next implementation is still one global configuration
persistence/binding owner and exact resolver, consumed by the existing
knowledge lifecycle and generated Connectome diagnostics. Subsequent packages
must demonstrate the same focus/pivot/correct/return oracle.

## Change-authoring checklist

- Draft PR brief named baseline, exact scope, current/target behavior, owners,
  failure oracle, research, trace/resource decisions and stop conditions before
  the first content edit.
- Existing behavior and docs were mapped to their current canonical boundaries;
  no code replacement requires a new characterization test in this package.
- Sources are directly linked with applicability/limitations; no text-mirroring
  tests or generic claim of all latest optimizations was added.
- Navigation generation, document validation and complete diff review precede
  ready-for-review; candidate/queue gates precede merge and branch cleanup.
- Only a package branch and session scratch PR/helper/evidence files were
  created. No services, containers, extra worktrees or stashes were created.
