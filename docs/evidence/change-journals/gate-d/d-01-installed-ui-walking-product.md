# D-01 installed UI walking-product evidence journal

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-01-installed-ui-walking-product`
**Owner:** package receipt for `D01-01-installed-ui-walking-product-v1`; the roadmap retains completion authority

This record reports one bounded D-01 prerequisite: a project can preview and
apply a deterministic local RRFlow installation, start the installed engine
from the primary executable, dynamically discover its public UI contract,
authenticate through the generated Rust client, shut down, reopen, and pass a
mutation-free quick verifier. It does not mark D-01, another gate, the alpha
objective, or a release complete.

## Bound authority and baseline

- Alpha prerequisite advanced: first locally usable installed engine and
  dynamic UI-discovery path for D-01.
- Starting revision: `2f686f99ac1e11c8cec11a67ffb47551c12e4b15`;
  tree: `426fb8a6272dacbec5ddd2009f5493aab71b48c1`.
- Planning-only commit: `f8806219195186e3e3ecf508aafec588f174549b`,
  whose only changed path is
  `docs/roadmap/rrflow-1.0-active-change.json` and whose sole parent is the
  starting revision.
- Branch: `agent/connectome-temporal-runtime-visualizer`; baseline worktree:
  clean; locked dependency digest:
  `316533e7512dbb9029e494386503ca8430b0c69853b3b78ef02319dcbab93a27`.
- Incremental destination remains `development` at
  `https://github.com/rrflow/rrflow-development.git`. `origin` at
  `https://github.com/rrflow/rrflow.git` remains the explicit-promotion-only
  remote. No push or official promotion occurred in this package.

## Change brief

At baseline, `rrflow version` and `rrflow install plan` both exited 2 as
unrecognized commands. The default CLI opened the legacy `.rrflow/rrd` path;
normal rrflowKV and manifest open paths could create state; credentials used a
load-or-create API; installation and attunement plan digests contained a clock;
security and initial runtime state could not be prepared for one physical
transaction; and the reusable server/client discovery surfaces were not
composed into the primary executable.

The target was one engine-owned path: clock-free content-bound preview; exact
digest-gated apply; explicit create-new/open-existing storage; a single
authoritative runtime-plus-control bootstrap transaction; owner-only generated
credentials; non-secret locator publication after readback; installed open;
shared-lock physical and semantic inspection; linked loopback server; generated
capability/endpoint/OpenAPI UI discovery; an authenticated ready challenge; and
one real SDK mutation/query/reopen proof. Installed open binds both the exact
executable and embedded profile digests, not merely the frozen product version.
`RrdEngine` remains the sole authority for durable effects. The CLI translates
arguments and renders typed values; it
does not own a second install record, schema catalogue, or database opener.

Unchanged behavior and stop boundaries were enforced:

- product version remains `1.0.0`; Gate D, Gate J, H-05, the alpha objective,
  and every linked POA&M item remain open;
- rrflowKV authoritative WAL durability, storage formats, endpoint dispatch,
  OpenAPI generation, transaction semantics, and promotion thresholds were not
  weakened;
- Arrow/DataFusion remain in-process, and all provider, generator, model,
  remote-mesh, and subprocess integrations begin inactive;
- installation requires no checkout command, sibling repository, external
  database, provider, network fetch, or companion executable;
- no implementation file was deleted, moved, renamed, or replaced without
  parity; the legacy supervisor, standalone initializers/controllers, and
  create-or-open paths remain non-acceptance inventory for a later direct-
  convergence package; and
- hostile concurrent pathname replacement, crash cleanup of a partially
  staged installation, repair/restore/salvage/uninstall, full verification,
  durable attunement execution, release assembly, and native platform
  qualification remain outside this package.

## Complete file review and implementation traceability

The following records were read completely before their dependent edits:

- `AGENTS.md` 1-161; `README.md` 1-238;
  `docs/objectives/rrflow-1.0-alpha.md` 1-65; canonical roadmap 1-165; Gate D
  record 1-31; POA&M index 1-43 and complete POAM-006, -007, -013, -016, -020,
  and -025 records;
- `docs/architecture/engine-data-flow.md` 1-1416; ADR-0002 1-122; installed
  lifecycle 1-364; agent bootstrap 1-324; installation research 1-203; the
  execution portal, change-authoring procedure, and implementation-navigation
  procedure in full;
- `rrd-contract` attunement source 1-1077, public library 1-6313, attunement
  tests 1-634, and complete golden fixture;
- `rrd-lsm` library 1-170, manifest 1-653, and database 1-2297;
  `rrd-store` library 1-98, rrflowKV 1-1804, control repository 1-347, and
  runtime repository 1-557;
