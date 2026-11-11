# The one-engine plan gains reasoning cycles and ephemeral traces

**Status:** active planning evidence; no package implemented and no gate advanced

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/one-engine-reasoning-cycles-plan-20261111`

**Owner:** the [one-engine joins plan](../../../roadmap/one-engine-joins-engineering-plan.md); follows [the joins plan journal](one-engine-joins-plan-20261111.md)

**Starting revision:** `development/main` at `abc29393`.

**Owner direction, 2026-11-11.** "properly look at phoenix dev tool… QL is suppose to do cycles and grow and grow in one session depending on the effort slider. We get the 8-12 second marker be the target for our baseline default". The owner also asked for these:
- user preferences honored, the owner's own included;
- a replayable reasoning graph;
- ephemeral traces that keep the last N in dev mode;
- a layman customer view, with developer and advanced views one drill-down away.

## Change

The plan's package table gains J11 and J12, and section 5 gains decision 4 and new research.
- **J11, reasoning cycles scaled by effort.** It sits under G-05 and runs inside the existing durable task budget.
- **J12, ephemeral traces and replay.** It builds on J1.
- **Decision 4** maps the owner's words onto the existing effort contract. It does not create a second one.

No code changes.

## Files read in full

- the joins plan
- `docs/reference/inference/lfg-swag.md`, from "Inheriting effort from the calling AI" through "Public task effort flow"
- `docs/roadmap/rrflow-1.0/gate-g.md`, from the aggregate reasoning-budget acceptance through the public effort flow

## Files read in part

- `docs/roadmap/rrflow-1.0/gate-g.md`: the G-05 rows
- `crates/authority/rrd-engine/src/engine/reasoning_budget.rs`: the module documentation and public items
- `crates/authority/rrd-engine/src/engine/context.rs`: searched for any cycle, escalation or iteration; none exists

## Research

A research agent read primary pages on 2026-11-11, and section 6 cites them:
- Arize Phoenix and OpenInference;
- vendor effort controls;
- ReAct, IRCoT, FLARE, Self-RAG, Adaptive-RAG and compute-optimal test-time scaling;
- anytime algorithms;
- OpenTelemetry tail sampling and Go's flight recorder.

Phoenix is licensed ELv2. So J12 makes OpenInference export an optional, off-by-default adapter, and nothing is bundled.

## Acceptance

`python scripts/knowledge/render_navigation.py` and `python scripts/ci/check_documentation.py`.

## Not established

- The 8 to 12 second Balanced target is the owner's stated target. It is not yet measured, and decision 4 requires it to be calibrated on a fixed question set.
- The Phoenix replay scope, model-call spans only, comes from Phoenix's documentation. It was not tested.

**Roadmap/POA&M change:** none; G-05, H-06 and D-06 keep their status and order.
