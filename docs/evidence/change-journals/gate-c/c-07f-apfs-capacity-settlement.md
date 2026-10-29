# C-07f APFS disposable filler capacity settlement

**Status:** active implementation evidence; native APFS candidate pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07f-apfs-capacity-settlement`
**Owner:** OBJ-02 / C-07f / POAM-035 / PR140

## Baseline and failure

[PR140](https://github.com/rrflow/rrflow-development/pull/140) published its
brief before implementation. Driver: current Codex session. Current public
main baseline `9a441efd5cbef924fc01f6cc47b6e6c5c844d4e6` includes merged
PR130/131; opening revision `483e25c627e579b0e2618908f9494af8f0d3987d`.
No unmerged code dependency or direct-main push.

PR134 candidate `4e2dbaffad29039d742f6b151f7636378ce3e7fe`, run
[40266752462](https://github.com/rrflow/rrflow-development/actions/runs/40266752462),
macOS job121391135697 failed while preparing fresh WAL capacity pressure.
Initial data fill: 252706816 bytes in8105ms,60 interval file barriers, one
capacity barrier,7856391us total barrier time. Metadata preparation reached
1613 files,1687 create attempts,86 settlement calls and25006ms elapsed against
the original25-second limit. This is a fixture timeout before selected WAL
failure/recovery proof, not demonstrated acknowledged-data loss.
Full retained log: `target/pr134-macos-121391135697.log`, 207174 bytes,
SHA-256 `bc6a0f5fdf9feef0d53c789cdb99cf1ee9ecbc079c317f0eec39e6e209e2b382`.

## Exact change and replacement proof

`capacity.rs::synchronize_allocation` replaces the unconditional periodic
file sync schedule at `Filler::fill` with an explicit disposable-filler policy.
APFS skips interval file synchronization; Windows/Linux retain it. Every final
capacity boundary still calls file synchronization followed by the existing
waited owned-volume settlement. `finish_allocation_barrier` retains capacity
and noncapacity error semantics. The same `fill_data` loop consumes newly
available allocation and requires a fresh one-byte capacity failure after the
final barrier. Metadata create denial, joint pressure convergence,25-second
aggregate clock, byte/file caps, ownership/cleanup and independent installed
history checks are unchanged. No engine durability, runtime API or production
source change. Diagnostics count actual file barriers, so skipped intervals do
not falsely inflate the synchronized-work count.

The first regression drives the real fill algorithm through60 periodic
intervals and allocation released by final volume settlement. Before the
policy change it failed with62 file syncs versus the required2. Afterward it
checks both platform policies: APFS requires two final file/volume barriers,
other platforms retain62 file syncs, all delayed bytes are consumed and native
capacity-code denial remains necessary. This demonstrates scheduling and
pressure semantics, not APFS wall-clock success. Existing noncapacity,
partial-settlement, metadata, byte/time-cap and installed crash tests remain.

Source scope: existing `crates/adapters/rrflow-cli/tests/support/capacity.rs`.
Supporting scope: this journal, POAM-035, generated navigation/file inventory.
No new runtime or fixture lane, source moves, thresholds or dependency changes.
Stop for weakened final pressure proof, swallowed errors, unexpected native
failure, runtime changes or invented timing/qualification claims.

## Research and reviewed boundaries

Apple's [fsctl definitions](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/fsctl.h)
and [handle_sync_volume](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/vfs/vfs_syscalls.c)
show FSCTL_SYNC_WAIT selecting waited volume synchronization. That existing
boundary remains. Deferring intermediate disposable-file syncs is our inference
from source and measured fixture costs; Apple does not guarantee this runner's
timing. No upstream code was copied and no engine durability policy changed.

Read in full: capacity_metadata.rs, capacity_apfs.rs, prior batched-allocation
journal, PR template and change-authoring procedure. Targeted reads: README,
AGENTS, C-07/POAM owner, capacity.rs data/refill/metadata/joint-pressure/cleanup
and synchronization tests, actual candidate log. Changed helper, closure,
regression and full diff are reviewed before commit. Trace/resource decision:
retain original phase timing, native error receipts, original budgets and final
settlement; do not relax a deadline to mask the preparation cost.

## Verification

Windows locked shared build, dev/test debug level zero:

- Focused new regression initially failed:62 file syncs versus2, exit101. Log `target/c07-apfs-final-before.log`.
- `cargo test -p rrflow-cli --test installed_crash --features crash-test --locked -- capacity::filler_tests --nocapture`:11 passed,0 failed.
- `cargo test -p rrflow-cli --test installed_crash --features crash-test --locked -- --nocapture`:25 passed,0 failed,2 owned-volume qualification tests ignored;20.30 seconds test execution. Real installed crash/publication/reclamation histories remain covered.

Logs: `target/c07-apfs-final-after-0.log`, `c07-apfs-final-after-1.log`;
strict lint results and final checks follow before commit. Native APFS image
repetitions, NTFS/ext4 owned-volume qualification and24-hour soak were not run
locally. Candidate CI must prove the real selected native failures and history;
local allocator tests cannot replace that. C-07/POAM-035 remain open.

Checklist: pre-effect PR/owner/baseline, source/failure grounding, replacement
map, research/provenance decision, exact first failing and corrected oracles,
retained limits and errors, linked journal and generated projections, complete
diff review and normal candidate/merge gate. No roadmap completion change.

Strict lint `cargo clippy -p rrflow-cli --bin rrflow --test installed_crash
--features crash-test --locked -- -D warnings` passed in 38.27 seconds; log
`target/c07-apfs-final-after-2.log`. Formatting initially reported one new
assertion's line wrapping; explicit Rust 2021 formatting corrected it without
changing semantics. Documentation/navigation/inventory checks and complete
source/diff review follow the correction. No native success claim follows.

Final checks passed: workspace and explicit support-file formatting,
documentation (612 statuses/610 coordinates), navigation (17 indexes,
611 nodes/2420 edges), inventory (1738 records), workflow policy and whitespace.
Changed helper, closure, test and complete diff were reread. The original
fill_data algorithm, final barrier/error implementation and byte/time bounds
are unchanged. Full native candidate remains the decisive acceptance step.
