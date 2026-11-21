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

## Resolution and verification

Ordinary source merges: 37602fda and 2e74f30f (kernel), 50378e51 (HNSW),
3ddf13fa (query/quantization/object), 1d379dad (public contract). The third-parent
source is reachable through its original stash, and its canonical files remain
tracked in the current tree. No source or test delta remains versus green main.

Kernel third-tree files are runtime/{audit,data_snapshot,encoding,graph,ids,
model,projection,stamps,tests,transaction,validate}.rs. HNSW third-tree files are
hnsw/{config,graph,index,score,tests,validate}.rs. All map to the same current
module paths, with current read/graph and numerical validation retained.
Query syntax maps to syntax/{ast,parse,parser}; quantization to turboquant/
{codes,math,query,tests}; object persistence to object/{local,store,tests}.
The incoming architecture monolith maps to current workspace_architecture
storage/support modules; their module_source traversal preserves checks across
split files. Direct-read allowances still name runtime/tests.rs.

Contract conflicts were current engram naming, typed operation markers and
explicit read/keyed-mutation retry semantics, scoped-configuration admission,
identifier tokenization, centralized digest code and generated OpenAPI identity.
All remain current. The incoming former-name authority file maps to current
engram_authority.rs and was not reintroduced. The original split journal was
already present; its historical acceptance is not this run's result.

Reads were source module declarations and third-tree type/test inventories,
source direct-read and architecture changes, current module-aware storage
rules, the original split journal, and the contract conflict blocks. This is
not an exhaustive line-by-line review of every module. The named tests below
establish the selected behaviors, not performance parity or complete feature
qualification.

With CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target, CARGO_BUILD_JOBS=4 and
--locked on every cargo command:

- cargo test -p rrd-core --lib runtime: 9 passed.
- cargo test -p rrd-vector --lib: 86 passed, 1 ignored.
- cargo test -p rrd-query --lib: 21 passed.
- cargo test -p rrd-store --lib object: 15 passed.
- cargo test -p rrd-contract: 207 passed across the package targets.
- cargo test -p rrd-engine --test direct_read_source_closure --test
  workspace_architecture: 1 + 35 passed.

Logs are module-splits-{core,vector,query,object,contract,architecture}.log in
D:/.usr/Temp/rrflow-project-policy-20261121. No live provider, performance,
visual comparison or whole-engine qualification ran. No ignored test was newly
ignored. No roadmap status changed. Current module suites and selected source
boundaries passed; unselected package tests are not claimed.

Change-authoring checklist: PR476 brief before source merges; untracked source
trees explicitly enumerated; reusable behavior retained at canonical owners;
newer guards/catalogue preserved; focused oracles executed; read scope and
limitations recorded; no extra trace/research/resource mechanism; regenerated
index after joining PR475. Candidate and merge queue still govern landing.
