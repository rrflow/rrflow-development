# C-07f: reduce redundant probes during APFS capacity refill

**Status:** active repair candidate; native candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07f-apfs-refill-probe`
**Owner:** OBJ-02 / C-07f / POAM-035 / PR150

## Failure and source boundary

Baseline development main `86adada6dab6411cc58cb9aaa931e35472c0f452`, tree
`42f1a022bf06f1a5c0593203f9a553e1c7d837e0`; empty opening commit
`888d4df22bc5d74404e63dcc13de1ca4420c2ba3` opened
[PR150](https://github.com/rrflow/rrflow-development/pull/150) before edits.

C-07 manifest candidate `ab078b833ebe321a509d2da860db0db2e4f03720`,
[run 40500380910, macOS job 122176298531](https://github.com/rrflow/rrflow-development/actions/runs/40500380910/job/122176298531),
failed the fresh `current_before_create` owned APFS capacity case. Initial
data pressure reached 251723776 bytes in 8116 ms; metadata pressure reached
2606 owned files, 2725 creates and 138 barriers with 11608787 microseconds in
settlement. A later data refill reported zero new bytes, eight write attempts,
16-byte next request and the existing 25000-ms aggregate deadline. The fixture
failed before proving the selected engine operation. The raw retrieved job log
is retained as `target/c07-ab078-macos-failure.log`, 501144 bytes, SHA-256
`d9651e7cb6e05b02e6db58380741bd0c7113005628e8dadf6455dc042d1a601c`.
There is no evidence here of lost acknowledged writes.

## Repair and traceability

In `crates/adapters/rrflow-cli/tests/support/capacity.rs`, the initial fill
retains its original bulk-first behavior. A subsequent refill now begins with
a single-byte write; if it succeeds, bulk writes resume immediately. A
capacity failure still requires the existing file/volume settlement and a
fresh one-byte failure after the barrier. Successful writes reset that proof,
so allocation released by settlement is drained before the next final denial.
The aggregate clock, 8192-entry bound, volume ownership, byte allowances,
cleanup, metadata convergence, error classification and production engine
durability are unchanged. No files, paths or behaviors were removed or moved.
The existing tests and new regression stay at the same fixture boundary.

The first Windows focused test failed before the change: a stable refill made
the requests `[896, 56, 3, 1, 896, 56, 3, 1]` rather than `[1, 1]`.
Afterward it passes with one capacity settlement and the same final denial.
Another test makes settlement release 17 bytes, requires their consumption,
requires resumed bulk requests and verifies the second final settlement and
one-byte denial. Existing tests preserve byte/time bounds, noncapacity errors,
metadata/refill convergence and sync scheduling. The complete focused
`cargo test --offline --locked -p rrflow-cli --test installed_crash --features
crash-test capacity::filler_tests:: -- --nocapture` passed 13 tests.
The complete installed-crash target passed 27 tests with two native-owned-volume
fixtures ignored when not invoked by their native runner.

The prior [APFS settlement journal](c-07f-apfs-capacity-settlement.md) owns
the Apple waited-volume-sync research. This change does not copy upstream code,
alter the barrier, add a dependency, or claim the elapsed-time improvement is
proven on macOS. Review included the failure log, `capacity.rs` fill/refill and
tests, `capacity_metadata.rs`, `capacity_apfs.rs`, README, C-07 and POAM-035.
Existing native timings and error receipts remain the diagnostic evidence.
An initial combined journal/POAM patch failed because its POAM context did not
match; the two changes were applied separately using the correct context.

## Remaining checks and status

Strict Clippy for the primary binary and crash/process tests, generated
navigation/inventory, documentation and workflow policy, and formatting passed.
The complete code diff was reviewed. Exact candidate CI remains required.
The actual APFS owned-volume case must pass on the new candidate before this
failure is considered repaired; the complete sustained profile and real
24-hour soak remain open. This journal and POAM entry do not change roadmap
completion status or transfer evidence between candidate revisions.
