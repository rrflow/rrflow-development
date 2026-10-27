# C-07g: integrate the APFS refill repair with manifest sharing

**Status:** active integrated candidate; exact candidate CI and native qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-integrated-refill-repair`
**Owner:** OBJ-02 / C-07f,g / POAM-035 / PR142

## Integration

The manifest candidate before this merge was
`ab078b833ebe321a509d2da860db0db2e4f03720`. Its exact
[run 40500380910](https://github.com/rrflow/rrflow-development/actions/runs/40500380910)
failed the macOS `current_before_create` APFS pressure case at the existing
25-second limit. The full failure, raw log identity, test-first fixture repair
and separate native candidate requirement are recorded in
[the refill repair journal](c-07f-apfs-refill-probe.md) and
[PR150](https://github.com/rrflow/rrflow-development/pull/150).

The PR142 brief was updated before an ordinary merge of exact PR150 head
`824ada03bc9b04888c854a2f66155200aba65587`. Its capacity fixture change,
tests, journal and POAM entry are retained alongside the existing manifest
sharing source and journal. The only merge conflict was the generated
`docs/roadmap/rrflow-1.0-file-plan.jsonl`; the existing side was selected and
the combined Git inventory regenerated. No deadlines, pressure proof, runtime
storage format, or configured limits were relaxed. The PR150 candidate must
pass and land through its own required controls; this integrated candidate
must pass fresh CI before PR142 can land.

## Validation and limits

The separate PR150 Windows fixture suite passed 13 tests and its complete
installed crash target passed 27 with two owned-volume fixtures ignored.
The combined branch passed 13 focused fixture tests and five manifest lifetime
tests. Generated navigation/inventory and documentation/workflow checks also
passed. It must publish its exact candidate and complete fresh CI. These local
tests are not native APFS
qualification. No complete sustained corpus or actual 24-hour soak is claimed.
Old candidate CI, native capacity and release-build evidence remain bound to
their original source identities and are not transferred to this merge.

Reviewed the original failure log, both source diffs and journals, PR briefs,
C-07/POAM owner and the change-authoring/CI procedures. No source adaptation,
new dependency, new diagnostic authority or implementation removal is involved.
One journal-update patch missed its expected context and was reapplied correctly.
No roadmap or POA&M completion status changes.
