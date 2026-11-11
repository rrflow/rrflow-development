# POA&M: verified gaps from the one-engine code audit

**Status:** active POA&M evidence; no code changes and no gate advanced

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/poam-one-engine-audit-20261111`

**Owner:** the [RRFlow 1.0 alpha POA&M](../../../poam/rrflow-1.0-alpha.md)

**Starting revision:** `development/main` at `de435238`.

**Owner direction, 2026-11-11.** The owner said the engine "was never using QL from the start". They want read, write, modify, add and delete to be canonical and consistent as one engine, with clean files and folders so each contributor can be tuned and optimized.

## Change

- **New entries.** POAM-075 to POAM-079 are added.
- **POAM-072 widened.** It gains the code cause of its live-probe finding, and its scope now covers vectors and geo values as well as relations.

Findings already owned elsewhere were not duplicated:
- POAM-071 covers serialized writes.
- POAM-052 covers fixed context fusion.
- POAM-005 and POAM-048 cover vector and native access.

## Evidence

A read-only code audit was run on 2026-11-11 at `44d8f5f0`. I then re-read these items at `de435238`:
- the retire branch at `semantic_commit.rs:650-668`;
- that the only reader of `index_source_deltas_since` is a test;
- the context constants at `context.rs:10-14`;
- the MCP schema limits against `default-estate-configuration-v3.toml:50-52`;
- the configuration's immutability comment at `deployment.rs:643-645`;
- the cold-tier default against the shipped file;
- the count of 30 mixed-case `engram` identifiers, and that `ResourceKind::Engram` has no uses;
- the counts of 115 `RrdEngine::open(` and 29 `open_installed(` call sites.

## Acceptance

`python scripts/knowledge/render_navigation.py` and `python scripts/ci/check_documentation.py`.

## Not established

- No probe was run; these entries come from reading code.
- Counts are `grep` matches, which include definitions and comments.

**Research decision:** none. **Roadmap/POA&M change:** five new entries open; POAM-072's scope is widened; no status changed.
