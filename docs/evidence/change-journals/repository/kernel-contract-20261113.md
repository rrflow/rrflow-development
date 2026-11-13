# Kernel contract, baseline and competitor probes

**Status:** active planning evidence; no runtime behavior changed

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/kernel-contract-20261113`

**Owner:** the [RRFlow kernel contract](../../../roadmap/kernel-contract.md)

**Starting revision:** `development/main` at `297bd98f`.

**Owner direction, 2026-11-13:**
- Plan RRFlow as the "kernel of development" with schema-driven canonical consistency.
- Run ArangoDB and SurrealDB and see how they actually process the same questions, so that nothing is assumed.
- Replace the competing plans with one. The owner answered "yes" when asked whether the contract should supersede the seven planning records.

**Method:**
- **Code surveys:** four read-only surveys of `main` and Connectome `19ee075c` (endpoints and routing, Connectome visuals, engine triggers, fragmentation), recorded in the [as-built baseline](../../../architecture/as-built-baseline.md). Two of their claims were spot-checked by hand: `is_native_chat_adapter` has no caller, and Connectome routes with wouter.
- **Web research:** adaptive retrieval (Adaptive-RAG, IRCoT, CRAG, Stop-RAG, RouteLLM, semantic-router), code-context evaluation (Cursor, AugmentQA, SWE-QA, CoIR, CodeRAG-Bench), scale norms (Zoekt, RocksDB), System One decision models (Jev, Cloudflare Clef-flash), provider-neutral endpoints (LiteLLM, OpenRouter, Portkey, Cloudflare AI Gateway) and tracing (OpenInference, OTel GenAI). Vendor claims are labelled as such in the contract.
- **[Planner probe](../../planner-probe-20261113/README.md):** ArangoDB 3.12.13 and SurrealDB 3.3.0 in local containers, given the same corpus from this repository and the same indexes and questions, with explain and profile output kept raw.
- **[Multi-project probe](../../surreal-projects-20261113/README.md):** SurrealDB namespace isolation and noisy-neighbour latency, comparing a shared server with a separate process.
- **Plan inventory:** a lossless extraction of the seven superseded records, covering every package, acceptance number, owner decision and open question, plus 22 overlaps and 15 contradictions. Each package appears in the contract's "Carries" column, and each contradiction is resolved in its section 5.

**Changed paths:**
- `docs/roadmap/kernel-contract.md` (new).
- `docs/architecture/as-built-baseline.md` (new).
- `docs/evidence/planner-probe-20261113/` and `docs/evidence/surreal-projects-20261113/` (new, with scripts and raw results).
- The status lines of the seven superseded records, which now carry a Superseded-by link.
- The index rows in `docs/roadmap/README.md`, `docs/architecture/README.md` and `docs/evidence/README.md`.
- One paragraph each in `docs/roadmap/rrflow-1.0.md` and `docs/roadmap/rrflow-1.0/gate-d.md`.
- This record.

**Checks:** `render_navigation.py` and `check_documentation.py` both OK.

**Not established:**
- No package is implemented.
- The probes ran on one shared laptop. Their wall times are not a speed comparison, and 909 files were too few to reach any planner's cost-based switch-over.
- Answer quality has not been measured; the judge model and its spending limit remain the owner's decision.
- The baseline and inventory come from code reading, not from runs.
