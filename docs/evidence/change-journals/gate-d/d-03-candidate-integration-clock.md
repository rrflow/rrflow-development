# Inventory candidate integration and replan test chronology

**Status:** active candidate evidence; protected landing remains pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-candidate-integration-clock`
**Owner:** OBJ-14 / D-02 / D-03 / POAM-006; PR224 driver: active SWAG session

## Bound integration and source identity

[PR224](https://github.com/rrflow/rrflow-development/pull/224) recorded the
integration and later clock-repair briefs before their respective edits.
Starting Inventory head `63265630d3ab3ec9095e55145c4875717e3f79e8` normally
merges selected-plan dependency `f6185c2c3ed4acae090349599f0d799e609fad3e` as
`6b2b2c205fb1c06df6906eb986309d11dc397f97`, tree
`1df64a8ee84ea872c6a7cf2c6214254f78217af7`. The merge was clean: two offline
harness scripts, two existing evidence records and their generated index.
Inventory runtime code was unchanged. The staged tree with the later test-only
repair, before this journal and POAM update, is
`0a21befd07a6aa2bf694522a09fc70f6a053ebab`.

The included [dependency repair](d-02-offline-harness-admission-fix.md) keeps
native isolation and exact every-case execution, admits the actual seven-case
CLI harness and adds a bounded preflight. Reviewed that complete patch, its
runner and policy test, the compiled harness list, owning CI workflow and prior
selected-plan/inventory journals. Its 18 policy tests pass locally.

## Protected failure and correction

PR221's candidate run `41153836631` passed. Its protected merge-group run
`41156697702` did not thereby pass: the completed optional-engine job
[`124404956228`](https://github.com/rrflow/rrflow-development/actions/runs/41156697702/job/124404956228)
failed on revision `0c67078b71f9d31cd0d0dd539da2e76b73b84e89` with
`Contract("attunement transition time must advance")` in
`replan_receipt_read_is_principal_scoped_and_does_not_replay`, at the second
replan acceptance. The exact completed-job log and digest are retained locally.
The first download refused terminal escape sequences; explicit raw capture
succeeded. No native failure was discarded or cleared by a default-profile pass.

The test submitted BASE+1600 for Start and BASE+1800 for a later effect, while
Start records its actual elapsed completion. Under load the later supplied time
can precede that record. Reviewed the complete test and
`engine/attunement.rs::accept_attunement_replan`/control elapsed-time checks.
The only authored code change derives later receipt/read/Start/replan/reopen
times from the last returned job completion. It preserves principal scoping,
exact historical receipt equality, no additional replan effects, lease bounds
and the production monotonic-time check. No sleep, arbitrary larger timeout,
retry-until-pass or runtime validation change is introduced. The focused test
passes after this change. Native protected requalification is still required.

## Isolated build evidence

A separate PR225 investigation documented inconsistent artifacts when its two
divergent worktrees shared one Cargo target. This does not prove contamination
of PR224's earlier dedicated source-inventory cache. To remove that uncertainty
for the combined candidate, the source was frozen at `6b2b2c20` and built into
a previously absent worktree-specific directory:
`target/candidate-6b2b2c205f`. No other worktree/cache was removed. The clock test
was edited only after this baseline process finished.

With `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_BUILD_JOBS=2`:

- `cargo test -p rrd-engine --lib --locked`: 276 passed, four existing ignored.
- `cargo test -p rrflow-cli --test installed_lifecycle --locked`: seven passed.
- `cargo build -p rrflow-cli --bin rrflow --locked`: passed.
- `cargo clippy -p rrd-engine --all-targets --locked -- -D warnings`: passed.
- `python scripts/ci/run_offline_lifecycle.py --check-harness <isolated harness>`:
  passed, exactly seven cases; harness SHA-256
  `d461c355058fe28e8b151b9a63e5f2f0f5e6b254499b6029881aa3dbdc864380`.
- With `RRFLOW_TEST_BINARY` bound to a separate copy of that primary,
  `uv --directory sdks/python run --frozen pytest -o addopts= --color=no -q tests/test_installed_attunement.py`:
  two real HTTP tests passed. Executable SHA-256
  `4db1c73a4ed08b6652c2d71c13f6b5186a9d4ddbb0819f7e616f8b80480c65a0`.
  This unoptimized binary predates the test-only clock edit; it is not a release.

The first all-target/all-feature run passed all 276 library tests (four existing
ignored), including the clock repair, but failed `direct_read_source_closure`.
Inventory's four bounded test-only change-page reads were absent from the exact
source allowlist. Reviewed the entire scanner and each call: two compare initial
and edited mutations/retirements; two check partial completion and resume
uniqueness. The owning `engine/mod.rs` includes these under `#[cfg(test)]`.
Production Inventory uses selected authenticated canonical reads, not log replay.
The PR brief was extended before adding one exact path/kind/count=4 oracle
entry. No scanner, normal-path prohibition, other entry or count was relaxed.
The focused source-closure target then passed. The staged clock-only tree above
predates this additional allowlist edit. The final
`cargo test -p rrd-engine --all-targets --all-features --locked --no-fail-fast`
passed **339 tests across 13 targets**, with four existing ignored helpers;
the library accounts for 276 of those passes. The exact protected profile is
therefore exercised locally on Windows, not inferred from the default build.
`cargo clippy -p rrd-engine --all-targets --all-features --locked -- -D warnings`
also passed. Cargo formatting, diff whitespace, documentation/workflow policy,
generated navigation and the 2,096-record execution inventory passed. No Linux
or macOS qualification is inferred from this Windows run.

Local raw logs and source/binary/target identity records are under
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/`, prefixed
`integrated-candidate-`, `integrated-receipt-clock-` and `dependency-221-queue-`.
No credentials, fabricated model outcomes or training labels are added.

## Review and remaining work

The README, dependency spine, D-02/D-03 owners, POAM-006/039, change-authoring
procedure and CI gate define this integration. Full changed-file/diff review
preserves the existing public proposal/admission, tamper, SDK, restart and
offline assertions from exact PR221 ancestry and the bounded Inventory behavior
from PR224. PR225's separate descriptions work is not included or modified.
Only the existing receipt test, its Inventory-test source-closure entry, linked
evidence, POAM entry and generated navigation are authored here. No source owner
was moved or deleted.

No upstream adaptation or external algorithm research was needed: the native
failure log and elapsed-time code identify the test chronology mismatch.
Existing bounded logs, digests and committed receipts provide trace evidence;
no new runtime telemetry or resource guarantee is introduced. The change brief,
owner review, exact failure oracle, correction and validation form the package
checklist. Broader Inventory limitations remain in the
[bounded publication journal](d-03-bounded-inventory-publication.md).
No D-02/D-03/alpha gate closes. Candidate and protected CI must pass before
landing; neither a local merge nor a passing earlier candidate is merge evidence.
Stop on missing dependency behavior, changed production authority, unresolved
conflicts or a failed required check. Do not close a consolidated dependency
until its retained behavior has a verified landed destination.
