# E-04d integration and projection-boundary review

**Status:** active immutable change-journal evidence; E-04 and POAM-028 remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04d-current-main-integration`
**Owner:** PR #191 integration and QJL boundary evidence.

## Evidence record

```text
gate/package and alpha outcome: E-04d / OBJ-13 / POAM-028; no gate or release status change.
starting revision/tree and pull request: PR #191 f34451ee319156ebb8b53e3866c0473bb3284b15 / ba35c9b1cde65440eaa80189097a6337a457c99d, normally merging prerequisite PR #190 b2f209712cb7c62f019e0c44d7fac3dcdad4d9e6 / 969a6275d798a60838c04d903ebd708cf9019b6b, which contains public main 6fe8b7590d913beda9b009551e5d2d2644e406d9.
change brief: reconcile both source branches and regenerate contract projections; check the public QJL input boundary; retain E-04c and E-04d functionality and tests, no new authority or gate claim. Stop on lost semantics, generation failure or unresolved engine failure.
files and symbols reviewed: original E-04d PR description and journal; original TurboQuant paper abstract; rrd-vector qjl.rs, TurboQuantInnerProductCode::encode and TurboQuantQuery::inner_product; the E-04c incremental test module and conflict; combined rrd-contract result and estimator fields, OpenAPI and SDK generators; vector quantization lifecycle claim sections; rrd-store object publication and rrd-lsm native rename/sync paths after the observed engine failure.
implementation and evidence paths: resolve the incremental test comment and 20 contract/generated/inventory conflicts; regenerate five SDK endpoint/schema generators and seven signal projections. In qjl.rs make project return a Result, reject mismatched/non-finite inputs and non-finite results, and bind projection identity to its seed. In turboquant.rs reject mismatched seeds in the public encode and query paths; adjust direct callers/tests. Correct the reference and code comments so paper guarantees and finite-precision fixture observations are not conflated. Add this journal and regenerate its index.
research: TurboQuant (https://arxiv.org/abs/2504.19874) describes the theoretical unbiased estimator using a QJL residual; this finite seeded implementation is not proved unbiased or cross-platform bit-identical by a local test. No upstream code was copied. Original E-04d journal holds the adaptation and empirical matrix.
trace and resource decisions: retain original E-04d seeded fixtures and E-04c byte-equivalence and restart tests; added no new telemetry or dependencies. Projection dimensions remain bounded at 4096; total native resource qualification remains J-04.
first failure: a test using an 8-dimensional projection seeded 11 with encoder/query seed 12 failed before the repair because encoding returned success. After the repair qjl tests reject mismatched seeds, input lengths, NaN, infinity and non-finite projected results.
local results: cargo test --release --offline --locked -p rrd-vector --lib qjl passed 5 tests on Windows with debug assertions disabled. cargo test --offline --locked -p rrd-vector -p rrd-contract passed. All five SDK generators and their check modes passed; python scripts/ci/check_generated_surfaces.py --check passed with 62 HTTP operations and OpenAPI 3390bdc6904f6b419b38de4ace9dae7729da7b9f593bd651e220fb90ff0537ef. CI workflow, documentation, navigation and inventory generation passed; cargo fmt --all -- --check and cargo clippy --offline --locked -p rrd-vector -p rrd-contract -p rrd-engine --all-targets -- -D warnings passed.
surfaced failure and stop: cargo test --offline --locked -p rrd-engine --lib failed with 253 passed, 1 failed, 3 ignored: racing_builds_publish_only_generations_equal_to_a_full_build returned object store access denied on Windows. Synchronized independent tests reproduced that shared object-publication race; PR #219 and POAM-047 own its remediation. PR #190 candidate CI also failed repeated macOS APFS output capture. Neither blocker is waived, and this PR remains draft.
checks not run: exact-head candidate or merge-group CI, macOS/Linux qualification of the new QJL boundary, full native performance and cross-platform bitwise tests. Local fixtures do not establish theoretical guarantees, C-07, or alpha readiness.
remaining errors and owner: #219 / POAM-047 object concurrency and Windows crash durability; #215 macOS native capture diagnosis; POAM-028 and J-04 remain open.
review: exact conflict hunks and public projection APIs reviewed; generated outputs compared to the owning generators. No implementation behavior was deleted or moved.
change checklist: brief and expanded boundary recorded on #191 before edits; conflict intent reconciled; before/after failure and focused/package tests recorded; linked journal and generated navigation required; no merge claim.
pull request and CI: https://github.com/rrflow/rrflow-development/pull/191; draft and dependent on #190 and #219; no candidate or protected merge evidence.
```

## Links

- [E-04d original implementation journal](e-04d-turboquant-inner-product.md)
- [Object-publication repair](https://github.com/rrflow/rrflow-development/pull/219)
- [TurboQuant original paper](https://arxiv.org/abs/2504.19874)
