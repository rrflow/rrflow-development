# Proportional candidate CI lanes

**Status:** active change-journal evidence; a repository CI contract change, not alpha or release evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-proportional-lanes-20261109`
**Owner:** [CI and runner operations](../../../operations/ci.md#proportional-lanes); repository engineering procedure in `AGENTS.md`

## Owner direction

On 2026-11-09 the repository owner said the pre-production repository could not
spend 180 minutes of the full native matrix on a change of two to six files,
that they did not want the gate bypassed or changes merged blind, and asked for
a real path that merges small changes in minutes while keeping protection where
it matters. This reverses the 2026-11-01 decision to run the complete matrix
on every candidate; the owner confirmed the reversal before this package.

## Change

Starting revision: `development/main` at `a9f1b0fb`.

- `scripts/ci/select_ci_lanes.py` (new) maps changed paths to lanes: `docs`,
  `scripts`, `engine` (changed crates tested, workspace dependents compiled),
  `sdk`, and `full` for storage, installer, native harness, workflow, root
  manifest, toolchain and unrecognized paths. It fails closed. Dependents come
  from `cargo metadata`. `scripts/ci/test_select_ci_lanes.py` (new) holds 13
  table-driven regressions.
- `.github/workflows/ci-reusable.yml`: a first `changes` job runs the
  classifier against the pull-request or merge-group base (`scope: full`,
  manual dispatch or a missing base selects everything); three light jobs
  `docs-policy`, `scripts-policy` and `engine-fast` serve the light lanes;
  every existing partition now depends on `changes` and declares exactly its
  lane condition; `ci-gate` requires success for a selected lane and a skip for
  an unselected one, and fails on any other result.
- `.github/workflows/ci-nightly.yml` (new) runs the complete matrix against
  `main` daily and on dispatch, so a dated full result for main always exists.
- `scripts/ci/check_workflow.py` encodes the contract: the lane constants, a
  `verify_lane_selection` policy, a `verify_nightly_caller` policy, lane-aware
  gate rules, the storage-preflight and ARM64 rules adapted to the lane
  condition, and the documentation and J-04 checks required once in `verify`
  and once in their light lane. `scripts/ci/test_ci_lanes_policy.py` (new)
  holds 9 regressions; `test_candidate_admission.py` and
  `test_linux_arm64_policy.py` are updated for the new gate wording.
- `docs/operations/ci.md` describes the lanes, the nightly run and the gate.

Not changed: the thin caller `ci.yml` and its admission envelope, the main
ruleset, the merge queue, any partition's commands, budgets or assertions, and
the required check name `pipeline / ci-gate`.

## Evidence

Local, Windows x86_64, Python 3.11:

- `python scripts/ci/check_workflow.py`: CI policy OK, 7 workflows, 15
  substantive jobs.
- Policy regressions, each OK: `test_candidate_admission.py` (21),
  `test_linux_arm64_policy.py`, `test_native_sample_policy.py`,
  `test_sdk_attunement_policy.py`, `test_rrflowdb_standalone.py`,
  `test_cache_warm_policy.py`, `test_inspect_merge_conflicts.py`,
  `test_inspect_candidate_runs.py`, `test_select_ci_lanes.py` (13),
  `test_ci_lanes_policy.py` (9), `test_execution_inventory.py`.
- Every command the `scripts-policy` lane runs passes locally; the
  `docs-policy` lane's documentation, navigation and inventory checks pass.
- Classifier against live `cargo metadata`: a storage change selects `full`;
  a `rrd-vector` change selects `engine` with `-p rrd-vector` tested and seven
  dependents compiled; a workflow change selects `full`.
- This package touches `.github/workflows` and `scripts/ci`, so its own
  candidate run is the complete matrix. That run is the acceptance.

Not run here: the lanes on a live GitHub event; measured lane durations. The
first days of pull requests after this lands will supply both. If a lane's
wall-clock does not match its table entry in `ci.md`, correct the table from
the measured runs rather than widening the lane.
