# C-07g: pin the Windows qualification harness image

**Status:** active draft evidence; exact candidate and qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-windows-harness-image-pin`
**Owner:** OBJ-02 / C-07g-h / POAM-035 / [PR195](https://github.com/rrflow/rrflow-development/pull/195)

## Brief, baseline and traceability

Starting PR195 head `1b13e243705afb8e5d003cb0be0cfaad6169c56c`, tree
`c0180ffdf9324fff8a56924554328602affd6492`. The PR description records this
image-identity scope and CI refinement before edits. `run_sustained_capacity.py`
previously hashed an ambient Windows harness pathname and started its owned
Job without retaining the image. A local probe launched a copied, held actual
test executable and successfully renamed its running pathname. A read handle
without write/delete sharing denied the tested original-name rename; deleting
a separate hard-link name was allowed. This confirms a pathname race, not the
cause of the earlier sampling-gap failure.

New `scripts/ci/windows_harness_image.py` opens absolute drive-path ancestors
without delete sharing, rejects reparse and nondisk objects and volume changes,
and retains an ordinary read-only image handle without write/delete sharing.
It rejects multiple hard links at admission and alternate-stream path syntax,
hashes at most 2 GiB through that held handle, and checks its identity and
extent after reading. `run_sustained_capacity.py::windows_allocation_proof`
compares the existing receipt digest with that held image and retains the
guard throughout `run_owned` and its Job cleanup. The Windows Job regression
runner uses the same context, proof and owned process boundary. The Linux and
nonqualified launch paths are unchanged. No engine implementation, candidate
job, test, allocation cap or qualification threshold is removed.

New `test_windows_harness_image.py` uses a copied actual test executable and
owned temporary paths. Its real Windows tests cover the unguarded running-image
rename, live pin denial of a different-path rename, ancestor rename and
conflicting writer, wrong receipt digest, preexisting hard link, preexisting
writer, alternate stream and reparse leaf/ancestor. On the tested NTFS host an
additional hard link can be created while pinned and a same-file rename can
succeed; the test verifies the resulting bytes still match and denies replacing
the pinned pathname with a different file. The pin does not claim to forbid
every namespace operation. The existing Windows installed-reopen step now
runs the image tests and owned-Job runner under `bash` with
`set -euo pipefail`; the existing policy and negative cases require both
commands and the fail-fast shell.

Reviewed the prior Linux image journal and POAM-035, `native_allocation.rs`,
`Process` monitor, full `owned_windows_process.py`, relevant
`run_sustained_capacity.py::proof` and launch branch, current workflow/policy
and test runner, `rrd-lsm::filesystem::AnchoredPath` and native validation,
and the new files and complete staged diff. The launcher guard follows the
existing anchored no-reparse/retained-parent behavioral boundary without
introducing a production authority or importing upstream implementation.
Microsoft documents [CreateFile sharing](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)
and the [limits of file identifiers](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/ns-fileapi-by_handle_file_information).
Existing bounded traces and 2 GiB test caps are preserved; no performance
measurement or new product telemetry is introduced.

## Actual Windows results

Final seven workflow/implementation/test source paths, sorted as `path SHA256`
lines, have manifest SHA-256
`19cd95ac45e33d8ad0fc6ffa4548e7929d5a38eef1df029a3cb1232e5f695d76`.
On Windows x64 with pinned Rust 1.99.1, the actual Git Bash step using debug
and offline options ran `python scripts/ci/test_windows_harness_image.py
--profile debug --offline` (6 passed, none skipped), followed by
`python scripts/ci/test_native_sample_worker.py --profile debug --offline`.
The latter passed the real Job engine/harness/helper membership, cancellation
and reuse test and the verified-owned abrupt-parent/held-helper cleanup test.
Its 2 GiB aggregate/process and 4 GiB declared-host test receipt named harness
SHA-256 `09c07f69a41c62ad991a3324ab61ad750bd1a11cc22f75045cf18b4a57ff80c1`.
`python scripts/ci/test_sustained_launch.py` passed 3 tests;
`python scripts/ci/test_native_sample_policy.py` passed 5; workflow policy
and Python compilation passed. An independent fail-fast shell probe returned
nonzero and did not execute its second command after `false`. The retained
local log `target/c07-win-pin-v3.log` SHA-256 is
`3938f2d2633afe600df6c05493067a25233a4e367e16ec9001bd458a209d9a66`.
The separate initial running-image and hard-link probes are retained under
local `target/` and use only owned fixtures. No hosted release step or real
frozen qualification campaign was run.

## Remaining stops and checklist

A guard acquired after a binary was built does not prove its build provenance
or bind an already-running image before acquisition; nonqualified launches and
macOS image identity remain unproven. The tested hard-link behavior does not
prove all Windows filesystems or adversarial mount/namespace behavior. An OS
termination or reap failure can still leave cleanup unconfirmed; the original
100,000-record sampling-gap cause, native matrix and 24-hour soak remain open.
PR177 and PR194 are prerequisites; exact candidate and protected merge-group
CI for this head have not run. No roadmap, POAM, C-07 or alpha status changed.
Checklist: pre-edit brief and bounded effect scope, existing launcher/worker
traceability, real negative and positive tests, both CI commands and fail-fast
policy, known limitations, linked journal, generated navigation and inventory,
and full diff review before commit.

The unguarded rename is an observed characterization, not a portability
requirement: a host that already denies it may proceed to the pin assertions.
Final repository checks: candidate admission 11 passed, standalone policy 7
passed, inventory tests 10 passed, documentation policy passed (752 statuses,
750 classified coordinates), and navigation check passed 18 indexes.
