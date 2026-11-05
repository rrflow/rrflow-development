# E-04c integration with main after the object-publication repair

**Status:** active immutable integration evidence; E-04, POAM-028, POAM-035, POAM-045 and OBJ-13 remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04c-main-after-object-publication`
**Owner:** PR #190 integration and conflict-resolution evidence.

## Evidence record

```text
gate/package and alpha outcome: E-04c / OBJ-13 / POAM-028 prerequisite; no gate closure.
starting revisions and pull request: PR #190 head a1af96b3150bbbfeaeb14db2fafc5ebf57e426ba, integrating public main 5b4039fe. The owner approved landing #190, #191, #211 and #217 in order on 2026-10-31.
change brief: recorded on #190 before the push. Main landed #228, a non-replacing object publication that verifies an existing winner, for the Windows race that the previous integration repaired with #219's draft locks. Take main's repair, leave the #219 integration out of this branch, keep every E-04c contract field, implementation and test, and regenerate the projections. Stop if the racing build fails on main's repair, an E-04c field or test is lost, or a generated surface disagrees with its generator.
files reviewed: #190, #219 and #228 pull request descriptions; the previous E-04c integration journals; POAM-035 on main; the a1af96b3 object-publication integration diff (13 paths); every conflicted path; scripts/ci/check_generated_surfaces.py and the five SDK generators.
conflict analysis and traceability: 21 conflicting paths: two from the #219 integration (object.rs and the Gate C journal index), the authored OpenAPI digest in rrd-contract lib.rs, six authored reference digest citations, the generated signal reference, and eleven generated SDK files. Each conflict hunk took main's side; E-04c's non-conflicting contract fields base_generation and encoded_versions stay in lib.rs. The generated files were rebuilt from the merged contract export, and the authored citations now name its digest, 43205ef142fdbe8a7a5f2ca970c05bdbf789a6aeddaba4742a94be9e37067ca2. The two Gate G journals that cite main's previous digest are history and stay unchanged.
removed from this branch: the #219 integration from a1af96b3 — the object.rs and filesystem.rs repair, tests object_publication_race.rs and object_publication_trace.rs, journals c-07-concurrent-object-publication.md and c-07-object-publication-guard.md, POAM-047 (concurrent identical immutable-object publication), the object contract reference addition and e-04c-object-publication-integration.md. Those paths now equal main. None of them reached main; #219 keeps them. Main tracks the finding under POAM-035 with c-07-object-publication-observation.md and c-07-object-publication-repair.md. The E-04c implementation and tests are unchanged.
correction during integration: the first resolution took main's whole file for the eight contract and reference paths, which dropped E-04c's two contract fields; the merged digest then equalled main's, which exposed it. Those paths were re-merged and only their conflict hunks resolved.
research/trace/resource decision: none beyond the #228 and E-04c records; no upstream source adapted.
first oracle: cargo test --offline --locked -p rrd-engine --lib incremental_generations: 7 passed. The racing-build test that failed on Windows before #219, engine::tests::vector_index::incremental_generations::racing_builds_publish_only_generations_equal_to_a_full_build, then passed ten consecutive --exact runs on main's repair.
acceptance commands: the Go, Python, Java, .NET and TypeScript generators and their --check modes passed (TypeScript under fnm Node 24, Python through uv offline). python scripts/ci/check_generated_surfaces.py --write then --check: 71 HTTP operations, OpenAPI 43205ef142fdbe8a7a5f2ca970c05bdbf789a6aeddaba4742a94be9e37067ca2. cargo test --offline --locked -p rrd-vector -p rrd-contract: 215 passed. cargo test --offline --locked -p rrd-store: passed. cargo test --offline --locked -p rrd-engine --lib: 297 passed, 4 ignored. cargo clippy --offline --locked -p rrd-vector -p rrd-contract -p rrd-store -p rrd-engine --all-targets -- -D warnings: passed. cargo fmt --all -- --check: passed. Navigation, execution inventory, documentation, workflow and version checks are run after this record.
checks not run: candidate CI and protected merge-group CI for this head; native macOS, Windows crash recovery, 100,000-record and soak qualification.
remaining errors and owner: the earlier macOS candidate failure (command output did not close) applies to an older head; this head's candidate run will show whether it persists. POAM-028, POAM-035 and POAM-045 remain open. #219 must take a free POA&M number if it lands.
roadmap/POAM status change: none.
pull request and CI: https://github.com/rrflow/rrflow-development/pull/190; candidate CI pending for the integrated head.
```

## Links

- [Previous E-04c integration](e-04c-current-main-integration.md)
- [Original E-04c journal](e-04c-incremental-turboquant-generations.md)
- [Object publication repair on main](../gate-c/c-07-object-publication-repair.md)
