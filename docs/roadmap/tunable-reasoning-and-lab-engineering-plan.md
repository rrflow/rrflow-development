# Tunable reasoning and RRFlow Lab: engineering plan

**Status:** active proposed engineering plan; no package below is implemented
**Coordinate:** `rrflow://rrflow-instance/data/work-package/tunable-reasoning-and-lab`
**Owner:** supporting plan for J-04 ([Gate J](rrflow-1.0/gate-j.md)), D-06 ([Gate D](rrflow-1.0/gate-d.md)) and G-05 ([Gate G](rrflow-1.0/gate-g.md)); it extends the [one-engine joins plan](one-engine-joins-engineering-plan.md), the [D-02 attunement plan](d02-evidence-driven-attunement-engineering-plan.md) and [ADR-0005](../decisions/0005-second-brain-personal-and-org-engrams.md)

This plan cannot change the order or status set by the [README](../../README.md), the [alpha objective](../objectives/rrflow-1.0-alpha.md), the [roadmap](rrflow-1.0.md) or the [POA&M](../poam/rrflow-1.0-alpha.md). It joins work already planned into one tunable engine and one Lab, and names the terms and methods it uses from the research cited in section 8. Code claims were read at `development/main` `181b0e67` on 2026-11-12.

## 1. The owner's direction, 2026-11-12

- RRFlow is "the culmination of folding in all the optimizations and micro optimizations under one fully configurable reasoning engine", taking the best of others "while having our own LSM and CLD be the critical aspect that allows it to do deep reasoning highly fast".
- Configuration must be "layman friendly" and let people "adjust and be able to attune the engine to their needs".
- The guidance for graph reasoning must be tunable, and people must be able to influence it.
- The Lab is available during development and optional in the main package: included or left out.
- Use the AI field's terminology and approaches that research shows succeed, and continue established work.

## 2. Terms

| Term | Meaning here | Where it comes from |
|---|---|---|
| Setting | One typed, named configuration value with a unit, range, default and reload class | TiKV `online_config`, PostgreSQL `pg_settings` |
| Preset | A named set of settings chosen as one; Quick, Balanced, Thorough or Custom for effort | Vespa rank profiles, pgtune, model effort settings |
| Effort | One control that expands into budgets: reasoning cycles, traversal width and depth, rerank window, deadline | OpenAI `reasoning.effort`, Anthropic `effort`, J11 |
| Guidance | Everything that steers graph reasoning: seeds, personalization, relation weights, pins and exclusions, width and depth | HippoRAG 2, Think-on-Graph, Neo4j GDS |
| Personalization vector | The weights a Personalized PageRank walk restarts to; a user or policy shifts them to steer | HippoRAG, Neo4j GDS `sourceNodes` |
| Route | The choice between flat retrieval and graph reasoning for a question | Adaptive-RAG; the 2025 RAG versus GraphRAG comparison |
| Fusion | Combining ranked lists: reciprocal rank fusion, or a tuned convex weight | Cormack et al. 2009; Bruch et al. 2023 |
| Explanation | Each result's score broken into what each path and setting contributed | OpenSearch hybrid score explanation, `pg_settings` sources |
| Experiment, run, sweep | One configuration tested; one execution; a search over settings | MLflow, LangSmith, Weights & Biases |
| Pareto front | The configurations no other beats on every objective: quality, latency, tokens | Optuna multi-objective |
| Safe region | Settings predicted to stay above a quality floor, seeded at the defaults | SafeOpt, Bao |
| Tuning, validation and sealed test sets | Separate question sets so a tuned configuration is not judged on what it was tuned on | nested cross-validation; the reusable holdout |

## 3. What exists and what this plan joins

