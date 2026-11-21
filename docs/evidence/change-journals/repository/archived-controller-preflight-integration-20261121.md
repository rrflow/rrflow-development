# Archived controller preflight integration

**Status:** active integration evidence; no controller quality or performance claim
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-controller-preflight-integration-20261121`
**Owner:** G-02 controller task proposal, reasoning budgets and governed effects; alpha recovery prerequisite

## Before-edit brief

Baseline 1ddebb97377bde5ba5fbb0aadea8b524737c79fc. Integrate three saved
c07-pr206-preflight stash objects in order:
fc03c4483254d5e6e6404e058a751e2fb07b336c,
467a81fa2440e4bd2468e486d306d6c94fc4eaa9,
06ab73248e9380cdd7f55d44848d84a2054918a6.
Their two-parent source objects share tree cc505e48b99d41f21047a3a3519c2a70025eee49.
Ordinary merge history into current canonical owners, preserving newer worker
and assessment admission. No connected provider activation is authorized by
this repository integration, and configured is not proof of participation.

Read the source controller proposal/capacity and reasoning budget changes,
completion/proposal tests, inference transport, credentials, public contracts,
handler and installed proof/fault-relay script changes before resolution.
Current destinations are endpoint_proposal (including capacity), split
reasoning_budget reserve/finish/persistence/public, task ownership and typed
contract capabilities. The current scope/policy/authority guards, bounded
capacity across proposals/workers/assessments, single attempt output and
truthful may-continue state must survive. Current generated SDK catalogue
identity supersedes the trial's earlier counts and digest. Historical installed
receipts are evidence, not new runtime configuration or this session's results.

Before/after oracle: each saved source becomes merged without losing proposal
admission, failure settlement, idempotency, cancellation, stale configuration
and source/authority checks; focused engine reasoning-budget and endpoint
proposal tests, contract/inference tests, architecture and relevant Python
proof tests must pass. No live paid provider call, new benchmark or external
research is needed for this history package. Existing sources and tests own
failure semantics. Stop on any unmapped behavior, changed safety guard,
new dependency or red CI. Journal/index are the expected new files; preserve
unrelated submodule work. PR473 is independent native history integration.
Continue remaining source archives after this package lands.

## Resolution and verification

The three original objects were merged in order as 772229ee, 61c28390 and
0a1ee3b1. Merge 11687003 joins the green PR473 main. No engine, SDK, test or
proof-script implementation changes remain against that main; these merges
connect the saved work to the current canonical implementation.

The reviewed controller proposal/capacity source reserves capacity and a durable
attempt atomically, rechecks authority before dispatch, and settles completion
under the same guards. Current endpoint_proposal and reasoning_budget
reserve/finish/persistence/public retain those behaviors and the newer worker,
assessment and controller-flight accounting. The old prepared chat transport
is superseded by the current prepared controller boundary. Single output,
revocation, changed configuration, cancellation, uncertain remote completion,
reopen and non-replay oracles remain in the current completion/proposal tests.
The former monolithic public contract test maps to the split public contract
suite; generated catalogue counts and digests stay current.

Read: source endpoint_proposal and capacity, reasoning budget changes, selected
completion/proposal characterization cases, inference usage/error changes,
CLI endpoint_credentials, contract/handler changes, the complete original fault
relay and installed controller proof; current proposal, reserve, finish,
capacity and relevant conflict regions. The proof binds executable provenance,
explicit credential mapping and supplied task text; it records pre-dispatch
denials, controlled in-flight faults, read-only recovery, restart equality and
non-replay. Current scripts retain that behavior and later resume/worker proof.
Historical receipts were retained as historical evidence, never newly observed
provider results. No credentials were read and no provider was called here.

Commands on the merged source (CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target,
CARGO_BUILD_JOBS=4; all cargo commands --locked):

- cargo test -p rrd-engine --lib reasoning_budget: 79 passed, 2 ignored.
- cargo test -p rrd-engine --lib endpoint_proposal: 1 passed.
- cargo test -p rrd-contract --test endpoint_proposal --test endpoint_policy
  --test endpoint_selection: 7 + 3 + 9 passed.
- cargo test -p rrd-inference --features endpoint-chat --test endpoint_chat:
  12 passed.
- python scripts/inference/test_installed_controller_fault_relay.py: 14 passed.

Logs: controller-preflight-{engine,capacity,contract,inference,relay}.log in
D:/.usr/Temp/rrflow-project-policy-20261121. This is contract and local test
evidence, not controller quality, real-provider performance or concurrency
capacity qualification. Full installed live-provider proof was not rerun.

Change-authoring checklist: bound G-02 outcome and current/target behavior
before implementation; mapped original behavior and tests to canonical owners;
read conflict regions before resolution; preserved newer guards and generated
contracts; executed focused failure oracles; retained receipts without changing
their meaning; no new trace path or research claim; no roadmap status upgrade.
Architecture, documentation, workflow and navigation checks are recorded below
before candidate readiness. The merge queue remains the landing authority.

Final local checks: cargo test -p rrd-engine --locked --test workspace_architecture: 35 passed; python scripts/ci/check_documentation.py, python scripts/ci/check_workflow.py, python scripts/knowledge/render_navigation.py and git diff --check all passed. No new deficiency or roadmap completion was claimed.

Queue follow-up: the first ready event evaluated the brief revision while the final push was still being processed; the final source revision had only a skipped draft job. Re-emitted ready_for_review for that exact head, without bypassing a failed check. After PR475 merged, regenerated the conflicting repository index from both records. No source or test changes were made.
