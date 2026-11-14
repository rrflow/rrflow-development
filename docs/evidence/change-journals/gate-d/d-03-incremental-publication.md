# Incremental Inventory publication from authenticated current facts

**Status:** active implementation evidence; full qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-incremental-publication`
**Owner:** OBJ-14 / D-03 / POAM-006; PR226 driver: active SWAG session

## Baseline and first failure

[PR226](https://github.com/rrflow/rrflow-development/pull/226) was opened draft
with its bounded brief before implementation. Its separate worktree starts from
public main `f256c8e1337d85fa01f2a0c61e895a4d213163fa` and fast-forwards required
PR224 candidate `168e93c649fe6548187b01db22bd47eb5552e077`. PR224 must qualify
and land before this dependent package. Neither PR225 nor PR224 is modified
by this worktree.

The initial installed regression deliberately selects three Inventory steps
over the same source. Before the production edit, its second Inventory
published **14 runtime mutations**, failing the expectation of three new
completion records. Source entries and containment were being rewritten even
when their complete facts were unchanged. The fresh dedicated target built
successfully; the failing assertion, rather than a compiler failure, is the
characterization oracle. The exact test and log hashes are retained locally.

## Canonical implementation and recovery

The new `engine/attunement/inventory/reuse.rs` filters the existing proposal's
publication groups through authenticated identity reads at the current runtime
stamp. An entry and its containment remain one group. Equal current facts can
retain their existing versions; different, missing, future-dated or retired
facts cannot authorize reuse. Reuse never assumes that the previous complete
snapshot still describes current physical records: partial publication or
intervening writes may have superseded them.

`records::load_prior` and reuse share one remaining storage-key budget.
The complete proposal is still bounded and charged before filtering; the
writer reservation is not reduced. Publication retains its source, lease,
deadline, cancellation, authorization, expected-cursor and control-CAS checks.
The PR brief was extended before moving the existing final source recheck into
the shared completion boundary, so both small and grouped writes revalidate
after reuse work. This adds a full walk on the small path; fewer mutations do
not by themselves prove lower latency. A zero-cursor runtime skips pointless
current-identity lookups because it contains no committed runtime facts.
Every completion still commits a new snapshot, change set, root relation,
authenticated head, phase checkpoint and receipt at the existing engine owner.
Reusing facts does not reuse an old completion receipt.

Private progress format 2 binds an ordered, timestamp-independent publication
work-set digest in addition to the proposal, prior head and authenticated read.
A recomputed delta after interruption can be smaller; an offset is reused only
when all identities still match. Otherwise publication starts at the beginning
of the currently missing groups. Empty group sets can finish their complete
metadata transaction. Old format-1 progress is rejected; no binary-upgrade or
progress-migration claim is made.

The new tests are under `engine/tests/attunement/inventory_delta.rs`, included
by the existing attunement module. Additional changes are the Inventory entry
point, records, publication, this journal, POAM-006 and generated navigation.
No public operation, separate inventory database, provider scheduler, runtime
index or native filesystem freshness cache is introduced.

## Observed checks

- Initial unchanged-Inventory regression: failed, 14 mutations versus three.
- Corrected regression: passed, repeated completion and reopen retain identity.
- Current-record tests: passed for changed entry facts, changed containment and
  a retired entry. Each republishes its affected group; the next unchanged
  Inventory reconstructs the complete snapshot and publishes only metadata.
- Whole attunement module: 33 passed, two existing ignored process helpers.
  This includes partial-publication interruption/reopen without duplicate entry
  versions and refusal of stale completion after an edit during publication.
- The first all-target/all-feature run passed 277 library tests and all 63
  integration tests but failed the large-project case at Connect, before the
  reuse code executed, with `DeadlineExceeded`. Four existing helpers were
  ignored. The cause is not established; the failure is retained rather than
  attributed to reuse or hidden by increasing a lease. The final-boundary
  revision passes the focused large case and full-profile validation with
  unchanged budgets: **341 tests across 13 targets**, four existing ignored
  helpers (278 library and 63 integration passes). The unexplained earlier
  Connect failure is not claimed fixed. Strict all-target/all-feature Clippy
  passes; it did not run after the initial failed suite.
- A separate clone of the same actual public source revision `d1b8a26a`, tree
  `c4cc385a`, retains 1,870 tracked paths for the installed binary trial. Earlier
  failed and successful estates remain intact. The new trial has not run yet.

The first expanded test compile exposed missing explicit core-type imports;
those imports were corrected and both focused tests passed. No failure log was
overwritten. Raw evidence and the actual-repository trial script are retained
under `D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/` with
the `delta-` prefix. Authored correctness fixtures are not training data or
model-performance evidence.

Reproduce the checks below from this worktree. Observed builds used the dedicated
`target/delta-168e93c6`, debug information disabled and two Cargo build jobs;
focused filters below select the individual oracles described above.

```text
cargo test -p rrd-engine unchanged_inventory_publishes_only_completion_metadata --locked
cargo test -p rrd-engine inventory_reuse_observes_current_records_edges_and_retirements --locked
cargo test -p rrd-engine engine::tests::attunement --locked
cargo test -p rrd-engine inventory_larger_than_one_physical_batch_publishes_and_reopens --locked
cargo test -p rrd-engine --all-targets --all-features --locked --no-fail-fast
cargo clippy -p rrd-engine --all-targets --all-features --locked -- -D warnings
cargo fmt --all -- --check
python scripts/knowledge/render_navigation.py
python scripts/ci/build_execution_inventory.py --check
python scripts/ci/check_documentation.py
git diff --check
```

## Review decisions and remaining acceptance

Reviewed the D-03/dependency/CI owners, Inventory proposal and change identities,
current-record reconstruction and partitioning, publication progress and commit
guards, native source observation, authenticated versioned reads, the kernel
digest wrapper, and installed recovery tests. No upstream adaptation or
external research is needed: this change reuses the existing read/transaction
contracts and makes no new filesystem freshness assumption. Existing bounded
logs, source/binary digests and observed cursors provide its trace; no new
runtime metric or resource guarantee is inferred.

Formatting, documentation/navigation and the 2,100-record execution inventory
pass. The full suite includes the source-closure and 32 architecture checks.
The pre-effect brief, retained failing oracle, owner mapping, bounded source
change, recovery analysis and scoped checks form the package checklist. Finish
real binary trials and required candidate/protected CI before acceptance. No D-03 or alpha status
closes here. Full content scans, prior reconstruction and bounded authenticated
current reads remain; this is write reuse, not zero-content-read attunement.
RSS, final-manifest limits, old partial-observation retention, native race
qualification, safe upgrades and the complete controller/worker/learning
journey remain open under their existing owners.