- `rrd-security/src/lib.rs` 1-1338; `rrd-engine` public library, module root,
  core, token-key, memory-estate, and test roots in full;
- `rrflow-cli` manifest, main 1-169, command 1-1347, and development doctor
  1-580; `rrd-client/src/client.rs` 1-87; `rrd-server` public library and HTTP
  server 1-324; and
- both repository validators used by this package, including the complete
  1,905-line starting execution-inventory generator and the navigation
  renderer.

The replacement map is package-local. `ManifestStore::create` and
`open_existing` plus `Database::create/open_existing` are the canonical
explicit lifecycle boundary; the new read-only inspector owns no repair path.
`RrflowKvStore::create_new/open_existing` and `RrflowKvInspector` provide the
semantic boundary. Control-transition preparation and runtime semantic
preparation now share one storage transaction. Security initialization can be
prepared without publishing. `RrdEngine::installation` is the canonical
coordinator and `rrflow-cli/src/installed.rs` is only its outward adapter.
Legacy paths were retained because this package did not yet prove enough parity
to delete them.

The first vertical coordinator is intentionally one canonical module while
crash/replay behavior is being pinned. Its non-authoritative refactor seams are
already explicit: profile/types, pure planning and inventory, apply/publication,
installed open, and read-only verification. A later D-01 convergence change may
split those seams into submodules without changing the public operations or
creating another lifecycle owner; this package does not disguise a file move as
behavioral completion.

## Implementation and evidence paths

Created paths:

- `crates/persistence/rrd-lsm/src/inspection.rs` and its focused test;
- `crates/persistence/rrd-store/tests/installed_lifecycle.rs`;
- `crates/authority/rrd-engine/assets/install/default-profile-v1.json`,
  `src/engine/installation.rs`, and its focused test;
- `crates/adapters/rrflow-cli/src/installed.rs` and the real-process installed
  lifecycle test; and
- this linked journal.

Changed paths comprise the declared contract fixture/source/export/test;
rrd-lsm database/library/manifest; rrd-store library, rrflowKV, control and
runtime repositories; security library; engine core, memory-estate, module,
public export, token-key, and test root; client discovery; CLI manifest,
command, main, and doctor; the cited research record; and the execution
inventory generator. The Gate D journal index and global file plan are
deterministically regenerated. No file was deleted, moved, or renamed, and
`Cargo.lock` did not change.

## Research, trace, resource, and debugging decisions

