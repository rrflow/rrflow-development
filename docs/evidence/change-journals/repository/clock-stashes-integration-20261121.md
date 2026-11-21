# Recovered clock stashes integration

**Status:** active integration; candidate and queue pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/clock-stashes-integration-20261121`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), D-01 and [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md)

Baseline 15ed675d403b30eabfe69882bda39e1a0ce34c6c. Integrate recovered clock
stashes 5a3e0f4aa0a69affb92509219d5f4a28be68875a,
04182088ee3f3f6faee00bd2b976e57e321ab6bf and
c0fcd87de0ad08d8344a4142ecae934b93acd8f4 chronologically through ordinary
merges. All three third-parent trees are empty. Current main already has the
engine clock boundary and later configuration, anchored filesystem and recovery
changes. Retain those implementations while integrating every recovered effect
and assertion. Do not restore format-2 Estate assets over current Engram config.

Scope: clock.rs; the stash-touched CLI, contract, install/recovery modules and
tests and their current split destinations; workflow and generated projections
only when owning behavior requires change; this journal and generated index.
The retired active-change plan remains unchanged. No parallel legacy path.

Read recovered clock implementation and journal, current clock implementation,
source install/recovery/contract and test changes, current admission/open
boundaries and existing clock test locations. Resolve each remaining conflict
against its current owner before removing an incoming obsolete inline path.
Preserve fallible engine-owned observations, immutable recovery anchor, explicit
rollback tolerance, fail-before-write admission, read-only failed diagnostics,
nonzero CLI verification exit, and portable real-process reopen coverage.

First oracles: rollback/unavailable apply/open/replay and interrupted recovery
reject without changing owned bytes. Then owning engine installation tests and
contract deployment tests; existing parser integration's full contract and
installed lifecycle run may be reused only if the application tree is identical.
Run workflow, documentation and formatting checks plus required candidate and
queue. Stop on missing behavior, relaxed tests, changed time authority or red CI.
No new time synchronization, trusted-clock, performance or alpha claim. No new
research or trace is required; existing bounded clock traces remain. Record actual
resolutions, tests and limits below; other recovered history remains unfinished.
