# C-07g: verify Windows Job cleanup after a timed-out sampler

**Status:** active draft evidence; native qualification and exact CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-windows-timeout-supervisor`
**Owner:** OBJ-02 / C-07g-h / POAM-035 / [PR195](https://github.com/rrflow/rrflow-development/pull/195)

## Brief and traceability

Starting head `b603e40f3d6150e3f7e80d40800d181e5e139546`, tree
`675b302085b18e095fe953a66037f88403a32c30`. The PR description recorded the
fixture, safety boundaries and stop conditions before implementation. The
preceding bounded-response change reports an acquisition timeout while an
unfinished monitor remains owned, but it did not prove what the external
supervisor does if the harness exits with a live helper.

The new ignored Windows-only `native_job_timeout_cleanup_fixture` verifies its
actual launcher receipt and Job membership, starts a real held helper, and
deliberately substitutes a fresh uncancelled flag to model a monitor that does
not act on the acquisition cancellation. It requires a timeout and an explicit
incomplete-cleanup result, verifies that the helper is still alive in the
same Job, reports its PID, and exits only its own verified fixture process
with code 73. This is injected unresponsiveness, not a real OS-stuck child.

The existing `scripts/ci/test_native_sample_worker.py` uses the same pinned
qualification proof and `run_owned` Job supervisor for the new case. Its
independent controller retains parent and helper handles, checks both against
the exact newly created named Job and checks that the controller is outside
it, observes both live, waits for the fixture's expected failure and marker,
and requires the Job owner to quiesce the Job and the retained helper handle
to signal termination. The existing abrupt-parent and membership tests remain
at the same boundary; cleanup removes only known files inside its own
validated fixture directory. The existing guarded Windows CI step already
invokes this runner; artifact selection now requires all three private test
entries. No production engine, runtime policy, qualification threshold,
workload or existing test is changed.

Reviewed the complete changed fixture and runner, the existing Windows Job
owner and `quiesce_job`, prior bounded-acquisition and image-pin journals,
POAM-035 and the candidate workflow. [Microsoft's termination documentation](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess)
states that termination can remain pending on I/O; a controlled killable
helper cannot prove such a process has exited. No upstream source was copied.
Existing Job caps and bounded output remain; no performance claim or new
product telemetry is introduced.

## Local results

The two changed code paths, sorted as `path SHA256` lines, have manifest
SHA-256 `16edca1ef570f83a8c05310b9e0f801d14058ee8af51bd9095f348bd1cd5af54`.
On Windows x64, the final Git Bash native step using debug/offline options
passed six image tests and all three real Job tests: membership/cancellation/
reuse, externally terminated parent, and the new self-exiting timeout case.
The timeout case observed the live helper inside the Job before exit, then
its retained handle signalled after Job cleanup. The receipt used a 2 GiB
process and aggregate test limit, 4 GiB declared host, 24 host vCPUs, and
harness SHA-256 `16acff0987ca2b0725af91273398cc5f238f5553aa6c1ebaa1b5306ec98ed5db`.
The owning installed-reopen suite passed 27 with 13 ignored; the explicit
installed smoke passed 20 attempts and 28 observations matching checkpoints
with cancellation and reopen; strict Clippy, six native policy tests and
workflow policy passed. Local log `target/c07-image-windows-supervisor-final.log`
SHA-256 `75b591dc677ea303e65ce950fe6489d1ec4605676074f7d94449b42245e499f6`.

On Linux x86_64 WSL2 with Rust 1.99.1, the owning installed-reopen suite passed
28 with 13 ignored, the installed smoke passed 20 attempts and 37 matching
observations, and strict Clippy passed. The debug harness SHA-256 was
`db6655056d5a98b04d950ffe2203d95c7f42a8c7314a95040d6a5a1df3d3ebe3`;
local log `target/c07-image-linux-supervisor-final.log` SHA-256
`3231265226c94515928016032874e4095603a7958d3642c17394ab95d3906aae`.
The Windows-only timeout fixture was not executed on Linux. Neither local
debug run is hosted release CI or the frozen capacity profile.

## Remaining stops and checklist

The controlled child is killable; actual OS termination or wait failure,
Linux cgroup supervisor cleanup, macOS image identity, Windows build provenance
before pin, and uncontained launches remain unproven. The original 100k
sample-gap cause, frozen 100,000-record profile, native matrix and 24-hour
soak remain open. PR177 and PR194 must land first; candidate and protected
merge-group CI for this head have not run. No roadmap, POAM, C-07 or alpha
status changed. Checklist: pre-edit brief, existing Job/worker traceability,
independent retained handles, real child and explicit injected failure,
normal Job and owning suites, known omissions, linked journal, generated
navigation/inventory and diff checks before commit.

Final repository checks: candidate admission passed 11 tests, standalone
policy 7, inventory tests 10, documentation policy passed (754 statuses and
752 classified coordinates), navigation check passed 18 indexes, and
`cargo fmt --all -- --check` passed. The current ignored inventory is
regenerated and checked before commit; staged and unstaged diff whitespace
checks passed.
