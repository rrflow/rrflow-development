# Windows atomic publication under concurrent readers

**Status:** active package evidence; reproduced engine failure, native repair pending

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
