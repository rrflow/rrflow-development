# Leftover branches without a pull request, absorbed

**Status:** active integration evidence; build-and-test absorption, no gate acceptance claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/leftover-branches-absorbed-20261110`

**Owner:** the repository owner's 2026-11-10 direction that every branch is absorbed or deleted so main is the one branch; `docs/operations/ci.md`

**Starting revision:** `agent/integration-20261110` at `d5e66d43` (PR #314).

**Current behavior.** After #314 the development remote still held 136 branches. 120 are ancestors of the integration branch. Sixteen are not; eight of those carry hand-written content the integration branch lacks, and none has a pull request.

**Target behavior.** Everything those eight branches wrote that is still true lands here; what a later package superseded is named, not silently dropped. Every branch then becomes deletable.

| branch | last commit | carried here | superseded, and by what |
| --- | --- | --- | --- |
| `agent/source-bound-replan-design` | 2026-10-29 | `docs/architecture/source-bound-plan-design.md` (the later "immutable job identity" revision), its link from `engine-data-flow.md` and the architecture index, journals `d-02-immutable-job-replan-design.md`, `d-02-source-bound-replan-design.md`, `d-02-source-plan-inventory-integration.md` | none |
| `codex/d02-historical-plan-replay` | 2026-10-29 | its two journals are byte-identical to the files carried above | its earlier revision of the design doc (210 lines): the later revision replaced its six-line "a conflict means refresh and retry" paragraph with the immutable-job-identity rule, which is the correction that revision exists for |
| `codex/ci-conflict-reconciliation` | 2026-10-30 | `scripts/ci/test_merge_conflict_policy.py`, the sparse-ledger `poam_ids_valid` and first-parent `historical_poam_ids` in `check_documentation.py`, the verify-job line, `governance.md` paragraph, journal `ci-conflict-reconciliation.md` | none; `inspect_merge_conflicts.py` was already on main |
| `agent/vector-generation-read-path-wip-20261109` | 2026-11-09 | journal `e-04-hnsw-read-main-integration-20261107.md`, the POAM-048 paragraph, a pointer from `docs/reference/vector/search.md` to that evidence, and two HNSW oracles ported into `hnsw_read_path.rs` (`distinct_source_addresses_receive_distinct_hnsw_indexes`, `an_emptied_hnsw_generation_does_not_answer_for_another_collection`) | its installation, attunement and lfg-swag edits, by #241, #249 and #304's final forms; its reference paragraph describing the empty-generation defect, by #304's repair (the defect is recorded in the carried journal) |
| `agent/d04-source-spans-reconcile-wip-20261109` | 2026-11-09 | journal `d-04-parser-main-reconciliation-2026-11-07.md` | its engine, contract and client edits, by #241's final form |
| `agent/d03-snapshot-visibility-wip-20261109` | 2026-11-09 | the "current-main integration and Windows red baseline" section of `d-03-snapshot-consumer-20261107.md` | its workflow and policy edits, by #303 and #311 |
| `codex/ci-macos-cache-warm` | 2026-10-29 | journal `ci-cache-warm-standalone-integration.md` | its one `check_workflow.py` line, by #303's lanes |
| `agent/poam-flush-stalls` | 2026-11-07 | journal `e-04-flush-stall-record.md` | none |

The other eight non-ancestor branches hold nothing the integration branch lacks: `agent/lab-definitions-automatic` (#274, carried by #306), `codex/connectome-real-graph-evidence` (#234, empty), `codex/ci-alpha-throughput` (empty), `codex/attunement-job-discovery` (#231: its operation is on main, its POAM-040 paragraph is in #315, the remaining lines are a formatting variant of the same test), `copilot/fix-installed-domain-crash` (#300: its retry loop was superseded on main, its cleanup is in #314), `agent/c07-windows-characterization-evidence` (three rows of the generated `rrflow-1.0-file-plan.jsonl` inventory, regenerated from the current tree), and the two `gh-readonly-queue/*` refs GitHub manages.

**One file restored.** `docs/evidence/change-journals/gate-e/e-04-hnsw-generation-read-path.md` existed on main and was absent from the integration branch: the lab branch (#306) had deleted it when it reset #278's engine files, and the merge carried the deletion. It is the only file main had that #314 lacks (`git diff --diff-filter=D da3edfb8 agent/integration-20261110`), and it is restored here byte for byte.

**Acceptance.** On Windows at this branch: `cargo test --locked -p rrd-engine --lib -- hnsw_read_path`: 14 passed, including both carried oracles, so #304's source-identity repair satisfies the WIP branch's red oracle without change. `cargo fmt --check` and `cargo clippy --locked -p rrd-engine --all-targets -- -D warnings` clean. `python scripts/ci/test_merge_conflict_policy.py` (10 tests), `test_inspect_merge_conflicts.py`, `test_candidate_admission.py`, `test_ci_lanes_policy.py`: OK. `check_workflow.py`: CI policy OK, 16 substantive jobs. `render_navigation.py` and `check_documentation.py`: OK (1118 statuses). `git diff --diff-filter=D da3edfb8 HEAD` is empty: nothing main had is absent.

**Research decision:** none.

**Checks not run:** Linux/macOS, candidate CI.

**Roadmap/POA&M change:** none; the sparse-ledger rule from `codex/ci-conflict-reconciliation` lands with its governance paragraph.
