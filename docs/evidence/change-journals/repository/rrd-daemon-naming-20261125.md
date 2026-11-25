# RRD names the daemon; the order of work is fixed

**Status:** active planning evidence; no code changed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/rrd-daemon-naming-20261125`
**Owner:** [Code structure and conventions](../../../architecture/code-structure-and-conventions.md), sections 5, 6.1, 6.8 and 6.10.

## Change brief

Baseline `d98830da1`. On 2026-11-25 the owner said the running product is RRD, the Reason Ready Daemon; that rrflowDB, rrflowKV, rrflowTX, rrflowCX, rrflowQL and rrflowMX must each be a real boundary in code; that ACID and every SurrealDB capability are proven in the engine before Connectome's folders are changed to mirror it; and that macOS must be built and tested properly, or the work halts.

Current behavior: the plan retired the `rrd-` prefix entirely and named the engine crate `rrflow-core`. Target: `rrd-` is kept by the four daemon crates only (`rrd-engine`, `rrd-server`, `rrd-client`, `rrd-contract`), which matches the system overview's existing definition of RRD; every component crate is `rrflow-<name>`. Package 14 is gated on the two proofs. A macOS job that did not run is not a result. Scope: the structure record, one sentence of the system overview, a correction in the Connectome pointer journal, this journal. No code. Stop if an objective, roadmap or POA&M status would have to change.

## Reading of the direction

Keeping `rrd-` on exactly those four crates is my application of the owner's statement and of the system overview's row for RRD. The owner has not named the crates.

## macOS, measured

- Connectome: in the last twelve Connectome CI runs the macOS job passed ten times and was cancelled twice. Run 45668600633 (merge queue) passed all six jobs, macOS included, on `c39d8888`, the commit now on `main`. Run 45672191968 (push, same commit) had its macOS job cancelled after fifteen minutes with no runner.
- The earlier statement in the pointer journal that no macOS result exists for that commit was wrong and is corrected there.
- Not examined: what the engine's macOS jobs cover, and whether the owner's concern is about something a hosted runner cannot show.

## Verification

- `python scripts/knowledge/render_navigation.py` regenerated the index; `check_documentation.py` and the `workspace_architecture` test results are in the pull request.
- Not run: any build; no code changed.

No objective, roadmap or POA&M status changes.
