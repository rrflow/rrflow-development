# Selected-worker model boundary observation

**Status:** active observed execution gap; implementation and installed acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-model-boundary`
**Owner:** G-01b / G-02a / POAM-009; driver: active SWAG session

## Observed baseline

This worktree starts at public main `f256c8e1`, then fast-forwards to the exact
published controller dependency `952e0928c55f809b4f9fcce4dcf3808878a0bd65`.
Its tested implementation is `9e167aad`; that binary passes installed CLI and
SDK checks but does not execute a selected worker.

The actual exported public schema contains 71 operations, including
`task-endpoint-propose`, `task-attempt-read` and `task-embedding-execute`, with
no worker operation. Schema SHA-256 is
`f1856602eb3b5ba419f600b3be0e9626d62ad533032fe220a2728795bd28048a`.
`observe-worker-boundary.py` and `worker-boundary-observation.json` retain this
read-only observation under the local SWAG evidence directory. No provider call
was made. Absence is also confirmed in the engine and executable catalogue,
rather than inferred solely from an operation's spelling.

`endpoint_proposal.rs` explicitly grants no worker effect. Its durable attempt,
authority guards, budget and capacity mechanisms are reusable. `task.rs`
groups intended work and orchestrates nothing. `flight.rs` already prepares
authorized canonical context at one read stamp; its only provider arm is
`observe`. `endpoint_chat.rs` shares native HTTP safety with selector-specific
choice decoding, so ordinary model text cannot currently pass that decoder.

## Architecture evaluation

Retain the existing engine as the single authority. A selected-worker model
call must resolve a retained accepted controller choice, current task and
configuration, explicit worker settings, authorized context and remaining
budget. Commit its prepared identity before dispatch and accept bounded output
only through existing guarded completion. Its text remains a draft, never a
project mutation, verified task outcome, quality label or training consent.

Reuse the existing stateless HTTP boundary while separating bounded text
observation from strict selector decoding. Do not reinterpret an existing
selection response as worker output, add a second task scheduler, infer provider
rankings, or silently omit selected effort. Native subscription CLI adapters
remain explicit future transports requiring their own identity, credential,
containment and cancellation qualification. Logged-in Claude Max is not proof
that such an adapter exists or that an HTTP API call uses that subscription.

The [official chat protocol reference](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create)
documents text/JSON formats, distinct output-token fields and model-dependent
effort support. These are wire semantics, not proof that any configured service
supports them. Settings and effort mappings must be explicit and digest-bound.
No upstream implementation is copied.

## Required next evidence

Implement the worker request/result and configured generation policy, shared
bounded transport, engine admission/reservation/completion and public handler
together. Preserve selector, context, authorization and replay oracles. Exercise
denial before effects, changed task/configuration, Stop, malformed/late output,
unknown outcome and reopen without repeat. Retain an actual configured worker
call separately from protocol fixtures. Every check and remaining gap belongs
to a later immutable implementation journal; this opening record reports
inspection and catalogue evidence only and claims no new executable capability.

Project-source mutation and ordinary-edit successor authorization remain at
their existing engine owners. This package cannot claim source-current project
edits from task text or canonical-context provenance alone. End-to-end UI/worker
effects, RROFLOW correlation, independently reviewed learning and public release
remain required by the full SWAG objective. Product version stays `1.0.0`.
