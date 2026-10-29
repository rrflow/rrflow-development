# Bounded endpoint-choice validation on actual BF16 output

**Status:** active implementation prerequisite; runtime admission and dispatch missing
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-endpoint-choice-validation`
**Owner:** G-01b / POAM-009; OBJ-12 prerequisite
**Pull request:** [198](https://github.com/rrflow/rrflow-development/pull/198), after [196](https://github.com/rrflow/rrflow-development/pull/196)

## Baseline and change brief

Public main was `291fee511887932572ecc9935e9786e098727677`. The isolated
branch integrated PR196's `db0ae5927848a87a674616f91f5bd06d09778f32` by
ordinary merge, producing opening head
`add51483f251888889a2fbdcef8521b3e1dbcd68`. The draft PR and change brief
existed before implementation. PR196 must land first; no private repository
push or direct main mutation is permitted.

Previously no endpoint-choice decoder bound model output to a configured
candidate allowance. The new pure contract validates that relationship, finite
limits and declared effort choices. It does not claim supplied candidates or
effort records are authenticated or qualified. Engine admission, verified
descriptors, task/source identity and provider dispatch remain later work.
The existing recipe/branch/context union and public operation catalogue retain
their behavior. No runtime lifecycle, network adapter or credential resolver is
introduced. Stop on added authority, hardcoded routing, invented observations,
changed route semantics or bypassed CI.

## Inspection and implementation map

Read the existing endpoint configuration module and endpoint binding/router
contracts; inspected their validation/digest helpers, public catalogue entries,
task-budget types and engine configuration/inference methods. Read README,
AGENTS, change-authoring procedure, Gate G, POAM-009, PR196's inference reference
and the package brief. Full-file reread of newly authored Rust module, example
and focused tests accompanies the diff review; inspection of existing large
owners was scoped to the named symbols rather than an all-code audit.

Created `crates/transport/rrd-contract/src/endpoint_selection.rs` and exported
its types in `src/lib.rs`. Added `examples/endpoint_selection.rs` as an offline
bounded verifier and `tests/endpoint_selection.rs` with retained inputs under
`tests/fixtures/endpoint-selection-observed/`. Updated the LFG/SWAG reference,
this journal and generated nearest index. No existing implementation is removed
or replaced. The ignored execution inventory is regenerated from Git.

Protocol ceilings reuse existing resource bounds; installation limits and
candidate facts are explicit inputs. No model/provider names, task keywords,
rankings or endpoint URLs occur in implementation behavior. Declared unknown
effort remains unknown. Validation bounds raw response bytes before parsing and
counts serialized allowance bytes without allocating another encoded copy.
Errors do not echo raw model output. No third-party algorithm or library was
added or adapted, so external research is not applicable to this contract step.
Runtime trace/resource qualification belongs to the later admitted adapter.

## Actual observations and provenance

On 2026-10-29, an offline operator capture used the actual PR198 maintenance
task at its opening head and the retained native endpoint configuration,
revision 3. This is a real repository task, not the user's still-pending first
end-to-end trial or a human-labelled routing example. Two capture directories
retain requests, raw provider responses, extracted choice bytes, configuration,
allowance, task, effort observation, receipt and a per-file SHA-256 index.
Credential header values were excluded and checked against copied bytes.

Each capture first made a default-effort request about the actual task; its
successful transport response supports only the observed omission of effort.
The second call requested a choice using the Rust-generated schema. Both
captures used the configured experimental service alias
`swag-qwen3-coder-intent-20261028`. An alias is not independent attestation of
loaded weights. Four HTTP calls returned 200. No chosen worker was dispatched.

| Retained directory | Actual choice bytes SHA-256 | Observed response |
|---|---|---|
| `available` | `f00ec144e84b0093c575418c45be469717d1725ba11736e3f0070c2fa666e6a2` | Selected the supplied `swag-custom-bf16` binding, omitting effort. |
| `withheld` | `3c54cfeb084b86ecaaa6d6b3a48f465afa1ac42943f44402ae7a0f37d8f1ae5c` | Abstained with `no_eligible_candidate` after the operator supplied no candidates. |

Both bind configuration digest
`4621b748db8fbfa1aee8e49dffafcc635394301a1ed58c796db537cc98ca6063`.
Tests hash every retained file, link request/response/content digests and model
aliases to receipts, and bind the declared effort evidence. No human review,
training admission, authorization or routing qualification is asserted.
Single-call timings in receipts are observations, not performance benchmarks.
Fixture names and model identifiers describe captured history; product code
does not special-case them.

## Verification and limitations

The first oracle was absent decoding support for actual configured endpoint
choices. Eight focused tests now pass, including actual choice/abstention and
in-memory denial perturbations for changed configuration, candidate identity,
duplicate candidates, unknown/unsupported effort, byte limits, malformed JSON,
duplicate fields and invented authorization fields. Perturbations are validation
probes only; they are never retained as provider output or learning records.

`cargo test --locked -p rrd-contract --test endpoint_selection -j 2` and
`cargo test --locked -p rrd-contract -j 2` passed. The owning suite includes the
unchanged three-decision router characterization. The example compiled and
verified both actual captures. Initial example compilation exposed sha2 0.11's
digest array lacking `LowerHex`; explicit per-byte hex encoding corrected it.
`cargo fmt -p rrd-contract -- --check` passed.

The first staged whitespace check identified CRLF in retained configuration and
receipt bytes. A fixture-local `.gitattributes` preserves exact JSON bytes across
checkouts and recognizes CR at end of line; captured evidence is not rewritten
to change its hashes. Request/response provenance tests remain unchanged.

`cargo clippy --locked -p rrd-contract --all-targets -j 2 -- -D warnings`
passed. Documentation policy passed with 732 statuses and 730 classified
coordinates; workflow policy passed. Navigation generation and `--check` passed
with 18 indexes, 731 nodes and 2,832 edges. The staged whitespace check passed
after the byte-preservation correction; all 20 captured files also matched their
recorded hashes when read directly from the Git index.

Change checklist: brief before edits; bounded owner/scope and unchanged route
semantics; actual-output characterization followed by package tests/lint;
new immutable evidence and generated discovery; full authored-file reread and
diff review; failures and limitations retained. No roadmap or POA&M completion
status changes. Final inventory checks include this evidence paragraph.

Full candidate CI and protected landing remain pending; no engine, cross-platform runtime,
restart, cancellation, end-to-end integration, reviewed quality, training or
quantization acceptance is claimed. G-01b and POAM-009 remain open.
