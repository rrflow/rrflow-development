# RRFlow engineering instructions

`README.md` is the bootstrap product entry point and knowledge map. It owns
product identity and current status, and it links to the single owning memory
record for each detailed subject. The RRFlow 1.0 release checklist is owned by
`docs/roadmap/rrflow-1.0.md`. Other Markdown files are supporting design notes,
contracts, evidence, or history and cannot override either owning record.

`AGENTS.md` is the provider-neutral instruction entry point for repository
work. Provider-specific instruction files may import or point to this file, but
must not copy its rules or create another product, planning, recall, or
lifecycle authority. The planning chain is:

1. `README.md` owns identity, invariants, current maturity, and warp points.
2. `docs/objectives/rrflow-1.0-alpha.md` owns the measurable alpha outcomes.
3. `docs/roadmap/rrflow-1.0.md` owns dependency order and completion evidence.
4. `docs/poam/rrflow-1.0-alpha.md` owns the open-gap and remediation ledger.
5. Supporting records explain or prove one of those owners and cannot change
   their status independently.

The product version is frozen at `1.0.0` during pre-release convergence. Do
not use version changes as progress markers. A later version change requires
the repository owner's explicit instruction after the alpha objective, release
gates, optimization evidence, and release decision are complete.

All first-party code that implements the RRFlow engine or a bundled RRFlow
adapter, plus its schemas, bootstrap templates, default configuration,
installation logic, release assembly, verification, and recovery tooling, must
live in this repository.
Do not add a sibling-checkout dependency, escaping local path, Git submodule,
undeclared generator, or install-time/runtime fetch. Locked third-party source
dependencies are permitted build inputs, but the signed default distribution
must contain every linked binary and runtime asset needed to install, start,
persist, recover, and verify RRFlow after the bundle has been acquired.
Connectome, project databases, mesh services, providers, and other external
systems remain optional public-contract integrations and cannot be required
for default RRFlow readiness.

RRFlow is an independently installable and operable reasoning and recall
engine. A project's code or schema generator, build/test/evaluation harness,
CI system, database, mesh, model provider, or development tool may be
discovered during attunement and integrated through a typed, configurable
adapter, but it is never an undeclared prerequisite or a second RRFlow
authority. Discovery does not authorize activation. Installation must expose
fresh-project and existing-project modes; preview the exact versioned
scaffolding, records, adapter bindings, commands, permissions, budgets, and
digests; require explicit configuration and authorization before invoking an
external capability; and make RRFlow-owned integration removable without
damaging project-owned state. Generator output must re-enter the authorized
project-inventory and mutation flow, and harness results are evidence rather
than lifecycle state.

Upstream and experimental Rust implementations are reference inputs, not
architecture. Before adapting code from SurrealDB, Qdrant, Lance, Fjall, or
another implementation, map the useful behavior, algorithm, failure
semantics, and provenance to one canonical RRFlow boundary and its owning
roadmap gate. Adapt it to RRFlow naming, types, transaction authority,
physical model, budgets, and tests. Do not copy an upstream crate tree,
public model, compatibility surface, or hidden runtime and rename it RRFlow.

For project work:

1. Read `README.md` and follow its relevant owner warp point, then inspect the
   current worktree, implementation, tests, and existing diff before changing
   files.
2. Preserve unrelated user changes. Keep each change coherent and reviewable.
3. Use the existing `rrd-engine` composition boundary and provider-neutral
   contracts; do not add provider-specific state or a parallel source of truth.
4. Keep every first-party build, install, and runtime dependency within the
   repository and make optional external integration explicit at the contract
   edge.
5. Apply the independent-engine, project-integration, and source-adaptation
   rules in
   `docs/reference/agent-bootstrap.md#independent-engine-and-project-integration`
   before changing installation, attunement, scaffolding, or an adapter.
6. Journal every bounded package in
   `docs/roadmap/rrflow-1.0-execution-map.md` using its evidence-record
   template before commit. Record the starting revision, complete files read,
   changed paths and behavior, exact commands and results, surfaced failures,
   checks not run, remaining errors, and any roadmap-status change. Add a
   verified new deficiency to the POA&M; do not use chat or a Git commit alone
   as the work record.
7. Verify with the smallest relevant test first, then the owning package suite.
8. Report what actually passed, what failed, and what was not run.
9. Treat existing types, files, compilation, and mocks as implementation
   inventory, not proof. Update objective, roadmap, or POA&M status only with
   the acceptance evidence named by the owning record.
10. Before deleting, moving, merging, or rewriting implementation, update the
   implementation-requirements traceability in
   `docs/roadmap/rrflow-1.0-execution-map.md`. Map the current behavior, source
   modules, characterization tests, canonical destination, owning gate, and
   replacement evidence. Git ancestry, merge status, a path rename, and a
   successful compile are not consolidation proof.
11. RRFlow 1.0 has one current pre-release implementation. Carry reusable
   behavior and tests into their canonical boundary, then remove conflicting
   paths directly. Do not create a parallel fallback, migration, or
   historical-code lane to avoid completing that convergence.

RRFlow has no editor- or provider-owned automatic hooks. Recall, reasoning
lifecycle, and mutation authorization are explicit capabilities composed
through `rrd-engine`; clients must not create a parallel lifecycle authority.
