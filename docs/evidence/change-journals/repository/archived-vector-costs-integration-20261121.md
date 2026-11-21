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

Before-edit reads: source TurboQuant optimization/tests, HNSW estimate oracle additions, two engine HNSW isolation tests, parent cost probe, audit bypass; current turbo_segment config/search/tests locations, hnsw index estimator and tests, runtime planner and cost-selection tests, invocation append_invocation_audit, calibration example. Complete conflict regions and source third-parent example mapping before each merge commit. No whole-codebase review claim.

Canonical destinations: rrd-vector turbo_segment/{config,encoding,segment,tests}, hnsw/{index,tests}, runtime/{vector_runtime,tests}; engine tests/vector_index/hnsw_read_path; engine/invocation. The existing vector_cost_calibration example remains an optional measurement harness, never a product authority. Preserve current vocabulary, source addresses, exact verification, bounded candidate heap and typed operation identities. Only those owners and a linked recovery journal/index may change if missing behavior is found.

Acceptance: cargo test -p rrd-vector --lib --locked; cargo test -p rrd-engine --lib hnsw_read_path --locked; engine --lib invocation; engine --test workspace_architecture; docs/navigation and diff checks; candidate and queue. Dedicated target D:/.usr/Temp/rrflow-root-target, four jobs. Do not execute comparison/calibration workloads or claim fresh performance, recall-quality or parity results. Existing correctness oracles remain required.

No new algorithm, external research, runtime instrumentation or provider needed. Stop on an unmapped behavior, discarded assertion or failed gate. Record every source merge and actual acceptance result, keep known roadmap qualification gaps open, then delete the merged branch and continue remaining archives.
