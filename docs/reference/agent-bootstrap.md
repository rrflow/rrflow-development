# Provider-neutral agent bootstrap and specialization

**Status:** active target reference; checkout forwarding exists, installed specialization does not
**Coordinate:** `rrflow://rrflow-instance/data/reference/agent-bootstrap`
**Owner:** provider-neutral AI instruction, installation, and specialization boundary

The repository root [`AGENTS.md`](../../AGENTS.md) is the single checked-out
instruction entry point. It tells an AI how to find authority; it does not copy
the product memory into every prompt and it is not a database. The root
[`README.md`](../../README.md) owns product identity, current maturity, and
warp points. Installed RRFlow records will replace Markdown as durable project
memory after the persistence and installation gates pass.

## Host boundary

| Host | Checkout behavior | Rule |
|---|---|---|
| Codex | Discovers `AGENTS.md` hierarchically. | Consume `AGENTS.md` directly. |
| Claude Code | Loads `CLAUDE.md` and supports file imports. | Root `CLAUDE.md` imports `AGENTS.md` and contains no copied policy. |
| Gemini CLI | Loads `GEMINI.md` hierarchically and supports `@` imports. | Root `GEMINI.md` imports `AGENTS.md` and contains no copied policy. |
| Grok | Current tooling discovers the `AGENTS.md` file family. | Consume `AGENTS.md` directly; do not add hooks or a second instruction body. |
| LFG or another host | No behavior is assumed. | An explicit adapter must prove how it supplies the same digest-bound specialization and public RRFlow operations. |

