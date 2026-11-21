# Recovered source planning and job discovery branches

**Status:** active integration evidence; no end-to-end planning completion claim
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/planning-branches-integration-20261121`
**Owner:** Gate D source planning and public installed job discovery recovery

## Before-edit brief

Baseline 24cff5ae546d13eafc2d87a1702fc21b2f951484. Gate D and POAM-011
own the behavior and source recovery respectively. Ordinarily merge PR197
e3eaa78b3272bd19b58f93cfd6c8a6365ac4a01d, PR148
326957e9716225b9e18246dc0298c1cfde9dca1d, PR231
8d47500869329f1a5604821138f1a6c8efc349a5 and the empty PR234 opening
f0223d3e160817d6e82f2952d19b77fbc7a5c30f. Current source-bound plan design
and installed-job discovery exist. Integrate source history while retaining
current immutable job identity, successor/replay rules and bounded authorized
job discovery; repair any missing behavior at its current owner.

Scope: source-bound-plan-design, job discovery in engine/contract/client/server,
its public catalogue projections and tests, source journals and generated
indexes. Read original design changes, discovery code and assertions against
current split modules before conflict resolution. Preserve later Engram naming,
policy, current projections and evidence. Do not restore duplicate old modules.

First oracles: contract discovery, engine discovery, source-bound replanning
and client local discovery. Then owning attunement tests (reuse this session's
same-tree full run only when unchanged), docs and formatting plus required
candidate/queue. No research or new telemetry needed; existing source identity,
authorization and receipts remain. Stop on changed effect authority, weaker
assertions or red CI. This is not a new planner, live UI proof or alpha completion.

## Resolution evidence

PR467 merges PR197 at a2561c08, PR148 at bdb54f44, PR231 at bb6d1745 and
PR234 at ce94b6a4. Application tree is unchanged. The design already contains
the immutable job/plan correction; kept current effect-scoped source binding
rather than restoring a blanket snapshot prerequisite for exploratory work.
Engram naming and historical versus proposed behavior remain explicit.

Read current engine and contract discovery implementations, incoming changes
and conflicted discovery assertions, current client operation and real local
client successor/readback assertions, catalogue and server dispatch locations.
The existing engine discovery traverses stored predecessor/successor links,
requires EngramRead and rejects a changed control sequence. Kept later support
for a succeeded read-only predecessor; the old branch allowed only cancelled
predecessors and would regress completed-project follow/replan discovery.
Kept later engine-clock-aware fixture times. Contract page/count/byte bounds,
lineage/digest/instance validation and source verification NotChecked remain.
Client list_attunement_jobs lives in operation/attunement.rs, server operations
remain in current handlers/dispatch, and current generated catalogue/SDK
projections are retained. No old inline module or stale digest is restored.

The PR234 opening has no tree change and adds no graph proof. Earlier this
session the identical application tree passed the full engine attunement suite
(87 passed, eight existing helper cases ignored) and full contract/attunement
checks (270 passed). Focused discovery/replan and local-client tests are rerun
for this boundary; results and required candidate follow. No roadmap/POAM
completion status changes.

## Local-client failure and bounded test correction

The first rrflowdb_local run passed four and failed the typed replan test at
its initial discovery: HTTP 409, Conflict, retryable=true, control state changed
during collection. The isolated reproduction passed. This is a permitted
optimistic-read outcome, not a valid partial page: the engine's deterministic
concurrent-control test already requires that conflict. The live server runs
storage maintenance alongside requests; this record does not claim attribution
to a particular writer from the returned error alone.

Before correction: scope extends only to this test's four discovery calls.
They must honor the declared fresh-read retry response with at most four fresh
read requests, short asynchronous spacing and new correlation IDs. Every other
error and exhaustion fails the original test; all lineage, reopen and readback
assertions remain. The public client continues to return the conflict, with no
new hidden retry policy. Rerun the five real-server cases after correction.

Final local checks: `cargo test --locked -p rrd-contract --test
attunement_discovery` passed 5; `cargo test --locked -p rrd-engine --lib
attunement::discovery` passed 5; `cargo test --locked -p rrd-engine --lib
attunement::replan` passed 14 with one existing ignored helper. After the
bounded fresh-read test correction, `cargo test --locked -p rrd-client --test
rrflowdb_local` passed all 5 real-server cases, including unchanged durability,
relations, atomicity, historical queries, cancellation and receipt replay.
Formatting, documentation and diff checks passed. Candidate and queue remain
required. The correction is test caller behavior, not a production engine fix.
