# Bi-temporal memory, lexicons and behavioural state: survey

**Status:** active survey of external systems and literature; not an inventory, RRFlow architecture, plan or status
**Coordinate:** `rrflow://rrflow-instance/data/research/bitemporal-lexicon-and-behavioral-state-survey`
**Owner:** what other systems do for bi-temporal storage, multi-session folds, controlled vocabularies and behavioural state, and what that suggests RRFlow must decide; not delivery planning
**Reviewed:** 2026-11-16
**Method:** web search summaries on 2026-11-16, plus one arXiv abstract. No upstream source was pinned or read, so nothing here meets the source-adaptation rule in [AGENTS.md](../../AGENTS.md); an adaptation needs a pinned `<subject>-capability-inventory.md` first

The owner asked on 2026-11-16 for research before any build: SurrealDB, ArangoDB and reasoning engines that handle a dictionary or lexicon, planned as an engine part ([direction](../roadmap/bitemporal-reasoning-owner-direction-20261116.md)). This survey is the first pass. Vendor claims are marked as such. Where a source did not say something, it is listed under gaps rather than filled in.

## What the systems do

| Subject | Finding | Source quality |
|---|---|---|
| Zep and Graphiti (agent memory) | Two time axes, event time and transaction time. Every fact carries `created_at`, `valid_at`, `invalid_at`, `expired_at`. A contradicting fact closes the old edge instead of deleting it. Retrieval combines cosine similarity, BM25 and graph traversal ([Zep blog](https://blog.getzep.com/stop-using-rag-for-agent-memory/), [Neo4j write-up](https://neo4j.com/blog/developer/graphiti-knowledge-graph-memory/)) | Vendor and secondary write-ups; the paper is arXiv 2501.13956, not read |
| XTDB v2 | Bi-temporal by default on every table, following SQL:2011: valid time is user-managed, system time is when information entered. It uses a single writer, stated as needed for auditing and for generating bi-temporal timestamps; no sharded multi-writer ([Time in XTDB](https://docs.xtdb.com/about/time-in-xtdb.html), [README](https://github.com/xtdb/xtdb)) | Vendor documentation |
| Graph-native bi-temporal store for agents | Closed-open valid and transaction intervals on versioned content nodes; time-travel retrieval. On 60 of 500 LongMemEval questions, time-travel recall on knowledge-update questions was 80% R@10, but temporal-reasoning recall fell from 50% to 37.5%, attributed to post-filter dilution ([arXiv 2607.26520](https://arxiv.org/abs/2607.26520)) | Abstract only; one preprint; small sample, no variance |
| SurrealDB 3.0 | Presents itself as agent memory. SurrealKV keeps the history of every key for a configurable retention or for ever, read with a `VERSION` clause; the storage engine is flagged beta ([SurrealKV docs](https://surrealdb.com/docs/surrealkv), [release 3.0](https://surrealdb.com/releases/3.0)). The repository's own [SurrealDB inventory](surrealdb-capability-inventory.md) pins v3.2.4 and covers its temporal graph | Vendor documentation and a skeptical third-party review |
| ArangoDB | ArangoSearch analyzers combine tokenization, stemming and normalization; stemming and normalization can be switched off per analyzer; pipeline analyzers chain them; query terms must get the same processing as indexed fields or matches fail ([analyzers](https://docs.arango.ai/arangodb/stable/indexes-and-search/analyzers/)). AutoGraph and AutoRAG were described only by a third-party page | Official docs for analyzers; the AI-suite claims are unverified |
| Letta (MemGPT) | A background "sleep-time" agent rewrites memory blocks between sessions; reported compute savings differ between sources and are unverified ([Letta memory blocks](https://www.letta.com/blog/memory-blocks/)). A 2026 paper says most systems update memory per session with prompted heuristics and lack explicit cross-session consolidation ([arXiv 2605.20616](https://arxiv.org/pdf/2605.20616)) | Vendor and one preprint |
| Cognee | An ontology is optional; without one, node labels and relation names come straight from the LLM. With one, extracted entities are replaced by canonical names and validated. A small controlled set of relation types is recommended to stop one concept fragmenting across edge labels ([Cognee ontologies](https://docs.cognee.ai/core-concepts/further-concepts/ontologies)). The MOSS paper argues against external vocabularies and for an ontology that emerges from the corpus ([arXiv 2607.04391](https://arxiv.org/pdf/2607.04391)) | Vendor documentation; one opposing preprint |
| Soar and CoALA (cognitive architectures) | Separate procedural, semantic and episodic memory, with the boundaries enforced by the architecture. Soar learns by chunking: on an impasse it reasons in a substate and compiles the result into a rule. CogRec asks an LLM at an impasse and chunks the answer ([Laird](https://arxiv.org/pdf/2205.03854), [CogRec](https://arxiv.org/pdf/2512.24113)). A 2026 survey says LLM agents usually keep memory types as tagged text, not as enforced boundaries ([arXiv 2606.30306](https://arxiv.org/pdf/2606.30306)) | Primary for Soar; surveys for the rest |
| Event sourcing | Aggregate state is a left fold over an ordered event log; the same holds for projections ([Kurrent](https://kurrentdb.kurrent.io/blog/projections-1-theory/)). One low-authority page says events can be folded in parallel and combined when state forms a monoid | Mixed; the monoid claim is a sketch |

## What follows for RRFlow

These are inferences from the findings, not facts about the sources, and each needs the owner's decision.

1. **Timestamp assignment is the serialization point.** XTDB's single writer exists because system time must be assigned in order. The owner rejected one writer per engram ([storage direction](../roadmap/storage-and-devassist-owner-direction-20261116.md)). The open design question is whether system time can come from one cheap sequencer at commit (as with group commit) while many writers prepare their work in parallel. No source reviewed shows this done for a bi-temporal store.
2. **A multi-session fold is only parallel if its merge is associative.** A left fold over an ordered log is a catamorphism; splitting it across sessions needs a state whose merge is associative, and, to merge in any order, commutative and idempotent. Whether RRFlow's claims can be given that shape (for example, resolving by the pair of valid time and system time) is undecided. This is the owner's "one thinking is a multi-session catamorphism", and no reviewed source implements it.
3. **Apply the time filter inside the index scan, not after it.** The only measured bi-temporal result found shows recall falling when time is a post-filter. The repository already says an index artifact newer than a read is invalid for it ([index catalogue](../reference/query/index-catalogue.md)); whether the planner filters inside or after the scan is unchecked.
4. **A lexicon is a versioned part of the index identity.** ArangoSearch requires identical processing at index and query time. RRFlow's storage research already lists a "versioned analyzer identity" for BM25 ([storage research](rrflowkv-rust-storage-engine-architecture-research.md)). Whether the lexicon is supplied (Cognee) or emerges from the corpus (MOSS) is a product decision with opposing evidence.
5. **Behavioural state needs enforced boundaries and an impasse rule.** Soar's separation of memory types and its learn-on-impasse rule is the nearest sourced analogue to "learn the techniques to hold behavioural state". Letta's sleep-time agent is the nearest analogue to idle-time crawling. Nothing reviewed describes a state machine driven by telemetry that detects attention shifting and nudges an operator; that part has no support here.

## Gaps

- No ArangoDB capability inventory exists in the repository, only a probe script; the official sources for ArangoDB's AI suite were not read.
- No source pinned or read for Graphiti, XTDB, SurrealKV or Letta. Figures quoted above come from search summaries.
- The Zep paper, the XTDB source and the SurrealKV whitepaper were not read.
- Not read: [build, debug, trace and optimize research](rrflow-build-debug-trace-optimize-architecture-research.md), the nearest record to the owner's Connectome development build with "clear stats, optimizations, tuning across all parts of the engine".
- The engine's own code was not read for `ReadStamp`, claim resolution or the changefeed, so no statement here says what RRFlow does today.

## Next pass (proposed, not started)

1. Pin and read primary sources for Graphiti or its paper, XTDB's temporal design, SurrealKV's whitepaper, ArangoSearch analyzers and Soar; write inventories under the repository's rule.
2. Read RRFlow's `ReadStamp`, claim resolution and changefeed, and produce a read-only gap table against the fold and the lexicon.
3. Read the build, debug, trace and optimize research, and list the per-stage measurements a Connectome development build must show.
4. Put the decisions in the owner's questions below; write specifications only after.

## Questions for the owner

1. Is a single sequencer that assigns system time at commit acceptable, with many parallel writers preparing work, or must every writer assign its own?
2. Does a lexicon come from you and the project (supplied), from the corpus (emergent), or both with project terms winning?
3. What finishes the sentence "State Machine driven by telemetry, metadata, ..."?

## 2026-11-21 follow-up: memory freshness and vectorization

This bounded follow-up read the primary sources below directly, through
November 21, 2026. It does not retroactively qualify the search-summary claims
or resolve the concurrency questions in the original survey. No exhaustive
"latest everything" claim, source-code adaptation or RRFlow benchmark follows.
The papers are research evidence; new November results need independent
replication on RRFlow's workload before adoption or performance claims.

| Primary source and date | Finding read | RRFlow implication and limit |
|---|---|---|
| [Chroma Context Rot](https://www.trychroma.com/research/context-rot), July 14, 2025; introduction and evaluation scope | Controlled evaluations show performance varies with input length and distractors. | Measure selective bounded context against larger context. This does not establish one universally optimal token budget or prove that every contemporary model behaves identically. |
| [LongMemEval v2](https://arxiv.org/abs/2410.10813v2), March 4, 2025; abstract | Evaluates extraction, multi-session and temporal reasoning, updates and abstention, separating indexing, retrieval and reading. | Score those stages independently and include correction/abstention cases; benchmark results do not qualify RRFlow. |
| [RECON v1](https://arxiv.org/abs/2607.16716v1), July 18, 2026; abstract | Tests multi-hop evidence, cascading invalidation, source conflict and temporal constraints, including conclusions that retain independent support. | Test dependency-aware correction beyond replacing one retrieved fact; its domains and corpus are not RRFlow project workloads. |
| [StaleBench v1](https://arxiv.org/html/2611.19752v1), November 15, 2026; abstract, task/results and limitations | Similarity may retrieve obsolete versions; a correct top-ranked result does not rule out stale evidence elsewhere in the context. Entity linking on write and query affects freshness. | Measure stale-context contamination and reader correctness alongside recall. Supplied alias inventories and source validity labels limit transfer to automatic real-world entity resolution. |
| [LEDGER v1](https://arxiv.org/html/2611.20927v1), November 16, 2026; abstract, mechanism and result limits | Separates proposed facts from explicit update/trust/delete rules and evaluates both emitted evidence and answers. | Preserve provenance and governed correction/forget. Its coding workload is synthetic; overlapping confidence intervals on LongMemEval and incomplete ablations prevent universal superiority claims. Do not import its trust hierarchy or runtime. |
| [Qdrant hybrid queries](https://qdrant.tech/documentation/search/hybrid-queries/), current documentation checked November 21, 2026 | Supports dense and lexical/sparse result fusion and multistage scoring. | Use as a retrieval pattern reference, not evidence that hybrid always wins or that Qdrant supplies RRFlow policy/temporal semantics. |
| [Qdrant incremental embedding updates](https://qdrant.tech/documentation/tutorials-operations/incremental-embedding-updates/), current documentation checked November 21, 2026 | Distinguishes unchanged input, changed content, reuse and removal; checks model/pipeline identity. | Adapt the principle of digest-bound incremental work. RRFlow retains bitemporal revisions rather than copying an overwrite-in-place example; a source inventory must be complete before absence can justify deletion. |

### Design decisions carried into existing owners

Temporal limits matter: StaleBench's retention setup assumes in-order arrival;
LEDGER's A4 equates valid-time start with the reporting event. Neither proves
RRFlow's independent valid/recorded-time behavior for late or backdated
corrections. The joined oracle therefore adds those cases explicitly.

- [ADR-0006](../decisions/0006-scoped-policies-and-preferences.md#freshness-and-semantic-retrieval)
  requires complete exact policy resolution. ANN/top-k and summaries may assist
  discovery, never decide whether a mandatory requirement is enforced.
- [Knowledge construction](../reference/data/knowledge-connectors.md#memory-freshness-and-vectorization)
  binds canonical corrections to eligibility, derivation dependencies,
  embedding generations, source stamps and context evidence under one engine.
- [Gate D](../roadmap/rrflow-1.0/gate-d.md#connector-and-knowledge-ingestion-packages)
  and the existing E/H/J owners retain implementation and acceptance. A
  focus/pivot/correct/return journey joins them; no new roadmap is created.

### Keeping the guidance current

For each adopted technique retain the dated primary source/version, the exact
capability it supports, applicability, local baseline and candidate evidence,
cost/quality limits, and its owning gate. Recheck mutable documentation when
implementing, changing a model/schema/lexicon or investigating a regression.
Propose changes through the same governed configuration/projection boundary;
a newer publication is not permission to change defaults automatically.

Research-derived heuristics remain candidates until held-out evaluation earns
their activation. Source revision changes trigger affected checks and rebuilds;
they do not justify embedding the entire corpus again. This is a maintenance
requirement for the owning packages, not an implemented scheduler or guarantee
that documentation, memories or model answers cannot become stale.
