# Serving-record false absence during Windows replacement

**Status:** active repair evidence; local Windows acceptance passed, candidate and queue pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/serving-record-false-absence-20261120`
**Owner:** OBJ-01 / [D-01](../../../roadmap/rrflow-1.0/gate-d.md), [POAM-092](../../../poam/rrflow-1.0-alpha/poam-092.md)

[PR450](https://github.com/rrflow/rrflow-development/pull/450) was opened with
its scoped brief before editing, at `d1d9c5435a199306a5d41602b40a26e6cc4f39ab`,
from main `95cd44d86065e76c242ec5e5a7cc9266cb208044`.
[Alarm449](https://github.com/rrflow/rrflow-development/issues/449) holds the
independent timestamp PR448 in draft. POAM-092 is reopened on new evidence;
the earlier [native publication repair](windows-atomic-publication-20261120.md)
and its historical passing checks remain recorded.

## Failure and boundary

The PR448 full library suite produced 522 passed, one failed and 12 ignored
in 417.42 seconds. The existing concurrency test's reader found `None` during
replacement at `engine/tests/serving.rs:149`, then its join failed at line 164.
This differs from the earlier access-denied publication failure. PR448 changes
no serving or filesystem source. Its log is `cold-load-time-engine-suite.log`
in `D:/.usr/Temp/rrflow-selective-recall-audit-20261120`.

Readback returns `None` only after anchored metadata reports NotFound. Temporary
operation diagnostics distinguished the native open, mount admission, object
validation and final metadata calls without changing error identities. The
new filesystem-only probe uses four readers and one writer, replacing one
published name 5,000 times; it does not remove that name. Each reader rejects
NotFound and must have completed a successful observation. A finish flag and
scoped joins retain writer errors and do not leave a reader running on failure.

The first probe and eleven repeated runs passed. Run 12 failed in 3.62 seconds:
three readers observed native open error 2 (`ERROR_FILE_NOT_FOUND`). There was
no engine involved. That failed run checked reader absence before unwrapping
the writer result, so its output does not establish the writer outcome. Log:
`serving-metadata-filesystem-repeat-12.log`. This localizes the symptom to the
canonical filesystem path, but does not establish a particular kernel defect.

The old-held-reader metadata probe passed, including native
GetFileInformationByHandle after replacement (`serving-metadata-held-probe.log`).
Thirty executions of the unchanged engine concurrency test passed; they are
inconclusive about the intermittent absence, not a repair. Their outputs are
`serving-metadata-repeat-1.log` through `-30.log`. A diagnostic full engine run
is retained separately as `serving-metadata-full-probe.log`.

## Traceability and research

Current behavior: `ServingPublisher::publish_once` calls `replace_file`, which
uses `AnchoredPath::rename_to` and the one Windows FileRenameInfoEx implementation.
Readers use anchored metadata/open and bounded validation. Target behavior is
old-or-new visibility with absence reserved for an actually unpublished name.
The same filesystem boundary owns native repair; the serving oracle stays.
No delete-first path, compatibility implementation or unbounded retry is allowed.

Read: serving publication/read/replacement and its concurrency fixture;
`filesystem.rs` retained-root, traversal, metadata and rename methods; native
object/mount checks and the complete Windows rename helper; held-reader,
exclusive-winner and containment test bodies; POAM-092 and its earlier repair
journal; README/AGENTS/session/change-authoring and D-01 owners. Locked
cap-primitives 4.0.3 Windows open implementation, open_unchecked, MaybeOwnedFile
and manual-open call path were inspected; no dependency source was modified.
No whole-codebase review is claimed.

Microsoft documents retained old handles and new-name opens for
[FileRenameInformationEx POSIX replacement](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information).
[Microsoft/STL issue5501](https://github.com/microsoft/STL/issues/5501) reports
intermittent missing-name reads on NTFS, including a comment about POSIX rename.
It is a similar reported symptom, not independent proof of this machine's cause.
No upstream implementation is copied. The local native-error oracle decides
whether a proposed change is acceptable.

The replacement now retains the old target handle through FileRenameInfoEx.
It requests only FILE_READ_ATTRIBUTES and keeps no-follow/native object checks;
the held target is released after the namespace operation. Existing absence
still permits publication, and exclusive publication does not open the target.
This adds one short-lived metadata handle for a replacing operation, without
reading target contents or rewriting additional files. Source and ancestor
pinning and the existing rename flags remain. Temporary operation logging was
removed; `filesystem.rs` has no final diff.

The initial target-retention experiment passed 25 consecutive runs (125,000
replacements), recorded in `serving-metadata-target-pin-repeat-1.log` through
`-25.log`. That version requested read access; the final form reduces this to
metadata access. This supports the target-lifetime diagnosis but is not a
Windows kernel trace or proof of universal NTFS behavior. The unchanged probe
is retained to detect recurrence; no passing rerun of unchanged production is
represented as a fix.

The final-form filesystem target passed all 16 tests in 5.30 seconds, including
held readers, metadata, exclusive winner, directory publication, containment and
junction denial (`serving-target-final-filesystem.log`). `cargo test --locked
-p rrd-lsm` passed 276 tests, zero failed, three existing ignored, across 37
targets (`serving-target-final-lsm.log`). Strict LSM all-target clippy passed
(`serving-target-final-clippy.log`). The unchanged engine serving suite passed
15 tests in 2.00 seconds (`serving-target-final-engine-serving.log`). The final metadata-only form then passed another 25 consecutive probes
(125,000 replacements), `serving-target-final-repeat-1.log` through `-25.log`.
After review, the test checks its writer result before reader assertions so a
future failed writer cannot be hidden by a reader failure; all 16 filesystem
tests passed again in 3.91 seconds (`serving-target-final-oracle.log`). No
production change followed the final-form repeated probes. The full `cargo test --locked -p rrd-engine` then passed 591 tests, zero
failed, 12 existing ignored across 13 targets. The library portion completed
523 tests with 12 ignored in 373.60 seconds. Log:
`serving-target-final-engine-all.log`. This covers the production/test blobs
below on the baseline with this package's changes; integration with later
main PR447 is recorded separately, not silently inherited.

The diagnostic full engine run on pre-retention source with temporary logging
passed 523 tests, zero failed and 12 ignored in 482.15 seconds. This is explicitly
inconclusive, like the 30 earlier focused passes. It is not final-repair
acceptance. An attempted stop was guarded by exact owned-process identity;
the process had already exited and the guard refused the action. Nothing was
terminated. Both changed Rust files and the complete final diff were reviewed.

## Acceptance and limits

Use the existing checkout-specific Cargo target in the scratchpad and four
build jobs. Acceptance: the captured filesystem failure, retained original
serving concurrency oracle, owning LSM/engine suites, strict lint, format,
documentation, candidate and merge-queue gates. Results are recorded as they
finish. Stop on unestablished cause, a containment regression or material scope
expansion. No release, capacity or overall native-storage qualification is claimed.

PR447, independent of this failure, merged through candidate 44833873955 and
merge-group 44835662427 at `3a8f8cd7dec52113c2cadbe511ab18df6da8f460`.
Its twelve changed paths match source `388a7bfd4c9afcf1035cc4d5c61c6d34444dd3c9`;
`root-merged-447.bundle` was verified before deleting its local branch. The
remote branch was automatically deleted. This is merge cleanup evidence, not
proof of every historical branch's behavior.

Navigation regenerated 21 indexes, 1,282 nodes and 4,509 edges; inventory
contained 3,691 paths. Code-shape passed with its existing two oversize files
and 51 listed pattern sites, none grown. Documentation passed 1,285 statuses
and 1,281 classified coordinates (`serving-target-docs.log`). A later journal
link/content update requires final regeneration/check before commit.

A final documentation check exposed a Windows text-encoding mistake in the
POAM edit: Python's default cp1252 read re-encoded the UTF-8 heading's em dash.
The original UTF-8 text was restored explicitly; the checker was not changed.
Final production blob: `bd279f1c6bd718ebf7245307d61832e8a298926c`; final filesystem
test blob: `6dbfd3fd4be42cb5a098870ccf9134ea984d8b8a`.

Final documentation recheck passed 1,285 statuses and 1,281 coordinates;
navigation now contains 4,510 edges after the POAM journal link. Log:
`serving-target-docs-recheck.log`. Checklist: scoped PR brief before edits,
native failure and before/after oracle, existing canonical owner, primary-source
research, bounded resource decision, preserved refusal/concurrency tests, full
changed-file/diff review, journal/index and local package checks complete.
Candidate/merge-queue checks and refresh onto latest main are pending. No
broader release, parity, power-loss or hardware qualification advances.
