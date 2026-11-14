# Inventory completion-relative test chronology

**Status:** active candidate correction; native and protected qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-completion-relative-tests`
**Owner:** OBJ-14 / D-02 / D-03 / POAM-006; PR224 driver: active SWAG session

## Baseline and observed failure

Starting source is `61bfe5a60f8c5385cbff856b40540f48704e8959`, tree
`858aa85c0709db09d181084676ae090817a74615`, with a clean worktree.
[PR224](https://github.com/rrflow/rrflow-development/pull/224) received this
follow-up change brief before implementation. The
[prior integration record](d-03-candidate-integration-clock.md) repaired a
different receipt test; it did not prove all fixture timestamps safe.

Candidate run `41160199235`, Linux engine-authority job
[`124415867194`](https://github.com/rrflow/rrflow-development/actions/runs/41160199235/job/124415867194),
failed `replan_refuses_an_unreconciled_failure` at line 1570 with
`Contract("attunement transition time must advance")`. The raw completed-job
log was retained before editing. That library invocation reports 276 passed,
one failed and four ignored; other jobs cannot clear this failure.

The fixture supplies BASE+3000 to Inventory and BASE+3020 to Parse, while
Inventory records completion including elapsed work. The caller's later time
can therefore precede the committed job. Auditing the complete test module
identified the same 20 ms assumption in three receipt/detail/reopen tests.

## Bounded correction and review

Only four functions in `engine/tests/attunement.rs` change:

- `replan_refuses_an_unreconciled_failure`;
- `attunement_receipt_readback_preserves_original_result_after_reopen`;
- `attunement_detail_reports_failure_and_does_not_claim_source_verification`;
- `attunement_installed_connect_failure_and_exact_receipts_survive_reopen`.

Dependent Connect, Inventory and Parse calls now follow the predecessor's
returned `updated_at_unix_ms`. Later reads, reopen, resume and inspection
follow the last returned completion. All original assertions remain: known
versus uncertain failure, principal denial, original receipt equality, no
additional receipts, checkpoint preservation, lease generation and unavailable
Parse. Fixed times intentionally testing early/expired workers are untouched.
No helper rewrites, sleeps, larger timing windows, runtime changes or retries
are added. These correctness fixtures are not learning or routing data.

Reviewed README/AGENTS, the change-authoring and CI procedures, D-02/D-03 and
POAM-006, prior journal, fixture helpers, all four complete functions and their
diff, and production control/Inventory completion checks. Source adaptation
and external research are unnecessary: the native log and elapsed-time code
explain the mismatch. Existing bounded logs provide the failure trace; no new
telemetry, resource measurement or performance claim is introduced. Additional
paths are this immutable journal, POAM-006 and generated navigation. No source
owner, runtime behavior or public contract moves.

## Validation

Tests use the existing dedicated worktree target
`target/candidate-6b2b2c205f`, with development/test debug information disabled
and two Cargo build jobs. The target is not shared with another worktree.

- Focused all-feature library test for the native failure: one passed.
- All-feature `engine::tests::attunement::` module: 31 passed, two existing
  ignored process helpers; includes large Inventory and process recovery.
- `cargo test -p rrd-engine --all-targets --all-features --locked --no-fail-fast`:
  339 passed across 13 targets, four existing ignored helpers. The library
  accounts for 276 passes; focused passes are not additional unique tests.
- `cargo clippy -p rrd-engine --all-targets --all-features --locked -- -D warnings`:
  passed. Formatting, diff whitespace, documentation and workflow policies pass.
  Generated navigation is current; execution inventory checks 2,097 records.

Raw logs are retained under
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/`, including
`candidate-41160199235-authority-124415867194.log` and `completion-relative-*`.
The initial format check requested one assertion wrap; `cargo fmt` applied it.
Documentation policy initially rejected this journal's status without the
required active/historical classification; the record now declares active.
No runtime binary is rebuilt or requalified by this test-only source change.
Installed CLI/SDK and fresh repository trials were not rerun: their production
source is unchanged and the prior integration record retains their exact
binary evidence. The full native candidate remains required for final source.

## Remaining work and checklist

The pre-effect PR brief, native failure, canonical owner inspection, bounded
test correction, preserved assertions and scoped validation form this package's
change checklist. Native candidate and protected merge checks must pass on the
corrected revision. No Linux/macOS pass is inferred from local Windows tests.
Stop on weaker assertions, runtime changes, expired real budgets or another
unexplained failure. Other fixed-time fixtures are not universally qualified
by removing these four specific post-Inventory assumptions.

No D-02/D-03 or alpha gate closes. Source reuse, later Parse/worker execution,
safe binary upgrade and the complete Connectome/SWAG/RROFLOW journey remain
with their existing owners. No training, model qualification or release is
claimed. PR221's retained behavior still needs a verified landed destination
before consolidation can close that dependency.
