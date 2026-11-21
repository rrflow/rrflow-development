# Archived vector cost and visibility integration

**Status:** active integration evidence; vector qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-vector-costs-integration-20261121`
**Owner:** Gate E E-03/E-04 source recovery

## Before-edit brief

Baseline e9d65f5a18bf8a8b615bf303081bed067a52568a, green PR481.
Join PR482 after its queue passes before this candidate becomes ready.

Integrate the original bounded vector cost and version-resolution sources, retaining search auditing.

Outcome: Gate E E-03/E-04 vector planner/retrieval source recovery. Driver: this integration session after PR482. Sources: 73c9cecd2ec556e3ea8b59ed4539121cb063e6f0 (parent probe 112060f83 and third-parent example tree 2f11ef1605b403a279692622eae4960f6569ee2c), eddb6e264669b59e8d666ac15c55cb67fa903125, 0268d309270faef12d450e66fc400b489ea8d2b5, a8138e659dc6db1159ebaa6187ca14bcd0e33aea, cb07c15150b151649d65af6b55da41a1194be6b0. The broad read-path WIP snapshot remains a later reconciliation package.

Current code already uses ordered TurboQuant versions, linear visible-version selection and admission-before-cloning; HNSW cost samples; configured codec costs and exact reranking/overlay costs; source-address-isolated HNSW generations. Target: ordinary source merges without losing those behaviors, current module boundaries or vector invocation auditing. The measurement-only source explicitly says never ship its VectorSearch audit early return; its original ancestry is integrated but that bypass is not restored.

Before-edit reads: source TurboQuant optimization/tests, HNSW estimate oracle additions, two engine HNSW isolation tests, parent cost probe, audit bypass; current turbo_segment config/segment/tests locations, hnsw index estimator and tests, runtime planner and cost-selection tests, invocation append_invocation_audit, calibration example. Complete conflict regions and source third-parent example mapping before each merge commit. No whole-codebase review claim.

Canonical destinations: rrd-vector turbo_segment/{config,encoding,segment,tests}, hnsw/{index,tests}, runtime/{vector_runtime,tests}; engine tests/vector_index/hnsw_read_path; engine/invocation. The existing vector_cost_calibration example remains an optional measurement harness, never a product authority. Preserve current vocabulary, source addresses, exact verification, bounded candidate heap and typed operation identities. Only those owners and a linked recovery journal/index may change if missing behavior is found.

Acceptance: cargo test -p rrd-vector --lib --locked; cargo test -p rrd-engine --lib hnsw_read_path --locked; engine --lib invocation; engine --test workspace_architecture; docs/navigation and diff checks; candidate and queue. Dedicated target D:/.usr/Temp/rrflow-root-target, four jobs. Do not execute comparison/calibration workloads or claim fresh performance, recall-quality or parity results. Existing correctness oracles remain required.

No new algorithm, external research, runtime instrumentation or provider needed. Stop on an unmapped behavior, discarded assertion or failed gate. Record every source merge and actual acceptance result, keep known roadmap qualification gaps open, then delete the merged branch and continue remaining archives.

## Source merges and conflict resolution

Each source was joined with an ordinary two-parent merge, without rewriting
source history. Source ancestry is distinct from behavioral verification below.

| Original source | Integration commit | Canonical behavior retained |
| --- | --- | --- |
| 73c9cecd2ec556e3ea8b59ed4539121cb063e6f0 | ad3424c4e | TurboQuant ordered versions, linear visible-version runs, heap admission before cloning; parent cost probe and third-parent example mapped |
| eddb6e264669b59e8d666ac15c55cb67fa903125 | 923ec70b6 | Same scan optimization; existing per-round calibration harness retained |
| 0268d309270faef12d450e66fc400b489ea8d2b5 | 6b0abae2b | Five HNSW bounded-estimate/filter oracles and test-only visibility counter retained in split modules |
| a8138e659dc6db1159ebaa6187ca14bcd0e33aea | 2e6934a06 | Both source-address isolation tests retained; existing journal's actual acceptance retained instead of incoming placeholder |
| cb07c15150b151649d65af6b55da41a1194be6b0 | bbe09f541 | Measurement ancestry joined; audit bypass and measurement-only pinned-memory increase excluded |

PR482 merged as f734c29cf3d3402b83a5194f1ac1a2e5d9762c7b and was joined
in cb546af65. Its remote branch was automatically deleted; local deletion
initially refused before that join, then succeeded after it. No forced deletion.

The last source also joins the measurement integration chain b201c5f02,
32504df0d, fc2ce3469, 6cab3a452, f3ddda189, 515b60b3b, cd171544a
and the measurement-only 62aa760d3. Combined merge resolutions were inspected:
the JSON float-round-trip and typed-record context tests remain in
engine/tests/context.rs; collection identities and cost samples remain in
hnsw/index.rs. Digest/page-charge/repair merge commits introduce no combined
resolution patch. The current search operations/results split retains source
binding, canonical verification, shared read budgets and cold-source accounting.
The monolithic search body and old colon-bearing fixture identity were not
restored. Current fixture bytes are checked by the golden test. Current pinned
memory remains the configured 1 GiB default, not the measurement's 4 GiB, and
append_invocation_audit still unconditionally appends its audit event.

The optional calibration source was read in full. The current harness covers
the same eight codecs and dimensional inputs, with paired exact/codec rounds
and median ratios plus ranges rather than independent five-round medians.
It remains the single harness; no measurement was executed. The third-parent
2f11ef1605b403a279692622eae4960f6569ee2c contains only that example.

Additional reads: complete 0268 source patch and current estimator/test locations;
segment.rs visible-run, retirement and heap-admission sections; current engine
HNSW source-isolation tests; old/current calibration diff; current invocation
audit function; current HNSW collections/sample construction and search result
source-binding section. Truncated broad merge-base output was not treated as
review coverage; individual source commits and affected owners were examined.
One guessed context directory did not exist; recursive search located context.rs.
No whole-codebase or source-qualification claim is made.

No Rust, test or fixture delta remains against green main after these merges.
This package establishes original history joins and verified preservation of
existing behavior, not a new retrieval capability or performance improvement.

## Verification

Test tree: cb546af65, target D:/.usr/Temp/rrflow-root-target, four Cargo jobs.
Completed checks:

- `cargo test -p rrd-vector --lib --locked`: 86 passed, one ignored long lab-size fixture.
- `cargo test -p rrd-engine --lib hnsw_read_path --locked`: 14 passed.
- `cargo test -p rrd-engine --lib invocation --locked`: nine passed.
- `cargo test -p rrd-vector --test golden --locked`: one passed.
- `cargo check -p rrd-vector --example vector_cost_calibration --locked`: passed; harness not executed.
- `cargo test -p rrd-engine --test workspace_architecture --locked`: 35 passed.
 No comparison or calibration
workload, full recall qualification, provider integration, installed UI or
cross-platform runtime run is claimed. E-04 and the remaining archive recovery
stay open; the next package integrates the archived kernel/query source changes.

Navigation regeneration, documentation policy and `git diff --check` passed.
Candidate and merge-group results remain owned by PR483 checks.
