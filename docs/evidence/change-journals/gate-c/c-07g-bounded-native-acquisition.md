# C-07g: bound a native sample response during unfinished cleanup

**Status:** active draft evidence; native qualification and exact CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-bounded-native-acquisition`
**Owner:** OBJ-02 / C-07g-h / POAM-035 / [PR195](https://github.com/rrflow/rrflow-development/pull/195)

## Brief and traceability

Starting head `0c71ce907767cdb0497bfceb1dec9974c8f15298`, tree
`e4d21a0756dce15a80ef6920cc6ab185f631b30e`. The PR brief was extended
before this implementation. `Owner::acquire_request` previously awaited its
worker's response channel without a deadline. If the `Process` monitor could
not complete native termination or reaping, the current-thread scheduler was
not itself blocked, but the acquisition could remain pending indefinitely.

`native_worker.rs::Owner` now bounds the response wait by the existing
ten-second worker deadline. On expiry it returns an explicit error and its
cancellation guard signals the worker; it does not emit a resource sample.
While the owner remains alive, its `Active` retains the monitor and denies a
new acquisition until the monitor finishes. Existing finite `shutdown` still
reports an error if cleanup has not finished. No campaign threshold, native
allocation, installed engine operation or observed resource data is changed.
The existing `Process` monitor and owner remain the canonical implementation;
no implementation is moved or removed.

The new focused test launches a real held helper. A test-only wrapper delays
the monitor's completion report *after the helper has actually been cancelled
and reaped*, simulating an outstanding cleanup report rather than creating an
unkillable OS process. It verifies the acquisition timeout, the exact
previous-worker-still-active rejection, a finite incomplete-cleanup error,
continued current-thread scheduling, and successful reuse after releasing the
monitor. The exact test is required in the existing installed-reopen CI lane
on each matrix platform, with negative policy checks for removal or bypass.
Reviewed the complete worker and process-monitor ownership paths, relevant
segment/drain cleanup, workflow and policy diffs, README, C-07 and POAM-035,
and preceding Windows and Linux image journals. The prior primary-source
research on [Tokio cancellation](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
and [Windows process termination](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess)
continues to apply; no upstream source was adapted. Existing bounded traces
and Job limits remain unchanged.

## Local results

The four changed code/workflow/policy paths, sorted as `path SHA256` lines,
have manifest SHA-256
`f9d5dc2a5bab6e70b74326e5ddb2859ffa6ca0f94f44bd5e6b66f39d309e1f12`.
On Linux x86_64 WSL2 with pinned Rust 1.99.1, the focused held-helper test
passed. The final `cargo test --offline --locked -p rrflow-cli --test
installed_reopen` passed 28 with 13 ignored; the explicit ignored installed
capacity smoke passed 20 attempts and 37 observations matching checkpoints,
including interruption and reopen; strict Clippy passed. Its debug harness
SHA-256 was `6e75fa24662921bb2260f55c0ce85a276217e4c24b232d9e1955652e28abe63e`.
The retained local log `target/c07-image-linux-deadline-v2-final.log` SHA-256
is `baf8ea5d33e670409f966cd61da6fd7c808b60e717be6602d97c238fb5df5d84`.

On Windows x64, the focused test passed; the final installed-reopen suite
passed 27 with 12 ignored, and the explicit installed smoke passed 20
attempts and 36 observations matching checkpoints with interruption/reopen.
The Git Bash native CI step passed six image tests and the real Windows Job
membership/cancellation/reuse and abrupt-parent tests. Strict Clippy, six
native policy tests and workflow policy passed. The debug Job harness SHA-256
was `417a688565a0e4b4d555585963791b8d12e5124d0bcf95f55bb296b9d045b3d8`;
retained local log `target/c07-image-windows-deadline-v2-final.log` SHA-256
is `0a35d445f0f59ece6355836a2e84099b11fc32d0e6c1266ca5d4fe0d29dfb9b5`.
These debug/WSL tests do not establish hosted release or native matrix results.

## Remaining stops and checklist

A timed-out owner can still be dropped while its monitor is unfinished; the
process guard's native `kill` and `wait` failure modes remain unproven. This
change neither proves an OS-stuck helper is gone nor licenses an incomplete
cleanup to count as a successful campaign. Binary provenance before the
Windows pin, uncontained launch identity, macOS identity, Linux cgroup, the
original sample-gap cause, frozen 100,000-record profile, native matrix and
24-hour soak remain open. PR177 and PR194 remain prerequisites; no hosted
candidate or protected merge-group CI has run on this head. No roadmap, POAM,
C-07 or alpha status changed. Checklist: pre-edit brief, preserved original
owner/monitor, controlled failure and real helper, owning suites and strict
lint, explicit unrun checks, new journal, navigation/inventory and diff review
before commit.

Final repository checks: candidate-admission policy passed 11 tests,
standalone policy 7, inventory tests 10, documentation policy (753 statuses
and 751 classified coordinates), navigation 18 indexes and `cargo fmt --all
-- --check` passed. Current ignored inventory is regenerated and checked
before commit; staged and unstaged diff whitespace checks passed.
