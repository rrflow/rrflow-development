# RRFlow engineering instructions

`README.md` is the bootstrap product entry point and knowledge map. It owns
product identity and current status, and it links to the single owning memory
record for each detailed subject. The RRFlow 1.0 release checklist is owned by
`docs/roadmap/rrflow-1.0.md`. Other Markdown files are supporting design notes,
contracts, evidence, or history and cannot override either owning record.

For project work:

1. Read `README.md` and follow its relevant owner warp point, then inspect the
   current worktree, implementation, tests, and existing diff before changing
   files.
2. Preserve unrelated user changes. Keep each change coherent and reviewable.
3. Use the existing `rrd-engine` composition boundary and provider-neutral
   contracts; do not add provider-specific state or a parallel source of truth.
4. Verify with the smallest relevant test first, then the owning package suite.
5. Report what actually passed, what failed, and what was not run.

RRFlow has no editor- or provider-owned automatic hooks. Recall, reasoning
lifecycle, and mutation authorization are explicit capabilities composed
through `rrd-engine`; clients must not create a parallel lifecycle authority.
