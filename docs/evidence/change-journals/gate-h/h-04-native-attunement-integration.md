# Native controller and attunement projection integration

**Status:** active draft integration evidence; installed and protected qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-native-attunement-integration`
**Owner:** H-04 native client projection consolidation

Date: 2026-10-30. Package: H-04, dependent on G-01 / PR206. Driver: SWAG
controller session. This advances the native client prerequisite for Connectome;
it does not close H-04 or establish worker execution or public release readiness.

## Baseline and change brief

[PR222](https://github.com/rrflow/rrflow-development/pull/222) began this integration
at `61ddb26416811ba7e56edf2e28b8cef3acad5930`, tree
`94af63352d3f63867ba70d4527c1495efb118e21`. Its description recorded the integration
scope and consolidation map before repository mutation. The incoming ordinary
merge is PR206 `71d53e1e86e17bcc3663dd4d82a7f03451b991dd`, incorporating main
`a76faf7046b48819a06c31219e9db973b7f6a06b` and its landed attunement/replan work.
Read-only merge preview found two conflicts: client module composition and the
generated Gate H index. Neither is an engine admission conflict.

Before: this branch generated fixed-session POST Rust markers with Python,
while incoming main introduced a Cargo-time Rust generator for attunement.
After: the landed `rrd-client/build_support.rs` produces one operation module,
including the fixed-session POST metadata needed by the existing sealed
`ValidatedOperation` API. The public proposal call and typed attunement helpers
retain their contracts. Metadata alone cannot invoke an unsupported operation.

## Implementation traceability

The superseded `scripts/ci/rust_client_projection.py`, its Python tests and
`rrd-client/src/generated/operations.rs` are removed. Their deterministic,
metadata-change, malformed-input and removal checks move to the existing Rust
`tests/generated_catalogue.rs`; CI invokes that test instead. The surface
checker no longer maintains a second Rust output. Cargo generates the one
compiled projection directly from `rrd_contract::endpoint_catalogue()`.
No handwritten endpoint registry, provider ranking, retry or lifecycle is added.

The proposal's sealed call, bounded single-send transport, request identity
validation and four transport/real-server tests match the previous branch
byte-for-byte after newline normalization. Incoming engine proposal/attunement
owners and the real-server installation/replan tests likewise match their parent.
The attunement and legacy operation modules differ only in the generated module
import. Nine file comparisons are retained in the local evidence directory.

Full-file reads covered AGENTS, the authored generator and its replacement,
generator tests, client module composition, sealed call, and attunement helpers.
Focused reads covered README owner links, change-authoring, H-04/POAM-011,
Rust SDK reference, endpoint catalogue validation, transport, existing proposal
and replan tests, surface generation and CI registration. Reviewed diffs bind the
consolidation to those owners; incoming source identities are retained rather
than claiming a new review or qualification of every upstream implementation.

Research decision: no external algorithm or dependency adaptation; this is a
repository-owned projection consolidation. Trace/resource decision: preserve
bounded transport and existing receipts; no prompt/credential logging, GPU call,
performance measurement, dataset label or training. Stop on admission/retry
changes, missing behavior migration, duplicate authority or unrelated failures.

## Verification

The first focused build failed with Rust E0446: the public metadata implementation
exposed private `GeneratedOperation` associated types. The correction emits the
public contract types directly from the same descriptor; it does not make the
private dispatch trait public. The original failure log remains retained.

The corrected focused run passed all five generator and four controller tests.
Generator probes modify copies of the real catalogue solely to test compilation;
they are not provider observations or training/evaluation records. Controller
tests retain actual archived responses for explicit corruption/drop probes and
use a real engine socket for authorization/configuration denials. Archived
response replay is not a new live provider result.

- `cargo test --locked -p rrd-client --test generated_catalogue --test controller_proposal`:
  nine passed after the visibility correction.
- `cargo test --locked -p rrd-client -p rrd-contract`: 182 passed, zero failed or
  ignored, including the unsupported-operation compile-fail example and the
  typed attunement control/replan/reopen tests.
- `cargo clippy --locked -p rrd-client -p rrd-contract --all-targets -- -D warnings`
  and `cargo fmt --all -- --check`: passed.
- `python scripts/ci/check_generated_surfaces.py`: passed, 71 HTTP operations,
  seven signal projections, OpenAPI digest
  `f1856602eb3b5ba419f600b3be0e9626d62ad533032fe220a2728795bd28048a`.

The final documentation, workflow, generated navigation, execution inventory and
whitespace checks are retained with command/exit-code logs. The journal needed
the normal Status/Coordinate/Owner header before generated navigation discovered
it; the generated Gate H index now links this record.

Change checklist: scope/owner and stop conditions bound before mutation;
consolidation destination and replacement oracles recorded; original failure
preserved and corrected at its generator; parent semantic owners preserved;
authored changes reread and reviewed against both parents; generated discovery
refreshed; no new dependency, secret, model data or completion authority. Historical
journals and live receipts remain unchanged. The Rust reference also removes a
stale historical-table introduction left by the otherwise clean document merge.

## Remaining evidence and gates

Existing desktop and BF16 receipts retain their original client/engine identities.
No new installed executable, Connectome SDK pin, endpoint call, container or
installer is qualified by this integration. The installation/replan client suite
uses an in-process engine/server with a test installation identity; it cannot
substitute for a packaged executable or whole-project worker workflow.

PR206 remains draft behind the unresolved protected macOS APFS failure at
C-07 / POAM-035. PR222 stays draft and has no new candidate/protected acceptance.
Connectome refresh and actual installed verification follow this dependency.
Roadmap and POAM completion status are unchanged. Local command logs, the first
failure and parent-preservation receipt live under the operator's
`native-client-attunement-integration` evidence directory for this date.
