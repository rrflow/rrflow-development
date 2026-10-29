# D-02: integrate the shared native capacity fixture repair

**Status:** active integration evidence; fresh candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-attunement-status-apfs-integration`
**Owner:** OBJ-07 / D-02a / POAM-006 / PR146

The preceding status-read candidate was
`57f4b27656865cac5464c8f9c20b0e133512d850`. Its
[run 40501264917, macOS job 122179242946](https://github.com/rrflow/rrflow-development/actions/runs/40501264917/job/122179242946)
failed during the `current_before_create` APFS capacity pressure fixture. The
log reports zero refill bytes, two attempted writes and 25011 ms elapsed at
the unchanged 25-second aggregate limit. The failure preceded the selected
engine operation; it does not establish a status-read defect or lost
acknowledged writes. The retained local log
`target/d02-status-macos-failure.log` is 502130 bytes, SHA-256
`bda16210c86785cdf350241480bae07d9dacf87f3b8640a01c7197724e11747f`.

After preserving that log, the driver requested cancellation of the
already-failed run rather than wait for unfinished lanes; those lanes are not
passing evidence. The PR146 brief was updated before an ordinary merge of the
shared fixture repair [PR150](https://github.com/rrflow/rrflow-development/pull/150)
head `824ada03bc9b04888c854a2f66155200aba65587`. Its
[journal](../gate-c/c-07f-apfs-refill-probe.md) records the test-first repair,
unchanged bounds and native validation still required. Git reported no merge
conflicts; generated navigation and inventory are regenerated for the
combined tree. The status read remains a read of the stored engine plan and
job with source verification explicitly unchecked.

The combined candidate passed 13 focused capacity fixture tests and the real
installed status-read/source-edit/reopen test on Windows. Navigation, inventory,
documentation and workflow checks passed after regeneration. It must receive a
fresh full candidate pipeline.
PR150 must pass and land through its own merge controls before this dependent
PR lands. No source revalidation, replan, memory-effect audit, native APFS
success or roadmap completion is claimed here.
