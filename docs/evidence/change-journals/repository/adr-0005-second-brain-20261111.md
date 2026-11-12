# ADR-0005: the second brain's home, decided

**Status:** active decision evidence; no code changes and no gate advanced

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/adr-0005-second-brain-20261111`

**Owner:** [ADR-0005](../../../decisions/0005-second-brain-personal-and-org-engrams.md)

**Starting revision:** `development/main` at `b2d0d722`.

## Owner input, 2026-11-11

- **The component map.** The owner pasted a component map whose "Role (your statement)" column gives the second brain as "Global and user preferences that apply to all projects" and the engram as "One instance per major project, replaces 'estate'". The map's pass order puts the second brain's placement first.
- **Their direction the same day.** Reasoning is "graph reasoning driven with the second brain tuned for global/operato/org preferences depending on their account indie vs enterprise / team rbac based".
- **Their choice.** Asked to choose, the owner picked "A plus an org engram". In the same exchange they confirmed rrflowCLD is the cold tier, as decided earlier that day. They also directed that LFG is "moving it to be a sub repo of that main". Neither is part of this record, and both go to the naming pass.

## Files read

- `docs/decisions/README.md` and ADR-0003's structure
- `docs/architecture/instance-topology.md`, lines 50–125
- `docs/architecture/system-overview.md`, the workspace and LFG rows
- `docs/reference/data/rrflowdb-capability.md`, project links and current facts
- the H-10 row in Gate H
- `crates/transport/rrd-contract/src/deployment.rs:20-29`

## Research

Two research agents read primary sources on 2026-11-11 on layered settings, account tiers, relationship-based authorization, filtering before retrieval, and memory scopes. The ADR cites them.

## Acceptance

`python scripts/knowledge/render_navigation.py` and `python scripts/ci/check_documentation.py`.

## Not established

- Nothing in the ADR is implemented.
- H-10 project links, the resolver, and personal and org engram installation are all still to be built.
- The open questions in the ADR are for the owner.

**Roadmap/POA&M change:** none. Once pull request #357 has merged, a POA&M row for the missing resolver will follow.
