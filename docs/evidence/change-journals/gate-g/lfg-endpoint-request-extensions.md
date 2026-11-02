# LFG endpoint request extensions

**Status:** active implementation evidence; candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/lfg-endpoint-request-extensions`
**Owner:** forward-compatible task endpoint proposal request evidence

**Package:** Gate G-01/G-03 and POAM-009 integration prerequisite; no roadmap
or alpha status change. **Driver:** LFG/RRFlow integration working session.
**Starting development main:** `e6f3632b32f1c9be33c1951886a9d9a2be648ddf`.
**Draft:** [PR246](https://github.com/rrflow/rrflow-development/pull/246).

## Change brief

The owner requested additional request fields rather than a decoder that rejects
them. `EndpointSelectionRequest` previously denied all unknown fields. The
request now accepts and ignores them, so the typed request, operation digest,
authorized controller packet, selected endpoint, budgets and permissions do not
gain new caller-controlled inputs. In particular, sending an LFG observation as
an unknown field does not make the engine read or admit it. Prompt or credential
contents are not retained by this typed request. Normal transport body limits and
known-field validation remain in force. A separately implemented and authorized
engine path is necessary before LFG observations can affect a proposal.

Read for this package: `AGENTS.md`, `README.md`, Gate G, the inference reference,
change-authoring procedure, `rrd-contract/src/endpoint_proposal.rs`, endpoint
selection tests, `rrd-engine/src/engine/endpoint_proposal.rs`, reasoning-budget
proposal tests, native client example, controller-proposal client tests,
generated-surface tooling, SDK generators and their configuration. No external
code was adapted. This compatibility change uses existing HTTP body bounds and
records no additional trace content; resource and model-quality claims are out
of scope. No model call is needed for the decoding oracle.

Implementation changes `EndpointSelectionRequest` deserialization and its
OpenAPI-derived schemas, adds contract and engine tests, and corrects the
inference-reference status note. All five SDK endpoint and signal projections
are mechanically regenerated because they bind the OpenAPI digest. This journal
and its navigation index are evidence, not another implementation authority.

## Evidence and limitations

- Before the implementation change, the focused contract test failed with
  `unknown field endpoint`, reproducing the reported behavior.
- After the change, `cargo test --locked -p rrd-contract --test endpoint_selection`
  passed all nine tests. The new test checks that injected endpoint and credential
  fields decode to exactly the original typed request and do not serialize back.
- `cargo test --locked -p rrd-engine proposal_tuple_rejects_cross_attempt_config_and_usage_substitution`
  passed the targeted engine test. It checked that a request carrying an extra
  endpoint still validates against the same authorized proposal tuple.
- The initial `cargo fmt --all --check` showed formatting changes needed in the
  new test; `cargo fmt --all` applied them. The first generated-surface check
  then reported the stale TypeScript OpenAPI digest; all SDK projections were
  regenerated through their owning generators and the signal projections through
  `scripts/ci/check_generated_surfaces.py --write`.
- The first full `rrd-contract` suite then failed its deliberate OpenAPI digest
  golden check, and the generated-surface check flagged stale reference
  citations. The canonical digest and six current reference citations were
  updated to the newly exported schema; historical evidence was not rewritten.
- Inspection of the first regenerated TypeScript schema showed that merely
  omitting `additionalProperties` did not expose an index signature to typed
  clients. The schema now explicitly sets `additionalProperties: true`, with
  a schema assertion and a TypeScript payload test. All projections and the
  canonical digest were regenerated again. Unknown values remain ignored by
  the engine, not promoted into the controller packet.
- After the schema correction, `cargo test --locked -p rrd-contract` passed
  its full suite and `cargo test --locked -p rrd-engine --lib` passed 316 tests
  with four native subprocess fixtures ignored. The TypeScript SDK's
  `corepack pnpm check` passed generation, lint, typecheck and seven tests,
  including the extra-field type test. Its first run failed import ordering;
  that was corrected before the passing run.
- `python scripts/ci/check_generated_surfaces.py --check`,
  `python scripts/knowledge/render_navigation.py --check` and
  `python scripts/ci/check_documentation.py` passed. The tested OpenAPI digest
  is `9ddcf2c3965d140af0495f661f09a426d37892e2e10396889366f1da4133e656`.
- `cargo clippy --locked -p rrd-contract -p rrd-engine --all-targets -- -D warnings`
  passed on the local Windows host. Implementation and generated diffs were
  reviewed; generated digest changes are derived from the contract exporter.
- The tests above are in-process contract/engine evidence, not a real provider
  call, installed runtime, live LFG connection, or routing-quality result.
  Candidate CI, the remaining cross-platform and installed-runtime checks,
  and merge evidence remain pending and are owned by the pull request.

No access rights, engine dispatch, LFG admission, provider capability or user
subscription is inferred from an accepted extra field. Release gates remain open.
