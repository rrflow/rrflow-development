# ADR-0004: Host hooks as alias stubs

**Status:** active accepted architecture decision
**Coordinate:** `rrflow://rrflow-instance/data/decision/0004-host-hooks-as-alias-stubs`
**Owner:** whether and how an editor or AI provider hook may connect a host to RRFlow
**Decision date:** 2026-10-28

## Context

Editors and AI coding tools run hooks: commands that their configuration binds
to host events such as a session start, a prompt, a tool call or a stop.

[ADR-0001](0001-single-engine-authority.md#rejected-alternatives) rejected
provider- or editor-owned lifecycle hooks, because a client cannot become
RRFlow's reasoning or mutation authority. Later records widened that into a
ban on any editor or provider hook:

- `AGENTS.md` stated that RRFlow has no editor- or provider-owned automatic
  hooks;
- Gate I's title and I-04 required RRFlow to ship none, and J-01 listed them
  for removal;
- the agent bootstrap, the project command contract, the CI record and
  POAM-027 repeated the ban.

That wording entered in commits `54487226`, `2013f947` and `32aac720`, between
2026-09-03 and 2026-09-05.

On 2026-10-28 the repository owner said the ban is wrong. Editor- and
provider-owned hooks are allowed, as alias stubs that align to one clear,
consistent source, so that everything stays updated and linked, management is
reliable and context does not rot. The owner's intended start path depends on
it: a user starts RRFlow in a project folder, the host's hooks fire, and the AI
asks whether they want indexing to begin.

The repository already uses this pattern for instructions. The root
`CLAUDE.md` imports `AGENTS.md` and copies none of its rules, so updating the
one owner updates every host.

## Decision

RRFlow permits **host hooks as alias stubs**. The
[host hook contract](../reference/agent-bootstrap.md#host-hooks) owns the
exact rules.

1. **A hook is an alias stub.** It invokes one public RRFlow command or
   operation, or imports one owning record. It carries no copied rule or
   instruction text, decision logic, state, schedule, retry loop or credential
   of its own.
2. **One owner projects it.** An RRFlow-owned host binding declares the stub,
   and the stub is generated from that binding with its digest. Changing the
   binding regenerates or retires its stubs, and verification reports a stub
   that no longer matches.
3. **Installation is explicit and removable.** The install plan previews each
   stub and its exact host file. RRFlow writes it only after authorization and
   removes it with the rest of its integration, leaving project-owned host
   configuration as it was.
4. **A hook firing is a signal, not consent.** A stub reports a host occurrence
   as a typed engine event or asks RRFlow what applies now. Indexing, project
   mutation and every external effect still need explicit authorization
   through `RrdEngine`, for example the AI asking before attunement starts.
5. **A hook is not evidence.** A hook that ran proves only that the host
   invoked it. Readiness, progress and completion come from the engine.

ADR-0001's rejection stands for hooks that own lifecycle, state or authority.
This decision replaces the wider ban.

## Consequences

- The owner's start path becomes buildable. Gate I's I-04 binds host-event
  adapters to hook stubs, and I-06 previews, installs and removes them.
- Instructions and hooks follow one rule: one owner and thin host-specific
  entries generated or imported from it.
- A host whose hook would need logic of its own is supported only after an
  RRFlow adapter carries that logic behind a public operation.
- Until I-04 and I-06 land, RRFlow installs no hook. The current installation
  leaves project-owned `AGENTS.md`, provider instructions and host
  configuration unchanged.
- No hook can replace the repository's pull request and candidate CI gate.

## Rejected alternatives

- **No hooks at all**, the previous rule. It blocks the owner's start path and
  leaves each host to be wired by hand, with nothing keeping that wiring
  current.
- **Hooks with their own logic, state or instruction text.** Each would become
  a second authority that drifts from its owner, and the engine could neither
  see nor verify what it did.
- **Hand-maintained stubs.** They drift silently when the owner changes.

## Links

- [Host hook contract](../reference/agent-bootstrap.md#host-hooks)
- [Automation, routine, and skill flow](../architecture/engine-data-flow.md#automation-routine-and-skill-flow)
- [Gate I](../roadmap/rrflow-1.0/gate-i.md)
- [ADR-0001 single-engine authority](0001-single-engine-authority.md)
- [ADR-0002 adaptive reasoning with governed effects](0002-adaptive-reasoning-governed-effects.md)
