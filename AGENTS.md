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
5. Verify with the smallest relevant test first, then the owning package suite.
6. Report what actually passed, what failed, and what was not run.
7. Treat existing types, files, compilation, and mocks as implementation
   inventory, not proof. Update objective, roadmap, or POA&M status only with
   the acceptance evidence named by the owning record.

RRFlow has no editor- or provider-owned automatic hooks. Recall, reasoning
lifecycle, and mutation authorization are explicit capabilities composed
through `rrd-engine`; clients must not create a parallel lifecycle authority.
