# ADR-0006: Scoped policies and preferences

**Status:** active accepted architecture decision; resolver and global persistence binding not implemented
**Coordinate:** `rrflow://rrflow-instance/data/decision/0006-scoped-policies-and-preferences`
**Owner:** global user configuration, project configuration, policy enforcement and effective-value provenance
**Decision date:** 2026-11-20
**Supersedes:** [ADR-0005](0005-second-brain-personal-and-org-engrams.md)

## Context and terminology

The owner requires platform terminology for the user's overall constraints,
preferences and minimum requirements, with an engram and RRFlow instance
reserved for each project. This replaces ADR-0005's personal/org engram design.
SecondBrain, DevUniverse, DevWorld and DevU are historical names, not current
product components or alternate APIs.

Use **User policies and preferences** in documentation and UI for global user
configuration, **Project policies and preferences** for project configuration,
and **Effective configuration** for the resolved result. The architectural
term is **scoped configuration**. A user profile may describe identity or a
saved preset elsewhere; it is not the name of this enforcement boundary.

| Term | Meaning in RRFlow |
|---|---|
| User scope (global) | One identified user's policies and preferences across projects explicitly bound to that user configuration. Global does not mean every account, every user or unrestricted sharing. |
| Project scope | Requirements and preferences for one project, recorded in its engram and served by its RRD instance. |
| Policy | An enforced constraint: a prohibition, required behavior, minimum requirement, maximum budget, allowed set, denied set or locked value. Users as well as projects and authorized organizations may author policies. |
| Preference | An overridable default or guidance within applicable policies; for example language, answer detail or preferred reasoning effort. |
| Effective configuration | The values and obligations admitted for a particular user, project, operation and temporal read, with their sources and explanations. |
| Organization policy | Optional, explicitly bound requirements from an authorized organization. An organization is not a required parent of a local installation. |

## Primary sources and limits

Reviewed during the 2026-11-20 documentation session. These sources support
terminology and design patterns; RRFlow's exact composition rules below are
its own decision, not a claim of an industry-wide algorithm.

