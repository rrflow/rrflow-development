# E-04 catalogue-only refusal admission

**Status:** active draft; E-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journals/gate-e/e-04-catalogue-refusal-admission`
**Owner:** OBJ-13 / E-04 package driver

## Scope and baseline

[Draft PR #276](https://github.com/rrflow/rrflow-development/pull/276) holds the
change brief and integration handoff. Starting public main commit
`e012a51d4c01975ddab70538df14333a70ba7aed`, tree
`95ebae426247adbe05a473e26ef5d6b17a496d6d`. The comparison-lab draft #253
recorded the unmerged POAM-055: at 100,000 vectors an unsupported filtered
approximate request could read the full corpus before returning no path. Its
multi-second measurement is not independently reproduced by this package.

The existing named dense `require_approximate` route first reads an
accumulator-authenticated catalogue at the request stamp. When no compatible
catalogue binding exists, ask the existing planner at source cursor zero before
reading vector versions. If it rejects there, propagate its existing validation
or no-path error; if it admits any candidate, conservatively continue through
the unchanged general path. Eligible TurboQuant and other paths keep their
existing execution, budgets, authorization, catalogue checks and exact
reranking. This does not add automatic indexing, change fallback behavior when
an eligible path exists, or bound the size of a supported vector scan.

## Reviewed boundaries and decisions

Reviewed the changed portions of `engine/vector/search.rs`, the stamped
catalogue source and descriptor reconstruction in `runtime/vector_catalog.rs`,
`rrd-vector/src/plan.rs`, the relevant TurboQuant overlay fixture and tests,
the installed quantization driver portions, the OBJ-13/E-04 owners and the
comparison-lab POAM-055 detail. The changed engine source, engine test,
installed test, generated index and this journal were reread, and the complete
staged diff was reviewed. No whole-repository review is claimed.

The existing planner owns acceptance and error wording; no second filter
predicate or persisted state is introduced. Using source cursor zero is
conservative because this check is only used to reject: a planner success
continues to the original path. Catalogue evidence uses the existing scoped,
stamped read; the caller's key budget remains in force. No upstream code is
copied. [Qdrant's filtered-search description](https://qdrant.tech/documentation/search-patterns/vector-search-filtering/)
is context, not a claim of equivalent planning. The public error does not carry
read evidence, so a catalogue-sized key-budget test is the no-extra-key oracle.

## Characterization and local execution

On `clyffy-01`, Linux aarch64, cargo/rustc 1.99.1, isolated directory
`/tmp/rrflow-poam055.8aqErQBq`, the base Git archive SHA-256 was
`0c29484656cfee0e00d66bd567e26aa1b6dba45a08e66ead9db9b8861c8a1393`.
The test-only patch SHA-256 was
`7e5f7231fa1263f9ad74fbdc6934a668793ba10afd3d04309e9d29104bbb6693`;
the engine patch SHA-256 was
`385551edff68659dcf1deac329f585aa2fe2ca80d75ad65b04ee0ba81a85bf90`.
The installed-driver patch SHA-256 was
`e068075bf2d23f3ce423e0cc1275d2eb156a75e563d78f8b068826d820fbafa8`.
The remote directory contains the source and command logs; it is not release
or CI evidence and is not deleted by this package.

- Before the engine patch, `cargo test --offline --locked -p rrd-engine --lib
  unsupported_filter_is_rejected_before_vector_versions_are_read -- --nocapture`
  failed: with catalogue budget 21, the old path attempted key 22 and returned
  `ConfigurationLimit` rather than the no-path error. `red-test.log` retains
  the repeated failure.
- With the engine patch, the same focused command passed for fixtures with zero
  and 64 unrelated vectors; `green-test.log` retains the output.
- `cargo test --offline --locked -p rrd-engine --lib turboquant_overlay --
  --nocapture` passed five tests, including writes, reopen and bounded search.
- `cargo test --offline --locked -p rrd-engine --lib catalogue_reads --
  --nocapture` passed three tests, including a retained HNSW generation.
- `cargo test --offline --locked -p rrflow-cli --test installed_quantization
  installed_engine_builds_activates_and_searches_turboquant -- --nocapture`
  passed on the staged primary executable. It installs and starts the engine,
  activates TurboQuant, searches three manually supplied vectors successfully,
  and checks the unsupported-filter error through the Rust client. This is not
  a 100,000-vector or project-indexing test; `installed-test.log` is retained.
- `cargo test --offline --locked -p rrd-engine --lib -- --test-threads=2`
  passed 339 tests with four ignored; `engine-suite.log` is retained.
- `cargo clippy --offline --locked -p rrd-engine --lib --tests -- -D warnings`
  and `cargo clippy --offline --locked -p rrflow-cli --test
  installed_quantization -- -D warnings` passed; logs are retained.

## Comparison in progress

The existing comparison lab is running an isolated before/after diagnostic on
`clyffy-01`: 20,000 synthetic 384-dimensional vectors, ten queries, one client,
with the same manifest and debug build profile for both binary revisions. The
manifest digest is `dcbc17cffa2794c6323ea5e8aa2a5f87b66113c0171acc71a431fd8c0c5427e2`;
the baseline binary SHA-256 is
`f2c03f961f391b69b8939d816fcc6afc7c08014ca11b2307e43a506cd79c8236` and
candidate `49b0ef06ae0effdb3dabd984eee5521d567f397f89c233c5d90cc0d1c7fff2ec`.
The server's existing lab binary SHA-256 is
`671f9fb11f4721d9de7eca97f4fd858366aa2ff2755448708c7e235e4878771d`;
its source checkout is not a Git checkout, so its exact source revision is not
asserted. Runs and host samples belong to the isolated temporary directory.
The before run is active; no result or speed claim is made yet. The lab reports
refused-case wall time and attempts rather than individual refusal latencies.

## Remaining work

`cargo fmt --all -- --check`, `git diff --cached --check`,
`python -X utf8 scripts/knowledge/render_navigation.py --check`,
`python -X utf8 scripts/ci/check_documentation.py`, and the generated
execution-inventory build and check passed locally. Reconcile #271's
catalogue-writer changes and current main before ready. This PR and #273
both regenerate the Gate E index: if this PR lands first, merge updated main
into #273 and regenerate the index without dropping either journal. Candidate
CI and merge-group CI are not run. The
lab's 100,000-point refusal timing, 200,000-point budget ceiling, LFG/SWAG
routing, actual project indexing and C-07 qualification remain open. This
record does not change any objective, POA&M, roadmap or release status.
