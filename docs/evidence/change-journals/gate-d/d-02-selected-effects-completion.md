# Selected attunement effects and completed-job successor

**Status:** active draft evidence; adaptive default selection and project indexing remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-selected-effects-completion`
**Owner:** D-02/D-03, OBJ-07/14, POAM-006, [PR #241](https://github.com/rrflow/rrflow-development/pull/241)

Starting local candidate `4a2408d5ae7c6af4890330b01e406c9c64b5c614` is an
unpublished merge preparation containing dependencies that are not all on
public main. This package extends the existing PR brief before each effect;
it does not establish an integrated candidate. The driver is the installed
primary-process test `adaptive_installed::selected_inventory_completes_and_replans_across_primary_process_restarts`.

The old installation and plan validation required Verify even for an explicitly
selected Connect-only job. Connect and Inventory now finish when the last
selected checkpoint is committed, while an explicitly selected Verify remains
terminal and its passed verification envelope still requires a Verify
checkpoint. This means only that selected work succeeded. It does not claim
verification, parsing, retrieval, or a completed project index. The bundled
eleven-step defaults and automatic capability selection remain unchanged.

A completed Connect/Inventory job can now be the predecessor of an authorized
successor. Its snapshot remains unchanged under the same compare-and-swap guard;
the successor plan, job, link, and receipt are published atomically. The receipt
reader checks the completed predecessor against the stored job. Unresolved-job
fencing, existing plan and checkpoint digests, source freshness, authority,
leases, and idempotent replay retain their prior ownership. Source is reobserved
at acceptance, but its filesystem observation and the later engine commit are
not atomic; a concurrent edit after the observation is not excluded. The closed
Connect/Inventory allowlist is a fail-closed reconciliation boundary for
currently understood effects, not a prescribed universal reasoning sequence.

Changed source paths: `rrd-engine/src/engine/{installation,attunement}.rs`,
`rrd-engine/src/engine/attunement/inventory/publication.rs`,
`rrd-contract/src/attunement.rs`; test registrations and new focused engine
and installed-primary test modules, the existing contract and CLI tests, and
supporting installation/bootstrap references. No implementation or on-disk
record format was removed. Old plans retain their serialized digests. The
requested automatic selection, empty selection semantics, per-effect source
dependency tracking, parser execution and public HEAD generation repair are
not implemented here.

The review followed README, ADR-0002, the D-02/D-03/D-04 dependency spine,
the existing PR #241 and #252 briefs, plan/verification validation and job
transition/replan code, Inventory publication, installed client driver and
contract tests. PostgreSQL's [transaction isolation reference](https://www.postgresql.org/docs/18/transaction-iso.html)
distinguishes a statement snapshot from broader multi-statement guarantees;
the pinned checkpoint here is not a fix for the incomplete-refresh HEAD issue.
[Temporal's workflow documentation](https://docs.temporal.io/workflows) informs
receipt-bound replay; no external runtime or automatic retry is added. Existing
plan, source, checkpoint and receipt identities are the trace. No performance
or resource superiority claim is made.

Full-file review for this package covered `rrd-engine/src/engine/attunement.rs`,
`rrd-engine/src/engine/attunement/inventory/publication.rs`,
`rrd-engine/src/engine/installation.rs`, `rrd-engine/src/engine/tests/attunement.rs`,
`rrd-contract/src/attunement.rs`, `rrd-contract/tests/attunement_contract.rs`,
`rrflow-cli/tests/installed_lifecycle.rs`, both new test modules, and the
updated bootstrap and installed-lifecycle references. The generated journal
index was regenerated from its owning records; the complete changed diff was
reviewed. These reads do not extend to every file in the repository.

First, a shared-target Windows build failed because it reused an incompatible
`rrd-contract` artifact lacking the candidate's `SourceParsePolicy`. The
source contained that type. An isolated target directory was used thereafter;
its focused test selected one test and failed with `attunement proposal steps
or bounds are invalid`, exposing the mandatory Verify. After the implementation,
that same focused engine test passed and subsequently passed again after it
was extended to reject stale replan input, run the successor, reopen and replay.
`cargo test --offline --locked -p rrd-contract --test attunement_contract`
passed 22 tests; the complete `cargo test --offline --locked -p rrd-contract`
passed. `cargo test --offline --locked -p rrd-engine --lib` passed 344 tests
with nine ignored; the final strengthened focused test was rerun separately.

The source-built installed-primary test passed on Windows before adding public
source readback: it executed an explicit Connect/Inventory plan, terminated the
primary process, reopened and replanned after an edit, executed the successor,
then reopened and read both jobs. The strengthened checkpoint-pinned public
source query then passed in a fresh run of
`cargo test --offline --locked -p rrflow-cli --test installed_lifecycle adaptive_installed::selected_inventory_completes_and_replans_across_primary_process_restarts -- --exact --nocapture`:
one test selected, three primary-process lifetimes, and independent source ID
readback at each completed checkpoint after restart. The existing
`installed_plan_accepts_selected_attunement_and_rejects_tampering` test also
passed after its obsolete Connect-only denial was replaced by a nonterminal
Verify denial. Existing installed Parse remains a separate known failure and
is not weakened by this package. The full engine library suite was rerun after
the completed-predecessor receipt guard and again passed 344 tests with nine
ignored. The later primary-process run includes that guard.
`cargo clippy --offline --locked -p rrd-engine -p rrd-contract -p rrflow-cli
--all-targets -- -D warnings` and the package format check passed on Windows.
Navigation and documentation policy passed. The generated execution inventory
reported drift after newly created paths were staged; it is regenerated and
checked against the staged candidate before commit.

## Change checklist

- Owner and outcome: D-02/D-03 selected effect completion and safe successor;
  no alpha or gate status is changed.
- Current/target and scope: required Verify prevented short plans; completion
  and replan now bind only the selected, locally reconciled effects. Unknown
  execution is not enabled and the automatic default remains open.
- Failure and acceptance: isolated red test, full contract suite, engine library
  suite, strengthened focused test and installed-primary runs are listed above.
- Trace and research: existing checkpoint, read stamp and receipt identities;
  PostgreSQL snapshot and Temporal replay references above; no copied code or
  performance claim.
- Stop and remaining work: retain the #252 mixed-generation failure, preserve
  #241 parser memory and publication fences, and do not enable Parse or claim
  full indexing. Candidate and merge-group CI are still required.

No Linux, ARM, macOS, C-07 capacity, soak, full installed alpha, candidate CI,
merge-group CI or full repository line review is claimed. The confirmed
incomplete-refresh `KNOWN HEAD` mixed-generation defect remains open in #252;
checkpoint-pinned reads do not prove it fixed. Concurrent source mutation
after replan's final observation remains a separate snapshot/race limit in
POAM-006. Gate and POA&M completion
statuses are unchanged.
