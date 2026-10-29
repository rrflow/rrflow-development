# Native selector transport with actual BF16 observations

**Status:** active adapter prerequisite; engine admission and integration pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-native-chat`
**Owner:** G-01a/G-01b, POAM-009; OBJ-12 prerequisite
**Pull request:** [199](https://github.com/rrflow/rrflow-development/pull/199), after [196](https://github.com/rrflow/rrflow-development/pull/196) and [198](https://github.com/rrflow/rrflow-development/pull/198)

## Baseline, brief and scope

Public main was `291fee511887932572ecc9935e9786e098727677`. An isolated
branch integrated PR198 `63375dd42c77986ea8bd64a9c6034c07af2d3c9a` by ordinary
merge, producing opening head `c9de56cd47337f2d87fc1812392873208144a50d`.
The draft PR's change brief existed before implementation. Dependency landing,
full candidate CI and protected merge remain required.

The first failing capability was absence of a native transport for actual
configured selector calls. The prior Python experiment demonstrated only wire
compatibility. This package adds an optional stateless Rust chat adapter that
validates prepared request/configuration consistency, performs one bounded call
and decodes a choice. It does not create authenticated engine admission, a public
operation, automatic installation, worker dispatch, a scheduler or a new budget.
The existing embedding registry, durable task authority, load admission and
recipe/branch/context union remain unchanged.

Created `crates/compute/rrd-inference/src/endpoint_chat.rs`,
`examples/endpoint_chat.rs`, `tests/endpoint_chat.rs` and actual observation files
under `tests/fixtures/endpoint-chat-native/`. Added the optional feature, a
locked ureq dependency and module export. Updated the inference reference,
this journal and generated nearest index; regenerate the ignored Git inventory.
No previous implementation was removed or replaced.

## Inspection and design decisions

Read engine inference entry points, durable reasoning-budget reserve/finish and
repetition mechanisms, inference registry/execution control, endpoint
configuration/binding/selection contracts, transport dependency checks, README,
AGENTS, Gate G/I and package-readiness rules. Full-file reread covers the authored
adapter, example and focused tests; inspection of existing larger owners was
scoped to the named symbols, not an all-code audit.

The adapter consumes the existing cancellation/deadline mechanism and narrows
the deadline by configured binding timeout. Preparation performs no network or
credential resolution. Execution needs explicit network allowance and a caller
resolver for the configured credential reference; resolver errors are sanitized.
The resolver must itself obey the inherited control. No secret is serialized in
observations. TLS outside literal loopback, disabled redirects/proxies, finite
body limits and strict selection validation constrain transport behavior.
Generation settings and prompt content are explicit inputs; no task-specific
route, provider ranking or deployment endpoint is coded into the library.

The adapter has no independent concurrency admission or durable attempt record.
The engine must reserve capacity/budget, validate source and current permission,
and recheck before accepting results. Cancellation around blocking I/O rejects
late output; it is not a remote cancellation API or hard termination guarantee.
Failures after the call starts remain potentially dispatched. There is no
application retry. A service alias and reported usage are observations, not
independently attested weights or measured billing. Offline decoding leaves
HTTP status and timing unknown.

## Primary research and adaptation

Used ureq **3.4.2**, pinned in Cargo.lock, with optional Rustls support; no
handwritten HTTP/TLS. Reviewed its
[agent API](https://docs.rs/ureq/3.4.2/ureq/struct.Agent.html), configuration/body
APIs and downloaded `src/config.rs`/request source for proxy, redirects, bounded
body reading and whole-call timeout. The [vLLM chat reference](https://docs.vllm.ai/en/stable/serving/online_serving/openai_compatible_server/)
informed the narrow non-streaming chat envelope. Provider extension metadata is
ignored within the body cap; known envelope fields reject duplicates and the
choice contract rejects unknown authority fields. No upstream engine or model
algorithm was copied. The dependency's transport does not become lifecycle authority.

## Real calls and evidence

On 2026-10-29 the native Windows example used the actual PR199 maintenance task,
retained installed configuration revision 3 and experimental BF16 service. These
are operator transport probes, not the owner's first end-to-end task or training
examples. The actual requests, outputs, process exit codes, source/binary hashes,
configuration, allowance and task are retained in the fixture directory. A
per-file SHA-256 index and byte-preserving Git attributes retain exact evidence.
All copied bytes were scanned against the actual credential without printing it.

| Probe | Observed result |
|---|---|
| `available` | HTTP 200; configured binding selected; 2,295 ms observed. |
| `withheld` | HTTP 200; no-candidate abstention; 1,613 ms observed. |
| `deadline` | 100 ms operator ceiling; exit 1, remote outcome unknown. No response or success was invented. |
| `cancelled` | Stop signal after 100 ms; exit 1, late result rejected, dispatch may have occurred. No remote-stop claim. |
| `postdecode` | After adding a final post-decoding deadline/cancellation checkpoint, another actual call returned HTTP 200 and a valid selection; 2,043 ms observed. |
| `envelope` | After rejecting legacy function-call and declared reasoning payloads, the final adapter still returned HTTP 200 and a valid selection; 2,393 ms observed. |

The final raw response SHA-256 is
`15683a1a120ca67d4ef7eced30660a9fcab323216ae6253c9af58ae2376faad8`.
Source records distinguish the initial implementation from the final checkpoint
correction; live probes ran before the source commit and are not release builds.
The final capture records exact authored-source and binary hashes. Single calls
are characterization, not latency qualification. No chosen worker, human review,
learning admission or engine authorization is asserted by any receipt.

## Validation and checklist

Seven focused tests pass using retained actual records and explicit in-memory
denial perturbations. They cover source/configuration mismatch, denied network,
pre-dispatch cancellation/expiry, sanitized credential failures, non-loopback
plaintext denial, request/response bounds, wrong model, truncation, unsupported
effects, invalid choice, inconsistent/missing usage and captured-byte integrity.
Negative perturbations are not represented as real provider outputs or learning
records. Existing controlled embedding tests remain characterization of their
own mechanism, not model-quality evidence.

`cargo test --locked -p rrd-inference --features endpoint-chat -j 2` passed
33 tests; the example built and actual calls ran. Strict Clippy passed on the
final adapter. Formatting caught a newly extended test-case list; rustfmt
corrected it without changing captured source or behavior. Documentation policy
passed with 733 statuses and 731 classified coordinates; workflow policy passed.
Navigation generation/check passed with 18 indexes, 732 nodes and 2,834 edges.
The staged whitespace check passed. All 39 captured files matched their hashes
when read directly from the Git index, and the final live source hashes match
the authored adapter, example and lockfile. Final inventory generation/check
includes this evidence paragraph. No engine workspace, installed cross-platform, HTTPS
provider, remote hard-stop, model-quality or resource-peak qualification was run
locally; those claims require their own named evidence. Optional-feature CI
already exercises all inference features and remains required before landing.

Checklist: brief before edits; existing owner/mechanism reuse; bounded changes;
real transport oracle; focused then owning tests; reviewed source and diffs;
immutable evidence and generated discovery; failures and limits retained.
G-01b/POAM-009 remain open. No roadmap completion status changes. The next
package must compose task/source authorization, durable reservation and accepted
receipts at the existing engine boundary before this transport becomes a usable
controller workflow.
