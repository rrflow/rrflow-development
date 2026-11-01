# D-04 source parsing integration in progress

**Status:** active draft implementation; installed Parse, retrieval and alpha remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-04-source-parsing-in-progress`
**Owner:** OBJ-07 / OBJ-14 / D-04; [PR #241](https://github.com/rrflow/rrflow-development/pull/241)

The starting public main was `20ab8aac9e91ef2a74f39e05288ddb29da4b3185`,
tree `d108e2d3fdca583b074d935c49ed9f6eec1f48f5`. An empty bootstrap commit
`08d2c30dde0973acc4c5ae522692398dbf3fe0a6` opened the draft PR so its change
brief preceded code. The public `development` destination resolved to
`https://github.com/rrflow/rrflow-development.git`; the pushed ref was
`refs/heads/agent/d04-source-spans` at that exact commit. No main push occurred.

The current engine implements Connect and bounded Inventory, and returns
`phase-unavailable` for Parse. The PR target is source- and policy-bound Parse
through the existing engine, with installed durable readback; a pure parser is
only a prerequisite. The first focused test failed to compile with unresolved
`rrd_attunement::parse`, establishing that the parser boundary was absent.

The draft adds a pure parser proposal, explicit language
selection, source digest validation, exact byte ranges, named/error/missing
nodes, cancellation and input/node/depth/output/time limits. Rust, Python and
Go grammar sources and the Tree-sitter runtime are exact Cargo-locked build
inputs. Unsupported languages are returned explicitly. No project filesystem
read, persisted record, engine execution, schema change, source-index claim or
prompt behavior is added by this component. Source identity and content must be
authenticated by the caller; a matching caller-supplied digest is not authority.
The output limit counts the complete serialized proposal. The linked native
parser's tree memory is not hard-capped by this component; input size and time
bounds do not establish a byte-accurate memory ceiling. The exact cached
`tree-sitter-0.24.7/src/alloc.c` shows default allocation failures call
`abort()` and the allocator function pointers are process-wide. The upstream
[scanner documentation](https://github.com/tree-sitter/tree-sitter/blob/master/docs/src/creating-parsers/4-external-scanners.md)
also describes explicit scanner allocator reuse requirements. A shared
process-wide replacement is not an acceptable per-project enforcement proof.
This is a verified draft admission gap recorded in POAM-006; main does not
execute this parser.

Reviewed the README, alpha objective, first-complete-cycle spine, D-03/D-04/D-05
and Gate G owners, `agent-bootstrap.md` independent integration rules,
`lfg-swag.md`, `rrd-attunement/src/lib.rs` and the beginning of its Inventory
implementation and tests, the engine's Inventory dispatcher, source-walk,
record/reuse/publication boundaries and installed plan/configuration types.
The parser source and test are new. `Cargo.lock`, the attunement Cargo manifest
and module root are changed. No prior implementation is moved or deleted.

Research used the [upstream Tree-sitter Rust binding](https://github.com/tree-sitter/tree-sitter/blob/master/lib/binding_rust/README.md)
for pinned grammars, syntax ranges and old-tree edit requirements. The latest
API's progress callback is not present in locally available `0.24.7`; the draft
uses that version's parser timeout and input callback and fails if cancelled or
timed out. A cancellation callback panic is caught inside the C callback rather
than unwinding through it. No upstream implementation was copied. No external
service, credential or raw source content is logged. Parser allocation, native
ABI and platform portability still require evidence before engine admission.

Commands on Windows: `cargo test --offline --locked -p rrd-attunement --test
parse` passed seven; `cargo test --offline --locked -p rrd-attunement` passed
15 Git, 21 Inventory and seven Parse tests; `cargo clippy --offline --locked -p
rrd-attunement --all-targets -- -D warnings` and `cargo fmt --all -- --check`
passed. The first formatting check failed on the new files and `rustfmt` was
applied before the passing check. `cargo test --offline -p rrd-attunement --test
parse` resolved six exact cached third-party packages into `Cargo.lock`, then
passed the initial three tests. No installed Parse, memory cap, other OS/ARM,
candidate CI or protected merge-group test was run for this draft.

The existing `EstateConfiguration` is sealed, has no parser policy, and its
mutating configure-plan/apply operation remains future work. Selected phase
records originally retained a configuration digest, not the associated parser policy. The
engine must not infer configuration from the digest or silently hardcode an
extension/language mapping. The remaining implementation needs a verified
authorized policy binding, committed Inventory-to-source association, anchored
revalidation, bounded publication and checkpoint/replay, and a real installed
readback oracle. D-05 retrieval, configurable LFG/SWAG, full project-to-prompt
proof, C-07 capacity and soak and alpha acceptance remain open. Checklist:
brief before implementation; owner and affected boundaries inspected; focused
failure and package tests recorded; source/diff reread, navigation, inventory,
documentation and final integration checks remain pending. No roadmap or
POA&M completion status changed.

## Selected policy draft

The PR brief was amended before contract edits to add an optional versioned
`SourceParsePolicy` and extension/language bindings to the selected Parse step
and installation proposal. An absent field is omitted from serialized plans,
preserving the pre-existing golden digest and keeping those steps inactive.
Policy-bearing plans include their chosen values in the plan digest. Duplicate
or noncanonical extension mappings, invalid bounds and a policy on another
step are denied. Installation planning and sealed-preview revalidation now
check file count/bytes against the existing Inventory ceilings and a separate
per-publication byte ceiling against storage transaction admission. Aggregate
output can exceed one transaction or the memory ceiling; this does not
establish that streaming publication exists. This does not implement global
settings or establish enforcement of parser process memory.

The first contract test failed to compile because `SourceParsePolicy`,
`SourceParseBinding` and the plan field did not exist. After implementation,
`cargo test --offline --locked -p rrd-contract --test attunement_contract`
passed 20 tests, including the existing golden JSON and policy/digest checks.
The installed-estate policy/reopen test is running. Changed paths additionally
include `rrd-contract/src/attunement.rs`, its exports and attunement contract
test, `rrd-engine/src/engine/installation.rs` and existing attunement tests.
The first engine build of the reopen test failed with E0425 because the chosen
error-conversion helper was private to a different engine module. The call was
changed to the existing `ServiceError::Contract` conversion pattern; the
focused `cargo test --offline --locked -p rrd-engine --lib
selected_parse_policy_survives_installation_and_engine_reopen` then passed on
Windows. It checks actual installed-estate plan readback after reopen and
rejects a file limit exceeding the selected bundled profile without creating
an estate or changing the source file. This is policy evidence, not a Parse
execution or installed-primary process test.

The first generated-surface check reported a stale TypeScript schema, as
expected after the public plan shape changed. TypeScript, Python, Go, Java and
.NET operation generators and `check_generated_surfaces.py --write` ran with
Cargo offline; the six authored SDK/protocol digest citations were updated.
`check_generated_surfaces.py --check` then passed for 71 operations and seven
signal projections, OpenAPI SHA-256
`8326e4d27db9e488302a434ee853c79106838dabdfd905e1c3feb5130976adaa`.
That was the initial policy shape. A later explicit `max_commit_bytes` field
separated aggregate output from a single transaction: the first contract test
failed because the field was absent, then all 20 contract tests passed after
implementation. The focused installed-estate test passed again, including a
preview that permits aggregate output above one transaction when the selected
group fits, and denies a group above the transaction ceiling. The generators
were rerun and now report OpenAPI SHA-256
`fc10ce0ccffb5fb3314912befe8b3bc12d33430063649f255d671a3439a07dc7`;
the final generated-surface and SDK checks for that shape were then run below.
On the initial shape, TypeScript generation, typecheck and six tests, Python generation and seven
client tests, and Go generation and package tests passed. The local `pnpm`
shim initially failed to locate its executable; `corepack pnpm install
--offline --frozen-lockfile` used the cached 44 packages and succeeded. No
network download occurred. Java/.NET runtime suites and full candidate CI are
not yet run. The policy is not proof of an executable Parse phase.
`cargo test --offline --locked -p rrd-engine --lib` passed 306 tests with four
existing ignored subprocess helpers. Java and .NET operation-generator
`--check` commands also passed; their runtime suites were not run.

For the intermediate `max_commit_bytes` shape, `check_generated_surfaces.py --check`
passed at the `fc10...` digest above; TypeScript generation, typecheck and six
tests, Python generator and seven client tests, Go generator and package tests,
and Java/.NET generator checks passed. `cargo clippy --offline --locked -p
rrd-attunement -p rrd-contract -p rrd-engine --all-targets -- -D warnings`,
`cargo fmt --all -- --check`, workflow policy, documentation, navigation and
2,210-record execution-inventory checks passed. On the `max_commit_bytes`
shape, `cargo test --offline --locked -p rrd-engine --lib` passed 306 tests
with four existing helpers ignored. A later borrowed-policy validation was
added before proposal cloning, and the focused installed-estate test passed
after it; the full engine suite has not yet been rerun after that change. No
candidate CI has been launched.
The schema-boundary test then failed because the generated schema omitted the
policy version, binding-count and numeric ceilings enforced by validation.
After projecting those same structural bounds, the attunement contract suite
passed 21 tests, including the old golden JSON. Cross-field relationships,
source identity and authorization remain engine checks. The five SDK operation
generators and signal projections were rerun for OpenAPI SHA-256
`e49bc010c8797fed6dbebcf8dac3a82e50ac7c2f6812800db30b99b646ae20ea`.
For this shape, `check_generated_surfaces.py --check` passed, TypeScript
generation/typecheck/six tests passed, Python generation/seven client tests
passed, Go generation/package tests passed, and Java/.NET generation checks
passed. Strict Clippy on all targets for `rrd-attunement`, `rrd-contract` and
`rrd-engine` and `cargo fmt --all -- --check` passed. Java/.NET runtime tests,
native qualification and candidate CI remain unrun.
After the subsequent reference and POA&M edits, the inventory check reported
drift; rerunning `python scripts/ci/build_execution_inventory.py` and its
`--check` passed for 2,210 records. The inventory is a generated projection,
not source of lifecycle or completion authority.

The installed-lifecycle reference still said only Connect executed even though
PR #224 had merged bounded Inventory. The PR brief was amended before changing
the reference to name both and to describe the inactive policy. POAM-006
records the stale statement and the separate native-parser memory gap. No
roadmap acceptance status was changed.

## Contract and architecture suite follow-up

The full `cargo test --offline --locked -p rrd-contract` initially failed the
OpenAPI digest assertion: the approved constant still named the pre-policy
document. The baseline exporter at the same contract source revision produced
`8bc8cbdba9d90ab1239ea91624e639886971e58977c0f23a31007d1b2878a9c5`;
the candidate produced
`e49bc010c8797fed6dbebcf8dac3a82e50ac7c2f6812800db30b99b646ae20ea`.
A recursive comparison of both exported JSON documents found exactly 15 added
`parse_policy` properties in existing attunement plan positions, with no other
added, removed or changed JSON paths. After that review the owning constant
was updated and the full contract suite passed. Generated-surface parity
passed again for 71 HTTP operations and seven signal projections.

The first `cargo test --offline --locked -p rrd-engine --test
workspace_architecture` failed two checks: the dependency allowlist omitted
the four newly pinned Tree-sitter packages, and the new Cargo test target was
untracked. The PR brief was amended; only those exact dependencies were added
to the existing pure-attunement boundary check, and the new source and test
were staged. The same architecture suite then passed all 32 tests. No engine,
runtime or storage dependency was added to the attunement crate. The full
`cargo test --offline --locked -p rrd-engine --lib` also passed after the
borrowed-policy change: 306 passed, four subprocess fixtures ignored.

The existing `scripts/ci/owned_windows_process.py` contains a test-harness
example that assigns a suspended child to a Windows job before execution;
Linux CI scripts contain cgroup resource controls. These are research inputs,
not a shipped engine worker or proof of a cross-platform memory limit. An
engine-owned worker still needs a bundled executable identity, bounded IPC,
pre-execution resource admission, cancellation/reaping, and native failure
tests before Parse can be enabled. The Git working tree is still incomplete;
candidate CI, installed Parse/restart and the requested full prompt flow remain
unrun.

## Aggregate budget review

Code review found the initial policy incorrectly applied the parser's u32
single-source offset ceiling to aggregate input/output, memory and time. The
PR brief was amended before removing that cap for the other positive u64
limits. `max_file_bytes` retains the parser's u32 maximum; existing Inventory
and transaction ceilings still apply at installation. The focused contract
suite passed 21 tests, including aggregate input and memory above 4 GiB and
rejection of an oversized single file. The first formatting check exposed a
test assertion requiring rustfmt; formatting was applied. This is a policy
validation result, not a large-project execution or memory-enforcement test.
JavaScript safe-integer behavior for large JSON u64 values requires explicit
review before this public contract is finalized.

The five SDK operation generators and signal projections were rerun. The
baseline-to-candidate OpenAPI structural comparison again found only 15 added
optional `parse_policy` properties. The new reviewed OpenAPI digest is
`84429084f67c3c7489c42b5cbafc9add0eecd88fab4a7eb63683da59c3b9c6e1`;
the owning constant and reference citations were updated. Final follow-up
test results for this revised schema must be recorded before any candidate.

The later wire review found that accepting arbitrary u64 JSON numbers would
silently lose precision in the existing TypeScript number projection.
[ECMAScript specifies](https://tc39.es/ecma262/2025/multipage/numbers-and-dates.html#sec-number.max_safe_integer)
the exact safe-integer boundary. The PR brief was amended before the policy
and schema were constrained to that boundary; the parser's lower per-file
offset ceiling remains separate. The focused contract suite passed 21 tests,
including the inclusive safe-integer boundary, the next integer rejected, and
aggregate limits above 4 GiB. This is a wire-format constraint; larger exact
wire values would require a separately versioned representation. Regenerated
OpenAPI SHA-256 is
`c04d0b515716e4c14257a2ed9409114440b31fb27ef94aac8eba27d3d8abf46f`.
The structural comparison against baseline again found only 15 optional
policy additions. On this revised shape, `cargo test --offline --locked -p
rrd-contract` passed, and strict all-target Clippy for the attunement,
contract and engine packages passed. Generated-surface parity passed for 71
operations and seven signal projections. TypeScript generation, typecheck and
six tests passed; Python generation and tests passed with two skips; Go
generation and package tests passed; Java and .NET generator checks passed.
Java/.NET runtime suites and candidate CI remain unrun.

## Installed developer journey, Windows

The owner requested one runnable developer build and a joined real-project
oracle. Before editing, the PR brief was extended to cover the existing
`crates/adapters/rrflow-cli/tests/installed_lifecycle.rs` primary-process
journey. The initial attempt filtered on an internal helper and ran zero tests;
it is not evidence. The owning test, before modification, passed one test on
Windows: `cargo test --offline --locked -p rrflow-cli --test
installed_lifecycle installed_engine_plans_applies_serves_discovers_authenticates_and_reopens
-- --exact --nocapture`. It exercises actual installation, serving,
authentication, storage readback, restart, inventory and edit/replan. It did
not require Parse or a project-grounded prompt.

The local unmerged debug executable at `target/debug/rrflow.exe` starts and
reports `x86_64-pc-windows-msvc`, unqualified, with executable SHA-256
`fc2ef57639d4bd1356517f8402b594361f35a64aeecc0dfa0eef37a2db009500`.
Its build provenance is null; it was built from a dirty draft rooted at
`08d2c30dde0973acc4c5ae522692398dbf3fe0a6`, not a frozen candidate. A
`--version` invocation failed because the command is `version`; the latter
passed. This binary is not a release or complete alpha.

The same real-process test was then extended to create a Rust source, select a
versioned Rust parse policy, install, authenticate, inventory, edit, replan,
force restart, inventory again and require a Parse checkpoint. The exact
command above failed at the intended assertion: the last committed checkpoint
was Inventory and the public job returned `phase-unavailable`, with message
`This attunement phase has no installed executor; no source or index work was
performed.` The temporary test-owned child is terminated by its process guard
on failure. The test is intentionally red in this draft until the engine-owned
Parse implementation is safely admitted. It does not yet prove source spans,
rrflowQL source-grounded context, LFG/SWAG, or a complete prompt cycle. The
native parser memory isolation finding above remains a stop condition.
The oracle was rerun after selecting a 256 MiB worker-memory budget instead of
the initial 8 MiB fixture value; it failed at the same Parse checkpoint with
`phase-unavailable`. That budget is only an unexecuted policy selection, not
evidence that a worker uses or enforces 256 MiB.

A retained developer fixture lives under the ignored local directory
`target/first-cycle-demo/2dba2a974d7a4cc5a661d5787b5a9c00/`. Direct planning
with the Cargo executable failed because it has two hard links, including the
`target/debug/deps/rrflow.exe` link; the existing standalone-database POA&M
already records this class of failure. A standalone byte-identical executable
copy successfully planned and applied installation and passed quick verify.
The accepted installation digest was
`b7cde8cf973e4808e5c54b295d1db70111c479bb566e8dfc2408df18e1a2f1a0`
and selected attunement digest was
`69f5626d9ce31da24364b83c0e4f153576425e52c14765e8bf994f2c38a5e529`.
On the actual loopback service, an authenticated session advanced Connect and
Inventory, then Parse returned `phase-unavailable`. After stopping and
restarting the service, public attunement read returned the same selected plan,
Connect/Inventory checkpoints and failed Parse state for
`job-69f5626d9ce31da24364b83c`. No credential or bearer token is recorded
here. This proves only that retained partial journey and its failed state; the
fixture is not candidate CI or a completed prompt cycle.

## Source-bound Parse input draft

The next local implementation binds the pending Parse step to the persisted
Inventory head and its immediately preceding checkpoint. It compares project,
proposal digest and read coordinates before loading Inventory records, verifies
the stored source precondition against the selected plan, and rejects a head
that changes while being loaded. The caller must still authenticate the
session, fence the actual current job/lease, and repeat source and head checks
at publication. No Parse dispatch is connected by this change.

The anchored source reader selects an exact path and entry from the ordered
proposal using binary search, verifies regular-file identity, size, revision
and content digest, and streams fixed-size chunks into a disposable sink. Input
is charged before content reads and remains charged on cancellation or sink
failure. Output sent before final validation is untrusted: the future caller
must terminate/discard the worker on failure and may not publish partial
results. An initial whole-file engine buffer and a linear per-file selection
scan were removed after review. The existing Inventory reconstruction still
materializes a complete proposal and is bounded by the current inventory and
transaction admission limits; this change does not prove bounded memory or
100,000-file Parse scalability. Blocking worker pipes also need supervised
cancellation, and the parser still needs a proven native memory boundary.

On Windows, `cargo test --offline --locked -p rrd-engine --lib source_` passed
12 selected tests. These include an installed estate with an actual committed
Inventory, reopen, source read, rejected changed checkpoint and altered source;
the source-reader test also covers exclusions, byte budgets, cancellation,
partial sink failure and hard-link substitution. A new source-precondition
substitution case passed via `cargo test --offline --locked -p rrd-engine
--lib parse_source_is_bound_to_the_committed_inventory_after_reopen`. The first
run of that case failed when a changed checkpoint exhausted the read budget
before its identity was compared; the head comparison was moved ahead of the
Inventory reads. Initial strict Clippy reported a redundant clone and too many
arguments; the entry selection was consolidated and the clone removed, after
which `cargo clippy --offline --locked -p rrd-engine --all-targets -- -D
warnings` passed. A focused test invoked while the previous engine test binary
was still running failed to link with Windows `LNK1104`; after the existing
full suite finished, it passed sequentially. The full engine library suite
before the final source-precondition test edit passed 308 tests with four
ignored helpers. The real installed journey still fails at Parse, and its
follow-on spans, prompt context and LFG/SWAG have not been tested.

The same reopened-estate test then attempted a real rrflowQL traversal from
the committed snapshot to its source entry. It failed because an actual
numeric-leading digest was tokenized as a number and traversal accepted only
word IDs. PR #242 owns this independent query-parser repair. Its focused
quoted-identity syntax test and a persisted rrflowKV graph/reopen test passed
locally; its full package and candidate checks were still running or pending.
For a local joined test only, the exact uncommitted query-source patch from
#242 was applied to this worktree and the query uses a quoted snapshot ID.
That temporary patch was not an independent implementation or merged
dependency. With that exact local patch,
`cargo test --offline --locked -p rrd-engine --lib
parse_source_is_bound_to_the_committed_inventory_after_reopen -- --nocapture`
passed. The embedded installed estate reopened the committed Inventory and
an authenticated rrflowQL traversal reached the same source identity used by
the anchored Parse input reader. This is a real shared-engine read, but not a
primary-process or project-grounded prompt test and not Parse execution.

The reviewed #242 head `5432ec976f2c84bb1fbb54c9353d4ecd7ebd572d` was
then integrated into this worktree by an ordinary, conflict-free merge,
producing `618409814c9d08f1b9bcf732501f87641167c462`. Before the merge, the
temporary identical patch was removed and the D-04 work was stashed as
`a105804f84e80858b86050009dfbcbd606f34894`; after the merge, it was reapplied
with index state and its complete binary diff matched the saved SHA-256
`31703aefc5e0488d35b38934beb22c46a9fc2e8eb141a611c3b92ae64fb6afb5`.
The stash and a local ignored patch backup are retained. #242 candidate CI was
in progress; its merge into public main and this D-04 candidate remain unproven.
The same focused embedded installed-estate test passed again after that merge
with `cargo test --offline --locked -p rrd-engine --lib
parse_source_is_bound_to_the_committed_inventory_after_reopen` (one test).
The primary-process lifecycle was rebuilt and rerun on this integrated draft;
its executable SHA-256 was
`cbc474cec1f9f4e67606ba0138dad4241744e2a5b670739d2454082576a7a98e`.
It again reached committed Inventory and failed its Parse checkpoint assertion
with `phase-unavailable`. The new source reader is not connected to an
installed Parse executor, so the red result is expected and remains open.

After #214 passed its merge group and landed on public main at
`a8b3484d73887e31be06db9de83bebc78048c417`, the D-04 work was separately
stashed as `c5491c0613d419439ad21ca91856fdb9c1f1337e` and its binary diff
saved under ignored `target/` with SHA-256
`52059a6445d48c230cb3c856f44ea7b0ab5ede3cd9f937be619702f5f4f4361e`.
The conflict inspector found a clean merge and the ordinary merge produced
`342052837c8683af4a230b475d954ae54b3d0c34`. Restoring the stash merged the
shared protocol reference: the selected-policy OpenAPI digest and #214's
stale-transaction guidance are both present. All other restored diff sections
were byte-identical to the saved patch. Both stashes and backups remain
available; this local integration does not close C-07 or Parse.
The focused installed-estate checkpoint/source/rrflowQL test passed again on
this combined local tree: one test passed, 312 filtered out. The installed
primary-process Parse oracle was not rerun after the #214 merge and remains
known red from the immediately preceding integrated tree.

## Parser child protocol (2026-11-01)

The PR brief admitted a single-file framed child protocol as part of D-04.
`rrd-attunement::worker` now accepts versioned, size-bounded metadata followed
by exactly the declared source bytes, verifies their digest in the parser,
and returns a bounded syntax proposal or a fixed rejection code. It can emit
the header separately so the future supervisor can stream from the anchored
source reader. The primary `rrflow` executable exposes the internal entry via
`rrd-engine` before tracing/CLI initialization; the worker does not open
project files or commit state. Protocol bounds do not enforce process memory.

The first `cargo test --offline --locked -p rrd-attunement --test worker`
failed to compile because the requested worker module did not exist. After
implementation, the focused tests passed; after adding broken-pipe and
truncated-response coverage, `cargo test --offline --locked -p rrd-attunement`
passed 47 tests (15 Git, 21 Inventory, seven Parse and four worker). Strict
`cargo clippy --offline --locked -p rrd-attunement --all-targets -- -D warnings`
passed. `cargo test --offline --locked -p rrflow-cli --test parser_worker`
passed two actual primary-executable child-process tests, including a real
source span, digest mismatch, truncated input and oversized frame. Strict
`cargo clippy --offline --locked -p rrflow-cli --test parser_worker -- -D
warnings` passed before the final additional oversized-frame assertion.
`cargo fmt --all --check` and `git diff --check` passed before that assertion.
These are local Windows debug-build results, not a signed build or candidate
CI result.
On the same local worktree after the worker change, `cargo test --offline
--locked -p rrd-engine --lib
parse_source_is_bound_to_the_committed_inventory_after_reopen` passed one test
with 312 filtered out. It demonstrates the previously described committed
Inventory/rrflowQL/source identity path, not parser-worker publication.

The worker is not connected to the installed Parse dispatcher. The Windows
suspended process/job memory binding, explicit inherited-handle list, Linux
limit and parent-death behavior, executable identity pinning, supervisor
deadline/cancellation/reaping, bounded concurrent pipe handling and actual
source-to-checkpoint publication remain unimplemented and unqualified. Do not
activate Parse or claim the project journey, C-07 capacity or alpha from these
protocol tests. At the time of those tests the draft source and its existing red
installed-process oracle were local and had not yet been pushed to PR 241.

## Installed-process recheck before draft push

`cargo test --offline --locked -p rrflow-cli --test installed_lifecycle
installed_binary_replans_and_reads_receipt_after_forced_restart -- --nocapture`
selected zero tests: that name is a helper, not a registered test. It is not
passing evidence. The actual registered test,
`cargo test --offline --locked -p rrflow-cli --test installed_lifecycle
installed_engine_plans_applies_serves_discovers_authenticates_and_reopens --
--nocapture`, ran one test on Windows and failed at the Parse checkpoint with
`phase-unavailable`, after the real installed existing-project lifecycle and
restart reached Inventory. Its tested executable SHA-256 was
`fc4e052da6a8a829993baaa677d9662519cb65ae83b07313f044ca7653493847`.
This is the expected unresolved product failure; it must remain visible in the
draft rather than being removed or classified as a passing candidate.

Public main then advanced to `3622fa211a933bd05d9287c366da489c29b20b09`.
The conflict inspector reported a clean merge. Before integration the tracked
binary diff had SHA-256
`c607b8481ca2ff420e34f7fbc5dca7b896446b6296f373a7360f18df96aa45ad`;
the full tracked/staged/untracked work was retained in stash
`c53cf21040ec500c280b5d0e476311ec6da31117`. An ordinary merge produced
`d4083578f2425e466a9f509ea0d68b47bc756371`. Restoring the stash with its index
preserved reproduced that tracked diff digest exactly, and all three untracked
worker-source/test file hashes matched their pre-merge values. The stash and
ignored patch remain available. This synchronization does not turn the red
installed-process test into a passing result; rerun the relevant checks on the
final candidate.
After that merge, `cargo test --offline --locked -p rrd-attunement` passed
all 47 package tests and `python scripts/knowledge/render_navigation.py`
reported 18 indexes with no updates, 912 nodes and 3270 edges. This is a
draft push for review; the existing installed-process failure and native
isolation prerequisites prevent marking the PR ready or merging it.
