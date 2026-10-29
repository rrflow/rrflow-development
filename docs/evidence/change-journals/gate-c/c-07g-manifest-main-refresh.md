# C-07g: refresh shared-manifest candidate onto current main

**Status:** active integration evidence; new candidate CI and sustained qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-manifest-main-refresh`
**Owner:** OBJ-02 / C-07 / POAM-035 / PR142

## Candidate and integration

The preceding candidate `5e5dcb9144348c273b895e440f31433e41913eef` passed
[candidate CI run 40439001624 attempt 2](https://github.com/rrflow/rrflow-development/actions/runs/40439001624).
After main advanced, an attempt to enter the merge queue returned that the
required `pipeline / ci-gate` was expected despite the old head's successful
check. No rule was bypassed. The continuation brief on
[PR142](https://github.com/rrflow/rrflow-development/pull/142) identifies the
refresh before this merge. The branch was converted to draft; current public
main `86adada6dab6411cc58cb9aaa931e35472c0f452` was merged normally, retaining
both histories and all source changes. Git reported no merge conflicts.

The immutable-manifest sharing implementation and its prior source review are
owned by [the preceding package journal](c-07g-shared-manifest-generation.md).
This continuation does not change its storage code, resource bounds, native
thresholds or roadmap status. It regenerates navigation and the Git-derived
inventory for the combined tree. The new candidate requires fresh CI; prior
results are not attributed to it.

## Exact previous candidate native capacity evidence

The old candidate's installed domain crash jobs ran
`python3 scripts/ci/run_native_capacity.py` on actual hosted native platforms.
The retained logs report eight passing fresh/existing cases on each of Linux,
Windows and macOS, covering `wal_before_write`, `segment_before_create`,
`manifest_before_create`, and `current_before_create`. The separate macOS
repetition reported 24 passing cases. These are capacity-exhaustion and recovery
results for the old exact candidate, not the Desktop sustained growth profile.

- [Linux native job](https://github.com/rrflow/rrflow-development/actions/runs/40439001624/job/121974731607): harness SHA-256 `bd2eeb6937824060c9916e74309e63b2b952732611da9af66cb0ddfe3ba18cc1`.
- [Windows native job](https://github.com/rrflow/rrflow-development/actions/runs/40439001624/job/121974731611): harness SHA-256 `8d75de0efe81b2a73ca2dd576f0cc201fea04a5e25f6a3019adeb4d02a94af4c`.
- [macOS native job](https://github.com/rrflow/rrflow-development/actions/runs/40439001624/job/121974731565): harness SHA-256 `119b22a71397249a2466cfd7b855b924b5d98feb8178ebd5b02cdd943a304347`.

The earlier full-corpus deadline failure and the later KVM-denied local launch
remain recorded; no new complete full-corpus readback or 24-hour soak occurred.
Fetching three job logs concurrently initially hit a GitHub CLI zip-cache error
for two jobs; fetching those logs sequentially succeeded. Local raw logs remain
under `target/c07-pr142-{linux,windows,macos}-capacity.log`.

## Checks and remaining work

On the refreshed Windows source, `cargo test --offline --locked -p rrd-lsm
--lib segment::reader::lifetime_tests` passed all five tests. `cargo test
--offline --locked -p rrd-lsm --quiet` passed the complete suite with its
ignored fixtures reported; `cargo clippy --offline --locked -p rrd-lsm
--all-targets -- -D warnings` passed. Navigation and inventory regeneration and
checks, documentation policy and CI workflow policy passed. The exact new
candidate pipeline remains required. No independent speedup, sustained-profile,
soak or C-07 closure is claimed. The root cause of the stale required-check
state is not established merely by refreshing the branch.

Checklist: current main and former candidate identified; original code and
tests preserved; local focused oracle and owning suite passed; generated
navigation and inventory checked; candidate CI and merge controls required before landing; all
qualification gaps remain open.
