# Scoped user policies and project configuration

**Status:** active documentation evidence; no runtime capability or gate completed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/scoped-user-policies-20261120`
**Owner:** [ADR-0006](../../../decisions/0006-scoped-policies-and-preferences.md), D-01/D-06 configuration and [POAM-077](../../../poam/rrflow-1.0-alpha/poam-077.md)
**Starting revision:** `8abd7374e8cac17cee7a6480f143292c7155ff86`
**Pull request:** [454](https://github.com/rrflow/rrflow-development/pull/454)

## Change brief and traceability

The owner requested sourced platform terminology for global user constraints,
preferences and minimum requirements, reserving engrams and instances for
projects. ADR-0005 instead prescribed personal and organization engrams and
limited hard policies to organizations/projects. Its generic restrictive-value
wording did not define how minima, maxima or incompatible obligations compose.

ADR-0006 supersedes that design. README, the component map, topology, proposed
UI feature, kernel S1/S6/S7/S8 and K9/K11/K12/K16, Gate D configuration
acceptance and POAM-077 now reference the same scoped configuration model.
User policies and preferences, Project policies and preferences, and Effective
configuration are the product labels. Policies constrain preferences; global
configuration binding never grants cross-project knowledge access. The global
physical persistence owner and resolver remain unimplemented and must be bound
before code. This is an alpha documentation prerequisite, not a new runtime.

Replacement map: ADR-0005's accepted design becomes explicitly historical,
retaining its quotations, source links and original evidence. ADR-0006 is the
sole current configuration decision. The two November 16 direction records
receive current pointers without rewriting their historical quotations. The
November 13 as-built snapshot and already-superseded plans remain historical
observations. No implementation, source module, test or public API was deleted,
moved or replaced; therefore no runtime characterization test is claimed.

Scope is documentation only. Stop conditions are a new persistence authority,
runtime contract change, changed project cardinality, or any completion claim
without implementation evidence. None is introduced. The existing roadmap
order and alpha maturity remain unchanged; POAM-077 gains explicit acceptance
for the corrected requirement and remains open.

## Read coverage and sources

At the starting tree, reviewed AGENTS.md, the session standard, change-authoring
routine and PR template; README identity/status/navigation; objective opening;
roadmap delivery path; ADR-0005; system-overview terminology; instance topology;
code-structure feature layout; kernel configuration/instance/sync/reasoning
sections and packages; Gate D configuration; POAM-077; H-04 client convergence;
the November 16 DevWorld/storage direction records; historical second-mind and
tunable-plan status sections; and the original ADR-0005 journal. Complete
changed source documents and the final diff were reread before commit. This
does not claim a whole-codebase review or requalify the historical assertions
outside the changed subject. Git objects at the starting revision identify
unchanged source; the PR diff identifies changed paths exactly.

Primary research: official VS Code and Claude Code settings documentation for
user/project/managed scopes, PostgreSQL pg_settings for source/bounds/restart
diagnostics, and OpenFeature types for reason/error metadata. Exact URLs and
limits are recorded in ADR-0006. The precedence and policy-composition rules
are explicitly RRFlow decisions, not attributed as universal industry rules.
No third-party source code was adapted and no AI-performance claim was added.

Runtime traces, profiling, resource measurements and new telemetry are not
applicable to a documentation correction. The preceding source review at the
same baseline examined temporal/store snapshot and engine context/endpoint
boundaries; it did not establish a global configuration resolver.

## Oracle, validation and limits

The before/after oracle is the current documentation: active configuration
owners must use the new labels, keep project engram/instance isolation,
distinguish policy enforcement from preference precedence, preserve temporal
provenance, and explicitly retain the runtime gap. Remaining old names occur
in historical decisions, quotations or superseded research/planning records;
they are not compatibility aliases or an implementation lane.

Local validation:

- `python scripts/knowledge/render_navigation.py`: passed; 21 generated
  indexes, 1,288 nodes, 4,555 edges, one journal index updated.
- `python scripts/ci/check_documentation.py`: passed after the parent-index
  repair; 1,291 document statuses and 1,287 classified coordinates validated.
- `git diff --check` and `git diff --cached --check`: passed.
- Focused `rg` inspection of current owners: remaining old names are explicit
  historical quotations or supersession explanations, not current components.
  A source search initially named absent `templates`/`config` directories;
  rerunning against existing `crates` and `connectome/src` returned no matches
  for the retired names or the proposed `scope_ceiling` field (exit 1).

Candidate and merge-group CI attach to PR 454's exact candidate; they cannot
prove a resolver or UI was implemented. Runtime tests, an installed multi-project
configuration journey, UI checks and performance comparisons were not run for
this documentation-only change. The earlier temporal test result is not used
as evidence for the new configuration design.

One inventory-only Python one-liner failed because of PowerShell quoting; it
changed no repository files and was replaced with native PowerShell line counts.
The first documentation check rejected the new decision missing from its parent
index. The decision index is manually maintained (outside the navigation
generator's 21 indexes); it now links ADR-0006 and marks ADR-0005 superseded.
The generator's initial no-change result was not treated as validation.

## Change checklist and handoff

- Outcome, exact documentation scope, baseline, oracle and stop conditions were
  placed in draft PR 454 before the first content edit.
- Existing decision, topology and relevant implementation inventory informed
  the replacement; no code or historical evidence was silently discarded.
- Research, source adaptation, trace/resource/debug and acceptance decisions
  are recorded above; generated navigation is rebuilt from its owner.
- Full diff review covers changed policy semantics, sources, links, scope and
  historical status. No new test mirrors documentation text.
- No gate or objective completes. D-01/D-06 and POAM-077 own the remaining
  persistence/binding/resolver package, followed by generated Connectome views.
- Session scratch files contain the PR body, edit helper and validation output;
  no services, containers, extra worktrees or stashes were created.
