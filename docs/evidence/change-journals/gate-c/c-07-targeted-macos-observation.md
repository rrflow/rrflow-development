# C-07 single-case macOS preflight observation

**Status:** active characterization evidence; native matrix and repair pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-targeted-macos-observation`
**Owner:** OBJ-02 / C-07f / POAM-035
**Pull request:** [261](https://github.com/rrflow/rrflow-development/pull/261)

This is a later observation of the [targeted-preflight change](c-07-targeted-macos-preflight.md),
not a correction to its original journal or an acceptance record.

The manual command dispatched the branch with `point=wal_before_write`,
`mode=fresh` and `repetitions=1`. [Run 41667800353](https://github.com/rrflow/rrflow-development/actions/runs/41667800353)
completed successfully on macOS at exact head
`732ccf8020b150e8328de238dd5374e6b02698f4`. The test executable targeted
`aarch64-apple-darwin`, debug profile with assertions; the build identified
itself as unqualified. Harness SHA-256:
`f84bcff5c4e44a270976bbd5ce62a0f352ff489f1d37c1c59813740087d48d29`.
The installed executable SHA-256 was
`c2a6dd7db2d66b939e7747a6a9ab4847042667feaf054a3f7dfab95aecd1a036`.

On the disposable 256-MiB APFS image, the selected `wal_before_write`
operation recorded native code 28 for the WAL write. The receipt reported
27 logical calls, 28 attempts, 27 acknowledged, one uncertain, 27 resolved
present, zero unresolved and no violations; expected-history SHA-256 was
`cb32710d3a043f3ebc480b58fc8d8076b0f98183b0144f98b1be95c78add133e`.
The filler wrote 257,757,184 bytes. Metadata fill created zero files, made two
create attempts and one settlement, with 41 microseconds of measured settlement.
The job reported owned-volume cleanup and the harness emitted its completed
single-case suite. The workflow's deliberately injected create-error capture
also passed; it is a diagnostic test, not capacity evidence.

For comparison, the earlier failed merge-group run reported 758 files, 892
create attempts and 139 settlements before timing out; the earlier successful
candidate run logged zero metadata files across 32 native fills. The fixture
helper and harness paths did not change between those base revisions. This is
observed variability; neither comparison establishes its cause. A successful
single case does not erase the failure or prove the full matrix, installed
cross-platform qualification, 24-hour soak, C-07 completion or alpha readiness.
The run is tied to the head above, not to any later documentation commit.
