# POAM-004 — stamped streamed DataFusion provider

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-004`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

C-04's accepted evidence removes normal runtime-log reconstruction from query execution and
proves authenticated, budgeted direct semantic-version reads with rrflowMX/rrflowKV equality
and rrflowKV reopen/physical counters; C-04 is reopened because the vector catalogues still
replay history ([POAM-028](poam-028.md)). C-06g exposes a bounded synchronous projected storage stream with owned pinned
generations, selective page families, cancellation/drop release, compaction/GC survival, and
operation-scoped counters. C-06h now drives that stream through deterministic
independent-model histories, every injected storage boundary, stress replay,
cancellation/resource cases, and bounded sanitizer fuzz targets. Query execution still
materializes `QueryRow` values and allocates new Arrow arrays before DataFusion; there is no
stamped asynchronous `TableProvider`/`ExecutionPlan` proving provider backpressure, pushdown
semantics, shared request-budget enforcement, or borrowed `RecordBatch` lifetime through the
full analytical path.

The current `rrd-query/src/parquet.rs::export_parquet` writes a bounded,
uncompressed query-result artifact. It does not implement the requested
background Parquet cold-storage lifecycle, including atomic generation
publication, retention, hot/cold reads and recovery. F-06 tracks that missing
work independently from exchange/import.

The F-06 draft additionally exposed two constraints that remain open: a scoped
page of the global runtime change chain cannot independently prove that no
matching changes were omitted, and an interrupted or replanned publication can
leave an unreferenced immutable file without a bounded orphan-reclamation
policy. Runtime history also excludes the separate security/control audit
streams. Application backups omit cold artifacts and receipts, so a restore
does not preserve their generation identities. Cold-maintenance failures have
a bounded typed status when the control journal can record one, but detailed
resource/latency observations and failure-reporting under an unavailable
control store remain open. The source read can
wait behind native storage even though the preceding maintenance offer skips a
busy lock; nonblocking admission and starvation bounds remain unproven. The
draft cold schedule is configurable, but its behavior under sustained native
maintenance and foreground load still needs measurement. The draft verifies
an inclusion proof per scanned change, but its proof-node read cost and peak
memory under large source logs have not been measured or separately admitted.
Derived cold files must not replace or reclaim their source until
coverage, all intended streams, retention and recovery are proven.

## Impact

The physical prerequisite no longer requires a result-sized storage scan, but current
rrflowQL scan cost, memory, and latency can still scale with eager materialization;
installed real-process crash/lifetime qualification and the full provider
cancellation/resource path remain absent.

## Owning gates

C-07, F-01, F-02, F-04, F-05, F-06

Roadmap owners: [Gate C](../../roadmap/rrflow-1.0/gate-c.md) and
[Gate F](../../roadmap/rrflow-1.0/gate-f.md).

## Closure evidence

Retain C-04 source closure plus C-06g projected-stream and C-06h adversarial qualification
evidence; then prove bounded asynchronous batches, projection/filter/limit pushdown,
provider cancellation/backpressure, compaction safety, and measured
retained/read/decoded/borrowed/copied/allocated/decompressed/cached bytes through one
stamped DataFusion operation.
