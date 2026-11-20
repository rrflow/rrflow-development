# The lab states how RRFlow builds its graph

**Status:** active change-journal evidence; POAM-053, Gate J and OBJ-10 remain open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-explicit-native-hnsw-20261119`

**Owner:** Gate J comparison lab; OBJ-10 (reproducible competitive evidence)

**Starting revision:** main `8e5daf331`.

## Change brief

- **Current behaviour.** `rrflow-lab-systems` builds every RRFlow HNSW graph with two constants in the adapter, `HNSW_MAX_LEVEL = 6` and `HNSW_SEED = 7`. No pack states them, no setting changes them, and a system binding with an `rrflow_hnsw` block is refused as an unknown field. The lab's own saved manifest of 2026-11-06 (`eval/lab/evidence/arm-20k-20261106/manifest.json`) states `rrflow_hnsw` with a highest layer of 12, which today's lab can neither read nor reproduce.
- **Target behaviour.** A system that has RRFlow build an HNSW index states the graph's highest layer, seed and build policy in its binding. The adapter holds no value of its own. The engine's contract bounds them, asked through the same public request the adapter sends. A person can tune the highest layer and the seed on a machine, and a tuned run says so.
- **Scope.** `crates/evaluation/lab/rrflow-lab-systems` (`binding.rs`, `adapters/rrflow.rs`, tests), `crates/evaluation/lab/rrflow-lab` (`settings.rs`, tests), `eval/lab/systems/rrflow-hnsw.json`, the four checked-in manifests, the three generated lab schemas, `eval/lab/README.md`.
- **Unchanged.** The values today's runs use: the published definition states 6, 7 and the processor build, which is what the constants were. Every other system. The engine.
- **Failure oracle.** The new tests do not compile at the starting revision: `RrflowHnswParameters` and `SystemBinding::rrflow_hnsw` do not exist there.
- **Research decision.** None needed; this restores a behaviour the lab had.
- **Trace decision.** Not applicable.
- **Stop conditions.** Stop if the published values would change, or if a range of the lab's own would be needed beside the contract's. Neither happened.

## How it was found

The repository's deleted refs were read by what their tests assert. Two lab tests on the merge of pull request 253 (`f9b4957a`, 2026-11-07) had no counterpart on `main`: `rrflow_hnsw_requires_explicit_native_build_parameters` and `native_hnsw_request_uses_manifest_parameters_and_requested_filter`. `main` descends from the 2026-11-04 line that introduced the constants (`895010223`); the explicit form was on the other line and did not survive the lab's restructuring into crates. An earlier reading by code comparison had recorded the lab's vocabulary and files as present, which they are; the behaviour was not.

## Change

- `RrflowHnswParameters { max_level, seed, build_policy }` and `SystemBinding::rrflow_hnsw`.
- `SystemBinding::validate` refuses the block on anything but an RRFlow HNSW index, requires it when that index runs, and asks the engine's `EnsureVectorIndex::validate` whether the values are allowed. The lab keeps no range of its own.
- The adapter's two constants are removed; `hnsw_index_request` takes the binding's parameters and sends the build policy.
- `SystemTuning` gains `max_level`, and `seed` now also sets an RRFlow graph's seed. A system with no RRFlow graph refuses both.
- The published `rrflow-hnsw` definition and the four checked-in manifests state the block; the three lab schemas are regenerated.

## Constants

None added. Two removed (`HNSW_MAX_LEVEL`, `HNSW_SEED`); their values are now data in `eval/lab/systems/rrflow-hnsw.json`, with an override.

## Evidence

Windows 11, one run each, exit codes read directly.

| Command | Result |
|---|---|
| `cargo test -p rrflow-lab -p rrflow-lab-systems -p rrflow-lab-benchmarks -p rrflow-lab-definitions -p rrflow-lab-evals -p rrflow-lab-experiments -p rrflow-lab-datasets` | 215 passed, 0 failed, before the tuning test was added |
| `cargo test -p rrflow-lab --test automation` | 44 passed, including `an_rrflow_graph_is_built_with_what_this_machine_tunes` |
| `cargo test -p rrflow-lab -p rrflow-lab-systems`, after every change | 147 passed, 0 failed |
| `cargo test -p rrflow-lab-systems` | 30 passed, 5 of them in the new `tests/native_hnsw.rs` |
| `cargo clippy -p rrflow-lab -p rrflow-lab-systems --all-targets -- -D warnings` | exit 0 |
| `python scripts/ci/check_code_shape.py` | OK |
| `cargo run -p rrflow-lab -- schemas --out eval/schemas` | three schemas rewritten; the staleness test passes |

Surfaced while authoring: the first run failed `the_default_plan_equals_the_checked_in_manifests` and `generated_schemas_match_the_checked_in_projections`, because the manifests and schemas are projections of the definitions; both were regenerated. One fixture built an RRFlow graph without parameters and now states them and asserts the refusal.

## Not done and not proven

- No lab run was made with the engine. The request the adapter sends is decoded and validated as the engine's public type; it was not sent.
- The 2026-11-06 saved manifest is not made loadable as a whole: its other fields are of that date's format. Only its `rrflow_hnsw` block now has a place.
- The build policy is stated and sent but has no tuning setting.

## Roadmap and POA&M

No status change.
