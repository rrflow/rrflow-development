# C-07: prepare CI preflight with the assessment candidate

**Status:** active local dependency-integration evidence; candidate and merge-group CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-catalogue-ci-preflight-integration-20261107`
**Owner:** OBJ-02, C-07/J-04, [PR #293](https://github.com/rrflow/rrflow-development/pull/293)

## Change brief and boundary

PR #293 head `b49ae70f4dc82fc355ac813a99f487062011ad9e` and public main `b9aaa9abce3c25108a53cc43c70262947dda0f62` were checked before edits. PR #292 exact head `40870e5c5f8f79dfebc7126bf721458ccf789634` had passed candidate CI and entered the protected merge queue, but had not merged. The PR #293 brief was amended first. Its preflight/workflow/policy/documentation paths did not overlap the 53 paths changed by #292. An ordinary no-commit merge of that exact head produced the initial combined index tree `934d429c27e959c4739ac44e01398434838ac75e` with no textual conflict. This tree is local preparation and does not stand in for the eventual merge-group revision or main.

Read the existing C-07 preflight journal, CI owner and the complete preflight/workflow policy/test diff; compared both parents' changed paths, inspected the golden test's actual fixture reads, and checked live PR/CI state. No original behavior or test was removed or rewritten, and no third-party implementation was adapted. The candidate retains every native job, timeout, and aggregate success requirement. The preflight is a synthetic catalogue/fixture gate, not a capacity or soak test.

## Evidence

- On the combined tree, `python scripts/ci/test_candidate_admission.py` passed 15 tests; `python scripts/ci/check_workflow.py` reported five workflows and 11 substantive jobs. `python scripts/knowledge/render_navigation.py --check` and `python scripts/ci/check_documentation.py` passed.
- With `CARGO_PROFILE_TEST_DEBUG=0` and `CARGO_INCREMENTAL=0`, `cargo test --offline --locked -p rrflow-eval --test storage_contract generated_schemas_and_synthetic_receipts_match_goldens -- --exact` passed one named test after 20.38 seconds compilation and 0.12 seconds execution on this Windows host. This is not a CI time prediction.
- For the negative oracle, a local Python `try/finally` script temporarily replaced only `crates/evaluation/rrflow-eval/tests/fixtures/storage/manifest.json` with the version from main `b9aaa9abce3c25108a53cc43c70262947dda0f62`, ran the identical test, and restored the exact saved bytes. The named test failed with exit 101 as intended; the restored fixture SHA-256 was `b67be044bca1bf558624794e1d7dcd8c45dd9110b4f0b08792fe0f63f0204571`. This checks the observed stale-fixture failure mode locally, not GitHub's scheduler.

## Checklist and stop

[done] Brief before mutation; baseline and overlapping paths checked; current candidate test and stale-fixture negative oracle run; original fixture restored; no native gates disabled. [open] Regenerate navigation, review staged diff and exact combined tree, reconcile the actual merged main after #292 lands, run candidate and merge-group CI and inspect scheduling. Do not push or mark this draft ready against an unmerged dependency. If #292 changes or fails, reassess the combined tree. No alpha, C-07 capacity or soak status changed.

## Policy bypass repair

After combined-tree review, the PR brief was amended before changing `scripts/ci/check_workflow.py` and `scripts/ci/test_candidate_admission.py`. The first new mutation pair appended a false condition at test-step indentation and appended `|| true` to the final cargo command. The existing checker accepted both; the 17-test suite failed those two new cases. The checker now forbids conditions in the preflight job/steps and matches the final command to its line ending.

A second mutation pair placed a false condition after the aggregate step's run block and appended `|| true` to its preflight success check. The previous checker again accepted both; the 19-test suite failed exactly those two cases. The aggregate policy now requires one unconditional reduction step and an exact run body containing only the declared success checks. The existing job-level `always()` remains required. The same 19-test suite and `python scripts/ci/check_workflow.py` then passed. These are deliberate workflow-policy mutations, not evidence that the checked-in workflow was actually bypassed or that GitHub scheduling has been tested. The workflow itself and its native workloads were not changed by this repair.
