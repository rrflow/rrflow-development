# Selected worker admission and retained drafts

**Status:** active local implementation; qualification and publication pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-model-admission`
**Owner:** G-01b / G-02a / POAM-009; driver: active SWAG session

## Scope and source

This continues PR229's pre-edit worker-activity brief from published
`cea6c846a1fdc80c5f957933fdd3918117d3e850`. The prior
[transport/configuration record](task-worker-model-transport-policy.md) remains
immutable. Product version remains `1.0.0`. No roadmap gate closes here.

The executable catalogue, authorization action and HTTP handler now compose
`task-worker-execute`. The engine resolves an accepted selector receipt from
the existing attempt store, checks exact task/configuration/allowance identity,
rejects abstention, and resolves only explicitly configured worker settings.
It uses the canonical context assembler, bounded model transport, existing
task reservation/completion and durable readback. It creates no second scheduler,
credential store, effect ledger or project mutation path.

Worker and selector leases share the existing capacity key and per-binding
limit. Unknown remote execution survives reopen and continues occupying capacity.
Completion guards retain the security, session, configuration and selection
bytes. A changed task or authority withholds output; eligible subsequent work
uses the remaining budget and original deadline. Repetition identity excludes
incidental runtime read-cursor advances, while output retains exact packet hashes.
Worker drafts require TaskRead on readback and MemoryContextRead when derived
from canonical context. Context provenance is historical evidence, not current
filesystem freshness or permission to apply generated text.

## Verification

The initial missing-worker API test failed compilation before implementation;
raw `task-worker-admission-first-oracle.log` is retained in the SWAG evidence
directory, SHA-256
`19b0c755d02376d12e9469c125b90145fa08dd8a53495c94f8fc78e9ad2e95d9`.
Later compiler checks exposed a read-stamp field spelling and a
missing test import; their original logs remain. The first controlled server
inherited a nonblocking accepted socket on Windows; explicitly restoring
blocking reads corrected the test fixture without changing production transport.
The context probe initially exceeded the estate's configured storage-key ceiling;
the engine denied it before dispatch. The corrected probe requests 10,000 keys
within the unchanged 100,000-key configured ceiling.

`cargo test -p rrd-engine --lib --all-features --locked reasoning_budget::worker`
passes seven tests in `task-worker-admission-context-bounded.log`:

- Real engine selection and worker reservations precede their controlled HTTP
  exchanges; exact output hashes and unknown usage survive reopen; replay does
  not dispatch or consume more effort.
- Wrong, missing, stale and abstaining selections fail before worker effect.
- Every worker permission, including context access, is required.
- Configuration/task/authority changes, Stop and actual socket response loss
  withhold draft acceptance and survive reopen; uncertain capacity blocks both
  another worker and the controller on the same binding.
- A new selection and worker succeed after configuration invalidation with four
  spent steps and the original deadline and limits.
- Canonical seeded task context is actually assembled/exported; revoking context
  permission subsequently denies derived-output readback.

These are controlled protocol fixtures, not LLM outputs, task-quality evidence,
training examples, review labels or consent. Installed configured-provider
evidence and full composed SDK/installation conformance remain pending.

The canonical exporter initially rejected the new reference entry because it
was outside the reference catalogue's binary-search order. Correcting its order
allowed recursive comparison against the retained `cea6c846` export. Exactly
twelve structural changes remain: eight security-action enum additions, two
optional retained-output projections, the worker operation, and endpoint count
71 to 72. The reviewed OpenAPI digest is
`a0cc8919d190031c4e0e1493be97e66e6b9ae86479a407e29395dc66cee01f45`.
The first broad contract suite then exposed the shared conformance corpus's
old 71-operation expectation; it was updated to the observed 72, without removing
assertions. That failed log has SHA-256
`9e3f417f0922a9790d2b371258c2047e03a7a534cd7c8334161418090d521f4e`.
The corrected contract/inference/client run passes 218 tests, followed by all 298 engine
library tests (four existing helpers ignored). The engine log has SHA-256
`097530206ad39c5d4cd6739d2771d4026c5b7068477d71461d226bf15c2fd106`.

The Rust client now exposes a typed worker method through the existing generated
operation projection, validating request/result identity and limiting this call
to one transport attempt. Its authenticated real-server denial checks the new
HTTP route. Review found that an initial retry test returned an HTTP error which
the shared transport already would not retry; the revised oracle closes the
socket after receiving the complete request and observes until client completion.
Only the revised test can substantiate no retry under transport uncertainty.
That revised test and all 18 client tests pass; its log SHA-256 is
`5c8ce358bed3db148f21ed84fb3b3aed510d6e5b31a85940bc037e56fc672206`.
Strict all-target/all-feature Clippy passes for engine, contract, inference,
server and client, log SHA-256
`357ccef2495c9373147eac9e85844eb709e555c497a9058b03a8ef2fa646ea5e`.
Canonical TypeScript/Python/Go/Java/.NET generators and signal projections pass;
parity reports 72 operations and seven signals. TypeScript generation, lint,
typechecking and all six tests pass, log SHA-256
`60b0895eaa006fc0e34a6f2dbbdfa0eb4b142e69ca28b94f95edb6c1127430c4`.
The local `task-worker-admission-*-validation.json` records retain exact commands,
exit codes, elapsed time, log hashes and source-file hashes. The first failed
owner run and the initial context/protocol failures remain alongside them.
All 32 workspace architecture checks, Rust formatting and whitespace checks pass.
Architecture log SHA-256:
`3ac72e3365dead48401fc8ef9125cf94c0c46d29e8308c0f14565a3767f7c231`.

Owning acceptance commands (run from this worktree with the locked dependencies):

- `cargo test -p rrd-contract -p rrd-inference -p rrd-client --all-features --locked`: 218 pass.
- `cargo test -p rrd-engine --lib --all-features --locked`: 298 pass, four existing ignored helpers.
- `cargo test -p rrd-client --all-features --locked`: 18 pass after strengthening the socket-loss oracle.
- `cargo clippy -p rrd-engine -p rrd-contract -p rrd-inference -p rrd-server -p rrd-client --all-targets --all-features --locked -- -D warnings`: pass.
- `cargo test -p rrd-engine --test workspace_architecture --all-features --locked`: 32 pass.
- `pnpm --dir sdks/typescript generate`; `uv --directory sdks/python run --frozen python scripts/generate.py`; `go run ./cmd/generate` in `sdks/go`; `python sdks/java/scripts/generate.py`; `python sdks/dotnet/scripts/generate.py`: pass.
- `python scripts/ci/check_generated_surfaces.py --write`, its check without `--write`, and `pnpm --dir sdks/typescript check`: pass using pinned Node 24.22.0/pnpm 12.14.0.
- `cargo fmt --all -- --check`, `git diff --check`, `python scripts/knowledge/render_navigation.py`, `python scripts/ci/build_execution_inventory.py`, its `--check`, and `python scripts/ci/check_documentation.py`: pass. Inventory reports 2,170 paths; documentation reports 847 statuses and 845 coordinates.

These suites overlap and are not an additive total. Required candidate/protected
CI, an installed exact-source worker call, remaining credential/policy and process
interruption cases, and composed UI/SDK acceptance still gate completion of PR229.

## Authoring and remaining acceptance

Read the root instructions/README and alpha/POAM owners, existing PR brief and
transport journal, full capacity owner and proposal flow, task reservation and
completion sections, public readback, authorization/context owner sections,
contract request/output validators, and existing protocol/configuration tests.
New implementation/tests are read in full as authored. The large contract root,
security and context modules were inspected at relevant sections; no complete
review of those files is claimed. No upstream code is copied or new protocol
support inferred. Existing bounded errors and exact source/output hashes suffice;
no latency, cost, memory or model-quality claim follows from these checks.

Changed owners are `rrd-contract::endpoint_worker`, typed task-attempt output,
`rrd-engine::endpoint_worker`, shared endpoint capacity and existing attempt
readback/completion. Public catalogue/action/handler and generated clients must
remain one mechanical projection. Keep PR229 draft until its remaining acceptance
is satisfied. Native subscription adapters, governed project effects, UI worker
flow, RROFLOW correlation, reviewed learning and release qualification remain
requirements of the full SWAG objective.