| Source | Supported pattern | Limit of the analogy |
|---|---|---|
| [VS Code user and workspace settings](https://code.visualstudio.com/docs/configure/settings) | User settings apply across workspaces; project settings and managed policy have distinct scopes and precedence. | RRFlow's project is its own identity; a VS Code workspace is not an RRFlow engram or authorization boundary. |
| [Claude Code settings files and precedence](https://code.claude.com/docs/en/settings) | User, project and managed settings distinguish personal defaults, shared project configuration and organizational controls. | Precedence and merging vary by setting; RRFlow does not copy provider file formats or assume all settings use nearest-scope precedence. |
| [PostgreSQL pg_settings](https://www.postgresql.org/docs/current/view-pg-settings.html) | Effective values expose units, source, allowed bounds and whether restart is pending. | This is diagnostic precedent, not a global user-policy resolver. |
| [OpenFeature evaluation details](https://openfeature.dev/specification/types/) | Evaluation results can carry reasons, errors and metadata. | Feature-flag evaluation is not authorization; RRFlow requires policy provenance independently. |

## Scope, ownership and isolation

The [instance topology](../architecture/instance-topology.md) remains one
project, one engram and one logical RRD instance. User configuration is a
distinct logical configuration scope, not a personal project, personal engram,
organization engram or mandatory extra daemon. Project knowledge, scratch work,
memories and reasoning state stay in the project's engram.

Global user configuration must have one RRFlow-owned durable source, survive
project shutdown and remain usable by authorized project instances. Resolution,
authorization and governed changes belong at `RrdEngine`; Connectome and
plugins consume public operations. They cannot own a second editable copy or
silently promote project records to global scope. A cache is a revision-bound
projection, never a new authority.

The physical persistence owner, installation lifecycle and authenticated
cross-instance binding for that global source are **not implemented or fixed
by this logical decision**. Before implementation, D-01/D-06 must bind them to
one canonical RRFlow persistence boundary, including local operation without
cloud dependency. This ADR does not authorize a client preference database,
hidden service or install-time network requirement.

Binding a user configuration grants only the explicitly authorized settings
read. It grants no project-to-project knowledge access. Such access still
requires [H-10 project links](../reference/data/rrflowdb-capability.md#project-links)
with independent source identities and stamps. Configuration binding is not
blocked on implementing an artificial personal engram or project link to one.

## Resolution and enforcement

1. Resolve the caller, project, authorized configuration bindings, operation,
   schema revisions and temporal read identities before a governed effect.
   Each setting declares allowed scopes and composition semantics. Unknown
   ideas remain representable as proposals; unknown effects are not executable.
2. Gather applicable user, project and explicitly bound organization policies.
   A more specific scope cannot weaken a mandatory policy. Combine numeric
   minima by taking the greatest minimum and maxima by the least maximum;
   intersect allowed sets, union denied sets and retain all required
   obligations. Units and applicability must agree. Conflicting locks,
   impossible obligations, an empty required choice set or a minimum above a
   maximum produce an explicit conflict and refuse the affected effect.
   There is no generic ordering of arbitrary requirements called "strictest".
3. For an ordinary overridable preference, choose the first applicable value
   in this RRFlow order: run, project, user, optional organization default,
   engine default. Only declared scopes participate. Collections or structured
   values need explicit schema-defined composition; do not recursively merge
   arbitrary objects or lists. Organization defaults and mandatory organization
   policies are different records.
4. Validate the result against every applicable policy. An explicit request
   outside policy is refused with its reason. A stored preference that became
   incompatible is reported as a conflict, requiring a valid proposal; do not
   silently clamp it and claim the requested behavior ran. Policies never
   confer permissions; authorization and budgets remain independent checks.
5. An absent optional preference may use a declared engine default, reporting
   its source. An unavailable, stale beyond its admitted freshness, or
   unauthorized required policy source makes the affected operation unavailable
   or refused. Do not treat failed policy retrieval as an empty policy set.

For example, a user requirement to retain supporting evidence applies in every
bound project. A project may require additional evidence, not remove that
obligation. A project may override the user's preferred answer detail. A
minimum of three verification checks and a maximum of two is a conflict, not
permission to pick whichever scope was read last.

Behavioral observations and inferred preferences remain attributed proposals
until explicitly admitted as configuration. Repetition does not override an
instruction. Semantic routing, orchestration and optional System One advice
consume the effective configuration at their configured decision points;
controllers cannot weaken policy or become execution authority. Schema growth
governs durable records and effects, not private thought ([ADR-0002](0002-adaptive-reasoning-governed-effects.md)).

## Temporal provenance and diagnostics

Policy and preference revisions preserve both valid time (when a requirement
applies) and recorded time/read identity (what was known). Corrections and
supersession must preserve historical answers. A resolution binds every source
revision and the resolver/schema revision; it must not imply one atomic read
across independent stores. The implementation must define freshness and
revalidation at the governed effect boundary. Historical replay never grants
authorization to perform an effect under revoked permissions.

Effective configuration reports the selected value, units, applicable scope,
source records and stamps, overridden candidates, enforcing requirements,
resolution reason or conflict, and live/restart/rebuild application state.
Receipts retain the authorized provenance needed for replay without exposing
secret values or inaccessible policy content. Reason codes are generated from
the implementing contract, not invented separately by Connectome.

Connectome must show plain-language **User policies and preferences** and
**Project policies and preferences** controls, with **Effective configuration**
explaining why a value applies. It must distinguish configured, resolved,
unavailable and conflicted states. Project instance start/stop and project
knowledge sharing are separate controls; global defaults do not start an
instance or connect projects implicitly.

## Freshness and semantic retrieval

The 2026-11-21 follow-up binds this decision to the existing
[knowledge lifecycle](../reference/data/knowledge-connectors.md#memory-freshness-and-vectorization)
and its [dated research review](../research/bitemporal-lexicon-and-behavioral-state-survey-20261116.md#2026-11-21-follow-up-memory-freshness-and-vectorization).
Resolve all applicable mandatory policies by authenticated scope, identity,
validity and declared applicability rules. Never select that set by vector
similarity, top-k, an LLM summary or a token-budget truncation. If required
resolution cannot fit its budget or a required source is unavailable, refuse
the affected effect. A model may help propose an applicability rule; it cannot
silently decide an enforced requirement is irrelevant.

Optional embeddings may help discover related preferences, rationales,
examples and project memories. They are revision-bound retrieval projections;
the canonical records determine meaning and authority. A vector hit is neither
a policy grant nor proof that a fact is current. Mandatory constraints remain
enforced even when the embedding provider is disabled, fails or returns no hits.
Lossy summaries cannot replace their authoritative representation. For behavior
that cannot be checked mechanically, report the unverified obligation and its
evidence requirement rather than claiming enforcement from prompt inclusion.

Recency and use frequency may influence optional context ranking. They cannot
expire, weaken or promote an explicit policy. Source freshness leases, semantic
valid-time intervals and retention are separate: an old but still applicable
user requirement remains binding until an authorized revision changes it.
Maintenance can propose corrections and revalidation; it cannot refresh truth
merely by updating an observation timestamp or re-embedding old text.

## Implementation and acceptance

[POAM-077](../poam/rrflow-1.0-alpha/poam-077.md) remains open. D-01/D-06 own
configuration persistence, governed plan/apply and resolution; K2 carries the
settings contract; K11 carries instance isolation; H-06/K12 consume generated
public contracts in Connectome. H-07 owns optional synchronization. No gate or
runtime capability is completed by this documentation decision.

Required installed evidence includes two isolated projects bound to one user
configuration; a project preference override; enforced user minima and budgets;
conflicting policies; concurrent updates rejected or admitted against exact
revisions; historical correction/readback after restart; unavailable policy
sources; denied cross-project reads; project shutdown without loss of user
configuration; and identical public-operation results in Connectome and other
clients. Sharing/import must preview scope, provenance, permissions and changes
before admission; imported preferences cannot activate integrations or weaken
policy. These are acceptance requirements, not existing behavior claims.
