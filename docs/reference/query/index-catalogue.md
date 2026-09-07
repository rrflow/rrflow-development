# rrflowQL index catalogue

**Status:** active implementation reference; native incremental index convergence remains incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/query/index-catalogue`
**Owner:** implemented query-index definition, catalogue, snapshot-artifact, lifecycle, and selection behavior

This record describes the current query-index implementation without declaring
the final RRFlow index subsystem complete. The
[native multi-model boundary](../../architecture/system-overview.md#native-multi-model-boundary)
requires every derived index to remain subordinate to canonical rrflowDB data.
Roadmap [Gate E](../../roadmap/rrflow-1.0.md#gate-e--make-graph-and-indexes-native-incremental-access-paths)
owns native scalar, unique, BM25, graph, and vector access paths, while
[Gate F](../../roadmap/rrflow-1.0.md#gate-f--connect-rrflowkv-to-arrowdatafusion-correctly)
owns their stamped Arrow/DataFusion composition.

## Definitions and catalogue state

An index definition binds a stable projection ID to one non-recursive rrflowQL
source, an index kind, zero to sixteen kind-dependent fields, optional
supported filters, and an optional scalar uniqueness constraint. Current kinds
are scalar, count, grouped count, geospatial, materialized view, and BM25.
Creation binds fields against the captured query catalogue before changing
control state. Invalid or duplicate fields, incompatible kinds, cross-scope
catalogues, unsupported filters, and traversal sources fail closed.

Each entry carries a projection stamp:

- generation fences stale builders;
- source cursor identifies the covered authoritative runtime history;
- configuration digest binds the definition;
- artifact digest binds the serialized snapshot artifact; and
- state is `building`, `ready`, `quarantined`, or `retiring`.

The catalogue is authenticated persistent control state rather than the
artifact itself. Catalogue changes use compare-and-swap control transitions
and append to the control journal. A ready artifact is selected only when its
definition digest, source cursor, and valid-time coverage exactly match the
bound query. A newer artifact is not valid for an older bitemporal read.
Absent, building, quarantined, retiring, stale, or differently-timed entries
fall back to the current authoritative path.

## Build, publication, and selection

The current builder executes the definition's all-fields reference query at a
captured head and literal valid time. It validates uniqueness where requested,
builds any BM25 or analytical payload, serializes the sorted `QueryRow`
snapshot, and writes it under a content-addressed projection name. Publication
then revalidates generation, source cursor, digest, coverage, and state before
marking the entry ready.

Scalar and geospatial selection requires matching leading fields. A
materialized view additionally matches its frozen predicates. Multiple exact
candidates choose the longest matching prefix and then stable index identity.
BM25 artifacts carry their configuration, source cursor, schema revision, and
valid-time coordinate and return score, matched-term, and highlighting fields.
Count and grouped-count artifacts carry checked analytical summaries.

After the first complete build validates a unique scalar definition,
`RrdEngine` checks prospective transactions against that constraint even while
the read artifact is stale or rebuilding. This is real constraint behavior,
but its present implementation reconstructs current records from runtime
history; it is not yet the Gate E transactional unique-key access path.

## Verified implementation boundary

The focused catalogue corpus proves equivalent lifecycle behavior on rrflowMX
and rrflowKV, rrflowKV close/reopen, stable exact selection, stale fallback,
generation fencing, journal verification, idempotency collision rejection,
artifact corruption rejection, scalar uniqueness, and current scalar, count,
grouped-count, geospatial, materialized-view, and BM25 artifacts.

The artifact remains a serialized materialized snapshot. Rebuild first executes
the full reference query; `incremental_reconciliation` records a checked diff
against the prior artifact but does not mean the underlying postings or
secondary keys were incrementally updated. Scalar and unique checks still scan
or reconstruct broad state. BM25 is not yet a transactionally maintained
dictionary/statistics/postings structure, and vector/HNSW projections remain
in their separate current subsystem. Gates C and E must commit canonical data
and synchronous index deltas atomically before Gate F can stream their bounded
native outputs as stamped Arrow batches.
