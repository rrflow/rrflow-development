# Naming pass: rrflowLSM, rrflowCLD, engram, the second brain and LFG get canonical rows

**Status:** active terminology evidence; no code changes and no gate advanced

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/naming-pass-component-rows-20261111`

**Owner:** the [system overview's canonical component terminology](../../../architecture/system-overview.md#canonical-component-terminology)

**Starting revision:** `development/main` at `bae0d883`.

## Owner input, 2026-11-11

The owner pasted a component map giving each component's role in their own words:

- rrflowLSM: "LSM tree, started from fjall, tuned for AI and RRFlow reasoning".
- rrflowCLD: "Columnar format, started from Parquet, tuned for AI".
- rrflowQL: "The planning and reasoning graphs".
- LFG: "CPU-burstable classifier, prompt assistance, data points for SWAG".
- Engram: "One instance per major project, replaces 'estate'".
- Second brain: "Global and user preferences that apply to all projects".

When asked, the owner confirmed that rrflowCLD is the cold tier, not the LSM's segment format. They also said LFG is "moving it to be a sub repo of that main".

## Change

`docs/architecture/system-overview.md`:
- new rows for rrflowLSM, rrflowCLD, engram and the second brain;
- rrflowKV now names both tiers;
- rrflowQL carries the owner's statement;
- the LFG row drops "whether a model runs inside it is undecided", defers its internal design to LFG's own repository, and records the move to a submodule;
- the list of exact spellings gains rrflowLSM and rrflowCLD.

Each row's implementation column states what the code does today: hardcoded LSM constants, a CLD that nothing reads, context assembly bypassing rrflowQL, the half-applied rename, and an unimplemented second brain. Each links its POA&M item or plan package.

## Not done here

- **The rename itself.** The other rows of this document still say "estate"; the rename is POAM-078 and is its own package.
- **The LFG submodule.** It needs a URL a clean clone can fetch without credentials; LFG is private on the owner's account. It also needs the engine to be shown working on `main` first.
- **The owner's LFG decision dated 17 October.** It is in no repository record. The owner directed that it be recorded in LFG's own repository.

## Acceptance

`python scripts/knowledge/render_navigation.py` and `python scripts/ci/check_documentation.py`.

**Roadmap/POA&M change:** none.
