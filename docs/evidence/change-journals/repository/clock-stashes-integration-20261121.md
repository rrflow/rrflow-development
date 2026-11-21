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

## Resolution evidence

PR464 merges the three sources as 6e0792d1, 17f7a464 and 3ba8685a. Main's
merged Connectome pin was refreshed first. The source merge leaves no
application delta: current main already contains these clock behaviors and
later fixes. All original stash/index/untracked parent objects are retained
through the ordinary merges; this is not a newly implemented clock service.

| Recovered source | Current owner retained |
|---|---|
| clock.rs observations, assessments and traces | Same file; retain later malformed-assessment errors instead of restoring expect panics |
| Inline installation clock admission and recovery | installation/admission.rs, apply.rs, open.rs, recovery.rs, records.rs and storage.rs: immutable intent anchor, pre-open clock check, replay rejection and checkpoint anchor cross-check |
| Clock configuration and v2 asset | deployment/profile.rs and configuration.rs plus sole default-engram-configuration-v3.toml/default-profile-v3.json; bounded clock policy remains part of configuration and digest |
| CLI dispatch and failed verification | Current installed module split, main and verification rendering retain engine-owned time and failed-report exit status |
| Old inline lifecycle/http tests | Current tests/installed_lifecycle/ and tests/http_process/ targets; no restored duplicate rs test file |
| Rollback and recovery assertions | engine/tests/installation/apply_and_open.rs, configuration_and_adoption.rs and installation_recovery.rs; no-write unavailable/rollback cases, explicit tolerance and original recovery anchor retained |
| SDK configuration fixtures and generated identities | Current v3 fixtures retain clock policy and current digests; old generated catalogue/digest values not restored |
| Native lifecycle workflow placement | Current reusable jobs and workflow checker retain Linux, Windows and macOS lifecycle coverage; old job layout not restored |

Incoming obsolete v2 configuration, v1 profile and inline test paths were
removed from the merge index only after mapping their current destinations.
Retired active-change history was not edited. Kept current README, roadmap,
POAM, generated inventory and later clock journal evidence. The three stash
generations differ in the plan corrections and strengthened open/anchor checks;
current open/admission retain the final checks. Full original/current clock.rs,
recovered clock journal and incoming install/recovery/contract/test hunks were
read, together with current admission and open preflight, SDK fixture fields
and workflow lifecycle call sites. This is not a full review of unrelated code.

Local checks passed: `cargo test --locked -p rrd-engine --lib clock` (5),
`cargo test --locked -p rrd-engine --lib installation` (84, 1 existing ignored),
and `cargo test --locked -p rrd-contract --test deployment_configuration_contract`
(10). Workflow and documentation checks and `git diff --check` passed. Full contract (270
combined parser/contract tests) and installed lifecycle (17 tests) passed in
PR463 on the identical engine/application tree earlier this session; only
merge ancestry, evidence and optional child pin differ. Do not infer trusted
UTC, runtime-wide time policy or native release qualification from these tests.
