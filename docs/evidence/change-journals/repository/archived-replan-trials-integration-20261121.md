# Archived replan trial integration

**Status:** active integration evidence; no new attunement qualification
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-replan-trials-integration-20261121`
**Owner:** D-02/H-04 historical source recovery under POAM-011

## Before-edit brief

Baseline ed7f4933642984d44fc8fc1f8e04d9417fb1005c. Integrate five saved
c07-h04-stale-preflight stash objects through ordinary merges:
2d09e42e3182904f6c6654533d474eff0822f9ef,
445bd8b2d1ae851bf73174e88b1f8a30c44e0bbe,
fabd29be25038d045ec7c83e62ffc3393b5541c0,
6102267d5d885dbf1a0d261a514487c627809717,
5bdbc6371961d3602f9748536890b129ce7b9c3d.
They share one saved tree and have no untracked third-parent tree.

Scope: early replan receipt/source observation/generation, predecessor fencing,
public SDK/error/readback, installed/crash and contract oracles, CI and journals.
Map inline attunement/client/server/tests to their current split owners. Retain
the wider current Connect/Inventory/Parse checkpoint allowance, completed-job
successors and principal-scoped receipt lineage. Do not restore old generated
catalogue identities, earlier one-checkpoint-only restrictions or monoliths.
PR467's bounded retry of an explicitly retryable discovery read remains intact.
Read incoming changes, actual conflicts and current owners before resolving;
carry an unmatched assertion or behavior rather than silently dropping it.

Smallest checks: replan tests and contract operations, then installed/client
readback and relevant Python SDK error tests. Same-tree PR463/467 evidence may
be cited only where implementation is unchanged. Documentation, architecture
and exact candidate/queue remain required. No new research/profiler is needed
for source recovery. Stop on lost oracle, relaxed fence, unknown change or red
CI. These merges do not establish complete attunement or the remaining archives.


## Resolutions and traceability

Source merges: af1fe0e9, d5dc6bfa, 3acda133, cdf5d761 and 040d2280.
All five saved trees are identical; each history object was merged separately.
There is no application/test delta against ed7f4933 after the resolutions.
Read the incoming replan source/receipt/checkpoint changes, test declarations,
actual SDK/CLI/server/contract conflicts, current replan implementation,
current generated-client metadata and Python installed/error assertions.
The old 11 engine replan functions, two contract oracles and CLI/server/client
oracles all have current owners; test execution below supplies behavioral evidence.

Current attunement/replan.rs and its receipt tests retain predecessor revision,
source recheck, lease fencing, generation increment, principal-scoped lineage,
competing acceptance and durable replay. Their broader Connect/Inventory/Parse
and succeeded-job support must not regress to the saved trial's one-checkpoint
rule. Current inventory remains in its proposal/git-input modules. Current
Engram names, permission catalogues, public operation descriptions and error
mapping retain later functionality. The generated client keeps public metadata
and sealed operation traits instead of the early private marker-only generator.

The automatically added 352-line client test was a duplicate of the existing
installed_typed_attunement_controls_replan_and_reopen; its current form retains
the original assertions plus discovery and bounded read-conflict retry from
PR467. Kept it once. Removed incoming old contract/server test monoliths;
their operations/validation/installed-flow owners and assertions remain.
The Python installed test retains both stale-start and selected-after-progress
cases, current contract version and the CI requirement for two passing tests.
The old one-test expectation, generated digests and synthetic fixture identities
are not restored. Existing journals and later POA&M evidence are preserved.

The fifth merge was interrupted by a transient index.lock conflict. The lock
was absent on inspection; no lock or other process was deleted. Repeated only
the unfinished conflict resolution and committed after the index was clean.

## Validation

Engine replan filter: 14 passed, 1 child helper ignored. SDK attunement CI policy:
3 passed. Python client unit tests: 7 passed. The first uv attempt failed on an
access-denied lib64 entry in the existing checkout virtual environment; reran
with a dedicated UV_PROJECT_ENVIRONMENT under the session temporary directory,
using the frozen dependency lock and its required Python 3.15 interpreter.
No SDK dependency or existing environment was manually modified. Remaining
contract/client and policy results follow. Other archives are still pending.

Contract replan filter: 2 passed. Real local-client suite: 5 passed, including
installed controls, temporal reads, atomicity and reopen. Both Cargo exits 0.
Navigation generation and documentation policy passed. Same-tree broader
attunement/installed evidence remains in PR463/467; it is not a new run here.

Final local architecture suite passed all 35 tests; workflow policy, cargo
format and whitespace checks passed. No roadmap/POA&M status changed.
