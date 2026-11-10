# Candidate CI: repository invariants on every light candidate

**Status:** active CI contract change; no engine or release claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-repository-invariants-20261110`

**Owner:** `docs/operations/ci.md#proportional-lanes`; the candidate-CI gate in `.github/workflows/ci-reusable.yml`

**Starting revision:** main `cc139164`.

**Current behavior.** The proportional lanes landed by #303 run the engine's repository-wide architecture tests only when a change selects the engine lane for `rrd-engine` or the full matrix. #308 landed evaluation inputs through the docs and scripts lanes on 2026-11-09; its merge left main failing `retired_pre_release_identity_is_absent_from_the_active_repository` and `tracked_and_untracked_text_files_have_no_trailing_horizontal_whitespace`, first seen on #249's merge check, and every engine-lane candidate merged with that main inherited both failures. The data correction is #310; this record closes the lane gap.

**Target behavior.** A `repository-invariants` job runs `cargo test -p rrd-engine --locked --test workspace_architecture` on every candidate whose selection is not `full`, with the bounded Linux-heavy compiler profile and its own cache key; the full matrix skips it because `engine-suites` already runs those tests. The gate requires its success whenever the full lane is not selected and a skip otherwise. `scripts/ci/check_workflow.py` names the job in its ownership set, its lane condition, its gate expectation, its heavy-job profile rules and requires the exact test command; `test_ci_lanes_policy.py` gains a regression for the condition and the command.

**Changed paths.** `.github/workflows/ci-reusable.yml` (job, gate `needs`, result variable, `LIGHT_SELECTED`, expectation line), `scripts/ci/check_workflow.py`, `scripts/ci/test_ci_lanes_policy.py`, `docs/operations/ci.md`, this record.

**Acceptance.** `python scripts/ci/check_workflow.py`: CI policy OK, 16 substantive jobs. `python scripts/ci/test_ci_lanes_policy.py` (10 tests), `test_select_ci_lanes.py`, `test_candidate_admission.py`, `test_linux_arm64_policy.py`: OK. The workflow parses; the new job depends on `changes` and the gate depends on it. This change selects the full lane, so its own candidate run exercises the gate with the job skipped; the first light candidate after it exercises the job.

**Research decision:** none beyond the retained failure (#249 merge check and #310). No upstream code.

**Checks not run:** no hosted run of the new job before merge; its cost is one cached `rrd-engine` test build per light candidate and is to be read from the first runs.

**Roadmap/POA&M change:** none.
