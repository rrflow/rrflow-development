# Recovered manifest-sharing stash integration

**Status:** active integration; validation and queue pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/manifest-stash-integration-20261121`
**Owner:** [Gate C](../../../roadmap/rrflow-1.0/gate-c.md), C-07 and [POAM-011](../../../poam/rrflow-1.0-alpha/poam-011.md)

## Change brief

Start from ffb4c4b1779212aa8fa80e2d5908f3685f98d4ab. Integrate the recovered
stash a2004fd90840c8fd7061420e8b33f4b8bfed1a40 through an ordinary merge,
resolve conflicts into the current rrd-lsm ownership boundaries, retain both
original tests, validate and land through the green queue. The owner requests
actual integration of recovered work. This does not replace the historical
inventory or claim other stashes complete. The preceding PR458 fixture defect
has passed its complete candidate CI; its queue run remains pending.

Read the complete tracked stash patch: database.rs, segment/reader.rs and
segment/reader_lifetime_tests.rs. Current destinations include database.rs,
database/transactions.rs and segment/reader/views.rs. The behavior is one
shared immutable manifest generation per admitted view, surviving later
publication/database drop, with identity/content validation and released
ownership after refusal. Both characterization tests are retained under their
original names in reader_lifetime_tests.rs. No untracked third parent exists.

First oracle: cargo test --locked -p rrd-lsm --lib manifest. Owning suite:
cargo test --locked -p rrd-lsm. Require formatting, documentation/navigation,
required candidate CI and queue checks. Stop on unexplained conflicts, removed
assertions, weakened admission, lost lifetime guarantees or absent source.
No new external research, upstream adaptation, telemetry or benchmark is needed
for this recovered-source merge. No performance or full parity claim is made.

The source and recovery bundle are readable locally; git bundle verify passed.
Exact merge results, conflict resolution and commands will be recorded below.

## Integration and validation

[PR460](https://github.com/rrflow/rrflow-development/pull/460) retains the source
stash as a parent of ordinary merge 5dd3a5b2. Conflicts affected database.rs,
segment/reader.rs and reader_lifetime_tests.rs. The old large method bodies now
live in database/open.rs, transactions.rs, flush.rs, snapshot.rs, compaction.rs
and segment/reader/views.rs. Retained their shared Arc manifest ownership and
pointer/content checks, current admission arguments, engram terminology and the
additional memtable-release assertion. The original two test names and their
lifetime/content/refusal assertions remain. No fallback or duplicate module was
introduced. The resolved application tree is unchanged from the baseline; this
package integrates the recovered history and verifies its retained behavior.

Windows, CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target, CARGO_BUILD_JOBS=4:

- cargo test --locked -p rrd-lsm --lib manifest: two original tests passed.
- cargo test --locked -p rrd-lsm: 276 passed, zero failed, three ignored across 37 result groups.
- python scripts/knowledge/render_navigation.py: passed, nearest index generated.
- python scripts/ci/check_documentation.py: passed.
- cargo fmt --all --check and git diff --check: passed.

Logs: D:/.usr/Temp/rrflow-project-policy-20261121/manifest-stash-focused.log and
manifest-stash-suite.log. The first log-count reader assumed UTF-8 and failed;
reading the PowerShell log as UTF-16 recovered the counts. Tests themselves
passed. Original stash patch read in full; current mapped ownership/admission
regions and both original test bodies read for conflict resolution. This is not
an exhaustive review of the storage implementation. Candidate CI and queue
remain required. No new native performance, Fjall parity or broader historical
absorption claim; roadmap status unchanged.
