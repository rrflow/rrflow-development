# Worker model transport and configured policy

**Status:** active verified transport/configuration prerequisite; worker dispatch unavailable
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-model-transport-policy`
**Owner:** G-01b / G-02a / POAM-009; driver: active SWAG session

## Change boundary

[PR229](https://github.com/rrflow/rrflow-development/pull/229) starts at
`9466ed0c405797baa58a75748d7be811e2c37ace`, over exact controller dependency
`952e0928`. Its pre-edit brief and the
[opening observation](task-worker-model-boundary.md) bind the complete selected
worker activity. This record covers its transport/configuration prerequisite,
not completion of that brief or the end-to-end SWAG objective. Version stays
`1.0.0`; PR206, PR224/227 and PR228 remain dependencies.

The previous native HTTP adapter accepted only endpoint-choice content.
`PreparedModelChat` now prepares explicitly configured text/JSON requests and
retains bounded provider content with exact request/response/content hashes.
`PreparedEndpointChat` remains a strict allowance-checked selector wrapper over
the same transport. Existing selector wire bytes and retained response hashes
are preserved. Network policy, deadline/cancellation checks, credential
redaction, TLS/literal-loopback restriction, no retries/redirects/proxy, response
limits, unsupported-effect rejection and unknown usage remain enforced.
JSON-object mode rejects text and arrays; neither text nor JSON is a project
mutation, correctness label, or training permission.

`EndpointGenerationSettings` names the token parameter, response format and
optional temperature. An optional `EndpointWorkerPolicy` uses the existing
profile preview/apply/revision/digest boundary. Binding, prompt, limits and
effort mappings are explicit. Unsupported/default effort cannot be guessed;
missing policy does not enable calls. Declared protocol evidence is a reference,
not an attestation. Policy permissions require separate engine admission.

## Evidence and limits

The pre-edit `model_chat` oracle failed compilation because generation settings
and model-content APIs did not exist. The first raw result remains
`task-worker-model-first-oracle.log` in the local SWAG evidence directory,
SHA-256 `59d3896feb320602e284b33b2c8f02f57b186b0c83bd53b094793ac4f785827a`.
After implementation:

- `cargo test -p rrd-inference --features endpoint-chat --test model_chat --test endpoint_chat --locked`:
  five new protocol tests and eight existing selector tests pass. The loopback
  exchange checks actual wire bytes, content, digests and measured HTTP status;
  it is a controlled protocol fixture, not a live model-quality observation.
- `cargo test -p rrd-contract --test endpoint_worker --test endpoint_policy --test endpoint_binding_contract --locked`:
  14 tests pass, including absent-policy identity stability, changed-policy
  digest rejection, closed settings and explicit effort mapping.
- `cargo test -p rrd-engine --lib --all-features --locked endpoint_configuration`:
  six tests pass. The real engine rejects tampered approval, applies the exact
  worker settings, preserves them after reopen and rejects stale application.
  The existing selector-only case remains independently exercised.

The first broader run stopped at the pinned OpenAPI digest assertion. Its raw
log SHA-256 is `4ee4d1cdfa0997ca13a7d2656f4c0c5720c781e6daca0b22d4c34d14261dd40d`.
Recursive comparison against the actual prior exporter found exactly eight
added `worker_policy` properties on existing configuration request/response
profiles and no other structural changes. The intentionally reviewed digest is
`d4fbddb341893b19347e46c5f54605c888edc932ea5084dc81eb00fd422e754b`.
The focused schema assertion then passed, followed by 216 contract/inference/
client tests across 39 targets (none ignored), and 291 full engine-library tests
(four existing ignored helpers). These overlap the focused checks above.
Logs have SHA-256 `b69f6857e2266958f055fbd7a9610006f44169eb8cedd21415d0b2882b388b81`
and `ed58a4449c27b242bc1c6f4ca1e33dd81afab6d62923cd7bccccee5fc5ea3ee4`.

Strict all-target/all-feature Clippy for engine, contract, inference and server
passes. The canonical generators refreshed all affected SDK projections and
digest citations; parity still reports 71 operations and seven signal
projections. TypeScript generation/lint/typecheck and all six tests pass.
All 32 `workspace_architecture` tests pass, including tracked source closure.
`cargo fmt --all -- --check`, whitespace checks, navigation regeneration,
`build_execution_inventory.py` and its check (2,166 records), and
`check_documentation.py` (846 statuses, 844 coordinates) pass.
`task-worker-model-corrected-validation.json` retains each command, log hash,
elapsed time and the tested input-file hashes; its SHA-256 is
`a155ad7dae784a3082d0829a7d77ee4a100f6784a04dd0b99fc131276839872c`.
No installed worker binary, provider dispatch, subscription
adapter, project effect, RROFLOW correlation or training is claimed. The
repository's required candidate/protected checks remain mandatory.

## Authoring and traceability

The original selector's HTTP and envelope behavior stays at its canonical
`rrd-inference::endpoint_chat` owner; there is no parallel network adapter or
task scheduler. New generation/policy contracts live in `rrd-contract` and the
existing `endpoint_binding` profile owns their persistence and approval hash.
Tests extend `endpoint_configuration`, `endpoint_binding_contract`, and new
`model_chat`/`endpoint_worker` cases. Public export additions are confined to
the contract module declarations; generated projections must follow that owner.

Review includes the complete native chat implementation and selector tests,
new generation/worker contracts and tests, and the relevant profile
validation/digest, configuration test and public-export sections. Engine
proposal/capacity, task-budget completion, context and flight owners were
inspected for the remaining admission work; no complete-file review of the
large contract root or context assembler is claimed.

The opening record links the official chat-wire reference. No upstream code
is copied. Existing bounded errors, hashes and execution controls suffice for
this increment; no new runtime tracing or performance claim is warranted.
The first SDK install used the shell's older Node/pnpm and failed to parse the
pinned multi-document lockfile. Using already installed Node 24.22.0 and pnpm
12.14.0 succeeded without changing the lockfile. Both logs are retained.

The initial documentation check rejected this record's missing `active` status
prefix; the prefix was corrected and its generated index refreshed. No runtime
code or check was weakened. Diff review confirms the public root changes only
exports and the reviewed schema digest, the generated TypeScript changes only
the eight worker-policy projections plus digest, and the other SDK/reference
changes only their derived digests. Full installed/SDK conformance and candidate
CI wait for the complete worker operation; this component check is not a release.

Stop on a second authority, guessed provider support, lost selector validation,
secret exposure, automatic uncertain retry or draft text treated as an accepted
effect. Continue with existing task/context authorization, budget and shared
capacity admission, guarded output/readback and the generated public operation.
No roadmap gate or POA&M item closes in this increment.