The upstream behavior references are [Codex `AGENTS.md`](https://learn.chatgpt.com/docs/agent-configuration/agents-md),
[Claude Code memory](https://docs.anthropic.com/en/docs/claude-code/memory),
[Gemini CLI context files](https://geminicli.com/docs/cli/gemini-md/), and
[Grok instruction compatibility](https://docs.x.ai/build/features/skills-plugins-marketplaces).
These links justify only bootstrap-file behavior. They do not give a provider
authority over RRFlow lifecycle or state.

## Installed specialization

Installation must eventually persist one immutable, versioned specialization
manifest through `RrdEngine`. Its identity and digest bind:

- estate, project, instance, source snapshot, schema, catalogue, and policy
  coordinates;
- discovered languages, frameworks, build systems, data sources, and verified
  capabilities;
- eligible routine, trigger, skill, model, embedding, and adapter revisions;
- explicit budgets, permissions, secrets references, and network constraints;
- the current attunement job and checkpoint chain; and
- evidence showing why each capability is active, inactive, or unsupported.

Providers receive a bounded projection of this manifest through public RRFlow
operations. They never receive raw KV keys, plaintext credentials, unrestricted
workspace content, or authority to mutate the manifest directly.

## Installation and attunement

The frozen contract already defines fresh/existing project targets; the ordered
installation actions `initialize_instance`, `configure_project_locator`, and
`create_attunement_job`; and the following eleven attunement phases:

```text
connect -> inventory -> parse -> normalize -> entity-link
        -> lexical-index -> embed -> vector-index
        -> graph -> ground -> verify
```

Inventory is a committed data boundary, not an informal directory listing.
The exact record contract, Git/non-Git ignore behavior, symlink and secret
safety, deterministic tree digest, incremental change set, and acceptance
corpus are owned by
[Project-tree inventory and incremental attunement](../architecture/engine-data-flow.md#project-tree-inventory-and-incremental-attunement).
No parser, indexer, model, routine, skill, watcher, or provider adapter may run
ahead of that committed snapshot. Filesystem notifications can request another
bounded inventory pass only after Gate I exists and they enter as authenticated
engine events; they never mutate project knowledge directly. Before then,
installation and explicit refresh invoke the same inventory operation directly.

For a substantial existing project, 30–45 minutes is an installation-planning
estimate, not a completion guarantee. Preview must derive its estimate from
the discovered source count, bytes, parser work, model/index work, configured
resources, and reusable checkpoints. A new or unchanged estate may require
less work; a larger or resource-constrained estate may require more. Apply
reports phase progress and remains safely resumable rather than racing a fixed
wall-clock target.

The engine implementation must preserve these rules:

1. Preview resolves exact actions, inputs, estimates, configuration digest,
   template revision, and specialization revision without mutation.
2. Apply accepts that exact plan digest; it cannot silently re-plan.
3. Every phase records a durable, digest-chained checkpoint through
   `RrdEngine`, including an explicit no-work result.
4. Optional capabilities are selected from verified project signals and estate
   policy. Absence of a language, model, database, or provider is ordinary and
   does not fail the whole attunement.
5. Resume continues the next uncommitted phase. It never infers completion from
   a trace, file, hook, or client status.
6. Verify compares persisted records, indexes, graph state, specialization,
   public operations, and restart behavior against the plan.

## Project-operation preflight

The first engine step for development or maintenance work is not a model
prompt and not a host shell walk. The caller submits intent; `RrdEngine`
resolves the latest complete authorized project-tree snapshot, binds its digest
to the operation `ReadStamp`, and retrieves only the relevant tree, ownership,
symbol, dependency, policy, and evidence neighborhoods. If no current complete
snapshot exists, the operation returns `inventory-required` and may propose the
same bounded inventory capability used during installation.

Every proposed source change retains the input snapshot and affected entry
digests. After an authorized file activity applies the previewed change, RRFlow
re-inventories and commits the resulting change set before later reasoning or
index work proceeds. A provider's open-file list, editor callback, cached tree,
or model recollection cannot satisfy this precondition.

## Generic routine package

A routine is a versioned, resumable graph of authorized engine operations. A
generic package is eligible for every estate but inactive until attunement
proves its inputs and policy allows activation. Each package declares:

- canonical identity, revision, content digest, input/output schemas, and
  minimum engine capabilities;
- accepted canonical engine events and deterministic activation predicates;
- preconditions, ordered or branching steps, operation permissions, budgets,
  timeouts, retry policy, checkpoints, cancellation, and terminal states;
- required context intent rather than storage fields, index names, or model
  providers;
- verification assertions, evidence fields, rollback or compensation behavior,
  and retirement rules; and
- secret references and outbound-network permissions without secret values.

The canonical event-to-context-to-commit sequence and the distinction between
events, triggers, routines, skills, functions, MCP, and physical access paths
are owned by the
[automation, routine, and skill flow](../architecture/engine-data-flow.md#automation-routine-and-skill-flow).
A routine step names a public semantic operation or required capability. It
never names a Rust function, executable path, MCP server, storage key, graph
edge prefix, index implementation, DataFusion node, or model provider.

The first generic routine set is deliberately small:

| Routine | Purpose | Activation evidence |
|---|---|---|
| `project-inventory` | Produce or refresh the bounded authoritative project-tree snapshot. | Explicit install/refresh intent, or an authenticated filesystem hint accepted under estate policy. |
| `change-impact` | Resolve affected symbols, graph neighbors, tests, policies, and documentation. | A committed change set and required graph/index capabilities exist. |
| `error-resolution` | Correlate diagnostics with code, ownership, prior evidence, and verification commands. | A typed diagnostic event and a supported build/test boundary exist. |
| `verification` | Run the smallest owning checks, then broaden according to risk and changed boundaries. | A proposed mutation or completed routine requires proof. |
| `knowledge-maintenance` | Re-attune only stale records and projections, preserving unaffected identities. | A committed project-tree change set or parser, schema, embedding, or catalogue revision drift is proven. |

These names describe target packages, not current implementation. They become
real only after the routine contracts, persistence, authorization, restart,
and conformance evidence required by roadmap Gate I pass.

## Capability growth

```text
committed engine event
        -> bounded trigger match
        -> attunement/capability rule evaluation
        -> authorized routine proposal
        -> RrdEngine validation and commit
        -> checkpoint, trace, verification, and live delta
```

PostgreSQL, Turso, SQLite, Dragonfly, cloud services, UI clients, model
providers, and future tools enter through independently versioned adapters.
Detection records a possible capability; it never installs, activates, reads,
or writes that system without explicit configuration and authorization.

## Conformance

A provider or adapter is supported only when a real-process corpus proves:

- the same specialization digest and public operation schemas;
- equivalent authorization, budgets, read stamp, result, evidence, and denial;
- no provider-specific persistence, context assembler, routine scheduler, or
  mutation path;
- bounded handling of malformed output, stale state, cancellation, timeout,
  reconnect, and restart; and
- clean uninstall that removes only the adapter and leaves RRFlow state
  readable through every other conforming surface.
