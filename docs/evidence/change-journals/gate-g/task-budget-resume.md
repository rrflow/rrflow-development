# Explicit recovery under the original task budget

**Status:** active implementation evidence; installed provider and release acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-budget-resume`
**Owner:** G-01b/G-02a / POAM-009; PR230; active SWAG engineering session

## Brief, baseline and canonical boundary

[PR230](https://github.com/rrflow/rrflow-development/pull/230) carries the
pre-implementation brief. The branch starts at main `df26b35512176428bf4847784501c28d6a123474`
and ordinarily composes worker PR229 `d5151eefc9c85bfea674a0d9184cc6d4d81ba7dd`.
Composition `ec681d925405983d1b8798dd2ba277af51e27fb4` has the exact worker tree
`b59fe96db0bdde88e2603c4f54601a8d4f0565d6`; no new runtime behavior precedes
the published brief. PR206, PR222 and PR229 remain dependencies, not implicitly
landed code. The change advances usable task recovery in the existing alpha
project-to-learning journey without closing that outcome or a roadmap gate.

The [installed terminal-result history](task-worker-terminal-live.md) demonstrated
the missing behavior: another task could use released capacity, but the original
known-completed stopped task had no public resume operation. The operation
catalogue at the baseline exposes configure/read/stop, with no resume.

The new `task-budget-resume` request binds scope, task and the exact current
budget digest. `RrdEngine` permits only `cancelled` or `attempt_failed` budgets
with no active/unknown execution, an open task, unchanged estate configuration
and remaining original time/steps. The existing control/runtime transaction
compares budget, session and security authority bytes with the runtime cursor.
Concurrent revocation, task closure or budget changes cannot authorize a stale
commit. Resume clears only the current stop and Stop replay marker and advances
the observed clock. The original limits/deadline, spent steps/time, repetition
counts and attempt/output records remain intact. Prior Stop evidence remains
in the existing event/control audit; old Stop requests cannot stop resumed work.

Resume does not call a provider, clear capacity, authorize a worker or retry an
attempt. New attempts pass ordinary current admission and spend another step.
A lost resume acknowledgement requires public readback; stale digest retries
conflict. Unknown-outcome reconciliation and extending/resetting budgets are
outside this package.

## Implementation and review

The contract owns `ResumeTaskBudget`, request validation and native-response
identity/state validation. Its catalogue and operation reference generate the
public OpenAPI and language SDK surfaces. The HTTP handler calls the engine;
the Rust client uses its existing catalogue-derived, single-send carriage with
the contract validator. No additional lifecycle store, provider policy,
hand-maintained endpoint metadata or public authority is introduced.

Changed authored paths are the budget contract/catalogue/reference, the engine
budget module and new `resume.rs`, budget view visibility, worker recovery tests,
server budget handler/router, native client validator/catalogue/HTTP tests,
the inference reference, POAM link and this journal. Generated SDK/signal
projections, current OpenAPI digest citations and navigation/inventory follow
their existing owners. No runtime source is deleted or relocated.

Review followed AGENTS/README, the alpha objective, Gate G, POAM-009,
change-authoring and CI procedures. Implementation inspection covered budget
reservation/completion, public configure/read/stop, transactional persistence,
endpoint authority guards, terminal-response tests, HTTP handlers, operation
catalogue and native carriage. The new resume implementation and tests were
read completely; existing large owner files were reviewed at the affected
symbols and through their full change diffs. The nearest index is generated.

Research/source adaptation: no external algorithm or library change; existing
engine authority and transaction semantics own this increment. Trace/debug:
reuse the control audit and request/operation identifiers, without another
telemetry authority. Resource/performance qualification is not claimed for a
bounded state transition. Stop conditions prohibit clearing unknown leases,
resetting budgets, altering prior attempts or bypassing current authority.

## Verification

The first focused engine run passes both new tests. The expanded worker group
passes all ten tests, including matching terminal rejection versus lost response,
explicit resume after reopen, unchanged capacity, old-attempt rejection and
denial of active/uncertain work. Resume tests preserve exact budget fields and
attempt readback across another engine reopen; only a new admitted attempt
spends a further step. Denial cases cover unknown work, spent step limit,
deadline, clock rollback, changed configuration, invalid token, closed task and
stale budget digest without changing the budget.

The real authenticated HTTP/native-client test passes: read-only caller denial,
Stop/reopen/resume, preserved effort/deadline, stale resume and old Stop denial,
repetition-stop denial, and a separately admitted builtin embedding after resume.
These are engine/protocol probes. Retained real task inputs and controlled fault
envelopes are not generated training data, model-quality labels or evidence of
a new live BF16 call.

The initial full engine run failed its SDK corpus bootstrap check (300 passed,
one failed, four ignored): the corpus expected 72 endpoints. The canonical
catalogue now has 73. The reconciliation derives the count from the contract
export; a structural comparison proves `expected.endpoint_count` is the corpus's
only changed value. The subsequent full engine run passes 364 tests with four
existing ignored tests, including all 32 architecture checks. Log SHA-256:
`a0e834721069a08309dc387e57f43bab52e64c7a4f5b73247ea7ddcc953ae8f5`.

The first transport owner run then failed the intentional OpenAPI digest oracle:
the public advertised digest still named the previous schema. Review confirmed
the added authenticated operation/request schema and existing response contract;
the advertised constant now matches the actual export
`1344d5a23f04dc0a2cd9efb8743caaa945be0e3691c61868d2577d3f419c2139`.
The failure log is retained with SHA-256
`e85e7275cc1efb59c4b6b0b3e76d44e445687aaf9da1691009c9c814b984326a`.
Six engine bootstrap checks pass after this metadata-only correction. The full
engine suite above predates that constant correction; runtime resume code is
unchanged. The corrected combined transport run passes all 189 contract/client
tests and 29 server tests before the process-driver target reports a missing
`rrd-estate-controller` test binary. No assertion or runtime code is weakened.
The CI-owned prerequisite `cargo build -p rrflow-cli --bin rrd-estate-controller
--locked` supplies it; the complete server suite then passes all 33 tests,
including process-kill/reopen. Server log SHA-256:
`71695f2857d3acb8fc0a09c1acf5da14253eca52a2a129cfea1f3923cb7e481a`.
Strict Clippy across engine/contract/server/client, all targets and features,
passes with `-D warnings`; Rust formatting passes. Clippy first rejected an
unnecessary same-type `.into()` in the new test fault string. Removing that
conversion changed no runtime behavior; the failure remains recorded. Passing
lint log SHA-256:
`ded2757e3656d1a3b2850d9d3c24b956090bc6c48cb95112a6fb5078cfd5be61`.

All five language projections generate successfully; parity verifies 73 HTTP
operations and seven signal projections. TypeScript generation check, lint,
type checking and all six tests pass. No language SDK semantics or dependencies
changed. Navigation/documentation checks pass with 867 nodes, 3,124 edges,
868 document statuses and 866 coordinates; inventory includes 2,337 paths.

The initial SDK install used ambient pnpm 10.30.3 and rejected the newer lockfile;
no lockfile was changed. Re-running with the repository-pinned pnpm 12.14.0 and
Node 24.22.0 installed the frozen dependencies successfully. The failure remains
in the local evidence alongside the correction.
The first Python generator call lacked the locked formatter; the repository's
frozen `uv` environment resolved it. A local citation-update wrapper encountered
Windows default decoding; explicit UTF-8 completed the remaining citations.
Successful generators were not rerun for those wrapper corrections. No captured
model response, prior trial or failed test was replaced.

Acceptance commands (locked Cargo, all features/targets where stated):

- `cargo test --locked -p rrd-engine --lib --all-features engine::tests::reasoning_budget::worker`: ten passed.
- `cargo test --locked -p rrd-client --all-features --test task_effort`: real HTTP resume/readback/next-step check passed.
- `cargo test --locked -p rrd-engine --all-features --all-targets`: 364 passed, four ignored before the advertised schema-digest correction; six `installation_bootstrap` checks passed afterward.
- `cargo test --locked -p rrd-contract -p rrd-server -p rrd-client --all-features --all-targets`: component results and retained prerequisite failure above; `cargo test --locked -p rrd-server --all-features --all-targets` passes after the CI fixture build.
- `pnpm --dir sdks/typescript generate`; `uv --directory sdks/python run --frozen python scripts/generate.py`; `go run ./cmd/generate` in `sdks/go`; `python sdks/java/scripts/generate.py`; `python sdks/dotnet/scripts/generate.py`: pass.
- `python scripts/ci/check_generated_surfaces.py --write` and its non-writing check; `pnpm --dir sdks/typescript check`: pass.
- `cargo clippy --locked -p rrd-engine -p rrd-contract -p rrd-server -p rrd-client --all-features --all-targets -- -D warnings`; `cargo fmt --all -- --check`: pass.
- Navigation, execution inventory and documentation checks pass as above.

The full multi-language live conformance campaign, other native platforms,
installed BF16 resume, resource/performance qualification and distribution
checks were not run for this component increment. Required candidate CI and
the separately frozen installed trial must supply those applicable results.

Commands, exit statuses, log hashes and the validation driver are retained in
SWAG `.swag/evidence/20261031/task-budget-resume/`. No provider credential or
historical estate was changed. Existing AGENTS and historical journals remain
unchanged. Candidate/protected CI and dependency landing are still required.

## Remaining acceptance

Freeze a clean-source primary with the canonical build tool and exercise an
actual completed rejected BF16 attempt, public resume and separately authorized
new work on the same task, preserving original effort/deadline through restart.
The old binaries do not inherit resume. Connectome requires the exact composed
SDK/engine pin and an explicit recovery control with installed UI proof.
Unknown leases remain unresolved. Ordinary-edit UI replanning, governed project
effects, correlated RROFLOW outcomes, independently reviewed learning, model
qualification and native/container/public release remain open. No version or
roadmap/POAM lifecycle status changes.
