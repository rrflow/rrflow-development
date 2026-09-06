# Package command capability policy

**Status:** supporting pre-release target policy; not implemented or accepted
**Canonical destination:** `docs/reference/automation/package-workflows.md`
after the knowledge-bootstrap move in KB-05
**Authority:** the automation flow in
[`architecture/engine-data-flow.md`](architecture/engine-data-flow.md#automation-routine-and-skill-flow)

Package-manager scripts are discovered project facts, not permissions and not
RRFlow lifecycle events. A routine may invoke one only after installation or
attunement has proposed a versioned capability binding and an operator has
explicitly applied it through `RrdEngine`.

The generic routine refers to a semantic capability such as
`project.verify.typecheck`; it does not name pnpm, npm, Cargo, a repository
path, an MCP tool, or a provider. The project specialization binds that
capability to one exact direct-argument-vector command:

```toml
format = 1

[[capabilities]]
id = "project.verify.typecheck"
command = ["pnpm", "run", "typecheck"]
working_directory = "project-root"
allow_additional_arguments = false
required_projections = ["source-routing"]
max_source_lag_generations = 0
verification = "exit-zero"
```

This TOML is an illustrative target contract, not an implemented filename or
schema. Its eventual checked-in template is generic; the durable applied
binding, grant, revision, and status belong to the project's rrflowDB estate.
A checkout locator may identify that estate but cannot become mutable workflow
authority.

## Command boundary

A valid command capability has all of these properties:

- an immutable capability ID and revision;
- an exact executable plus argv array, never a shell command string;
- a logical working-directory coordinate resolved inside the authorized
  project root;
- explicit environment allowlisting and secret references, with no inherited
  ambient secret set;
- source, schema, policy, catalogue, and projection freshness requirements;
- bounded wall time, output bytes, process count, and cancellation behavior;
- one verification rule: `exit-zero` or `observe`; and
- a digest over the normalized binding used for authorization and evidence.

Pipelines, redirects, command substitution, implicit shells, unresolved
environment expansion, caller-selected paths, extra arguments, unknown fields,
duplicate capability IDs, and cross-estate bindings are denied. A command may
be useful evidence, but it cannot authorize its own mutation or declare a
roadmap gate complete.

## Engine-owned execution flow

```text
public operation or eligible routine step
                  |
                  v
              RrdEngine
 resolve applied capability revision -> authenticate -> authorize
 bind ReadStamp/freshness -> acquire lease/idempotency key -> enforce budget
                  |
                  v
       bounded local-process activity adapter
         exact executable + argv; no shell
                  |
                  v
 typed result: exit status + bounded/redacted evidence + digests
                  |
                  v
              RrdEngine
 validate result -> commit routine checkpoint/status/audit/engine event
                  |
                  v
       post-commit event/outbox notification
```

The activity adapter performs only the authorized host effect. It does not
advance a routine, write rrflowKV, emit authoritative events, or infer success.
`RrdEngine` owns leases, retries, idempotency, checkpoint transitions,
verification, and every durable mutation. HTTP, WebSocket, SDK, MCP, CLI, and
Connectome invoke the same public engine operation rather than implementing
package-command behavior independently.

## Evidence and redaction

The durable observation contains the estate, capability ID/revision, routine
run and step IDs when applicable, bound read stamp, command-binding digest,
start/finish times, bounded exit result, verification state, evidence digest,
and correlated audit/event IDs. It may retain approved redacted diagnostics.
It must not persist secret values, an ambient environment, unrestricted stdout
or stderr, or caller-supplied shell text.

## Explicit exclusions

There is no session-start, pre-tool, post-tool, stop, compact, editor,
package-manager, or provider-owned hook lifecycle. Host notifications may later
enter through removable adapters as untrusted `EngineEvent` proposals; they do
not execute a command or mutate state directly. MCP is an ingress/egress
adapter, not the routine scheduler.

## Required acceptance evidence

Gate I must add and pass tests for exact-argv execution, absent or stale
bindings, scope escape, extra arguments, shell composition, environment and
secret redaction, timeout, cancellation, retry/idempotency, restart from a
committed checkpoint, stale projections, verification semantics, post-commit
event ordering, and identical behavior through every public surface. Until
those files and results exist, this record makes no implementation claim.