| Part | Exists or planned | Gap this plan closes |
|---|---|---|
| Effort presets and reasoning cycles | J11 and J12 planned; the effort contract exists (`reference/inference/lfg-swag.md`) | Effort expands into cycle limits only; it must also set guidance budgets |
| One planner and fusion | J6 planned; context assembly hardcodes RRF constants (`engine/context.rs:10-14`) | Fusion settings and their explanation |
| Graph reasoning | Adjacency and `traverse:` exist; D-02 P4 plans a PageRank-ranked repository map; context expansion is snapshot BFS (`engine/context.rs:686-788`) | A guidance profile users can steer, and a route between flat and graph |
| Settings after install | Immutable ([POAM-077](../poam/rrflow-1.0-alpha/poam-077.md)); no vector, BM25, fusion, embedding or graph section | One typed tree that a governed plan and apply can change |
| Where a value came from | The ADR-0005 resolver, not built | Explanation of every setting's source and every result's score |
| Vector path costs | Priced from measurement since #363 ([POAM-080](../poam/rrflow-1.0-alpha/poam-080.md)) | The cost model joins the settings tree |
| LSM and CLD | Constants hardcoded (J9); no read reaches CLD (J8) | Their settings join the tree; the Lab tunes them one at a time |
| The Lab | Approved layout under `crates/evaluation` (#369, #370); vector comparisons only | Knob ranking, sweeps, multi-objective tuning, and packaging |

## 4. Design

### 4.1 One typed settings tree

- One tree with a section per contributor: `storage.lsm`, `storage.cld`, `vector`, `lexical`, `fusion`, `graph`, `reasoning`, `recall`.
- Every setting declares its type, unit, range, default, reload class (live, next-open, or rebuild) and measured-impact rank. The tree is generated from one owner into the schema, SDKs and Connectome, like every operation.
- Values resolve through ADR-0005's levels: run, then project, then user, then org, then core. Policy ceilings can only tighten. Every resolved value reports its source and reason (`DEFAULT`, `INHERITED`, `OVERRIDE`, `CLAMPED`, `LOCKED` or `REFUSED`).
- A change is a governed `configure-plan` previewed with its diff and effects, then `configure-apply` with a receipt. This is the operation POAM-077 and `deployment.rs:643-645` name and do not have.

### 4.2 Guidance for graph reasoning

A guidance profile is a section of the tree. A request may override it within policy.

- **Seeds** come from linking the question to the graph through lexical and vector hits. HippoRAG 2 links queries to relations rather than only to entities, which added 12.5% recall@5.
- **Personalized PageRank** spreads relevance from the seeds. Its damping and seed weight per node type are settings. HippoRAG 2 used damping 0.5, and its passage seed weight moved MuSiQue recall@5 from 77.9 to 80.5.
- **Relation weights** set how strongly each relation type carries relevance, for example `references`, `defines`, `imports` and `contains` from D-02 P4.
- **Pins and exclusions** let a person or policy force or forbid nodes, files or domains. They work like Bao's exclusion rules.
- **Width and depth** bound each reasoning cycle's expansion. Think-on-Graph's beam search defaults to width 3 and depth 3, and retuning them recovered quality on an incomplete graph.
- **Route.** A measured router sends each question to flat retrieval, one graph step or multi-step reasoning. A 2025 comparison found flat retrieval better on single-hop questions, graph reasoning better on multi-hop ones, and selecting between them better than either.

Every answer carries its explanation: the seeds, each path's contribution to each result, the settings used and their sources.

### 4.3 Effort, the single control

- Effort stays the existing contract. Quick, Balanced, Thorough and Custom expand into sealed limits (J11), and Balanced targets 8 to 12 seconds by measurement.
- This plan adds the guidance budgets to that expansion: width, depth, rerank window and route threshold. Compute allocated by difficulty was more than four times as efficient as best-of-N in the test-time compute study.

### 4.4 Fusion and reranking

- The default is reciprocal rank fusion with k = 60, which needs no labels. Its constant is a setting and is recorded with each plan (J6).
- Once an engram has labelled questions, the Lab proposes a convex weight per engram. Bruch et al. found a tuned convex combination beats RRF in and out of domain, with few labels.
- An optional rerank stage reorders a fixed window. Its window is an effort budget.

### 4.5 The tuning loop in the Lab

1. **Rank settings first.** Measure each setting's effect one at a time, holding the rest at defaults. On Cassandra, 5 of 155 settings reached 99% of the best configuration (Kanellis 2020).
2. **Search the top settings,** five to ten of them, with constrained multi-objective Bayesian optimization (quality, latency, tokens). Search only inside a safe region seeded at the defaults.
3. **Judge on held-out sets.** Each engram keeps tuning, validation and sealed test question sets. A proposal passes only if it beats the current default on the sealed set by recall@k, nDCG and answer correctness, without regressing latency or tokens beyond a declared tolerance.
4. **Propose, never write.** The Lab's output is a `configure-plan` proposal that a person accepts, never a direct write. Accepted proposals become presets that attunement can choose from.

Rules for the loop:

- **LSM and CLD settings join the same loop** once J9 makes them settings. `rrflow-lab-tuning` links `rrd-lsm`, `rrd-vector` and the CLD tier directly for component measurements, as the evaluation rules allow.
- **Answer quality needs a judge.** Faithfulness and answer correctness are judged by a reader and judge model. The repository records that none may run until the owner chooses one (D-02 plan).

### 4.6 Layman-first configuration

Most users never customize, and defaults decide outcomes. So:

- Attunement chooses a measured preset for each project.
- People see one effort control, and a quality-versus-speed choice where it applies.
- Raw settings sit behind an advanced view, which shows each value's source and what changing it would affect.
- Every result's explanation is one click away.

### 4.7 The Lab is optional

- **Development:** `cargo run -p rrflow-lab` and the Connectome Lab routes are always available in a checkout.
- **Release:** J-03's bundle gains an optional Lab component, holding the `rrflow-lab` binary and its definition packs. The default distribution leaves it out unless the person installing chooses it.
- **Engine boundary:** the engine never depends on Lab code (the #370 rule). The engine exposes the operations the Lab needs: settings read, explain and plan/apply; diagnostics; J1 traces; J12 replay. The Lab composes them.
- **Connectome:** it loads the Lab's routes on demand and shows them when a Lab is reachable. Connectome renders; the Lab and the engine do the work.

## 5. Packages, in order

| Package | Work | Depends on | Acceptance |
|---|---|---|---|
| T1 | The settings tree, resolver explanation and governed `configure-plan`/`configure-apply` | ADR-0005 resolver; closes POAM-077 | A live setting changes on a running installation without reinstall, its explanation names its source, and a refused value names its policy |
| T2 | Fusion, guidance and route sections, read by J6's planner; snapshot BFS and hardcoded RRF retired | J6, D-02 P4 | Context results equal the planner's at one stamp, each result carries its explanation, and changing a relation weight changes the ranking as predicted on a fixed fixture |
| T3 | Effort expands into the guidance budgets | J11, T2 | On a fixed question set, more effort widens and deepens guidance until measured evidence quality flattens |
| T4 | The Lab's crates move into `crates/evaluation/lab` and `qualification` | #369, #370 | Each move keeps every test; the evaluation rules hold |
| T5 | Knob ranking and sweeps in `rrflow-lab-tuning` and `rrflow-lab-experiments`, over the T1 tree | T1, T4, J1 | A ranking of every live setting by measured effect, reproduced twice within a declared tolerance |
| T6 | Multi-objective tuning in a safe region, held-out gates, and proposals as `configure-plan` | T5, owner's judge choice for answer quality | A proposal beats the default on a sealed set without a declared regression, or the report says it does not |
| T7 | Connectome Lab: file-based routes, `features/lab`, the report renderer registry, and the settings view with presets, advanced mode and explanation | T1; Connectome router migration | A setting changed in Connectome takes effect through `configure-apply`, and a Lab report renders from data with no new screen |
| T8 | The optional Lab component in J-03's bundles | J-03, T4 | A bundle built with and without the Lab installs and runs RRFlow identically; with it, `rrflow-lab` runs against the installed engine |

J1's per-layer timing comes before any tuning, because nothing can be tuned that cannot be measured. T1 comes before T2 to T7, because every one of them reads the tree.

## 6. Owner decisions

1. **The Lab runs as its own RRFlow process, outside the engine** (recommended), driving the engine through public operations. The alternative is Lab operations inside the engine. This refines the 2026-11-03 direction that the engine does the work and Connectome is the UI: RRFlow still does the work, in a process the default distribution can leave out.
2. **The reader and judge model for answer quality, and its spending limit.** This is open since 2026-11-03. Until it is chosen, T6 tunes retrieval metrics and latency only, and says so.

## 7. Not in this plan

- Learned routers or rerankers trained on RRFlow data. They come after T6 has labelled sets.
- Community summaries in the style of GraphRAG. They are a possible later guidance level, after T2's measurements.

## 8. Research

- HippoRAG 2, personalized PageRank, seed weights and ablations: <https://arxiv.org/html/2502.14802>
- HippoRAG: <https://papers.nips.cc/paper_files/paper/2024/hash/6ddc001d07ca4f319af96a3024f6dbd1-Abstract-Conference.html>
- Think-on-Graph: <https://arxiv.org/abs/2307.07697>; ToG-2: <https://arxiv.org/html/2407.10805>
- RAG versus GraphRAG, 2025: <https://arxiv.org/html/2502.11371v2>
- GraphRAG: <https://arxiv.org/abs/2404.16130>; dynamic community selection: <https://www.microsoft.com/en-us/research/blog/graphrag-improving-global-search-via-dynamic-community-selection/>
- Adaptive-RAG: <https://arxiv.org/html/2403.14403v2>; Self-RAG: <https://arxiv.org/abs/2310.11511>; CRAG: <https://arxiv.org/html/2401.15884v3>
- Test-time compute: <https://arxiv.org/abs/2408.03314>
- Effort settings: <https://developers.openai.com/docs/guides/reasoning>, <https://platform.claude.com/docs/en/build-with-claude/effort>
- Reciprocal rank fusion: <https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf>; convex fusion: <https://arxiv.org/abs/2210.11934>
- Neo4j GDS PageRank controls: <https://neo4j.com/docs/graph-data-science/current/algorithms/article-rank/>
- Bao: <https://ar5iv.arxiv.org/html/2004.03814>; OtterTune: <https://www.cs.purdue.edu/homes/csjgwang/CloudNativeDB/OtterTuneSIGMOD17.pdf>; knob ranking: <https://www.usenix.org/conference/hotstorage20/presentation/kanellis>; LlamaTune: <https://arxiv.org/abs/2203.05128>
- SafeOpt: <https://mlanthology.org/icml/2015/sui2015icml-safe/>; Optuna multi-objective: <https://optuna.readthedocs.io/en/stable/tutorial/20_recipes/002_multi_objective.html>
- Vespa rank profiles: <https://docs.vespa.ai/en/ranking.html>; progressive disclosure: <https://www.nngroup.com/videos/progressive-disclosure/>; customization study: <https://www.lri.fr/~mackay/pdffiles/CHI91.Triggers.pdf>
- Overfitting in model selection: <https://www.jmlr.org/papers/v11/cawley10a.html>; reusable holdout: <https://research.google/blog/the-reusable-holdout-preserving-validity-in-adaptive-data-analysis/>
- MuSiQue: <https://ar5iv.labs.arxiv.org/html/2108.00573>; SWE-bench: <https://arxiv.org/abs/2310.06770>; CodeRAG-Bench: <https://arxiv.org/html/2406.14497v2>
