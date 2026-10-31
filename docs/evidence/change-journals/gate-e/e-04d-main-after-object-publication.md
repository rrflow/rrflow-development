# E-04d integration with the updated E-04c and main

**Status:** active immutable integration evidence; E-04, POAM-028, POAM-035, POAM-045 and OBJ-13 remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04d-main-after-object-publication`
**Owner:** PR #191 integration and conflict-resolution evidence.

## Evidence record

```text
gate/package and alpha outcome: E-04d / OBJ-13 / POAM-028 prerequisite; no gate closure.
starting revisions and pull request: PR #191 head 72c8196a050c3a916eea38fb98dc1603b64a4bf3, integrating #190 head 70abe59d000910c5dc8cc62488b1793072edf5a3, which merges public main 5b4039fe. The owner approved landing #190, #191, #211 and #217 in order on 2026-10-31.
change brief: recorded on #191 before the push. Bring E-04d onto #190's integration, which takes main's #228 object-publication repair and leaves out the superseded #219 integration. Keep E-04d's estimator, QJL module, projection checks, contract additions and tests. Stop if the object-publication race returns, an E-04d field or test is lost, or a generated surface disagrees with its generator.
files reviewed: #191 and #190 pull request descriptions; the E-04c integration journal at 70abe59d; every conflicted path; the E-04d contract diff against main.
conflict analysis and traceability: 20 conflicting paths, all shared projections: the authored OpenAPI digest in rrd-contract lib.rs, six authored reference digest citations, the generated Gate E journal index, the generated signal reference and eleven generated SDK files. Only conflict hunks were resolved, each to #190's side; E-04d's non-conflicting contract additions stay. The projections were rebuilt from the merged contract export and the authored citations name its digest, 8bc8cbdba9d90ab1239ea91624e639886971e58977c0f23a31007d1b2878a9c5. The E-04d implementation and tests are unchanged.
research/trace/resource decision: none beyond the E-04d journal and the TurboQuant paper it cites; no upstream source adapted.
first oracle: cargo test --offline --locked -p rrd-engine --lib turbo: 5 passed. The two concurrent quantization-build tests, incremental_generations::racing_builds_publish_only_generations_equal_to_a_full_build and quantization_replay::concurrent_builds_with_one_key_publish_one_generation, each passed ten consecutive --exact runs.
acceptance commands: the Go, Python, Java, .NET and TypeScript generators and their --check modes passed (TypeScript under fnm Node 24, Python through uv offline). python scripts/ci/check_generated_surfaces.py --write then --check: 71 HTTP operations, OpenAPI 8bc8cbdba9d90ab1239ea91624e639886971e58977c0f23a31007d1b2878a9c5. cargo test --offline --locked -p rrd-vector -p rrd-contract: 225 passed. cargo test --offline --locked -p rrd-store: passed. cargo test --offline --locked -p rrd-engine --lib: 298 passed, 4 ignored. cargo clippy --offline --locked -p rrd-vector -p rrd-contract -p rrd-store -p rrd-engine --all-targets -- -D warnings: passed. cargo fmt --all -- --check: passed. Navigation, execution inventory, documentation, workflow and version checks are run after this record.
checks not run: candidate CI and protected merge-group CI for this head; native macOS, Windows crash recovery, 100,000-record and soak qualification.
remaining errors and owner: the full engine library failure recorded at 72c8196a (a concurrent object-publication test with Windows access denied) was the race #228 repaired on main; the full engine library now passes, and both concurrent-build tests passed ten runs each. POAM-028, POAM-035 and POAM-045 remain open.
roadmap/POAM status change: none.
pull request and CI: https://github.com/rrflow/rrflow-development/pull/191; it lands after #190, and candidate CI is pending for the integrated head.
```

## Links

- [Previous E-04d integration](e-04d-current-main-integration.md)
- [Original E-04d journal](e-04d-turboquant-inner-product.md)
- [E-04c integration with main after the object-publication repair](e-04c-main-after-object-publication.md)
