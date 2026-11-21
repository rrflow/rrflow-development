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
