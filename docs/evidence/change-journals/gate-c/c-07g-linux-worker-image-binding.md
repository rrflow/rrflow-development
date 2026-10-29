# C-07g: bind the Linux sampler to its running image

**Status:** active draft evidence; candidate and native qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-linux-worker-image-binding`
**Owner:** OBJ-02 / C-07g-h / POAM-035 / [PR195](https://github.com/rrflow/rrflow-development/pull/195)

## Brief and traceability

Starting head `c2c4921233f6cfbcf3c10eb39825a450e35d28f9`, tree
`9052565097338046b599e80d474acd23e4f9c492`, already integrates exact
PR194 and PR177 candidates. The PR195 description was extended before
implementation and CI edits. The prior worker used `current_exe` pathname
for helper launch and allocation receipt hashing, so replacement could
divert either operation. Linux now uses `/proc/self/exe` for both the running
harness digest and child helper command; open or spawn failure is an error
without a mutable-path fallback. Non-Linux retains its existing behavior, not
a claim of Windows or macOS image binding. The worker still uses the same
`Process` ownership, native observations, bounds and allocation verifier.

Changed `sustained/native_allocation.rs::{harness_image,harness_digest,receipt}`,
its `native.rs` exports and `native_worker.rs::run_helper`; added a private
Linux copied-executable replacement fixture and ordinary test in
`native_worker.rs`. The fixture supplies a synthetic receipt for admission
only; it does not claim a real cgroup allocation. It verifies the receipt and
actual running-image digest, renames its own copied executable, replaces the
old pathname with invalid bytes, verifies the receipt and digest again, and
obtains a real helper observation. A separate copied-process invocation rejects
a deliberately different receipt digest. The CI reopen lane now runs this exact
Linux release test; the existing `verify_native_sample_worker_ci` guard and
negative tests reject removing, duplicating, disabling, altering or bypassing
it. The Windows Job step remains required. No engine source or test was moved
or removed, no qualification threshold was changed, and no new telemetry was
added. Reviewed the complete allocation module, worker and process ownership
implementation, changed workflow/policy diffs, prior worker journals, C-07 and
POAM-035.

The [Linux proc executable link manual](https://man7.org/linux/man-pages/man5/proc_pid_exe.5.html)
and [Rust `current_exe` documentation](https://doc.rust-lang.org/std/env/fn.current_exe.html)
informed the change. A separate C and pinned Rust 1.99.1 exploratory probe
on WSL x86_64 first showed `/proc/self/exe` executing the original image
after its pathname was replaced; only the repository fixture below tests the
actual RRFlow harness. [Microsoft's process termination documentation](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess)
explains that termination can remain pending on I/O; the separate bounded
cleanup stop is not resolved. No upstream source was copied. Preserve existing
bounded phase traces and Job limits; no performance conclusion follows from
the smoke runs.

## Results and failures

Final six changed source/workflow/policy paths, sorted as `path SHA256` lines,
have manifest SHA-256
`08081abb97c9cd52b9d8d8b943597bf73ad40aca408dbff553e42a84013a387f`.
On Linux x86_64 WSL2 (kernel 6.18.33.2), pinned Rust 1.99.1 and a temporary
isolated toolchain/target, `cargo test --offline --locked -p rrflow-cli --test
installed_reopen` passed 27 tests and ignored 13, including the actual image
replacement and wrong-receipt test. The explicit ignored
`sustained::installed_capacity_workload_smoke` passed on the same final source:
20 attempts, 37 resource observations and checkpoint agreement, with a later
acknowledgement while the helper was held, sample cancellation and installed
kill/reopen/replay. `cargo clippy --offline --locked -p rrflow-cli --test
installed_reopen -- -D warnings` passed. The final Linux test executable
SHA-256 is `1698f2d68b4669411d366ed02faae3546c48907723f8de1b5a25e1e05f69aa4e`;
retained local log `target/c07-image-linux-final.log` has SHA-256
`86027e390468aca692a33e66882493026b20f69e4474a2930993e6b697f99aeb`.
The Linux smoke is observed-only and the replacement fixture does not verify
real cgroup membership; WSL is not the complete native qualification matrix.

On Windows x64, `python scripts/ci/test_native_sample_worker.py --profile
debug --offline` passed both the real synthetic Job membership/cancel/reuse
and abrupt-parent cleanup tests with the existing 2 GiB test cap; the test
executable SHA-256 was
`09c07f69a41c62ad991a3324ab61ad750bd1a11cc22f75045cf18b4a57ff80c1`.
The owning installed-reopen suite passed 26 with 12 ignored, the explicit
installed smoke passed 20 attempts and 36 resource observations with matching
checkpoints and interruption/reopen, and strict Clippy passed. Retained local
log `target/c07-image-windows-final.log` SHA-256 is
`2e029a00f097e65eb14cdd0dc66447803e07f13f45a362c344a002f4aeecfec3`.
`python scripts/ci/test_native_sample_policy.py` passed 5 tests and the
workflow policy passed. These are debug builds, not hosted release CI.

The first Windows build failed because the worker referenced the allocation
module at the wrong visibility boundary; corrected the export. The first
Linux build failed because the digest type did not implement `LowerHex`;
corrected its bytewise formatting. Initial format checks found formatting
drift; `cargo fmt` corrected it before final tests. The logs for those first
failures were tool output, not retained files.

## Remaining stops

This does not prove procfs-unavailable behavior in a separate mount namespace,
Linux cgroup containment, Windows/macOS executable image binding, or bounded
termination and reaping when the operating system cannot complete them. The
original sampling-gap root cause, frozen 100,000-record profile, native matrix
and 24-hour soak remain open. The prior macOS `hdiutil` failure remains
recorded. PR177 and PR194 must actually land before PR195 readiness; exact
candidate and protected merge-group CI have not run for this code. No roadmap,
POAM, C-07 or alpha status changed. Checklist: pre-edit brief, linked
traceability, real-process negative and positive oracles, owning suites and
strict lint, exact source/binary identities, failure and omission disclosure,
linked journal, generated navigation/inventory and diff review before commit.

Final repository checks: candidate-admission policy passed 11 tests,
standalone policy passed 7, inventory tests passed 10, workflow policy passed,
documentation policy passed (751 statuses and 749 classified coordinates),
navigation check passed 18 indexes, and `cargo fmt --all -- --check` passed.
Inventory generation and `--check` passed for 1914 records; staged and
unstaged diff whitespace checks passed. The final journal edit requires a
regenerated ignored inventory before commit.
