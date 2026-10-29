# E-04 TurboQuant persistent serving engineering plan

**Status:** active supporting work-package plan; no package is implemented by
this record; E-04, OBJ-13 and POAM-028 remain open
**Coordinate:** `rrflow://rrflow-instance/data/work-package/e-04-turboquant-persistent-serving`
**Owner:** implementation order and proof for OBJ-13's persistent-search part
and the public path a client uses to build and search with TurboQuant
**Repository baseline:** `81fce08e14497bbc280790beefe4666716355aec`

The canonical [RRFlow 1.0 roadmap](rrflow-1.0.md) owns completion status.
[OBJ-13](../objectives/rrflow-1.0-alpha.md#required-outcomes) owns the
measurable TurboQuant result, and the
[quantization lifecycle reference](../reference/vector/quantization-lifecycle.md#required-turboquant-scope)
owns the contract and its four parts. This record makes the first part,
persistent vector search, executable as ordered packages, and names the public
path a client needs to use it. It does not complete E-04, F-03, H-04, J-04,
POAM-028 or OBJ-13, and it adds no second vector store or quantization
authority.

## Owner direction

- **2026-10-24.** TurboQuant is a complete capability, not one codec
  ([PR118](https://github.com/rrflow/rrflow-development/pull/118), OBJ-13).
- **2026-10-28.** Build out systems that leverage TurboQuant first. Connectome
  shows and uses them, rather than describing a missing view.

## Verified starting point

Read at `81fce08e`:

- **Codecs and lifecycle exist in the engine.**
  - Scalar, product, binary and TurboQuant codecs exist.
  - `RrdEngine::build_vector_quantization_artifact`, `list_…`, `activate_…`
    and `retire_…` (`rrd-engine/src/engine/vector/quantization.rs`) take the
    contract's `BuildVectorQuantizationArtifact` and sibling types.
  - No public operation reaches them. None of the 57 HTTP operations builds,
    lists, activates or retires a quantization, so no client can turn
    TurboQuant on.
- **A build reads every vector version in the scope** (`read_vector_versions`)
  and encodes a complete new generation.
- **A search reads every vector version in the scope**
  (`read_vector_search_versions`) before it plans. Then:
  - an HNSW generation serves with an exact overlay of newer candidates
    (`rrd-vector/src/runtime.rs::prepare_search_at`);
  - a TurboQuant segment refuses a search whose required source cursor is
    newer than its own (`rrd-vector/src/turbo_segment.rs`), so any vector
    write after a build takes the generation out of service until a rebuild;
  - approximate candidates are reranked against every canonical candidate held
    in memory.
- **The commit path already writes the atomic index delta E-04 needs.** It
  writes one `VectorSourceDelta` per collection, named vector and field, in
  cursor order, carrying before and after version pointers and the value
  digest (`rrd-store/src/access/vector.rs::encode_vector_source_effects`). The
  only reader is `RuntimeRepository::vector_source_deltas_since`, which has no
  read stamp, no key budget, no read evidence and no engine caller.
- **The public `vector-search` already accepts approximate modes**
  (`allow_approximate`, `require_approximate`) with `exact_rerank` and
  `ef_search`, and reports the access path, the selected generation and its
  resource evidence.

## Ordered packages

Each package is one pull request with its change brief, linked journal,
focused oracle, affected suites and candidate CI. A package that changes the
public contract regenerates every generated surface in the same change.

| # | Package | Owners | Change | Oracle before handoff |
|---|---|---|---|---|
| 1 | **E-04a: stamped vector source-delta reads** | E-04, C-04, POAM-028 | Add `RuntimeVersionedSource::VectorDeltas { address, after }` to the existing `read_versioned_expanding`. It reads one address's deltas in (`after`, read cursor] under the request's key budget, then expands each delta into `Identity` reads authenticated against the stamped root, and fails closed when a pointer disagrees with its authenticated version. Remove the unstamped reader. Add the public read-evidence path `vector_source_deltas`. | A differential against a full exact read across put, update, move and retire; stamp exclusion and reopen on rrflowMX and rrflowKV; the budget denial; constant keys for one address as unrelated vectors grow; and tampered-pointer denial. |
| 2 | **E-04b: TurboQuant serves from its generation plus the delta** | E-04, POAM-028 | A search that selects an active TurboQuant generation reads that generation's catalogue entry and the stamped delta since its source cursor, not every vector in the scope. Changed and retired references override the generation. Exact rerank reads only the candidate references' histories. The segment's staleness refusal becomes an overlay, as HNSW has today. | Results equal the exact oracle. Writes after activation appear; updated, moved and retired points never return stale. Search keys stay flat as unrelated vectors and collections grow. `exact` mode and fallback are unchanged. |
| 3 | **H-04q: public quantization lifecycle** | H-04 (public operations), E-04 | Serve build, list, activate and retire through the catalogue-derived operation path, with OpenAPI, the Rust SDK, the generated SDKs and the CLI. The engine methods and authorization actions stay the owners. | Each surface binds each operation exactly once and lowers to the same engine method; authorization denial; conformance; and the installed-lifecycle process test builds, activates and searches. |
| 4 | **Connectome Vectors with TurboQuant** | H-06, Connectome Sprint 009 | Per named vector, build, activate and retire generations and show their states, bits, seed, generation, source cursor and sizes. Run exact and TurboQuant search side by side with recall@k against exact, latency and keys examined. | Connectome real-engine process tests, and before/after captures under its decision 17. |
| 5 | **E-04c: incremental generations** | E-04 | Build the next generation from the active one plus the stamped delta, atomically published, instead of re-encoding every version. | Equals a full rebuild across update, delete, move and reopen. Interrupted-build recovery, stale-generation denial, build amplification and RSS are reported. |
| 6 | **E-04d: TurboQuant_prod** | E-04, OBJ-13 | Add the [paper's](https://arxiv.org/abs/2504.19874) inner-product estimator, an MSE stage plus a one-bit QJL residual, as an explicit lifecycle codec choice. | Recall and score error against the MSE variant and exact search on one corpus; and estimator bias. |

F-03 then runs candidate generation and exact rerank in the stamped Arrow
pipeline, J-04 qualifies recall, memory and latency claims, and the reference's
other parts (the Gate G KV-cache format and the H-07 transfer profile) keep
their owners.

## Invariants for every package

- Canonical full-precision vectors and the exact scorer stay authoritative. An
  approximate hit is never reported as exact, and exact search remains
  available.
- One engine authority and one quantization lifecycle. No second store, no
  temporary index, and no change-log source on a vector path.
- Reads use the request's stamp and budget, and physical counters ship with
  the behavior that uses them.
- The 512-vector component matrix stays a component test. Claims beyond it
  come from J-04 evidence, and a paper result is not RRFlow evidence.

## Client consumption

Connectome's [Sprint 009 plan](https://github.com/rrflow/connectome-development/blob/main/.agents/sprints/009-connectome-ownership.md)
owns its steps. Package 4 is the first view built on this work. The owner's
direction of 2026-10-28 also names project knowledge paths, editable rrflowQL
graph results, and plan and handoff records together with the schemas they
produce. Those are real views over engine operations; a view does not ship as
a notice that it is missing.
