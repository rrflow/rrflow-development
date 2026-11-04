# Engine-owned LFG advisory checkpoint

**Status:** active implementation-stage evidence; draft and qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/lfg-engine-admission-checkpoint`
**Owner:** OBJ-12 / G-01a, G-01b and G-02 prerequisites / POAM-009
**Pull request:** [249](https://github.com/rrflow/rrflow-development/pull/249)

## Brief and baseline

The public draft began at `90042a3e7e5d412a2da529b6a4a0db609e9bc0c5`.
Current public main `ed3027b79e41dfcdf5b27e57f07346f11f6da347` was merged
without rewriting history, yielding local starting commit
`a37d6d3f34a82e8435cdc29255ccdffc099e903c` and tree
`cb6b3cdb0d4ad3c19854aaed020f9fc899de7edc`. The PR brief was updated before
implementation and again when the checkpoint and public projection became
necessary. The separate forward-compatibility PR 246 remains a dependency.

Before this change the LFG transport existed but engine selection did not
invoke it. The target is an explicitly configured, bounded, optional LFG call
inside the existing task reservation, followed by an engine-owned durable
advisory checkpoint and controller proposal. No worker execution, new engine,
automatic model ranking, trusted producer identity, default endpoint, or
external runtime dependency is introduced. Stop for unbounded I/O, unresolved
effect replay, bypassed guards, source-dependent project trapping, or a second
lifecycle authority. The PR remains draft until its dependencies, generated
projections, complete review, tests and candidate/merge-group checks pass.

## Changed behavior and evidence boundaries

`EndpointSelectionPolicy` accepts an optional loopback LFG configuration with
explicit network and unverified-identity permission, pinned assessment/rules
digests and limits. Absence preserves the prior serialized representation.
The existing engine reserves task effort and controller capacity before the
call, validates analyzer identity and bounds, checkpoints its advisory through
the task ledger, then constructs the controller packet. A stale task or
authority returns an invalidated attempt after recording the completed
observation when the change is detected before controller dispatch. Its
successor uses current inputs. The pre-dispatch check is not atomic with the
network call; a later change can reach the controller but cannot publish an
accepted result if completion detects it. An uncommitted or uncertain
external effect cannot be silently replayed; capacity remains held when the
checkpoint cannot be established. `task-attempt-read` exposes the checkpoint.
No remote identity attestation, remote cancellation guarantee, or quality claim
follows from matching digests. A crash after remote dispatch but before the
checkpoint can leave an unresolved attempt without an observation.

Implementation paths: `rrd-contract` endpoint policy/proposal, reasoning budget,
export and tests; `rrd-engine` endpoint proposal, reasoning budget persistence
composition and tests; generated SDK and signal projections and current
reference digest citations; inference reference; this journal and navigation.
No existing implementation was removed or relocated. The change does not
implement Parquet compaction, predicate pushdown, PostgreSQL compatibility,
multi-day reasoning, C-07 native capacity or the installed project journey.

Files read in full include the LFG advisory decoder and the macOS capacity
metadata and APFS helpers. Targeted reads covered the README and owner chain,
the change-authoring procedure, endpoint policy/proposal, LFG transport,
engine endpoint composition, budget reservation/completion/public persistence,
security authorization, existing tests, generated-surface owner and SDK
generators. These are not a whole-codebase or every-line review.

Research: [PostgreSQL WAL](https://www.postgresql.org/docs/current/wal-intro.html)
requires durable WAL ordering for crash recovery; an in-memory-only log is not
durable. [Arrow's Parquet row filter](https://arrow.apache.org/rust/parquet/arrow/arrow_reader/filter/struct.RowFilter.html)
documents late materialization and its workload-dependent tradeoff.
[pgrust](https://github.com/malisper/pgrust/blob/main/README.md) calls its
embedded variant future work and advises against production use; no upstream
code was copied. The adapter's configured deadlines, limits and retained wire
digests provide bounded diagnostics, not a benchmark. Existing control events
carry the attempt digest; raw task text and service responses are not logged.

## Local verification and remaining evidence

The first policy test failed to compile because the opt-in contract did not
exist; after implementation its five focused cases passed. The four focused
engine LFG tests passed: synthetic two-service HTTP composition and readback,
checkpoint persistence across reopen and replay denial, stale guard conflict,
and invalidation with a successor attempt. The invalidation/successor test
injects the invalidation. A subsequent engine HTTP test actually changed the
configuration while the analyzer response was held, then verified checkpoint
readback, invalidation, no controller connection and a released task budget.
A malformed controller response test verified that the LFG observation remains
readable and the uncertain capacity is retained. The synthetic HTTP servers do
not establish actual LFG/SWAG model behavior or performance.

The first complete Windows engine library suite exposed an inherited
nonblocking-socket mode in the new test; the test now explicitly sets the
accepted socket to blocking. The final `cargo test --offline --locked -p
rrd-engine --lib` passed 338 tests with four ignored. The first contract suite rejected the expected OpenAPI digest drift;
the generated digest and current reference citations were updated before the
suite was rerun. The Windows pnpm shim failed to launch; invoking
the pinned pnpm module and an offline cache succeeded. The Python generator
initially lacked Ruff on PATH; the pinned cached Ruff 0.17.1 was used instead.

After regenerating the TypeScript, Python, Go, Java and .NET endpoint
projections and seven signal projections, `cargo test --offline --locked -p
rrd-contract` passed all its active tests; `python
scripts/ci/check_generated_surfaces.py` passed with 75 operations and OpenAPI
SHA-256 `e9f6363967936948b3405c7ecbf9afa4f9be77eddf95d73727ae83aa51506971`.
`cargo clippy --offline --locked -p rrd-contract -p rrd-engine --all-targets
-- -D warnings` passed. These validate the current code and generated
metadata, not deployed cross-language behavior.
TypeScript `pnpm typecheck` (using the pinned module), `go test ./...`,
documentation policy and navigation check passed.
The LFG decoder and transport focused suites passed five tests; the real LFG
service test was ignored because no explicitly launched service was supplied.
Execution inventory generation produced 2,356 path records.

The earlier PR 256 merge-group [run 41600990700](https://github.com/rrflow/rrflow-development/actions/runs/41600990700)
failed on macOS at `capacity_metadata.rs:20`: 758 filler files, 892 creates,
139 barriers and 25 seconds elapsed before native pressure/recovery proof.
PR 256 candidate CI was green but does not erase that merge-group failure. An
accidental new queue entry was removed after inspecting it. POAM-035 owns the
existing native-capacity gap; neither this package nor a rerun closes it.

Review: changed production and test hunks and the new integration test were
reviewed; generated projections were checked against the contract exporter.
Every changed file's complete bytes were reread for the path/hash inventory;
this is not a semantic line-by-line review of all unchanged content in those
files or of the rest of the repository.

Pending: a source-edit-during-call integration test, actual
LFG/SWAG processes with identity evidence, installed/native qualification,
candidate and merge-group CI, and C-07 capacity/soak. No alpha, roadmap gate,
POA&M item, or merge completion is claimed. No release or promotion occurred.

Checklist: PR brief and owner/baseline recorded; first failure and changed
boundary identified; bounded trace/resource decisions and research recorded;
focused/owning tests and failures recorded; the full pipeline remains pending.
This record is evidence, not an
independent lifecycle or status authority.

## Real LFG engine flow and profiler preflight, 2026-11-04

On draft head `f8be2ab375e817ef7c460a60ac79fe8c7ee3b8e2`, the Windows
checkout was clean before this follow-up. Public main was
`746a0a462e512563b6d3aaa89520f8ab5cde9051`; this result has not been tested
on a candidate integrating that main. The already-running local LFG service
answered `/health` as `lfg-server` on `127.0.0.1:54330`. Its reported process
executable path was `D:\projects\lfg\target\worktrees\lfg-devtools\target\debug\lfg-server.exe`;
the file at that path hashed SHA-256
`91f79f04ea56b360654f0c5071925e648a8f0918ec84baa09c2b6145c003e7b0`.
Its source worktree HEAD was `fb6e7508203da6e5fddd4287cb4f24526fc5a715`.
These observations are not build attestation and do not prove that the file
on disk was the exact executable image loaded by the running process.

- `cargo test --offline --locked -p rrd-inference --features endpoint-chat
  --test lfg_transport real_lfg_service_returns_bound_advisory_data --
  --ignored --exact --nocapture`, with `RRFLOW_TEST_LFG_URL` set to the
  service above: **1 passed**, exercising the actual transport and synthetic
  prompt against the live service.
- Extended `crates/authority/rrd-engine/src/engine/tests/reasoning_budget/lfg_flow.rs`
  with an explicitly ignored, opt-in real-service lane that shares the
  existing engine and controller-fixture assertions. It derives assessment
  and rules digests from the existing fixture; it does not set a production
  endpoint or alter engine behavior.
- `cargo test --offline --locked -p rrd-engine --lib
  real_lfg_service_flows_through_engine_controller_and_reopen -- --ignored
  --nocapture` with the same URL: **1 passed**. The test admits the configured
  LFG call, checks that the observation enters the controlled HTTP controller
  packet, checks digest binding and durable task-attempt readback, reopens the
  engine and refuses a duplicate proposal. The controller was a synthetic
  fixture: this is **not** standalone SWAG, model-output, installed-binary,
  source-edit, or native CI evidence. The test's in-process engine is real.
- `cargo test --offline --locked -p rrd-engine --lib
  'reasoning_budget::lfg_flow::' -- --nocapture`: **3 passed; 1 ignored**.
  The opt-in real-service case was separately run above. `git diff --check`
  passed. `cargo fmt --all --check` initially failed on the new test; after
  `rustfmt --edition 2021` formatted the affected file, it requires a final
  check for this exact candidate.

For diagnostic research, [OpenTelemetry's Rust SDK status](https://opentelemetry.io/docs/languages/rust/)
reports traces beta and metrics/logs stable; the [GenAI conventions](https://opentelemetry.io/docs/specs/semconv/registry/attributes/gen-ai/)
identify optional content fields as potentially sensitive. This supports
bounded, redacted optional diagnostics, not automatic prompt capture or a
claim that RRFlow emits the complete standard. [Samply](https://github.com/mstange/samply)
was installed locally with `cargo install --locked samply --root
D:\localdev\devlabs\rrflow\target\rrflow-devtools`, version 0.13.1,
executable SHA-256
`256c6c5ec24a570b21287d52d8c4c600f4dca6c27832537a231d5329ad6f6f7d`.
The locked install warned that two transitive packages were yanked; no runtime
or distribution dependency was added. A save-only capture of the engine test
failed: `samply requires Administrator privileges for profiling on Windows`.
No profile or performance result was produced. Linux ARM host `clyffy-01`
reported `perf version 7.0.14` and `perf_event_paranoid=4`; no privilege or
system setting was changed and no Linux capture was attempted.

Remaining: reconcile the actual draft against current main, source-edit and
revocation histories, actual SWAG controller, installed executable and SDK
flow, native qualification and candidate/merge-group CI. The prompt-flight
provider remains observe-only: endpoint-selection is not a general model
provider invocation. This evidence does not change a roadmap/POA&M status.

Follow-up verification: `cargo fmt --all --check` passed after formatting. An
initial full engine suite was interrupted by the operator while running; its
partial output is not a suite pass. A separately launched full-binary rerun
is pending. On `clyffy-01`, `perf stat -- true` exited successfully but
reported CPU cycle events as `<not supported>`; `perf stat -e task-clock --
true` exited 1 with access limited under `perf_event_paranoid=4`. The first
command measured only the `true` process; neither is an RRFlow performance
profile. No system permissions were modified.

Documentation/navigation checks for this follow-up: `python -X utf8
scripts/knowledge/render_navigation.py --check` passed (18 indexes; no
updates); `python -X utf8 scripts/ci/check_documentation.py` passed. A
merge-tree preview against public main `746a0a462e512563b6d3aaa89520f8ab5cde9051`
identified 19 conflicting paths: contract source and generated SDK/reference
projections. Resolve from the combined owner, not by accepting one generated
side. PR #246 remains open; PR #235 is merged. These are integration findings,
not evidence that a combined candidate builds.
