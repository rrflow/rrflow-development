# D-03 deterministic inventory proposals

**Status:** active component implementation evidence; installed inventory remains unavailable
**Coordinate:** `rrflow://rrflow-instance/data/change-journal/d-03-inventory-proposals`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), D-03 / [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md)

## Outcome and baseline

[PR223](https://github.com/rrflow/rrflow-development/pull/223) supplies the planned
pure compute boundary for OBJ-14's real-project inventory prerequisite. Driver:
SWAG active implementation session. Starting main `a76faf7046b48819a06c31219e9db973b7f6a06b`, tree `064c306ddccbeeb6a9a7c207fb6183586bcf280c`.
An empty bootstrap commit established the draft PR brief before source edits;
it carried no planning file or product behavior. No prerequisite PR is needed
for this compute component. No roadmap or POA&M status changes.

## Change brief and traceability

Previously only `engine/installation/inventory.rs` produced an installation
precondition hash, and `engine/attunement.rs` rejected Inventory as unavailable.
The new `crates/compute/rrd-attunement/src/inventory.rs` builds bounded proposals
from explicitly supplied observations. It cannot open a file, read a store,
activate a provider, authorize effects, or commit a checkpoint. Existing runtime,
installation hash and wire contracts are unchanged; no implementation was moved
or deleted and no parallel persistence authority was introduced.

Stable identities bind estate, project, logical root and normalized relative
path. UTF-8 path byte ordering and bottom-up child hashes make traversal order
irrelevant. Root digests bind scope even for an empty tree. Entries retain typed
containment, policy exclusion/leaf decisions and explicit errors. Changes preserve
add/remove evidence; only unambiguous equal-content pairs yield rename candidates.
Candidates never reassign identities or claim an observed operating-system rename.

Policy/walker/detector revisions, configured limits and metadata must match before
reusing content digests. Metadata revision is an engine-owned observation contract,
not proof that an mtime alone detects edits. Metadata-only changes are rehashed
without inventing content changes. An unchanged supplied observation does not
request any content chunk. This is an iterator-level oracle, not a measured
zero-read filesystem traversal or persisted restart proof.

Incomplete enumeration, explicit error leaves or any rejected observation prevents
a complete proposal. Invalid paths, duplicate entries, missing/non-directory parents,
foreign/incomplete prior proposals, invalid revisions, oversized files and byte
ceilings fail explicitly. Failed read bytes remain charged. Empty chunks are
rejected so a stalled producer cannot evade byte bounds indefinitely. The caller
must enforce filesystem policy, cancellation, deadlines and freshness; a complete
proposal alone is never an engine receipt.

## Reviewed sources and scope

Complete implementation files reviewed: new `src/lib.rs`, `src/inventory.rs`,
`tests/inventory.rs` and crate `Cargo.toml`; workspace Cargo manifest and lock diff;
existing kernel `digest.rs` interface and compute crate manifest conventions.
Owner sections reviewed: root AGENTS/README, Gate D D-02/D-03 and real-project
acceptance, POAM-006 admission gaps, engine-data-flow's complete project-tree
inventory section, agent-bootstrap independent-engine section, and complete
change-authoring procedure. Relevant existing implementation sections reviewed:
installation inventory traversal/hash and attunement control/Connect dispatch.
These section reads are not claimed as full reads of the large owner files.

Created: the four crate files above and this journal. Changed: workspace
`Cargo.toml` membership and nine-line local-package `Cargo.lock` addition.
Changed: existing CI compute suite package selection to include the new workspace
member, and canonical layout/pure dependency/outward-import assertions in
`rrd-engine/tests/workspace_architecture.rs`. Generated: nearest change-journal navigation index. No upstream dependency version,
public API projection, installed template or source-policy default changed.

## Research, trace and resource decisions

No external implementation adaptation or new dependency: hashing uses the existing
kernel SHA-256 interface and serialization uses locked serde. Pure deterministic
composition needs no external research. Exact Git ignore and OS filesystem semantics
must be researched at the later engine read boundary. No runtime trace is applicable
to this pure component. Resource oracles exercise declared count/depth/path/metadata,
per-file and aggregate content bounds, retained failed-read charges and nonprogress.
These are adversarial correctness fixtures, never model training/evaluation data,
real workload observations, fabricated credentials, or performance qualification.

## Failure oracle and verification

The first `cargo test -p rrd-attunement --offline` compiled the new workspace crate
and produced 12 passes / one failure: a truncated file was rejected but could be
omitted while `finish(true)` returned a complete proposal. A focused locked rerun
confirmed it and retained the output. Every rejected observation now poisons
completeness; partial bytes remain charged and the regression passes. Review also
bound empty-root digests to source scope and added the corresponding oracle.
A later test declaration typo (`fn_` instead of `fn `) caused a parse failure;
correcting the declaration restored format/compilation. No failure was hidden.

Final commands on Windows:

- `cargo test -p rrd-attunement --locked`: 19 passed, zero failed/ignored.
- `cargo clippy -p rrd-attunement --all-targets --locked -- -D warnings`: passed.
- `cargo fmt -p rrd-attunement -- --check`: passed.

The SHA-256 test uses the independent published `abc` digest. Other oracles check
changed path sets, stable unrelated entries, unique/ambiguous renames, scoped empty
roots, invalid containment, rejected completeness, policy invalidation and a lazy
iterator that panics if unchanged content is requested. No filesystem/provider
mock is represented as an actual engine result.

Raw local command evidence is retained under the SWAG operator evidence directory
`20261030/source-inventory/`. Exact file hashes:

- `first-failure.log`: `88b5aa5b6a82522a9f20e29544caf8de2487519166e5f44846165ec85b61a3ca`.
- `final-tests.log`: `d7f677efff87fd7bf2f57c989ac88979683bc3afdd777ff477eb1f092b65ee60`.
- `final-clippy.log`: `35e6e89140d6866ff6d8232d527e7b9889da8ca18ce274e181850dbf624be1b1`.
- `final-format.log`: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

Repository checks found the newly added crate absent from CI's explicit package
selection. It was added to the existing compute suite, preserving all workflow
permissions, triggers, actions, runner settings and limits; workflow policy now
passes with 21 packages. The first documentation check rejected the journal's
missing active/historical status classification. Correcting that header required
regenerating its index; the intermediate stale-index failure is retained here.
Final checks:

- `python scripts/ci/check_workflow.py`: passed, 21 workspace packages covered.
- `python scripts/knowledge/render_navigation.py --check`: passed.
- `python scripts/ci/check_documentation.py`: passed; ownership, statuses,
  coordinates, generated parent indexes and local links accepted.
- `python scripts/ci/build_execution_inventory.py` and `--check`: passed,
  2063 current/generated/planned path records.
- `cargo test -p rrd-engine --no-default-features --features edge --test
  workspace_architecture --locked`: 32 passed. The edge feature supplies the
  test's serde dependency; these architecture checks inspect the full workspace
  metadata/source, not runtime execution. `CARGO_PROFILE_DEV_DEBUG=0`,
  `CARGO_BUILD_JOBS=2`; build 56.17 seconds, tests 1.44 seconds.
- `cargo clippy -p rrd-engine --no-default-features --features edge --test
  workspace_architecture --locked -- -D warnings`: passed with the same environment.
- `git diff --cached --check`: passed. Complete new crate/test files and all
  changed hunks, including the mechanical CI/layout registration, reviewed. Full candidate CI, native installed execution, secret/Git policy,
concurrent edits, cancellation and committed readback are not supplied by these
component tests. A testable compute crate is not a complete D-03 implementation.

## Checklist and next boundary

The PR brief preceded implementation; scope/owners and before/after oracles are
explicit; no replacement or unregistered dependency was introduced; failure and
remaining proof are retained. Changed source/test files and final diff were
reviewed. Candidate CI and merge remain required for this package.

Next: engine-authorized bounded enumeration/classification and strong file-change
observations, then atomic source records/change-set/checkpoint publication. That
package must research exact Git ignore behavior, bind a versioned admitted policy,
keep secrets unread, enforce lease/deadline/cancellation, reject racing/escaped
inputs and prove unchanged, edited, interrupted and reopened real-project runs.
Do not wire the new proposal into Inventory merely to turn an unavailable phase
green. PR221's plan-admission overlap must be integrated explicitly. Worker
execution, controller routing qualification and reviewed learning remain later
requirements of the existing dependency spine.
