# D-02 durable installed attunement control and Connect worker

**Status:** active partial implementation with local evidence; D-02 and alpha remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-durable-attunement`
**Owner:** OBJ-07, D-02a/b and POAM-006

## Brief, baseline and admission

Starting revision `fbddf214dba5b4cd1b832c4524a07b4d15f329d9`, tree `5a566de938fc76ef8d59831ff35052d859fabe36`.
Fetched development/main `dd225072bcc639c89b0eef968625cc73303ea998`; created an
isolated branch from it and fast-forwarded the required PR137 context. The
owner's GET IT DONE authorizes bounded runtime work after that planning audit.
[PR138](https://github.com/rrflow/rrflow-development/pull/138) carried the exact
implementation brief before edits. PR136/137 remain dependencies; no main push
or official-repository promotion is performed.

Before: installation persisted a pending job but no engine handler could run,
fence, pause, cancel or resume it. After: explicit authenticated embedded
`read_attunement`/`control_attunement` operate that same job. This is an additive
Rust engine API; existing HTTP/SDK operation catalogue and wire models are
unchanged. EstateRead admits status and EstateAdmin admits mutations. Neither
installation, task-open nor readiness activates work. Public client dispatch,
source-effect authorization and the complete D-02c settings surface remain open.

## Implementation and traceability

New `crates/authority/rrd-engine/src/engine/attunement.rs` owns Start, Resume,
Step, Pause, RecoverExpired and Cancel at RrdEngine. Canonical installation
plan/job key helpers are shared internally; no second job store exists. The
engine module/root export the Rust command/result types. New tests live in
`engine/tests/attunement.rs`, registered from the existing test module.

Each mutation checks current authorization, expected revision and relevant
plan/checkpoint/worker identity. Principal-scoped idempotency survives session
renewal; replay returns the original accepted snapshot and still requires
current authorization. Existing control commit_batch atomically persists the
job and receipt through authoritative rrflowKV durability. Control journal
entries retain actor, action and request/operation correlation. No prior
implementation is deleted or moved; existing installation key bytes remain
identical. The existing installation suite verifies that later job states
remain valid for installed inspection/reopen.

A lease binds the claimant session and monotonically advancing job revision.
A superseded, foreign, paused or expired worker cannot publish. Expiry recovery
persists Paused before a new claim. Cancellation invalidates old revisions and
then acknowledges terminal cancellation; this worker owns no asynchronous
external resources. Source/provider work is deliberately not authorized here.

Connect verifies installed identity/configuration and binds a real runtime
read stamp to its engine-computed checkpoint. The caller cannot submit phase
outputs or checkpoint hashes. Inventory produces a durable `phase-unavailable`
failure, retaining only the Connect checkpoint. Retryability allows a later
installed executor to resume the same phase; there is no automatic retry loop.
This is not source inventory, parsing, embeddings, graph production or a
query-ready project demonstration.

Encoding rejects beyond 64 KiB while writing, not after allocating unbounded
output. Lease duration is caller-selected within 1..60000 ms. Normal job work
stops at revision 1024, reserving at most two transitions for cancellation.
Admission uses the existing transaction gate; elapsed waiting/computation and
authorization time count toward the request deadline and worker lease before
publication. These are this package's bounds, not a claim of all D-02c or C-07
resource qualification. Full scheduling, total job budgets and UI configuration
remain separate unimplemented requirements.

## Code and evidence inspected

Read the full new implementation and its authored tests during review, plus
changed registration/key/export diffs. Existing targeted reads: contract
AttunementPlan, Job, Checkpoint, Lease, Resume and Cancel validators; installation
plan/job publication, verification and identity helper; knowledge.rs CAS/receipt
path; control.rs (full helper file); core installed fields/readiness; session
authorization and authenticated-session creation; repository control batch
implementation; installed test fixture/reopen helpers; engine test registration;
crate manifest/configuration. Documentation: README identity/status, Gate D
packages, POAM-006 admission gaps, AGENTS, execution-map/change-authoring routine
and agent-bootstrap independent integration/source-adaptation rules.

Research decision: existing repository transaction, fencing and control receipt
semantics are the reference; no external algorithm or upstream source is
adapted. No external provider, model, Nomen or cloud capability is invoked.
Native traces are the real installed job/checkpoint/control receipts, not model
chain of thought or a schema of reasoning.

## Failure oracles and results

Focused cases install both fresh and existing estates, then verify Start ->
Connect -> explicit unavailable Inventory, exact receipt replay after later
state changes and reopen, retryable phase resume, competing claims, foreign
workers/generations, pause/expiry recovery, cancellation fencing and terminal
persistence, invalid identity/token/deadline/lease, oversized encoding, finite
transition limits with a cancellation reserve, and isolation of two installed
projects. No manually fabricated indexed data is presented as ingestion proof.

The native subprocess case opens the installed estate, durably claims a job,
and is killed with its storage handle still open before the accepted response
is delivered to the parent. A new process owner replays the original receipt,
recovers the expired lease, resumes and publishes one Connect checkpoint.
The child uses the Rust engine test executable, not the shipped rrflow CLI;
this is Windows native installed-engine process evidence, not distribution,
all-platform crash-boundary or 24-hour soak qualification. Its ignored helper
is explicitly invoked by the passing parent test.

Surfaced errors: first cargo check found CorrelationId Display and core-error
conversion mismatches; fixed through existing as_str/error mapping. The first
focused build used the full plan digest as job identity while installation uses
its canonical 24-character prefix: all five original cases failed before work.
The fixture was corrected to the actual identity rule; no production validation
was weakened. Review moved final lease/deadline checks after authorization so
its elapsed work is included. Broad exploratory output truncations and guessed
paths were corrected with bounded reads/actual path discovery. No runtime
failure was relabeled as a passing test.

Local command results are recorded below after final execution. Logs live in
`target/d02-attunement-focused-final.log`, `target/d02-engine-suite-final.log`
and `target/d02-clippy-final.log` outside this source worktree. No credentials
are written to those logs. No new soak, actual hosted login, GUI demo, provider
call, source-index benchmark or full candidate CI is claimed. Existing upstream
PR136 candidate run 40218400444 failed macOS installed-domain-crash and ci-gate;
that independently remains a merge blocker, not a failure reproduced here.

## Review and remaining work

Changed owner records bind the embedded admission and actual partial progress;
no roadmap checkbox, objective or POA&M is closed. Public transport/Connectome
admission, committed source inventory, parsing/normalization/indexing, all-family
transaction coupling for those effects, scheduling/settings and full native
qualification remain open. Ingestion must use atomic runtime effects plus the
job checkpoint, not copy Connect's control-only transaction for source writes.

Checklist: brief/driver/dependency before implementation; one canonical boundary;
source/behavior/test traceability; explicit activation/grants and limits; no
public wire/version change; smallest oracle then engine suite; failures retained;
source/diff review; one linked journal and regenerated navigation/inventory;
actual passes/omissions and next work recorded. Stops remain false phase success,
implicit source effects, new authority or unsupported alpha completion. PR138
remains dependency-blocked draft until its candidate is eligible for full CI.


## Final local acceptance

- `cargo test -p rrd-engine --lib engine::tests::attunement --locked -- --nocapture`:
  8 passed, 1 ignored subprocess entry point, 0 failed (0.40 s test execution).
  The parent explicitly ran and killed that subprocess entry point.
- `cargo test -p rrd-engine --lib --locked`: 214 passed, 3 ignored, 0 failed
  (15.92 s test execution). Two ignored entries are subprocess helpers invoked
  by their parent tests; the third is the existing privileged Windows
  `foreign_owner_is_denied_without_repair` fixture, not run locally. This is the
  engine library suite, not every workspace or integration-test binary. The
  initial chat description incorrectly grouped all three as subprocess helpers;
  log review identified and corrected that claim before commit.
- `cargo clippy -p rrd-engine --lib --tests --locked -- -D warnings`: passed.
- `python -X utf8 scripts/knowledge/render_navigation.py --check`: 17 indexes,
  614 nodes, 2446 edges, no drift before final result entry.
- `python -X utf8 scripts/ci/check_workflow.py`: passed; 3 workflows, 9 substantive
  jobs, 5 engine suites, 20 default/6 optional packages, 19 container pins.
- `git diff --check`: passed.

Documentation validation initially rejected this new journal's Status prefix
because it omitted the required word active. Corrected the metadata without
changing implementation/evidence. Final documentation, format and inventory
checks passed after this correction: `check_documentation.py` reports 615
statuses/613 coordinates, `build_execution_inventory.py --check` reports 1738
records, and `cargo fmt --all -- --check` plus `git diff --check` are clean. The
new engine module replaces an already planned path. No runtime changes follow
the final Rust acceptance above. Inventory regenerated once more for this
result-only journal edit before commit.