Deep research was required because installation publication, credential
creation, offline inspection, UI discovery, and process boundaries have
security and durability consequences. The updated
[research synthesis](../../../research/rrflow-installation-repair-and-distribution-research.md#code-grounded-synthesis-for-the-first-installed-walking-product)
maps Rust atomic create-new and shared file locking, Linux `openat2` and
`fsync`, TUF provenance, OpenAPI discovery, and Arrow/DataFusion streaming to
RRFlow boundaries. Upstream behavior is evidence only; no upstream source tree,
runtime, or compatibility surface was copied.

Opt-in `rrflow::installed` traces report operation/stage, plan/profile/instance
digests or identities, bounded counts, semantic/runtime coordinates, physical
file/byte totals, address, replay disposition, and elapsed milliseconds.
Credential values and project content never enter trace or output. Installation
plans account for action order and estimated writes; apply returns a sealed
runtime/control/locator receipt; verification returns manifest lineage,
reachable segments, WAL extent, file digests, semantic read identity, and
individual check digests. Typed contract/storage/security/client errors remain
the failure oracle instead of being collapsed into a successful readiness
marker.

## First failures, corrections, and focused proof

The starting binary failures were retained: both absent subcommands exited 2.
Test-first engine work then failed on the intentionally missing installation
API before implementation. During validation, the following additional
failures exposed real boundary assumptions:

- the first engine test run spent more than 60 seconds hashing a 1.1 GiB debug
  test executable in three parallel cases. Only that test process was stopped.
  Tests now use a small immutable distribution surrogate through a hidden
  debug-build-only argument; release builds reject that argument and always
  bind the actual `rrflow` executable;
- the first stronger SDK commit returned HTTP 400, `commit requires
  deadline_unix_ms`; the caller was corrected to use the required explicit
  deadline rather than weakening the server contract;
- the first engine-wide architecture run rejected three untracked new Cargo
  test targets. Those exact declared files were staged, and the focused
  `every_workspace_target_source_is_tracked` rerun passed; and
- the first CLI-wide run reported the newly linked public `rrd-server` library
  as a forbidden physical bypass. The planning record was amended before
  editing the completely reviewed doctor; its closed allowlist now admits only
  client, contract, engine, and server libraries while continuing to reject
  storage/query/provider dependencies. The focused doctor rerun passed;
- an initially validly rehashed plan with a forged profile field passed basic
  digest validation. Apply now reconstructs the canonical preview from the
  embedded profile, project inventory, and executable, and requires byte-for-
  byte equality before any effect;
- treating every post-install project edit as installation corruption would
  have frozen the project against normal development. Apply still rejects a
  stale preview, while quick verification now reports the current project
  inventory digest and `attunement_source_current` without misclassifying
  source drift as physical installation corruption;
- the first serve announcement used the word `ready` before the authenticated
  challenge. It now reports only `listening`; readiness is claimed exclusively
  by `rrflow ready` after session, capability, endpoint, and OpenAPI discovery;
- final review removed generic credential serialization and public secret-field
  access, generated the initial grant set from the canonical security-action
  catalogue with schema-parity coverage, bound installed open to executable and
  profile digests, and changed the process proof to query the same claim it
  committed rather than an unrelated bootstrap record;
- full-file reread found that ordinary metadata calls followed static symbolic
  object/database paths. Create, writer-open, installed open, and read-only
  inspection now reject symbolic roots, directories, CURRENT/manifests,
  segments, and the active WAL; the remaining pathname gap is hostile
  concurrent replacement, not a pre-existing link;
- the first inspector required every ancestor manifest and its obsolete
  segments, contradicting the existing garbage collector's deliberate current/
  checkpoint reachability rule. It now authenticates the complete current
  physical closure and every retained parent manifest, accepts an absent-parent
  boundary produced by ordinary garbage collection without claiming why that
  ancestor is absent, and has a post-GC regression; and
- final evidence review found that the credential-leak assertion covered only
  selected successful JSON payloads. The process oracle now scans every
  captured stdout and stderr stream, including rejection, live-writer denial,
  server shutdown, and post-close verification; and
- final source review removed a redundant `unwrap()` from WAL inventory after
  retaining the already checked project-relative WAL path. The verifier now
  carries that validation result forward without a latent panic edge.

Focused acceptance results:

| Command | Result |
|---|---|
| `cargo test -p rrd-contract --test attunement_contract --locked` | passed, 8 tests |
| `cargo test -p rrd-lsm --test inspection --locked` | passed, 6 tests |
| `cargo test -p rrd-store --test installed_lifecycle --locked` | passed, 3 tests |
| `cargo test -p rrd-engine installation --locked` | passed, 6 installation tests |
| `cargo test -p rrflow-cli --test installed_lifecycle --locked -- --nocapture` | final focused rerun passed, 1 test in 1.81 s |

The real-process test uses one exact child, not the legacy broad kill matrix. It
proves: version 1.0.0; two byte-identical no-write previews; wrong-digest
rejection before `.rrflow` exists; exact apply; runtime cursor 2 and bootstrap
control sequence 7; byte-stable offline verification; idempotent replay; linked
server startup on an ephemeral loopback port; authenticated ready with live
capabilities, endpoint catalogue, and OpenAPI 3.1; one SDK transaction commit;
one rrflowQL query of that same committed claim; rejection of offline
verification while the writer owns the database; graceful SIGINT; and
successful post-close reopen/verification. It also scans every captured
lifecycle stdout/stderr stream for credential leakage.

## Widened acceptance

The following package suites passed after the focused oracles:

- `cargo test -p rrd-contract --all-targets --locked`;
- `cargo test -p rrd-lsm --all-targets --locked`;
- `cargo test -p rrd-store --all-targets --locked`;
- `cargo test -p rrd-security --all-targets --locked`;
- `cargo test -p rrd-client --all-targets --locked`;
- `cargo test -p rrd-server --all-targets --locked`; and
- final `cargo test -p rrflow-cli --all-targets --locked`.

The final workspace run's pre-existing `local_estate_driver` group passed four
tests in 228.57 seconds. That time is reported separately: it is legacy
supervisor, marker, process-identity, effect-gap, and bounded-kill regression
coverage, not evidence for the new installed-product path. In the same final
workspace run, the new real-process lifecycle passed in 2.50 seconds.

`cargo test -p rrd-engine --all-targets --locked` passed 73 unit tests and all
engine integration/architecture targets. The final
`cargo test --workspace --all-targets --locked` passed every workspace target.
Strict clippy across the eight changed package boundaries passed with warnings
denied. Generated-surface, documentation, navigation, inventory, export,
version, format, and diff results are recorded below before this journal is
committed.

An additional ordinary-operator release smoke did not use the hidden test
executable override. `cargo build --release -p rrflow-cli --bin rrflow
--locked` passed in 9m46s initially and in 2m03s for the exact final incremental
rebuild. It produced a 125 MiB x86-64 Linux ELF with SHA-256
`e14506179c72e0bfb1cbd6cc8d7a095d9b8a185e720725427e66ab81238556cd`.
The optimized build reported one pre-existing unused-import warning in the
unchanged `engine/estate_control.rs`; it was not hidden or pulled into this
planned package. The resulting executable reported 1.0.0, rejected the hidden
surrogate argument in release mode, emitted two byte-identical no-write plans,
applied its own executable-bound plan at runtime cursor 2/control sequence 7,
wrote both credential files at Unix mode 0600, and passed quick verification
with a current attunement source. The disposable estate was removed afterward.
This locally built ELF is verification input, not a signed or offline-complete
release candidate.

## Remaining errors, checks, and status

Remaining product deficiencies include installation crash cleanup
and resume; adversarial race-free path resolution on each native platform;
complete estate-directory ACL policy; full verification and
repair/restore/salvage/uninstall; executable attunement phases; project
generator/adapters; release-bundle acquisition, signatures, SBOM, and native
platform candidates; removal of the legacy supervisor and standalone
initializers/controllers; streamed rrflowKV-to-Arrow/DataFusion; and the strict
native storage promotion failure. Quick verification is
intentionally offline, fails while the writer lock is held, and currently
hashes the complete reachable physical closure rather than using a mature-
estate incremental verification index. The bounded project inventory rejects
symlinks and projects above 100,000 files or 1 GiB of included source; broader
inventory policy belongs to the attunement package. Fresh mode requires an
already-created empty project directory; complete scaffolding is not present.
Install apply owns a durable engine receipt and authenticated requests are
audited, but foreground `serve` start/stop currently has opt-in trace evidence
rather than its final durable process-lifecycle receipt.

The loopback server requires the generated API credential and rejects remote
binds, but the installed local client does not yet pin a server identity before
sending that credential to a same-host listener. Browser CORS policy, a
generated TypeScript SDK/package, and the actual UI are also not shipped in
this slice. The dynamic readiness/capability/endpoint/OpenAPI response is the
UI integration boundary, not a claim that the UI exists. The generic seat has
no provider persona or representation until an attunement executor is
explicitly authorized.

No benchmark or threshold was rerun or changed. The retained C-07 evidence
still attributes the dominant authoritative write time to WAL `sync_data`, and
H-05/native promotion remains open because native write throughput was below
Fjall in the retained strict comparison.

Final non-Rust checks passed: format check and `git diff --check`; documentation
policy over 264 statuses/262 coordinates and local links; navigation generation
and check over 14 indexes, 263 nodes, and 1,093 edges; six navigation tests;
execution-inventory generation/check over 1,152 records; eleven export tests;
the 1.0.0 version freeze; generated parity for 33 HTTP operations and seven
signal projections; the active-change gate over all 37 post-plan paths; and
outward help checks proving the test distribution argument remains hidden and
`verify --level`/`ready --json` remain explicit. No signed/offline-complete
Linux candidate, Windows/macOS native candidate, hostile concurrent path-
replacement harness, install crash-point matrix, full repair/uninstall,
durable attunement run, or fixed-hardware performance promotion is in this
package's authority.

Full-file reread and final diff review completed after generation and widening.
The final coordinator, CLI adapter, read-only inspector, install/credential
tests, profile, research, and this journal were reread completely; every
smaller cross-boundary diff and generated inventory delta was reviewed. That
review found and corrected canonical-plan forgery, source-drift semantics,
premature readiness language, build/profile substitution, manual grant drift,
generic secret serialization, unrelated query evidence, static symbolic-path
following, and post-GC false corruption before this record was closed.

Change checklist: the objective prerequisite, owner, current/target/unchanged
behavior, exact paths/symbols, first oracle, research, trace/resource/debug
decisions, failure semantics, stop conditions, implementation traceability,
and acceptance commands are bound by the planning commit. All 37 declared
post-plan paths reconcile; no undeclared path, dependency-lock change, move,
rename, deletion, or generated drift remains.

Roadmap/POA&M status change: **none**. D-01 remains open, as do D-02 through
D-11, F-01, H-05, J-01 through J-05, POAM-006/-007/-013/-016/-020/-025, every
alpha outcome, the release decision, and official promotion.

Result commit/development push evidence: this implementation/evidence commit;
no push was performed.
