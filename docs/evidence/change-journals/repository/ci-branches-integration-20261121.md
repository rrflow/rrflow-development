# Recovered CI and native evidence branches

**Status:** active integration evidence; native release qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-branches-integration-20261121`
**Owner:** repository candidate and native-check history integration

## Before-edit brief

Baseline 5d391dba612916e9c8f9f30411487b4c4ca87114. POAM-011 owns
source integration; Gate C owns native failure evidence; Gate H SDK conformance
and Gate J candidate checks remain independent acceptance requirements.
Integrate recovered PR151 9aea9987b21e4437c5f6d96a5c5832da9e00939f,
PR189 c15ed0b923dfb56ff947be0e14c04b10d8c650a1, PR192
a754de83acd20912a4d4dcc701c6fecb6e331e34, PR220
5697b208b5269f76817d437cb219f79b2434c500, PR300
d26a1322dd5d87ec19b417420a5074c077e09de1 and PR344
81059c9df60c64567414d14d0ed70f6dd10dc1a7 through ordinary merges.

Scope: native capacity/teardown and tests, private SDK fixture/standalone
conformance, merge-conflict tooling and POAM checks, workflow wiring and
recovered evidence. Read changed source and current owning implementations.
Preserve later bounded helper-aware APFS retry, original failure propagation,
owned-path checks, red-candidate alarm, queue CI, and current native matrix.
Do not reinstate retired cache experiments, prior workflow layout or missing
monolith files; map useful behavior to current owners before resolving.

First oracles: native capacity and lifecycle Python suites, merge-conflict
policy tests, private manifest/standalone tests. Then workflow/documentation
checks and required candidate plus queue. No real macOS/APFS claim from a
Windows mock test. No new research, runtime authority or profiling required.
Stop on relaxed failure tests, lost owned-resource checks or red CI. Record
exact merge decisions, commands and limitations below. Other branch recovery
continues separately; no whole-repository absorption claim.

## Resolution evidence

PR466 merges PR151 at 835de13e, PR189 at aecfd1bb, the empty PR192 opening
at 81b4d1b1, PR220 at 5f801a4a, PR300 at 77374fca and PR344 at e286c1f7.
There is no application-tree delta: current implementations already carry the
useful behaviors. The previously absent PR344 journal is retained as historical
evidence; it does not assert that its old per-case image loop is current.

| Recovered behavior | Canonical resolution |
|---|---|
| Windows characterization | Retained later POAM-035 failures and generated inventory/index; sole unmerged source commit was a main integration |
| SDK private manifests/standalone and cache policy | Current run_sdk_conformance, sdk_private_manifest, standalone driver and workflow steps retained, including platform-specific Python selection and later native/queue policy checks |
| Merge-aware POAM IDs and conflict inspection | Current historical_poam_ids, poam_ids_valid, inspect_merge_conflicts and their tests already contain the source; retained later real-Git regression tests in verify |
| Busy APFS create retry | Current darwin_template owns five bounded attempts, partial-image ownership/detach, timed-out refusal and capture-thread release; did not restore the older three-attempt per-case loop |
| Escaped helper handling | Current CommandOutputNotClosed.wait_closed and retained-failure flag plus test_macos_template_* tests preserve original failure and held image |
| Device-node wait | Deliberately removed by the existing held-output-retry-20261114 replacement journal: no create follows each case's detach after template adoption. Did not reintroduce that retired path or its four tests |

Read incoming workflow/POAM conflict hunks, native retry implementations and
assertions, current template loop and teardown, historical PR344 and its
replacement journals, conflict inspector and POAM ID policy. Current tests
retain stronger no-retry/no-sleep/no-retry-line failure assertions. Source
ancestry is integrated without claiming a new runtime capability or an observed
real macOS retry. Workflow and documentation checks passed. Python suites run
from scripts/ci; an initial invocation from repository root failed module
resolution and is not counted as a test result. The corrected command
`python -m unittest test_native_capacity test_offline_lifecycle
test_merge_conflict_policy test_inspect_merge_conflicts test_sdk_private_manifest
test_rrflowdb_standalone test_cache_warm_policy` passed: 114 ran, 100 passed
and 14 platform-specific skipped in 100.554 seconds. Required candidate and
queue results are still pending. No native qualification/status change.
