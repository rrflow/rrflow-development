# Governed controller settings in endpoint profiles

**Status:** active configuration prerequisite; engine-admitted selector execution pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-policy-configuration`
**Owner:** D-06/G-01b, POAM-009; OBJ-12 prerequisite
**Pull request:** [201](https://github.com/rrflow/rrflow-development/pull/201), after [196](https://github.com/rrflow/rrflow-development/pull/196), [198](https://github.com/rrflow/rrflow-development/pull/198) and [199](https://github.com/rrflow/rrflow-development/pull/199)

## Baseline and before/after oracle

Public main was `44cbfa7346fe458d84daead188d33cbac7392efd`. An isolated
worktree integrated PR199 `5ee39f39942f610e4414fec287ce21dbe88b2d8b` by ordinary
merge, giving opening head `b3e6fa78e173ab40e6c2088ffdb1adaa207f05e2`.
The draft PR held the change brief before implementation. Driver: the SWAG
controller working session. Dependencies and full candidate CI remain required;
draft checks are not landing evidence.

Previously, endpoint profiles could retain service/credential references but
could not retain approved controller prompts, generation settings or candidates.
Passing caller-supplied settings directly into transport would not establish
engine-owned configuration. This package adds optional `EndpointSelectionPolicy`
to the existing profile and its preview/apply/read CAS. No new configuration
authority, operation, provider dispatch, training record or routing default is
introduced. Stop conditions remain duplicate authority, invented evidence,
embedded provider ranking or required-check bypass.

The embedded engine test derives a proposed policy from retained actual BF16
request, configuration and allowance files. It uses a fresh test estate, not a
claimed replay of the captured installed estate. Preview leaves state unchanged;
changed settings fail old approval; apply, exact readback and idempotent replay
survive real store close/reopen; revocation invalidates policy projection and the
old plan. This advances durable configuration, not installed-binary execution.

## Source placement, inspection and review

Created `rrd-contract/src/endpoint_policy.rs` and `tests/endpoint_policy.rs` under
`crates/transport/`. Changed profile construction/validation/hash in
`endpoint_binding.rs`, module exports and OpenAPI digest in `lib.rs`, and existing
contract/engine configuration tests. Extracted the existing candidate checks from
`EndpointSelectionAllowance::validate_for` into one shared validator in
`endpoint_selection.rs`; the existing eight selection tests retain its oracle.
No implementation was deleted or moved into another authority. The authored
policy, tests and changed source sections were reread; existing larger files
were inspected at their relevant symbols rather than claimed as an all-code audit.

Reviewed README/AGENTS, change-authoring and Gate D/G/I owners; endpoint binding,
configuration and selection; engine configuration CAS; task reservation/completion
and inference admission; native transport preparation; SDK generators and schema
parity checker. Updated the endpoint and LFG/SWAG references, this journal and
generated navigation. Regenerated every supported SDK's endpoint projection,
signal projections and current digest citations. The ignored Git inventory is
derived after staging new files; its tracked retirement marker stays unchanged.

All policy fields bind profile/configuration/approval identity. An absent policy
omits the wire field and preserves previous profile hashes. Controller and
candidate identities come from the profile; the system prompt and generation
settings have no defaults. Limits bound encoded policy size and generation;
temperature uses exact integer thousandths. Capability evidence hashes remain
declared references, not attestation or authorization. Policy projection requires
the exact installed configuration and still validates the resulting allowance.

No external algorithm or dependency was introduced. Existing repository owners
and generators suffice; additional research, provider calls and GPU/resource
benchmarks would not test configuration persistence. Preserve prior transport
evidence without attributing new model performance or training to these tests.

## Validation, failures and limits

- Four policy tests pass using actual retained settings and explicitly derived
  in-memory denial probes. They cover absent-policy identity, changed prompts,
  candidates and limits, foreign bindings, size/generation bounds, closed fields
  and explicit empty candidates. No probe becomes a captured output or dataset row.
- `cargo test --locked -p rrd-contract -j 2`: 148 tests passed. The expanded policy
  tests passed again after the final review changes.
- `cargo test --locked -p rrd-engine --lib endpoint_configuration -j 2`: five
  tests passed, including persistence/reopen, stale approval and revocation.
- `cargo test --locked -p rrd-inference --features endpoint-chat -j 2`: 33 tests
  passed, preserving validation of actual retained native observations.
- Strict Clippy for contract and inference with `endpoint-chat`, all targets,
  and separately `cargo clippy --locked -p rrd-engine --lib --tests -j 2 -- -D warnings`
  passed.
- All SDK generator checks passed. TypeScript typecheck, Biome and six tests
  passed. Only TypeScript schema structure changes; other SDK projection diffs
  contain the new digest. Full cross-language process conformance remains CI work.
- Generated surface parity passed: 62 unchanged HTTP operations, unchanged signal
  catalogue, OpenAPI `c1c8c59f888c3c1425789feec81c008d3aa0cd0676f6155a526ccd96aca2c606`.
  Review found eight additive optional-policy schema occurrences and no removed
  schema fields. The frozen digest was intentionally advanced with all projections.

The initial extraction compiled with an extra borrow of a slice; correction to
the shared loop fixed both reported type errors. The first full contract run
correctly failed its frozen OpenAPI digest assertion; schema review and complete
regeneration corrected the mismatch. Rustfmt corrected two expanded assertions.
The local pnpm shim was broken; invoking the installed pinned 12.14.0 entry with
Node allowed frozen-lockfile installation and checks without changing dependencies.
A Go generator command used the wrong directory; the corrected SDK-directory
generation and check passed. No failed check was waived.

No full engine workspace, installed cross-platform lifecycle, provider capability
attestation, authenticated selector call, worker effect, model-quality or memory
qualification was run for this package. Public admission, shared durable attempt
reservation/fencing, credential resolution and source revalidation remain required
at their existing owners. G-01b, POAM-009 and OBJ-12 remain open; no roadmap or
qualification status changes. The next implementation must connect these settings
to actual engine admission, not expose caller-controlled transport as authority.

Checklist: brief before edits; canonical owner and shared validation retained;
actual-setting-derived persistence oracle; focused then owning checks; authored
source and generated-diff review; failures and excluded claims retained; immutable
journal and generated discovery. Documentation policy passed with 739 statuses
and 737 classified coordinates; workflow policy passed. Navigation check passed
with 18 indexes, 738 nodes and 2,847 edges; inventory generation/check passed with
1,967 records. Formatting and staged whitespace checks passed. Only the public
development remote and `codex/controller-policy-configuration` branch are the push
destination; record the exact post-commit revision before pushing. Dependency
landing and exact-candidate CI remain pending.
