# Windows atomic publication under concurrent readers

**Status:** active package evidence; native repair and local acceptance passed, candidate gate pending

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/windows-atomic-publication-20261120`

**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), D-01 installed lifecycle.

Starting revision: `081282df0af594ba3ded2554c9ee5acd2ab88308`, tree
`b7104b59a97127cb0879e1528333d43e1c0aa983`.
Driver: this RRFlow implementation session. Alarm [#441](https://github.com/rrflow/rrflow-development/issues/441).
This prerequisite lands before [#440](https://github.com/rrflow/rrflow-development/pull/440), then integrates by ordinary merge.

## Recorded failure

On Windows, the full `rrd-engine` suite on the controller candidate reported
522 passed, 1 failed, 12 ignored. The unchanged concurrent serving publisher
failed with `Storage("Access is denied. (os error 5)")` at
`engine/tests/serving.rs:161`. The original log is retained at
`D:/.usr/Temp/rrflow-selective-recall-audit-20261120/engine-tests.log`.
The controller candidate records POAM-092. The exact native failure is still
under investigation; a passing rerun cannot establish a repair.

## Bounded change brief

Repair the canonical `rrd-lsm` anchored Windows replacement boundary so open
readers retain old bytes while new readers receive the replacement, without an
absent-name window. Preserve no-replace publication, parent-handle containment,
file validation and synchronization. No delete-first publication or retry loop.
Scope: `filesystem/native.rs`, optional private Windows implementation module,
`tests/filesystem.rs`, and engine serving error context if needed. The existing
engine concurrency oracle stays unchanged. Stop on a containment regression,
unavailable native atomic semantics or material expansion.

The replacement map is existing `MoveFileExW` replacement and publication tests
to the same `native::rename` boundary. Exclusive publication remains canonical
at `rename_new_to`; Unix behavior remains platform-native. No upstream code is
copied. Research uses Microsoft FILE_RENAME_INFORMATION / FileRenameInfoEx docs
and the locked cap-primitives 4.0.3 sharing implementation. Native source and
held-reader tests will decide the cause, not a timing retry.

## Review and verification scope

Read README, AGENTS, agent sessions, change-authoring, Gate D D-01; filesystem
metadata/open/rename and native helper functions; publication/containment test
bodies; engine serving publication/read/replacement and its concurrency test;
and installation bounded read. No whole-codebase source-review claim.

Acceptance: held-reader replacement oracle, filesystem target, owning LSM suite,
original engine serving oracle, strict lint/format/docs checks. After dependency
integration, run full engine suite and installed source-read controller driver
on #440. CI and cross-platform qualification are pending. Use four Cargo jobs
and the same checkout target `D:/.usr/Temp/rrflow-selective-recall-audit-20261120/target`.
No new resource process or container is needed. No roadmap completion change.

## Native cause and replacement evidence

Repair PR: [#442](https://github.com/rrflow/rrflow-development/pull/442).
The new held-reader test failed deterministically before production edits:
`cargo test --locked -p rrd-lsm --test filesystem replacement_keeps_open_readers_valid_and_publishes_new_bytes`
returned error 5 at `pending.rename_to(&published)`. This isolates the replacing
MoveFileEx operation; an ordinary anchored reader is enough to reproduce it.

The canonical Windows rename now opens the source object with DELETE access,
rejects reparse/nondisk objects, retains the caller's pinned parent/ancestor
handles, and calls `SetFileInformationByHandle(FileRenameInfoEx)` once.
Replacing operations set REPLACE_IF_EXISTS and POSIX_SEMANTICS; exclusive
operations set neither. No legacy retry or delete-first implementation remains.
The source uses WRITE_THROUGH; existing caller file sync and directory-sync
boundaries remain in place. This is not a new physical-power-loss claim.
The variable-length UTF-16 buffer uses native pointer alignment, checked sizes
and a zeroed terminator. The target path derives from its held directory handle.

Primary references: Microsoft
[FILE_RENAME_INFORMATION](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information),
[FILE_RENAME_INFO](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info)
and [SetFileInformationByHandle](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileinformationbyhandle).
No upstream source is copied. Native unsupported semantics remain an error.

Intermediate failures: compilation required the existing cap-fs-ext follow
trait import. A relative-name/non-null RootDirectory request returned error 87
on this Windows host. The implemented Win32 form uses a null RootDirectory
and the full name resolved from the retained target directory, preserving the
existing pinned-parent guarantee; it passes without a retry or fallback.
The first filesystem target after repair passed 13 tests, including the new
held-reader oracle and existing containment checks. Added directory-publication
and held exclusive-winner tests will verify the shared primitive's other users.

Production changed only `crates/persistence/rrd-lsm/src/filesystem/native.rs`;
regressions live in `tests/filesystem.rs`. The linked journal and gate index
are the only evidence changes. Engine serving code and its oracle are untouched.

## Local acceptance and review

- `cargo test --locked -p rrd-lsm`: 273 passed, 0 failed, 3 existing ignored,
  across 37 targets; log `publication-lsm-tests.log` in the session scratchpad.
  This run includes the first held-reader regression; the two additional
  publication tests were run in the final target below.
- Final `cargo test --locked -p rrd-lsm --test filesystem`: 15 passed, 0 failed,
  including retained old readers, directory publication, an exclusive winner
  held without delete sharing, ancestor/junction containment and cleanup.
- `cargo test --locked -p rrd-engine --lib engine::tests::serving::`: 15 passed,
  0 failed, 519 filtered, in 1.70 s. The original concurrent serving test passed
  unchanged. Log `publication-engine-tests.log`.
- Documentation, code-shape and generated execution inventory checks passed.
  Formatting and whitespace checks passed. Full engine and installed driver
  will run on the integrated controller candidate; no pass is claimed here.

Complete changed-function and regression diff reviewed after tests. Review
covers the unsafe buffer's alignment, size arithmetic, UTF-16 copy and Win32
class/flags; held-directory lifetime; exclusive publication failure semantics;
and all three added tests. Existing callers were searched and identified,
including backup directory publication, which motivated its regression.
No foreign process, container, worktree, stash or archive was created or removed.
Checklist: bound D-01 prerequisite, traceability, primary research, native failure
oracle, replacement evidence, preserved denials, scoped review, and actual local
results recorded. CI/merge evidence remains pending; no roadmap status changed.
Next: required candidate gate/queue, then integrate into #440 and rerun its
full engine and installed journey checks. Alarm #441 stays open until integration.

`cargo clippy --locked -p rrd-lsm --all-targets -- -D warnings` passed.

## Final source-handle containment review

Before landing, review tightened the source open to READ/WRITE sharing only:
its DELETE access performs the rename while denying another DELETE-capable
open that could move that same source between validation and the native effect.
This preserves the name under its pinned parent through the operation.
The 15 filesystem tests and strict LSM clippy passed again. Full LSM and serving
checks are rerun for this final revision rather than inheriting the earlier run.
The first ready event raced the push and named the journal-only SHA; that CI run
was cancelled and the event corrected. The later pre-containment candidate run
was also cancelled when the review changed code. Neither cancellation is a pass
or an erased failing test. Required CI will run on the final pushed revision.

Final source-pinning LSM rerun: 275 passed, 0 failed, 3 existing ignored across
37 targets (`publication-lsm-final-tests.log`). The broader engine run completed
590 passed, 0 failed, 12 ignored across 13 targets on pre-pinning revision
`5de5d09f0fa9f1e284ca82d4f74ba1a60d16e87c`
(`publication-full-engine-tests.log`); it is not represented as the final
source-pinning revision's full-suite result. The concurrent final serving build
hit LNK1104 because that earlier engine test executable was still running.
After the earlier run exited, the final serving check was rerun sequentially;
this is an owned build-artifact lock, not a claimed runtime test failure repair.
The original linker log remains `publication-engine-final-tests.log`.

Final sequential engine serving check: 15 passed, 0 failed, 519 filtered in 1.68 s (`publication-engine-final-recheck.log`). No production changes followed these results.
