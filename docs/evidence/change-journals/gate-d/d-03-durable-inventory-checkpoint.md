# D-03 durable Inventory and ordinary-edit recovery

**Status:** active implementation evidence; full D-03 qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-durable-inventory-checkpoint`
**Owner:** OBJ-14 / D-03 / POAM-006; PR224 driver: active SWAG implementation session

## Bound change and baseline

[PR224](https://github.com/rrflow/rrflow-development/pull/224) declared this
durable integration before implementation. Baseline:
`9cc0f770fcd36602d1e77725c304619cfdf35a33`, tree
`62b24cbc022bba7a8c6e2b48151c866846c1892d`. Staged source tree before this
journal and generated indexes: `e0cd5ee68f4ae670a1d5ce4054f4f564e3133e1a`.

Previously the selected Inventory step returned `phase-unavailable`. It now
uses the installed bounded Git/native walk to prepare source records, then
publishes the snapshot, entries, containment, change set, command receipt,
authenticated head pointer and exact resulting phase checkpoint through the
existing atomic runtime/control transaction. The public operation catalogue,
selected-plan authority and installation configuration are unchanged.

The installed engine oracle commits Inventory, replays its receipt without
advancing the runtime, reopens, edits/adds/deletes/renames files, rejects the stale
plan, accepts its successor and commits a second Inventory. It observes the
nonempty change set, rename candidate and retirements of the original entry
identities. The real relocated primary additionally executes this path through
the generated Rust client, survives forced process termination, reads its exact
job back and accepts an ordinary edit after Inventory.

## Canonical ownership and implementation review

Read the existing selected-plan/control/replan implementation, installed tests,
installation walker/policy/Git-input modules, pure proposal builder/tests,
retained-root observations, transaction admission, store runtime/control builder
and canonical-record/schema/read patterns. Reviewed the changed paths and new
modules at their owning boundary:

- `crates/compute/rrd-attunement/src/inventory.rs` and its inventory tests:
  fallible bounded chunks cannot become clean EOF. Committed reconstruction
  checks expected digest, completeness, bounds, root/entry identities and tree
  structure. A checksum accompanying arbitrary input is not provenance.
- `crates/authority/rrd-engine/src/engine/installation.rs`,
  `installation/inventory.rs` and its new `proposal.rs`: one source walk supplies
  both the precondition and proposal. Excluded paths that were never inspected
  carry no invented filesystem observation. Native content reuse is disabled.
- `engine/attunement.rs`, new `attunement/inventory.rs` and
  `attunement/inventory/records.rs`: prior proposals come from the committed
  control pointer and authenticated runtime reads. Strict canonical tables store
  source facts; the pointer contains identity/read coordinates, not another copy
  of the source payload. Membership lookup uses a set. Connect and Inventory
  history permit a source-bound successor; unknown external outcomes still
  require reconciliation.
- `engine/transaction_admission.rs`: retain the existing writer permit during
  preparation and grow its serialized logical charge before publication,
  including the composed control transitions. Rejection preserves its prior
  reservation; drop releases it. No nested store or second admission gate.
- `crates/persistence/rrd-lsm/src/filesystem/source.rs`: expose observed POSIX
  execute-mode bits. Windows returns false, not an executable-ACL claim.
- `engine/tests/attunement.rs` and
  `crates/adapters/rrflow-cli/tests/installed_lifecycle.rs`: installed state,
  exact replay, changed-input recovery and actual primary-process acceptance.

Native source metadata is revalidated before returning the proposal. This is
not an OS filesystem snapshot. Every eligible content file is reread. Entries
are explicitly unclassified; no language/model-derived semantics are invented.
Existing authorization, lease, deadline, invocation cancellation and CAS checks
remain at the engine/store boundaries. Explicit Cancel serializes behind the
active writer; this package does not claim immediate mid-read Cancel behavior.

## Oracles, failures and validation

Local Windows validation used the existing Cargo target cache with
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=2` (the first broad engine run used
the default debug profile). No sibling source dependency or runtime fetch was
introduced. Authored correctness fixtures are not training/evaluation records.

- First integration compilation exposed missing adapter interfaces/imports;
  those failures are retained. The installed Inventory/reopen/replan oracle then
  passed against the actual engine.
- The first broad run passed 269 tests and failed four obsolete
  unavailable-Inventory expectations, with four existing tests ignored. Those
  tests now preserve their failure/replay/restart assertions at unsupported
  Parse. Final `cargo test -p rrd-engine --lib --locked`: **273 passed**, four
  existing ignored. Includes output-bound rejection through an explicitly
  installed configuration, unchanged runtime/job/head after reopen, stale-input
  retry with the same receipt key, and transaction reservation release.
- `cargo test -p rrd-attunement --locked`: **15 Git/config and 21 inventory**
  tests passed, including streaming-error rejection and prior reconstruction.
- `cargo test -p rrd-lsm --test filesystem --locked`: **11 passed** on Windows.
- `cargo test -p rrflow-cli --test installed_lifecycle --locked`: **seven passed**.
- The installed-primary test was then run against a copied single-link binary:
  **one passed**, six filtered, with unchanged recorded source hashes. Binary
  SHA-256: `078c772ef35c641927dce101cad9e33bc67ba394010eb9b32a5b01266bfcb23e`.
  This is an unoptimized engineering executable, not a signed release.
- `cargo test -p rrd-engine --test workspace_architecture --locked`:
  **32 passed**.
- Strict all-target Clippy for `rrd-attunement`, `rrd-engine`, `rrd-lsm`, and
  targeted CLI installed-lifecycle Clippy passed. Initial lint findings were an
  unnecessary clone and an eight-argument walker; use a borrowed slice and
  private walk options instead. Formatting and whitespace checks passed.
- `uv --directory sdks/python run --frozen pytest tests/test_installed_attunement.py -q`
  against the copied primary: **two passed**. The first attempt used system
  Python, which lacked pytest; its failure is retained. The frozen SDK
  environment supplied the declared tooling without changing global Python.
- Generated surfaces, documentation, regenerated navigation and execution
  inventory checks passed. Inventory now describes 2,091 paths; this count is
  discovery metadata, not a runtime or release acceptance measure.

The local evidence bundle is
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/durable-*`:
command logs (including failures), source-file hashes, staged source tree,
copied binary digest and installed-primary result. Tests exercise actual store
and process effects; no mock publication, fabricated checkpoints or model calls.
Logical byte limits are not an RSS measurement. No new upstream algorithm or
dependency required external research; existing bounded logs and storage evidence
sufficed without adding a tracing service.

## Remaining scope and checklist

The declared current-to-target change, canonical replacement map, failure
oracle, resource/debug decisions, read/review scope and acceptance evidence are
recorded above. No objective, roadmap checkbox or POA&M item is closed. PR224
remains draft; these local results are not candidate/protected CI or merge proof.

Full D-03 still requires safe native reuse and zero-content-read no-work
checkpoints, wider filesystem race/cancellation and resource qualification,
large installed Git/non-Git repository acceptance, and native Linux/macOS runs.
This increment rejects failed walks without publishing complete source state;
durable incomplete error-entry publication remains open. Loader key/output
limits do not establish a heap bound for historical record decoding. Parse and
later phases, full SWAG/Connectome worker execution, RROFLOW outcomes and
independently reviewed learning remain incomplete. No new training, quantization,
29 GB footprint, public endpoint or public release qualification is claimed.
