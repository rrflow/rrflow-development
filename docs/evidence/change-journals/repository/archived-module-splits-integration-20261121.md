# Archived module split integration

**Status:** active integration evidence; candidate and merge queue pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-module-splits-integration-20261121`
**Owner:** K13 code structure; alpha recovery prerequisite

## Before-edit brief

Baseline b9f197726ba5d3cacfd7b954445ac93aadc7115b. Integrate saved module splits:
441281439f326c55f05ae73f009549fd665fec21 and
4c83372a70b53b9dcadc26fd55e631f6c6a2af4d (kernel runtime),
c2163a8fc84de80a3a8e18ddf168bede08647aac (HNSW),
577b26dd4b0c67d05e243d36b8ef3973eb249677 (query syntax, TurboQuant, object store),
cf4cefc3af8dc10d08ac6e1f0c3559bfb49c0f13 (public contract).

The kernel and HNSW stashes have untracked third-parent trees, respectively
93a7958888e573e47e8d0b3fdf045f487a46b7e1 (11 module files) and
7716aa2584221dfb0da08a64267dc3aea4497333 (6 module files). Their code must be
accounted for, not just the tracked monolith deletions. Current runtime and HNSW
already use those module boundaries. Source declarations and characterization
cases include bitemporal graph traversal, causal audit/transaction identities,
read-stamp tamper refusal, preview read-your-writes, HNSW exact reranking,
connectivity, bounded filter estimates and scalar/SIMD agreement.

Read source module declarations, third-parent module/type/test inventory,
source direct-read and architecture gate changes, and current contract split
journal before implementation; read remaining conflict regions before resolving.
Canonical destinations are current runtime, hnsw, syntax, turboquant, object
and contract domain modules. Retain current engram vocabulary and generated
catalogue, later storage/read validation, and module-aware source gates.
No aliases, monolithic copies, new runtime paths or source authority are added.

Acceptance: original OIDs receive ordinary merges; current kernel runtime,
HNSW/TurboQuant, query syntax, object and public contract tests plus source-closure
and architecture gates pass. No performance or visual comparison, research,
new tracing or live provider operation is required. Source transfer preflight
finds no known oversized executable ancestry. Stop on unmapped untracked files,
lost tests, changed public behavior, unavailable source, or red candidate gate.
Expected delta is journal/index only if all split behavior is already current.
No roadmap completion is inferred from history. Remaining recovery continues.
