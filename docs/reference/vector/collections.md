# RRFlow vector collections and points

**Status:** active implementation reference; native point and payload-index access remain incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/vector/collections`
**Owner:** vector collection identity, named-vector configuration, and point administration

RRFlow's vector subsystem is one branch of the authoritative runtime, not a
sidecar vector database. `RrdEngine` authorizes collection administration and
vector mutation. Canonical point versions remain `RuntimeVector` mutations in
the same runtime commit vocabulary as records, relations, events, claims,
series, geo values, and objects.

## Collection catalogue

A collection contains one to 64 uniquely named vector definitions. Each
definition fixes its canonical field, dense/sparse/multi-dense shape,
dimensions, similarity metric, optional embedding-model identity and digest,
and memory-tier policy.

The current `VectorCollectionRepository` persists a JSON-encoded per-scope
catalogue in an rrflowKV control record. Compare-and-swap control transitions
advance the catalogue revision, append control-journal evidence, and retain
content-bound idempotency receipts across reopen. Collection and payload-index
generations, timestamps, identities, and configuration digests are validated
when loaded; corruption fails closed.

This is engine-owned persistent control state, but it is not yet the final
ordered rrflowKV key layout or an atomic physical projection of the logical
schema catalogue. Gates C-01 and C-03 own that convergence.

## Point mutation and reads

`CommitTransaction` accepts collection-addressed `put_vector` mutations beside
the other logical models. Before commit, `RrdEngine` resolves the collection
and vector name and validates field, shape, dimensions, optional model
provenance, and indexed payload types. Any invalid mutation rejects the whole
transaction. `retire_data` records vector retirement at an explicit valid time
without erasing prior versions.

Retrieve and scroll currently capture one read stamp, scan retained runtime
changes from cursor zero, select collection-addressed candidates, reduce
visible temporal versions, and then filter or paginate them. They are bounded
and restart-safe, but they are not native point lookups or payload-index scans.
Gate C-04 must replace the broad replay path.

## Payload-index definitions and safe deletion

Each collection may declare up to 256 boolean, integer, unsigned, decimal,
keyword, or digest payload-index definitions. Ensure, list, and delete use
distinct deny-by-default engine actions. Catalogue transitions are
digest-bound, idempotent, revisioned, and persistent. Point commits enforce
the declared value type, and active HNSW or quantization configurations prevent
removal of a payload definition they reference.

A payload-index definition is not proof of a general native scalar index.
Current administration records configuration and constrains vector artifacts;
Gate E must prove the persistent lookup structure, incremental maintenance,
filtered recall, and reopen behavior.

Collection deletion scans the current vector history within an explicit
budget, rejects live or future points, and rejects active HNSW, TurboQuant, or
quantization artifacts. The caller must first retire points and governed
artifacts. Historical runtime changes remain intact.

## Verified boundary and required convergence

Focused repository tests prove collection and payload-definition ensure,
reconfiguration, deletion, digest/idempotency conflict, journal persistence,
and rrflowKV reopen. Engine tests prove dense, sparse, and multi-dense point
commit/retrieve, payload-type rejection, live-point deletion denial, retirement,
and restart-safe administration.

Those tests do not prove E-04's atomic vector index delta, immutable HNSW plus
exact delta overlay, exact reranking, or recall threshold. They also do not
prove F-03's stamped Arrow operator path. Until C-03/C-04, E-04, and F-03 pass,
RRFlow has persistent vector semantics and administrative scaffolding, not the
finished persistent multimodal recall engine.
