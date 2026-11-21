# Governed project-policy controls: starting boundary

**Status:** package characterization; implementation and acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/project-policy-controls-start-20261121`
**Owner:** D-01/D-06 and POAM-077; H-06 consumes the public contract

Baseline ffb4c4b1779212aa8fa80e2d5908f3685f98d4ab. The scoped resolver reads a
shared user source but obtains project rules only from sealed installation
configuration. Endpoint configuration already provides authenticated preview,
exact-digest apply and control CAS, but does not edit numeric project policy.
The installed two-project test proves shared user updates, not project edits.
Connectome's native client pins an older engine and has no scoped-policy controls.

The package adds engine-owned project numeric policy read/preview/apply and
bitemporal correction history in the existing project control repository.
Installation ceilings and the user binding remain sealed; edits replace the
project numeric settings snapshot and cannot weaken user requirements. A bounded
history read selects project policy by valid time and known revision. Plans bind
project identity, installed digest, current head and observed user policy.
Apply revalidates authorization, expiry, source revisions and composition, then
publishes history and head atomically. Query/recall admission uses this state.

The same session drives the dependent Connectome package after this engine
contract lands: existing project UI, authenticated native client, exact preview
and approval, effective limits/provenance and visible conflicts. UI changes stay
in its repository; a later parent pin binds the joined test. No parallel client
policy database, global HTTP writes, arbitrary behavioral settings or complete
configuration/alpha qualification is claimed.

Read in full: ADR-0006, POAM-077, change-authoring procedure, scoped contract and
publication module, engine scoped resolution/publication/store/tests, endpoint
configuration contract/engine, client configuration operations, server endpoint
handlers. Relevant portions: README, Gate D, Connectome client contract, engine
core, operation catalogue/router/reference, Connectome README/package manifest.
This is bounded source review, not a whole-repository audit.

Source mapping: user change/revision types and history selection become shared
scope-neutral configuration primitives; existing user-publication and installed
correction tests remain required. New project state uses those primitives and
the same control CAS, never a second transaction authority. Contract/server/
client catalogue and generated projections change together. No source algorithm
is imported. Existing sourced ADR governs design; no new external research or
performance claim is needed. Existing authenticated audit and admission stamps
carry diagnostics. Build cache remains checkout-specific, four Cargo jobs.

Oracle: install and serve two projects bound to one user source; preview and
apply project limits without restart; reject unapproved, stale, foreign and
conflicting edits; retain user enforcement and project isolation; query/recall
returns actual admission revision; correction reads distinguish valid/known
coordinates after restart. Focused contract/history tests precede installed
HTTP tests, owning suites, generated-surface checks and exact candidate/queue CI.

Stop on unknown effects, unauthorized sharing, silently weakened limits,
unbounded history, duplicate authority or a missing production consumer.
No roadmap/POAM status changes. Implementation, complete diff review, commands
and failures will be recorded in the completion journal and PR. Scratch:
D:/.usr/Temp/rrflow-project-policy-20261121. No containers or extra worktrees.
