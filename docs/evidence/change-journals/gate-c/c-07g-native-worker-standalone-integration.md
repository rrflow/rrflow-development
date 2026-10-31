# C-07g: integrate native worker and standalone candidates

**Status:** active draft integration; prerequisite merges and qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-native-worker-standalone-integration`
**Owner:** OBJ-02 / C-07g-h / POAM-035 / [PR195](https://github.com/rrflow/rrflow-development/pull/195)

## Brief, baseline and traceability

Starting PR195 head `8968f2d45165456f364372225675afdce9296519`, tree
`4b104e77b5d0a8881eb677ec0e85041656cc5ae1`; ordinarily merge the exact
[PR194](https://github.com/rrflow/rrflow-development/pull/194) head
`a385b484529c8eefd6c957342e90d0a6f7c2441b`, which includes
[PR177](https://github.com/rrflow/rrflow-development/pull/177) head
`1ab64897bcee1440159d19846e1bf0a6eb3d02b2`. The PR195 description recorded
this scope and the dependency order before the merge. PR177 and PR194 must
land with their own candidate and protected merge-group checks before PR195
can be ready. This integration itself is not a new engine capability.

The merge preview and actual merge found content conflicts in
`.github/workflows/ci-reusable.yml` and `docs/poam/rrflow-1.0-alpha/poam-035.md`.
The resolution retains both independent policy commands and both factual
POAM observations. The native worker Windows Job step, standalone installed
process steps, SDK manifest tests, their policy guards, and the preceding
immutable journals remain in their canonical locations. The workflow and
policy diffs were inspected against both parents; of 22 merged paths, only
the workflow, CI operations record, POAM-035 and workflow checker had staged
content differing from both parents. The first conflict was expected; no
acceptance threshold or candidate gate was changed. The prior PR194 and PR195
journals own the implementation mapping and characterization; no source
behavior was removed or moved in this integration.

Reviewed the change-authoring procedure and PR195 brief in full, both parents'
relevant workflow, CI policy and POAM merge diffs, and the full prior
standalone and Windows worker integration journals. No external source was
adapted. No new tracing, resource cap, performance claim, or debug mechanism
was introduced. The real Job tests retain their existing test-only 2 GiB
process/aggregate cap and 4 GiB declared host with 24 host vCPUs.

## Focused Windows results

On the integrated working tree, `python scripts/ci/test_native_sample_policy.py`
passed 4 tests; `python scripts/ci/test_rrflowdb_standalone.py` passed 7;
`python scripts/ci/test_candidate_admission.py` passed 11; and
`python scripts/ci/test_sdk_private_manifest.py` passed 4 with one POSIX-only
case skipped. `python scripts/ci/check_workflow.py` passed.

With `CARGO_TARGET_DIR=D:/localdev/devlabs/rrflow/target/c07-native-worker-195-build`,
`CARGO_BUILD_JOBS=2`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_PROFILE_TEST_DEBUG=0`, `python scripts/ci/test_native_sample_worker.py
--profile debug --offline` passed both real Windows Job membership/cancel/reuse
and abrupt-parent tests. The test harness executable digest was
`da816630b61bca4cd093f2d89d6bf37038c505340b82acc74b1bc140c0b093cd`.
This was a reused debug build, not an exact-head hosted release run or a
frozen capacity-profile result. The prior standalone real-process evidence
remains in [PR194's journal](../gate-h/h-04-standalone-sdk-integration.md);
that process was not rerun for this merge.

## Stops and checklist

The helper image identity and bounded native termination/reap failure remain
unproven. Linux and macOS native behavior, the 100,000-record profile, native
capacity matrix and 24-hour soak are not established. The cause of the earlier
sample-gap failure remains unknown. The macOS `hdiutil` failure remains
recorded; an unrelated pass does not explain it. Candidate and merge-group CI
for the integrated PR195 have not run, and no alpha, roadmap or POAM status
changed. The package remains draft, with PR195's driver responsible for the
remaining stops. Checklist: pre-edit PR brief and explicit dependency; merge
preview; conflicting evidence and both CI protections retained; real Windows
Job and negative policy tests; omissions and failures recorded; new linked
journal and generated navigation/inventory before commit.

`python scripts/ci/test_execution_inventory.py` passed 10 tests;
`python scripts/ci/check_documentation.py` passed (750 document statuses and
748 classified coordinates); `python scripts/knowledge/render_navigation.py
--check` passed for 18 indexes; `cargo fmt --all -- --check` passed;
`python scripts/ci/build_execution_inventory.py` and `--check` passed for
1913 records; and `git diff --cached --check` passed. The inventory is an
ignored local projection, not a committed status authority.
