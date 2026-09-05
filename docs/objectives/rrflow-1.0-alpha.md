# RRFlow 1.0 alpha objectives

**Status:** active measurable outcome set; no objective is complete
**Coordinate:** `rrflow://rrflow-instance/data/objective/rrflow-1.0-alpha`
**Owner:** first usable RRFlow alpha outcome

The RRFlow 1.0 version is frozen while this baseline is established. The
repository root [README](../../README.md) owns product identity and current
maturity. The [engine data-flow architecture](../architecture/engine-data-flow.md)
owns the detailed transactional and analytical flow, the
[roadmap](../roadmap/rrflow-1.0.md) owns delivery order, and the
[POA&M](../poam/rrflow-1.0-alpha.md) owns observed deficiencies. This record
owns only the measurable result.

## Objective

Deliver one installable per-project RRFlow estate that can drive development,
maintenance, governance, reasoning, and recall through one persistent engine.
The estate must remain provider-neutral: Codex, Claude, Gemini, Grok, LFG, and
future callers consume the same public operations, authorization, read stamps,
context evidence, routines, and skills.

rrflowDB owns AI knowledge, temporal graph state, reasoning state,
indexes, evidence, automation state, and analytical storage. Application and
operator databases such as PostgreSQL, Turso, SQLite, Dragonfly, and hosted
services remain governed external sources unless an operator explicitly adds
an adapter. They never become an implicit RRFlow persistence backend.

## Required outcomes

An outcome is `complete` only after its end-to-end proof passes at an exact
revision. `Partial` means useful implementation exists but the named proof has
not passed. A type, fixture, mock, compile, or document alone is not proof.

| ID | Outcome | Current state | Completion evidence | Roadmap gates |
|---|---|---|---|---|
| OBJ-01 | One authority and vocabulary | Partial: `RrdEngine`, grouped crates, and canonical terms exist; documentation classification and the final boundary audit remain open. | Repository policy proves one identity/status owner, one objective owner, one roadmap, one POA&M, inward dependencies, and no competing graph, memory, query, lifecycle, or provider authority. | A |
| OBJ-02 | One hybrid rrflowKV persistence engine | Partial: WAL, MVCC memtable, manifests, row-record segments, recovery, and compaction exist. Arrow-compatible immutable segment pages and compatibility removal do not. | Atomic crash/reopen corpus proves WAL → MVCC memtable → immutable key/version spine plus Arrow-compatible column pages → manifest publication with no alternate backend or legacy reader. | C, J |
| OBJ-03 | Equivalent volatile and persistent semantics | Partial: rrflowMX and native rrflowKV compositions share engine contracts and a logical corpus; the complete transactional/access-path corpus is not proven. | The same transaction, snapshot, graph, index, and query corpus passes for rrflowMX and rrflowKV; rrflowKV additionally survives every recovery boundary. | C |
| OBJ-04 | Atomic multi-model records, graph, and indexes | Partial: canonical mutations and several index foundations exist; graph, scalar, BM25, and vector paths are not one proven incremental commit/read system. | One transaction atomically changes records, both graph directions, scalar/unique state, BM25 postings, vectors, index deltas, and the runtime log; exact oracles remain equal after reopen. | C, E |
| OBJ-05 | Real Arrow/DataFusion analytical execution | Partial: DataFusion executes stamped Arrow batches allocated from materialized `QueryRow` values. rrflowKV does not yet stream eligible segment buffers. | A stamped provider streams bounded batches from rrflowKV, pushes eligible work down, proves conditional zero-copy with copy/decode/allocation counters, and enforces memory, spill, scan, time, candidate, and output limits. | F |
| OBJ-06 | Dynamic governed reasoning and recall | Partial: bounded lexical, exact-vector, graph, fusion, context evidence, reasoning-tree, and router contracts exist; native indexes, persisted tree execution, adaptive routing, and LFG dispatch do not. | One authorized request chooses fast or analytical work, retrieves through eligible graph/BM25/vector paths, fuses deterministically, advances a persisted tree through CAS, and reproduces its evidence at the same stamp after reopen. | E, G, H |
| OBJ-07 | Install and attune a new or existing project | Contract only: the provider-neutral install/attunement envelopes and phase state machine exist; the CLI, engine job persistence, phase executors, and templates do not. | Preview, apply, interrupt, resume, verify, close, and reopen pass for empty and existing projects without credentials, caches, generated output, application databases, or provider lifecycle files becoming engine state. | B, D |
| OBJ-08 | Adaptive routines, triggers, skills, and adapters | Partial foundation: synchronous governed functions/triggers exist. Canonical engine events, resumable routines, skill packages, explicit hook adapters, and capability activation do not. | Generic packages are versioned, digest-bound, authorized, budgeted, observable, restartable, removable, and activated only from verified estate signals and policy. | I |
| OBJ-09 | Provider-neutral client operation | Partial: HTTP, current WebSocket behavior, SDKs, CLI/MCP, and a separate Connectome bootstrap exist at different levels of conformance. | Every supported client and model adapter produces equivalent operation, authorization, stamp, digest, denial, trace, and restart results without embedding engine logic. | B, H |
| OBJ-10 | Reproducible competitive evidence | Not established: local component evidence exists, but no complete fixed-hardware engine corpus proves the alpha. | Published raw results cover correctness, p50/p95/p99 latency, throughput, memory, copied/decoded/allocated bytes, disk/WAL/compaction, recall@k, filtered quality, restart, and failure behavior against declared baselines. | J |

## Alpha exit

The baseline exists only when all ten objectives are complete and their linked
roadmap gates carry reproducible evidence. Optimization continues within the
same frozen version until the repository owner makes an explicit release and
version decision.
