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

The local, uncommitted draft adds a pure parser proposal, explicit language
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
