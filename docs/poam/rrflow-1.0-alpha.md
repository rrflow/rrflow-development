# RRFlow 1.0 alpha POA&M

**Status:** active verified-gap ledger
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha`
**Owner:** remediation status for the RRFlow 1.0 alpha objective

This record tracks gaps observed in the checkout. The
[objective](../objectives/rrflow-1.0-alpha.md) defines the required result and
the [system overview](../architecture/system-overview.md) defines the component
and security boundaries. The
[engine data-flow architecture](../architecture/engine-data-flow.md) defines the
affected system flow. The [roadmap](../roadmap/rrflow-1.0.md) owns execution
order. A row closes only when the referenced gate records its acceptance
evidence; editing this table cannot declare a capability complete.

## Status vocabulary

- `Open`: the deficiency is present and its prerequisite gate is available.
- `Sequenced`: the deficiency is present but an earlier roadmap gate owns the
  next executable work.
- `Verifying`: implementation exists and the exact closure evidence is running
  or under review.
- `Closed`: the owning roadmap gate contains accepted evidence at an exact
  revision.

## Open deficiencies

| ID | Priority | Observed deficiency | Impact | Owning gates | Closure evidence | Status |
|---|---|---|---|---|---|---|
| POAM-001 | Critical | Flat supporting documents are not fully classified; the deterministic bootstrap package is not yet enforced in CI or imported through persistent rrflowDB; and source boundaries have not completed the final naming/dependency audit. | Agents still depend on checkout fallbacks, can follow unclassified or overlapping records, and cannot yet prove durable readback of the packaged knowledge through rrflowDB. | A-06, A-07, D-02, D-05, H-01, J-03 | Documentation topology and dependency/vocabulary checks pass with one owner and no duplicate active body; a reproducible manifest/JSONL package covers every eligible record or explicit exclusion; authorized import, reopen, readback, and warp resolution pass. | Open |
| POAM-002 | Critical | Native rrflowKV segment v3 stores compressed row records. It does not implement the target ordered key/version spine plus Arrow-compatible column pages. | Point/state workloads and analytical scans cannot yet share the intended durable physical substrate. | C-06 | Format vectors, property tests, crash tests, and comparative benchmarks prove the hybrid segment layout and exact reads. | Sequenced |
| POAM-003 | Critical | Fjall selection, pre-1.0 format readers, migration runtime code, and alternate-path success tests remain executable. | More than one persistent path can survive into the alpha and force continued dual reasoning. | C-05, J-01 | Dependency and repository searches plus fresh native reopen tests prove one rrflowKV opener and one accepted physical reader remain. | Sequenced |
| POAM-004 | Critical | Normal query execution can reconstruct or materialize authoritative rows and then allocate new Arrow arrays before DataFusion. | Scan cost, memory, and latency scale with materialized input rather than selected columns and rows. | C-04, F-01, F-02 | Physical counters prove bounded key/page reads, streaming batches, projection/filter pushdown, and measured copy/decode/allocation behavior. | Sequenced |
| POAM-005 | Critical | Record, adjacency, scalar, BM25, vector, projection, and runtime-log changes are not yet proven as one atomic native write/read system. | Context can be slow or an index can diverge from authoritative state across failure. | C-03, E-01 through E-05 | Failure injection and exact-oracle reopen suites prove atomic maintenance and bounded native access. | Sequenced |
| POAM-006 | High | Install and attunement are public contracts without an engine-persisted job, CLI preview/apply flow, generic template, or real phase executors. | A project cannot yet become an RRFlow estate through a repeatable supported workflow. | D-01 through D-10 | Empty/existing project installation and all phase interruption/resume/verification tests pass. | Sequenced |
| POAM-007 | High | `AGENTS.md` is the repository instruction source, but there is no installed, digest-bound specialization manifest or adapter-conformance proof across supported AI hosts. | Provider files can drift or claim invisible lifecycle behavior a host does not expose. | D-01, H-04, I-04 through I-06 | Provider stubs contain only verified forwarding behavior; all adapters resolve the same specialization digest and operations. | Sequenced |
| POAM-008 | High | The low-level synchronous function/trigger foundation is not the canonical engine-event, routine, hook-adapter, or skill system. | Automation cannot yet resume, adapt, explain activation, or remain consistent across providers. | I-01 through I-07 | Restart, denial, idempotency, budget, activation, update, and uninstall conformance passes for generic packages. | Sequenced |
| POAM-009 | High | Reasoning-tree and router contracts exist without persisted tree execution, model-manifest handshake, RouterBackend dispatch, or LFG conformance. | A local model cannot safely steer the fast and analytical paths. | B-03, G-01 through G-06 | Invalid-output corpus, stamped route packets, CAS/reopen tests, and separated model/storage measurements pass. | Sequenced |
| POAM-010 | High | Context routing uses useful components but lacks native incremental BM25/HNSW integration, adaptive access planning, and verified feedback-policy evolution. | Retrieval remains expensive and cannot prove that dynamic routing improves quality. | E, H-01, H-02 | Exact and approximate quality corpus, plan evidence, feedback replay, rollback, and regression gates pass. | Sequenced |
| POAM-011 | Medium | Public transports, SDKs, MCP, mesh resolution, and Connectome have not passed one cross-surface real-process corpus. | User-visible status can disagree with the engine or remain mock-driven. | B-04, B-05, H-04 through H-07 | Equivalent requests produce the same result/denial/stamp/digest and correlated traces across every supported surface. | Sequenced |
| POAM-012 | High | There is no fixed-hardware end-to-end benchmark and failure corpus for the complete RRFlow workload. | Performance or competitive claims cannot be evaluated honestly. | J-02 through J-05 | Raw reproducible correctness, latency, throughput, memory, storage, recall, crash, and deployment evidence is published. | Sequenced |
| POAM-013 | Critical | The workspace crates are currently local, but no enforced repository-closure check, self-contained release assembler, offline installer proof, or clean-rollout comparison exists; one SurrealDB differential still defaults to a host-specific build-cache path. | A checkout can pass locally while a release silently depends on this machine, a sibling repository, a cache, a fetch, or an external service and therefore cannot be deployed as one RRFlow system. | A-07, D-01, D-06, J-03 through J-05 | CI rejects every escaping first-party input and host-specific path; a signed manifest accounts for the complete default distribution; network-denied clean-machine install/reopen passes without sibling code or external database services; pinned SurrealDB/Qdrant rollout evidence is published before any ease claim. | Sequenced |
| POAM-014 | Critical | The execution inventory covers current files, but it does not yet prove implementation-to-requirement traceability for existing WAL/MVCC/recovery and hot-read behavior, transaction/security/audit behavior, embedding/vector/edge behavior, reasoning/context behavior, or Arrow/DataFusion behavior before direct consolidation. | A move, merge, rename, compile, or deletion can appear complete while useful semantics or their characterization tests are omitted from the one RRFlow implementation. | A-07, C, D-05, E, F, G, H, J-02 | A-07 records the current modules, tests, fixtures, required behavior, canonical destination, and owning gate for every capability family; each later gate preserves or deliberately replaces that behavior with equal-or-stronger acceptance evidence before removing the prior path. | Sequenced |

## Milestone discipline

The milestone for each row is its owning roadmap gate, not a date guessed
before implementation exposes the true work. Work follows roadmap dependency
order. If implementation reveals another material deficiency, add one row with
observable evidence and a gate mapping before expanding scope.
