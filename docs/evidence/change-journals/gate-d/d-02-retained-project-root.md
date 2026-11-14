# D-02: retain the installed engine's admitted project root

**Status:** active implementation candidate; candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-retained-project-root`
**Owner:** OBJ-14 / D-02b / D-03 / POAM-038 / PR149

## Scope and source boundary

Starting public development main was `86adada6dab6411cc58cb9aaa931e35472c0f452`,
tree `42f1a022bf06f1a5c0593203f9a553e1c7d837e0`. Empty opening commit
`ac43ee2e8998fdfa4a0d3783a6ad4ddf62536f26` opened
[PR149](https://github.com/rrflow/rrflow-development/pull/149) with its brief
before changes. The installed-open path already admits and validates the
project's rooted `EstateFilesystem`, but its engine retained only the private
storage container. This change retains a clone of the already admitted
`AnchoredPath` inside `RrdEngine` after installed identity and security checks.
Cloning shares the existing root ownership; it does not open another ambient
path or itself authorize a source read. Component and rrflowMX engines leave
the field absent. No public operation, source scan, project mutation,
attunement transition, plan or on-disk format is added or changed.

Changed `crates/authority/rrd-engine/src/engine/core.rs`,
`engine/installation.rs`, and `engine/tests/installation.rs`; added this
linked journal and regenerated navigation/inventory. Read README, ADR-0001/0002,
D-02/D-03, POAM-038/039, the installed open and `EstateFilesystem` paths,
`rrd-lsm/src/filesystem.rs::AnchoredPath`, existing install fixtures and the
change-authoring procedure. No upstream code, new dependency, new telemetry or
source-tree traversal is introduced. Retaining one shared root capability has
bounded lifetime tied to the engine; platform-specific handle and relocation
qualification remains with the installed lifecycle gates.

## Test results and limits

The first focused test run before implementation failed to compile because the
installed engine had no `installed_project_root` field. After the change,
`cargo test --offline --locked -p rrd-engine --lib
engine::tests::installation::installed_engine_retains_only_its_admitted_project_root_across_reopen`
passed. It installs two distinct projects, reads each through its own retained
root, rejects an escaping relative path, observes an edit, reopens the first,
and checks that non-installed and volatile engines have no project root. These
are test-only internal reads, not an exposed or authorized source API.
`cargo test --offline --locked -p rrd-engine --lib
engine::tests::installation::` passed 21 tests on Windows. The isolated build
target prevents the shared-target release-build ambiguity seen on PR142.

`cargo clippy --offline --locked -p rrd-engine --all-targets -- -D warnings`,
formatting, navigation, inventory, documentation and workflow checks passed;
the full authored code diff was reviewed. Candidate CI remains required before
landing.
The retained handle proves neither a complete source snapshot nor freshness,
atomic publication, stale detection, replan, memory effects, or alpha readiness.
Those gaps remain open under their existing owners; no gate status changes.
The existing source/status PR146 and C-07 PR142 remain independent candidates.
