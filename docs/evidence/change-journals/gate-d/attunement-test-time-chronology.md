# Attunement test time chronology

**Status:** active candidate evidence; protected qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/attunement-test-time-chronology`
**Owner:** OBJ-14 / D-02 / POAM-006; PR240 driver: Claude Code working session

## Baseline and defect

Starting source is `15f36a8758e32b33c930a20da836def3e5cc3207`, tree
`a674bdf4a428b4b0af0d4ec8ed921aee7f52c9b0`, on a clean dedicated worktree.
[PR240](https://github.com/rrflow/rrflow-development/pull/240) received the
change brief before any test edit. This carries out the repository owner's
2026-11-01 decision to keep full CI and fix flaky tests.

Protected run `41258683845` (PR190's merge-queue entry), job
[`124707222954`](https://github.com/rrflow/rrflow-development/actions/runs/41258683845/job/124707222954),
failed `replan_rechecks_inputs_and_fences_the_old_worker` at
`tests/attunement.rs:1476:6` with
`Contract("attunement transition time must advance")`; 297 passed, one failed,
four ignored, on revision `2b2d4f70ea42d7f3db81ac5c40cfccdc9d1a7642`. A later
queue run for that pull request, `41263871154` at `9d937f6d`, passed. The earlier
run `41156697702` failed a different test the same way; the
[candidate integration record](d-03-candidate-integration-clock.md) and
[completion-relative correction](d-03-completion-relative-tests.md) repaired
those tests one at a time and did not qualify the rest of the file.

A transition records its supplied time plus real elapsed time. The engine
refuses a later transition that does not exceed the job's `updated_at_unix_ms`,
and the contract refuses a status or a Resume/Cancel command that predates it.
Those rules are correct and this package does not touch them. A fixture that
supplies a fixed later time assumes the previous call finished within a fixed
real-time window, and on a loaded runner it may not.

The audit found the defect has three symptoms, not one:

- `attunement transition time must advance` (a later transition);
- `attunement status observation predates the job snapshot` (a later read);
- `attunement command predates the job snapshot` (a later Resume or Cancel).

## Correction

Every time supplied after a committed transition now derives from the
completion that transition returned, following the pattern already used by
`168e93c6`. No assertion, call order, sleep, retry, helper or production path
changes. A comment at `BASE` states the rule for later authors. Five calls that
discarded a returned status now bind it (`let accepted`, `let started`) so the
next time can follow it; the calls themselves are unchanged. A mechanical
comparison with the base finds 226 assertion macros on both sides and an
identical sequence of request and operation keys.

124 supplied times change across 22 functions:

| Function | Times |
| --- | --- |
| `inventory_checkpoint_and_source_records_commit_reopen_and_replan_together` | 12 |
| `inventory_source_rejection_preserves_runtime_and_allows_retry` | 4 |
| `inventory_output_bound_preserves_runtime_checkpoint_and_head_after_reopen` | 6 |
| `partial_inventory_recovery` (two tests; a 100 ms window after recovery) | 4 |
| `selected_plan_replans_after_progress_and_reopens` (two tests) | 8 |
| `attunement_follows_a_bound_repeated_step_plan_after_reopen` | 14 |
| `stale_attunement_can_be_replanned_without_reinstalling_or_losing_receipts` | 7 |
| `replan_rechecks_inputs_and_fences_the_old_worker` | 10 |
| `competing_replan_acceptances_publish_one_successor` | 1 |
| `replan_keeps_historical_control_receipt_after_restart` | 7 |
| `replan_lost_response_survives_process_kill_and_replays` | 1 |
| `replan_receipt_and_replay_reject_a_corrupt_successor_link` | 3 |
| `replan_receipt_is_not_visible_to_another_authorized_principal` | 3 |
| `attunement_receipt_readback_is_principal_scoped_and_authorized` | 4 |
| `attunement_receipt_readback_rejects_corrupt_identity` | 2 |
| `attunement_competing_workers_only_one_claims_and_foreign_worker_cannot_publish` | 4 |
| `attunement_expiry_recovery_and_pause_fence_prior_workers` | 10 |
| `attunement_cancel_wins_over_stale_work_and_is_durable` | 8 |
| `attunement_rejects_invalid_identity_auth_budget_and_changed_idempotency` | 2 |
| `attunement_control_encoding_is_bounded_and_cancel_remains_available_at_limit` | 7 |
| `attunement_native_worker_kill_recovers_claim_and_replays_lost_response` | 5 |
| `installed_startup_keeps_source_admission_at_the_dependent_operation` (`startup_binding.rs`) | 2 |

In `attunement_expiry_recovery_and_pause_fence_prior_workers` the early and
expired probes now follow the returned lease: early at its completion plus one,
expired at `expires_at_unix_ms` plus one. Both previously assumed Start finished
within about a second.

Two parents cannot learn a killed child's completion until a later read returns
it, so those two times take the later of the original literal and the returned
value: `(BASE + 4_300).max(...)` and `(BASE + 43_000).max(...)`.

### Fixed times left in place

86 `BASE` uses remain (77 in `attunement.rs`, eight in `startup_binding.rs`, one
in `inventory_delta.rs`):

- **75 have no preceding committed transition** in the test: the installation
  and first-session fixture, each test's first successful Start or accept, and
  calls before it that fail or only read. Every later time in those tests is
  derived.
- **Two are the floors above.**
- **Two assert on the literal relation:** `deadline_unix_ms = Some(BASE + 2_999)`
  supplied with `BASE + 3_000` in
  `attunement_rejects_invalid_identity_auth_budget_and_changed_idempotency`,
  which expects `DeadlineExceeded`. It precedes that test's first transition.
- **Seven follow a killed child whose completion has not yet returned:**
  `BASE + 4_000`, `4_100` and `4_200` in `replan_lost_response...` (reopen,
  session, and a replay that performs no time comparison), and `BASE + 40_000`,
  `40_100`, `41_000` and `42_000` in `attunement_native_worker_kill...`. The
  parent's 15 s wait bounds the child, so the second group cannot be reached.

`inventory_larger_than_one_physical_batch...`, `replan_refuses_an_unreconciled_failure`,
`repeated_ordinary_edits...`, `replan_receipt_read_is_principal_scoped...`,
`attunement_receipt_readback_preserves_original_result...`,
`attunement_detail_reports_failure...`, `attunement_installed_connect_failure...`
and `inventory_delta.rs` already derive their later times and do not change.

### Other test modules

No other engine test module has this pattern. `tests/installation.rs` and
`tests/installation_recovery.rs` never call a transition. Outside the engine,
`rrd-client/tests/rrflowdb_local.rs`, `rrd-server/tests/http_process.rs` and
`rrflow-cli/tests/installed_lifecycle.rs` run against the real clock, and
`rrd-contract/tests/attunement_contract.rs` has no engine elapsed time.

## Oracle results

The delay was a temporary `std::thread::sleep` in this worktree only. It was
never committed: `git diff` on `engine/attunement.rs` and
`attunement/inventory/publication.rs` is empty, and a search of `crates` for the
instrumentation finds nothing. `old` is the starting revision's tests and `new`
is this package's. The group is 36 tests: 34 run and two ignored child fixtures.

| Delay | Scope | Tests | Result |
| --- | --- | --- | --- |
| 1 s before `finished` in accept | fixed test | old | Fails at `attunement.rs:1476`, the cancel step, with `transition time must advance`: the same line and error as the protected job |
| same | fixed test | new | Passes |
| same | `attunement` group | old | 30 passed, 4 failed |
| same | `attunement` group | new | **34 passed, 0 failed** |
| 1 s before every `finished` (accept, control, Inventory) | group | old | 18 passed, 16 failed: 11 `must advance`, 3 `status observation predates`, 2 `command predates` |
| same | group | new | **34 passed, 0 failed** |
| 3 s, same sites | group | new | 32 passed, 2 failed (below) |
| 1 s before `at` in control | group | new | 34 passed, 0 failed |
| 12 s before `at`, child process only | kill test | old | Fails: `StorageConflict("attunement worker is not expired")` |
| same | kill test | new | Passes |
| 3 s before every `finished` | replan lost-response test | old | Fails: `status observation predates` |
| same | replan lost-response test | new | Passes (within the 3 s group run) |

The requested accept delay alone cannot measure most of the audit, because most
changed times follow a Start, Step, Pause or Cancel. The wider delays were added
for that reason. The `old-worker` call does not fail under the delay, because the
revision check at `attunement.rs:754` refuses it before any time check. The cancel
step is what fails.

The two failures at 3 s are real budgets, not chronology, and are left as they
are:

- `attunement_expiry_recovery_and_pause_fence_prior_workers`: `DeadlineExceeded`.
  The test sets a real 2 s lease, and the engine refuses to publish a claim whose
  lease expired while it worked. A 2 s stall inside Start, or spanning Start's
  completion and the early-recovery probe, still fails this test. Raising the
  literal to `start(30_000)` would clear it now that the probes follow the lease;
  that is outside this package's time derivation and is left for a decision.
- `repeated_ordinary_edits_are_not_stopped_by_a_fixed_generation_limit`:
  `SessionExpired`. 33 accepts at 3 s plus the test's 1 s gap outlast the 120 s
  session. This requires a sustained slowdown of about 2.6 s per accept, not one
  stall.

Related but different: `rrflowdb_local.rs:219` (1 s lease then a 1.2 s sleep) and
`http_process.rs:3540` (2 s lease) use short leases on the real clock, so a stall
longer than the lease fails them. They are not changed here.

## Validation

All commands ran on Windows with no delay in place, `--offline --locked`, and
`CARGO_TARGET_DIR` set to this worktree's own `target/`. Test and dev debug
information are disabled, as in CI.

- `cargo test -p rrd-engine --lib attunement`: 34 passed, two ignored.
- The same with `--all-features`, the `optional-features` CI flag: 34 passed, two
  ignored.
- `replan_rechecks_inputs_and_fences_the_old_worker`, ten consecutive `--exact`
  runs: 10 of 10 passed.
- `cargo test -p rrd-engine --lib`: 305 passed, four ignored.
- `cargo test -p rrd-engine --all-targets --all-features --no-fail-fast`, the
  failed job's exact command: 368 passed across 13 targets, four ignored.
- `cargo clippy -p rrd-engine --all-targets -- -D warnings`: passed, and passed
  again with `--all-features`.
- `cargo fmt --all -- --check`: passed. `cargo fmt --all` first wrapped three
  statements the audit script had left on long lines.
- `py -3 scripts/knowledge/render_navigation.py --check`,
  `py -3 scripts/ci/build_execution_inventory.py --check`,
  `py -3 scripts/ci/check_documentation.py`, `py -3 scripts/ci/check_workflow.py`
  and `py -3 scripts/check_version.py`: all passed, with 2,206 inventory records
  and 866 document statuses.

Not run: other packages' suites, because only `rrd-engine` test source changes;
any Linux or macOS check locally. Candidate and protected CI run on the pull
request. Raw oracle logs lived in the session's scratch directory and are not
retained; the failing lines that matter are quoted above.

## Review, research and trace decisions

Read in full: `AGENTS.md`, `change-authoring.md`, POAM-006's chronology section,
both prior chronology journals, `engine/attunement.rs`, `tests/attunement.rs`,
`attunement/inventory_delta.rs` and `attunement/startup_binding.rs`. Read at the
relevant symbols: `attunement/inventory.rs`, `inventory/publication.rs`,
`rrd-contract/src/attunement.rs`, `engine/session.rs`, `engine/clock.rs`,
`rrd-store/src/control.rs` and the other test callers named above. Every changed
file and the complete diff were reread before commit.

No external research or source adaptation is needed: the protected log and the
elapsed-time code explain the mismatch. No new telemetry, resource measurement or
performance claim is introduced. The temporary delay was the only instrument.

Changed paths: `engine/tests/attunement.rs`, `engine/tests/attunement/startup_binding.rs`,
POAM-006, this journal, and generated navigation and inventory. No source owner,
runtime behavior or public contract moves.

## Remaining work and checklist

The pre-effect PR brief, native failure, owner inspection, first-failure oracle,
bounded test correction, preserved assertions and scoped validation form this
package's change checklist. Candidate and protected merge checks must pass on the
final revision; local Windows passes do not stand in for them. No Linux or macOS
result is inferred.

The protected failures remain retained. This repair removes a test-fixture
assumption and proves nothing further about attunement: D-02, D-03, POAM-006 and
every alpha gate stay open, with no status change. Stop on weaker assertions,
changed call order, runtime changes or an unexplained failure; none occurred.
