# Second-brain visualizer: evidence review

**Status:** active external-evidence research; not RRFlow architecture, implementation status, or delivery authority
**Coordinate:** `rrflow://rrflow-instance/data/research/second-brain-visualizer-architecture-research`
**Owner:** evidence for the [second-mind program](../roadmap/second-mind-program.md)
**Reviewed:** 2026-11-11

Compiled by one AI research pass on 2026-11-11 and not independently re-checked claim by claim. Verify a source before relying on it for a decision.

Tags: **Strong**, **Moderate**, **Thin**, **Marketing**. "(abstract)" means only an abstract or secondary summary was seen.

## 1. Rendering in a webview

- **Moderate (authors measuring their own system, 2022).** GraphWaGu (WebGPU) held 10+ FPS at 100k nodes/2M edges on an RTX 2060. NetV was the only other library still rendering (3 FPS). Layout plus rendering stayed interactive to 20k/400k, against 2k/40k for D3-force. On an integrated GPU, D3 canvas stayed competitive with WebGL libraries ([paper](https://par.nsf.gov//servlets/purl/10384648)).
- **Moderate.** WebGPU layout up to 69.5x faster, near 100k/6M; layout only (abstract, [Dyken 2025](https://www2.evl.uic.edu/entry.php?id=2900)).
- **Marketing.** cosmos.gl claims "hundreds of thousands" with no FPS or memory figures; it is WebGL2 and needs float-texture extensions ([README](https://github.com/cosmosgl/graph)). Its co-author lists graphs to 475k/1M without timings and warns several million may not fit ([article](https://nightingaledvs.com/how-to-visualize-a-graph-with-a-million-nodes/)).
- **Thin.** Sigma.js stalled at 7k/166k; no hardware given ([Stanford](https://cesta.stanford.edu/news/seeing-shape-knowledge-network-graphs-text-embeddings-and-cosmograph)). Cytoscape WebGL on an M1: 1.2k/16k above 100 FPS, 3.2k/68k at 10 FPS, self-described as subjective ([blog](https://blog.js.cytoscape.org/2025/01/13/webgl-preview/)).
- **Vendor.** deck.gl: 60 FPS to about 1M points, allocation crashes between 10M and 100M; not graphs ([docs](https://deck.gl/docs/developer-guide/performance)).
- **Gap.** No published memory measurement for any graph renderer, and no independent head-to-head benchmark.

**WebGPU.** Windows: Chromium x86/x64 shipped, ARM64 flagged ([status](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status)). The only WebView2 report used a flag, in 2023; No official default-on statement ([Tauri](https://github.com/tauri-apps/tauri/issues/6381)). macOS: default from 26 only, and a forum reply says WKWebView follows that default ([Apple](https://developer.apple.com/forums/thread/770862)); a Tauri maintainer reports it works there, one user saw a dev-build failure (same issue). Linux: upstream `ENABLE_WEBGPU` defaults OFF ([WebKit](https://raw.githubusercontent.com/WebKit/WebKit/main/Source/cmake/WebKitFeatures.cmake)) and GTK options never set it (checked today); treat as absent.

## 2. Drilling down

- **Moderate.** Controlled EEG study, shortest-path only, 25-175 nodes: people struggled past 50 dense or 100 sparse nodes ([Yoghourdjian](https://arxiv.org/abs/2008.07944)). Rendering capacity is not comprehension.
- **Strong but dated.** "None of these approaches is ideal"; research "fails to provide clear guidelines"; distortion "largely failed" to show gains ([Cockburn](https://www.microsoft.com/en-us/research/wp-content/uploads/2008/01/cockburn-ComputingSurveys09.pdf)). Pan-and-zoom beat rubber-sheet focus+context; overviews did not improve performance though users liked them ([Nekrasovski](https://www.cs.ubc.ca/labs/imager/tr/2006/Nekrasovski2006CHI/)).
- **Thin.** Semantic zoom: one small ontology study reports better readability; no timings found (abstract, [Wiens](https://publica.fraunhofer.de/handle/publica/401362)).
- **Moderate, negative.** Bundling reportedly hurt performance, though that study's validity is disputed (secondary, [poster](https://diglib.eg.org/bitstream/handle/10.2312/eurp20181121/025-027.pdf); [Dang](https://ceur-ws.org/Vol-1947/paper05.pdf)). Confluent drawings conveyed connectivity more correctly ([Dwyer](https://research.monash.edu/en/publications/towards-unambiguous-edge-bundling-investigating-confluent-drawing/)).
- **Thin.** Degree-of-interest: no controlled comparison found; Refinery was a 12-researcher usage study ([paper](https://diglib.eg.org/bitstreams/6c3b0f87-18fa-4e02-9061-228ba007907e/download)).

## 3. Showing live change

- **Moderate, conflicting.** Small multiples beat animation on speed for all five tasks; animation made fewer errors on some; mental-map preservation had little effect ([Archambault 2011](https://pubmed.ncbi.nlm.nih.gov/20498503/)). Node-link with auto-animation was quickest and most accurate, playback most preferred; small graphs, 76 student submissions ([Filipov](https://arxiv.org/abs/2208.13716v3)). Preservation helped orientation but can hurt readability (abstract, [2013](https://rd.springer.com/chapter/10.1007/978-3-642-36763-2_42)).
- **Moderate, charts only.** Animated transitions improved tracking ([Heer](https://idl.uw.edu/papers/animated-transitions)).
- **Moderate.** Users missed major changes ([Nowell](https://www.pnnl.gov/publications/change-blindness-information-visualization-case-study)), even ones they fixated (abstract, [Vallières](https://cognitiveresearchjournal.springeropen.com/articles/10.1186/s41235-016-0033-4)).
- **Moderate.** Simple motion beat colour and shape in peripheral detection ([Bartram](https://ccom.unh.edu/publications/moticons-detection-distraction-and-task)); some motion is far more distracting ([2001](https://ccom.unh.edu/publications/moving-icons-detection-and-distraction)).
- **Gap.** No study of an always-on, continuously changing knowledge graph; the studies above use bounded graphs in discrete steps.

## 4. Second-mind cognition

- **Philosophy, not data.** Extended mind faces the coupling-constitution objection ([Adams](https://philarchive.org/archive/ADACS)); Clark's 2025 perspective says outcomes depend on integration ([Nature Comms](https://doi.org/10.1038/s41467-025-59906-9)). No controlled human-AI transactive-memory study.
- **Moderate.** Offloading raised performance and lowered later memory; a goal to remember reduced the cost ([Grinschgl](https://journals.sagepub.com/doi/10.1177/17470218211008060)). Saving helped new learning only when saving was trusted ([Storm](https://www.psychologicalscience.org/journals/psychological-science/0956797614559285/)). The "Google effect" failed replication, contested ([FORRT](https://forrt.org/flora-replication-atlas/doi/10.1126/science.1207745/)).
- **Strong direction, short horizons.** Unguarded GPT: +48% in practice, -17% on the later unassisted exam; a guardrailed tutor avoided harm ([Bastani](https://papers.ssrn.com/abstract=4895486)), one school. 45-day retention 57.5% versus 68.5%, n=120 undergraduates ([Barcaui](https://doi.org/10.1016/j.ssaho.2025.102287)). Worse unassisted performance and more quitting, N=1,222, preprint ([Liu](https://arxiv.org/abs/2604.04721)). Quiz 50% versus 67%, no significant speed gain, follow-up questioning helped; vendor-run ([Shen](https://www.anthropic.com/research/AI-assistance-coding-skills)).
- **Weak.** Observational deskilling, 19 endoscopists, confounded ([Budzyń](https://wrap.warwick.ac.uk/id/eprint/191005)); self-report ([Lee](https://advait.org/files/lee_2025_ai_critical_thinking_survey.pdf)); "Your Brain on ChatGPT" has 18 crossover participants and a published [critique](https://arxiv.org/abs/2601.00856v1).
- **Moderate.** Generative chatbots tripled false memories, persisting a week, n=200 ([Chan](https://arxiv.org/html/2408.04681v1)). AI-edited visuals raised false memories 2.05x ([CHI 2025](https://www.media.mit.edu/projects/ai-false-memories/overview/)), which matters for a hyper-visual memory.
- **Strong: what helps.** Practice testing and spacing rate high, rereading low ([Dunlosky](https://www.aft.org/ae/fall2013/dunlosky)). Constructing concept maps g=0.72; studying them g=0.43 ([meta-analysis](https://summit.sfu.ca/item/17789)). Merely viewing a graph is the weaker case.
- **Thin.** No controlled passive-resurfacing study found; Quantum Country data comes from its creator and uses active prompts ([notes](https://notes.andymatuschak.org/z93QR51f6HAUPLVDxE6KT1T)). Total capture is critiqued by argument ([Sellen](https://www.microsoft.com/en-us/research/?p=647886)). The one second-brain study found is a case study ([Obsidian](https://arxiv.org/abs/2509.20187)). No RCT of any second-brain tool found.

## 5. Tracing reasoning

- **Docs only; no usability data.** Langfuse: Tree, Timeline, and a Graph inferred from timing and nesting, Aggregated by default with an Expanded run view ([docs](https://langfuse.com/docs/observability/features/agent-graphs)). Phoenix: span tree via parent IDs ([docs](https://phoenix.arize.com/docs/ax/instrument/what-are-traces.md)). LangSmith: "full execution tree" ([vendor](https://www.langchain.com/resources/langsmith-vs-arize)).
- **Thin to moderate.** ReTrace, n=18: lower workload, no task-time gain, summary fidelity untested ([paper](https://arxiv.org/html/2511.11187)). A demo reports 56% faster insight, N unstated (abstract, [IJCAI](https://www.ijcai.org/proceedings/2026/987)). InteractiveKG beat text for finding errors, sample unknown (abstract, [record](https://synapsesocial.com/papers/W7151442302)). AgentGraph links graph elements back to exact trace locations (abstract, [AAAI](https://ojs.aaai.org/index.php/AAAI/article/view/42393/46354)).
- **Strong caution.** Models mentioned hints they used only 25-39% of the time, contrived setup ([Anthropic](https://www.anthropic.com/research/reasoning-models-dont-say-think)). Attention is not reliable explanation ([Jain](https://arxiv.org/abs/1902.10186)). KG paths can diverge from model predictions ([Zhai](https://arxiv.org/abs/2310.04910)). Attribution graphs: completeness 0.70, attention formation unexplained ([methods](https://transformer-circuits.pub/2025/attribution-graphs/methods.html)).
- **Mixed.** Forcing functions cut overreliance but were rated worst ([Buçinca](https://arxiv.org/pdf/2102.09692)); cheap explanations can cut it too ([Vasconcelos](https://ar5iv.labs.arxiv.org/html/2212.06823)).

## 6. Memory architecture

- **Documented scopes are logical.** Letta blocks attach per agent, last write wins, no isolation guarantee stated ([docs](https://docs.letta.com/guides/agents/memory-blocks)). Mem0: caller-supplied IDs; "Unmentioned entities are not constrained" ([docs](https://docs.mem0.ai/platform/features/entity-scoped-memory)). Graphiti: "A namespace filter does not replace application authorization" ([docs](https://help.getzep.com/graphiti/core-concepts/graph-namespacing)). Zep keeps superseded facts on a timeline ([paper](https://arxiv.org/abs/2501.13956)) and separates user graphs from shared context graphs ([docs](https://help.getzep.com/v3/graph-overview)). No system documenting physical personal/project separation.
- **Marketing.** Benchmarks are disputed vendor claims ([Zep](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/)).
- **Moderate; preprint and small synthetic setup.** Vector-to-graph expansion leaked (risk up to 0.95) through naturally shared entities, no attacker needed, on synthetic and Enron data; authorizing at the expansion boundary brought it near zero ([Thornton](https://arxiv.org/abs/2602.08668)). Prefiltering protected vector-only retrieval in the same tests. Ungated retrieval leaked in 98-100% of probes; gating added 19ms; three synthetic tenants ([Arceo](https://arxiv.org/abs/2605.05287)). Qdrant tells callers to filter by tenant on each operation ([docs](https://qdrant.tech/documentation/guides/multitenancy/)).
- **Moderate.** Inversion exactly recovered 92% of 32-token inputs from embeddings ([Morris](https://arxiv.org/abs/2310.06816)). Query-only memory injection succeeded 98.2% ([MINJA](https://arxiv.org/abs/2503.03704v1)). Black-box extraction of agent memory also works ([MEXTRA](https://arxiv.org/abs/2502.13172)). Production zero-click injection ([EchoLeak](https://arxiv.org/html/2509.10540v1)).

## Design implications

1. WebGL2 baseline; WebGPU only by runtime detection (§1, absent on Linux).
2. Measure FPS and memory on integrated GPUs before choosing a renderer (§1 gap).
3. Show about 50-100 detailed nodes; aggregate the rest (§2 Yoghourdjian).
4. Pan/zoom with level-of-detail; no default fisheye or bundling on paths (§2 Cockburn, Nekrasovski).
5. Batch change into staged steps with history and playback (§3 Vallières, Filipov).
6. Idle means still; sparing, simple motion (§3 Bartram).
7. Spaced recall and user-built links, not a passive feed (§4 Dunlosky, meta-analysis).
8. Attempt-first mode before AI answers (§4 Bastani; §5 Buçinca).
9. Label observed events versus model narration, and user versus AI-inferred memory (§5 Anthropic; §4 Chan; §6 MINJA).
10. Physically separate stores; explicit promotion into the brain; authorize every hop, with leakage probes (§6 Thornton, Morris).
