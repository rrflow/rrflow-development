# Controller testing bundle: refresh the engine dependency

**Status:** active reconciliation evidence; dependency and protected landing pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/controller-testing-bundle-main-reconciliation`
**Owner:** J-03/J-05 engineering preparation
**Pull request:** [212](https://github.com/rrflow/rrflow-development/pull/212), dependent on [206](https://github.com/rrflow/rrflow-development/pull/206)

## Scope and procedure

The controller packaging session published the bounded refresh brief before merging dependency `f278fa6f4233f32b069eff4e5fa00bf50ba12aac` into clean packaging head `811f892a856ad7b3af8833700c263fa7b3aca522`. The ordinary no-commit merge completed without conflicts. The incoming dependency contains landed main `6fe8b7590d913beda9b009551e5d2d2644e406d9` and its reconciled canonical contract projections. No runtime feature, policy, routing choice or artifact was changed.

Read the applicable instructions and packaging journal; reviewed the merge path inventory, actual workflow diff, package-only diff against the dependency, archive test command/interface, receipt manifest and existing current deployment boundaries. The original implementation audit remains in the [tooling journal](controller-testing-bundle-tooling.md). Main's new native tests and diagnostics remain registered. The packaging-only workflow addition still invokes archive rejection tests, and the optional proof helper still resolves a relative receipt binary path. No deletion, move or alternate semantic owner was introduced.

The existing ZIP and its 14 public receipts are immutable evidence of engine `54fb845d` and assembly `98d0de93`. Source reconciliation does not update either identity or qualify a new executable. The [artifact journal](controller-testing-bundle-artifact.md) retains the actual unsupported model-effort failure and its conservative recovery. No provider call, retry, learning record or model promotion was performed in this refresh.

## Validation and limits

- `python scripts/test_controller_trial_package.py --archive <existing controller ZIP> --sha256 46b85f824d712cd8e8cceda4a6b0cd01bb1585f3f25a1f8c64ef8b7657f05b6f -v`: all 18 passed, including six checks against the actual artifact/corrupted copies.
- `python scripts/ci/check_generated_surfaces.py --check`: passed, 63 HTTP operations and seven signal projections; OpenAPI `52b7e85144b8b7b4e75291033cf151a0674c97f87fff2054d66f23c97a4bfb6d`.
- `python scripts/ci/check_workflow.py`: passed.
- All 14 captured receipt byte counts and SHA256 hashes matched their manifest.

Command logs are retained in the operator's October 30 `controller-bundle-integration` evidence directory. This journal and its index are the only newly authored repository content in the refresh. Navigation/inventory and repository policy checks are regenerated and checked before publication. No external research or new trace/resource measurement is needed for the dependency-only operation. No package tests failed. Stop on changed historical receipt bytes, semantic merge conflicts, failed checks or weakened native coverage.

No new binary build, live endpoint trial, native macOS qualification, full candidate CI or protected merge was performed. PR206's protected APFS image-creation failure remains unresolved at C-07 / POAM-035. Keep both this dependent package and its qualification claims draft until the prerequisite evidence exists. No roadmap or POA&M completion status changes.
