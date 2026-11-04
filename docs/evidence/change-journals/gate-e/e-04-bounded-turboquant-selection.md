# E-04 bounded TurboQuant candidate selection

**Status:** active draft; E-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journals/gate-e/e-04-bounded-turboquant-selection`
**Owner:** current package driver

## Scope and decision

OBJ-13 and E-04 need useful, bounded vector retrieval under the one RRFlow
authority. The starting public main was `e012a51d4c01975ddab70538df14333a70ba7aed`.
[Draft PR #273](https://github.com/rrflow/rrflow-development/pull/273) records
the initial brief and implementation boundary. The baseline tree is
`95ebae426247adbe05a473e26ef5d6b17a496d6d`; the remote draft initially
contained only bootstrap commit `6d0f92c91d937fe26327961aca669cc4d4c893fd`.

Before this change TurboQuant scored every eligible candidate, retained every
hit, sorted them all, then truncated to the caller's rerank width. The new
private heap retains at most that many hits and applies the same total ordering
of score, reference and source cursor. Selection still examines the matching
vectors and builds a temporal visibility map over the artifact. Therefore the
change does not prove bounded total query memory, sublinear candidate search,
better latency, a working project-indexing journey, or Qdrant equivalence.
No stored artifact, public contract, routing mode, filter, score, or authority
changes. Existing independent exact scoring and exact reranking remain intact.

Read for the package: `README.md` identity and entry points, E-04 and the
vector-search and quantization-lifecycle references, the change-authoring
procedure, `rrd-vector`'s TurboQuant selection and tests, and the `SearchHit`
ordering and request validators. The complete changed Rust file
`crates/compute/rrd-vector/src/turbo_segment.rs` and the complete
`docs/roadmap/rrflow-1.0-execution/change-authoring.md` procedure were reread
after implementation; other supporting references were consulted for the
relevant sections, not represented as a complete codebase review. Qdrant's
[filtered planner](https://qdrant.tech/documentation/search-patterns/vector-search-filtering/)
and [quantization/rescoring](https://qdrant.tech/documentation/manage-data/quantization/)
are research inputs to the candidate-width and exact-rerank distinction; no
upstream code is copied. Request and generation evidence stay as before; no
new telemetry or production latency claim is added.

## Local observations

On Windows x86-64 with isolated Cargo target
`D:/localdev/devlabs/rrflow/target/turboquant-bounded-build`:

- `cargo test --offline --locked -p rrd-vector bounded_ -- --nocapture` passed
  the two new tests. One compares heap selection with an independent full
  sort of synthetic hits, including ties and signed zero. The other compares
  narrow artifact results with full-width results from the same selector; it
  is not an independent scoring oracle. It covers filters, temporal versions
  and retirement, seven estimator/bit-width combinations and owned readback.
  The heap length is checked after every synthetic insert.
- After extending the artifact differential to reopened memory-mapped bytes,
  `cargo test --offline --locked -p rrd-vector --lib bounded_ -- --nocapture`
  passed both tests again. The complete package suite listed below preceded
  this test-only extension.
- `cargo test --offline --locked -p rrd-vector` passed the complete package
  suite, including its existing recall and deterministic lifecycle checks.
- `cargo clippy --offline --locked -p rrd-vector --all-targets -- -D warnings`
  passed after the test-only extension.
- `cargo test --offline --locked -p rrd-engine --lib turboquant_overlay` passed
  four tests on the candidate, covering writes, reopen, budget and unrelated
  vector growth.
- `cargo test --offline --locked -p rrflow-cli --test installed_quantization
  installed_engine_builds_activates_and_searches_turboquant` was interrupted
  during dependency compilation. It did not reach or pass the installed test.

The local fixed-input comparison built baseline and candidate release binaries
with separately generated offline lab lockfiles (not the workspace lockfile).
On a Windows x86-64 host, three alternating-order processes per build each
queried 20,000 synthetic 64-dimensional vectors 12 times after one warmup,
using a scalar 4-bit MSE codec and candidate width 32. All artifact and query
result digests matched. Across the 36 timed queries per build the medians were
20.85175 ms baseline and 16.50245 ms candidate; per-process medians were
20.590, 20.888 and 21.031 ms baseline, and 16.779, 16.579 and 16.302 ms
candidate. This is a local component diagnostic on a shared, uncontrolled
machine, not a production latency or recall comparison; it measures neither
peak memory nor the joined engine flow. The complete output is local ignored
`target/turboquant-selection-lab/result.json` (SHA-256
`d93f6ec511fb16f4dd0c6b03a5e657ed96a6ce5c35e96a7c4cf5693779c0b967`);
the local harness source has SHA-256
`79b6de13a14f4a506d838467c2e1aa9a9ebec7792d3ade601b82d0c6bf7ec6c6`.
Baseline and candidate executable SHA-256s are
`a8711a7feb8ea035420e00a9ca22238174b26d6c8f6478340f00d8d9de3f74ce` and
`8d4ada03adcbd020b2e77804d0b105a69f6d295d9090ea1a19e8c75a0d45cc30`;
the common artifact digest is
`857812f90d12aff66baf75b53bfbb319ed4e13792afedeffd32c7da926797b69`.
The first benchmark invocation used incorrect executable names and failed
before starting a run; correcting them produced the results above. This local
lab is not the independently owned cross-engine benchmark package #253.

Three concurrent fresh builds exhausted the host's available memory and were
interrupted before completion. Later, after the engine overlay passed with two
Cargo workers, the sequential installed build was interrupted when the host
became unresponsive; an OS query returned 636 KiB available from 66,516,344
KiB visible. A later retry with one Cargo worker was also interrupted during
compilation after available memory fell from about 9 GiB to 915,900 KiB. Other
compiler/linker processes were present; the exact memory attribution was not
measured. Neither installed attempt reached the test. None of the interrupted
builds is passing evidence. No other user processes were intentionally stopped.

The last installed TurboQuant test on the earlier candidate
`864c03e387ee6044516a19ed976762fa184b88a0` passed with three manually supplied
vectors and explicit build/activation. It does not prove automatic selection,
project indexing, prompt reasoning, or this new candidate. Before/after source
and benchmark results must be evaluated on the same dataset; a single host
result is not a system or release performance claim.

## Repository checks and checklist

- `cargo fmt --all -- --check` and `git diff --check` passed for tracked
  edits. The first staged check then caught CRLF whitespace in this new
  journal; it was converted to LF and `git diff --cached --check` passed.
- `python -X utf8 scripts/knowledge/render_navigation.py` regenerated the Gate
  E index; `python -X utf8 scripts/knowledge/render_navigation.py --check`
  passed. `python -X utf8 scripts/ci/check_documentation.py` passed.
- The implementation and its complete diff were reread, including ordering,
  request validation, artifact readback, and existing temporal selection. The
  sole runtime path changed is `turbo_segment.rs`; the created evidence is this
  journal and the modified navigation index is generated. No path is moved or
  deleted. The installed proof remains pending.
- Research was necessary to identify the difference between candidate
  selection and exact reranking. No upstream source is copied; the existing
  RRFlow comparator and contract remain authoritative. A new tracing span was
  not added because this change alters only the internal selection algorithm,
  and the component comparison does not measure installed trace behavior.
- The first characterization was source inspection, not a failing pre-change
  test. The independent sort oracle checks the new selection invariant. The
  local before/after diagnostic verifies the same result digests on a limited
  synthetic input; none of this is a native capacity, soak, or user-journey
  qualification.

- `python -X utf8 scripts/ci/build_execution_inventory.py` and its `--check`
  passed with 2,359 current, generated, and planned records after staging.

## Remaining work

The PR brief binds scope, input identity, failure oracle, research and stop
conditions. No implementation is deleted or relocated. This journal and its
generated navigation link are required. The engine overlay checks passed.
Repeat the installed check when host memory is available, reread the final
evidence and complete diff, and reconcile any main overlaps before ready. A fetch during this work reported the baseline as
the current public main; check again before landing. CI, native
capacity/soak and merge-group evidence are not yet present. The joined
project-to-context/reopen driver, adaptive policy selection, LFG/SWAG dispatch,
full Arrow execution and resource qualification remain open. No roadmap,
POA&M, C-07, E-04, or alpha status changes are claimed.
