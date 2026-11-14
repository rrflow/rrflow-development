# C-07g: Windows native worker interruption and CI coverage

**Status:** active draft evidence; candidate and native qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-native-worker-windows-ci`
**Owner:** OBJ-02 / C-07g-h / POAM-035 / [PR195](https://github.com/rrflow/rrflow-development/pull/195)

## Scope and traceability

Starting PR195 head `dc8311374cc8e42aa9d9e2ef4e092c2b9c9625a4`, tree
`7f46f58b683c00808bcfa726b70ebc52331b9fc8`; the
[preceding candidate journal](c-07g-native-sample-worker.md) remains unchanged.
The brief was extended before adding an ignored held-worker parent fixture in
`tests/support/sustained/native_worker.rs`, repository-owned
`scripts/ci/test_native_sample_worker.py`, policy check and negative tests in
`scripts/ci/check_workflow.py` and `test_native_sample_policy.py`, and one
Windows-only step in the existing installed-reopen lane plus its policy test in
the existing verify lane. No existing job, threshold, engine operation,
qualified profile or original test is removed.

The native runner resolves the unique installed-reopen test executable from
locked Cargo JSON, binds each test receipt to its digest, and uses the existing
`owned_windows_process.run_owned` Job launcher. For abrupt parent death it
opens and retains parent and helper handles, checks both belong to its exact
new named Job and the independent controller does not, terminates only the
verified owned parent, waits for the launcher to quiesce the Job and observes
the retained helper handle signal termination. It removes only known marker
files under its own checked temporary directory after successful cleanup.
Missing identities, malformed markers, failed cleanup or timeouts fail the run.
CI policy rejects a removed, duplicated, disabled, altered or
`continue-on-error` Windows step and rejects bypassing its policy test.

Reviewed the complete added runner and policy tests, the existing Windows Job
launcher and quiescence implementation, sampler owner and test fixture,
CI-reusable and workflow-policy boundary, README, C-07/POAM-035, and preceding
journal. No third-party source is copied. The test uses a deliberate named
Job and a private harness gate; it does not activate an installed product hook.

## Evidence and results

On Windows x64, the v5 eight-file source manifest SHA-256 is
`f371e3286530bbc2aadc3c540c8d6f08ce3e0b0eb4d4560d89c69e0514c3aa2b`;
the test executable SHA-256 is
`da816630b61bca4cd093f2d89d6bf37038c505340b82acc74b1bc140c0b093cd`
(20,769,792 bytes). `cargo test --offline --locked -p rrflow-cli --test
installed_reopen --no-run` passed in the isolated target. The focused held
worker test passed; the real installed smoke passed with an acknowledgement
after the native hold, cancellation/reaping and engine reopen/replay. The
installed-reopen suite passed 26 with 12 ignored; strict Clippy passed.
Their retained log SHA-256 values are respectively
`494e13c4d8ef79575d816aa0d18391ec0b8920b902de996f41e7950349883ccc`,
`b5b6fd506e03abfcd134116613fc14fe21e99de8076abe1d5d59c5122d7ac451`,
`991c1ec97ae337430280774363d059a08163e6878b97987e2ef2a574dcebcb93`,
and `3e7c03959cf56e50e5240a6c87b9d6c923d048d0a8540f9eb747105dd3a54ad9`.

`python scripts/ci/test_native_sample_worker.py --profile debug --offline`
passed both Job tests using an explicit 2 GiB process/aggregate test cap,
4 GiB declared host capacity and 24 host vCPUs. The actual installed engine,
harness and helper membership, helper cancellation and reuse, bogus harness
denial, and abrupt-parent cleanup were checked. The final retained runner log
SHA-256 is `c59928c352dcd3371d0606f96d1380a8a5f97cb814c2dcc96f784186c09e8bd5`;
the checked-in runner SHA-256 is
`1c93f93403f60fbd310c4dcf1168586a2a9e3043e53b5bbbfeac5f47b6d72a23`.
`python scripts/ci/test_native_sample_policy.py` passed four tests,
`python scripts/ci/test_candidate_admission.py` passed 11, workflow policy and
Python compilation passed. This local debug run does not prove the future
Windows release CI step has executed.

## Remaining stop: helper executable identity

The helper currently starts using `std::env::current_exe()` as a pathname.
[Rust's official documentation](https://doc.rust-lang.org/std/env/fn.current_exe.html)
warns that the returned path can be replaced in certain circumstances. A
matching helper PID, Job membership, or pathname does not by itself establish
that the helper image is the intended tested binary. Linux's
[`/proc/pid/exe` documentation](https://www.man7.org/linux/man-pages/man5/proc_pid_exe.5.html)
describes access to the executed image on that platform; it is not a portable
Windows or macOS solution. Design and test an appropriate native image binding
and replacement-race failure oracle before claiming this identity.

The earlier unbounded native termination/reaping failure case, Linux and
macOS native containment, uncontained abrupt parent death, realistic worker
overhead, the 100,000-record profile, native capacity matrix and 24-hour soak
remain open. The original sample-gap cause remains unknown. The Job test is
one Windows synthetic allocation, not the frozen qualification. Candidate CI,
protected merge and alpha are not claimed. Checklist: pre-edit brief, existing
owner/traceability, one new journal, no changed status, real native and
negative checks, explicit omissions and next stops, generated navigation and
inventory before commit.

## Final local repository checks

`cargo fmt --all -- --check`, documentation policy (741 statuses / 739
classified coordinates), navigation check (18 indexes), inventory check
(1900 records), workflow policy, the four native-worker policy tests, and
`git diff --cached --check` passed after adding this journal. The eight Rust
source files still match the v5 manifest. These results precede the commit and
do not replace exact-head candidate or merge-group CI.
