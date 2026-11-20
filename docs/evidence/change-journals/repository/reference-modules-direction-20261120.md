# The reference-module direction recorded, and one wrong line corrected

**Status:** active change-journal evidence; records only, one new POA&M item, no code

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/reference-modules-direction-20261120`

**Owner:** [reference modules and the reasoning fabric](../../../roadmap/reference-modules-and-reasoning-fabric-owner-direction-20261120.md)

**Starting revision:** main `33159f8e2`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage; [kernel contract](../../../roadmap/kernel-contract.md) K10.
- **Current behaviour.** The records said Vortex is the product format of rrflowCLD. No record said what a reference module is, and no POA&M item owned the product's Parquet dependency.
- **Target behaviour.** The owner's words of 2026-11-20 are recorded; Parquet, Vortex and Fjall are stated as reference modules with five conditions; the engine is mapped against the fabric direction; POAM-090 owns the Parquet deficiency.
- **Scope.** The new direction record, POAM-090 and its ledger row, and corrections in the 2026-11-19 direction record, the kernel contract and the system overview.
- **Failure oracle.** `python scripts/ci/check_documentation.py`.
- **Stop conditions.** No roadmap status changes.

## What was read

The Parquet facts were read on 2026-11-19 by a pass over `rrd-query`, `rrd-engine` and the install configuration, and the dependency lines were re-read today. The context-plan and contract facts come from the same day's pass over `rrd-contract`. The Vortex crate's version, licence and Rust version come from `cargo info vortex@0.93.0`.

## Evidence

| Command | Result |
|---|---|
| `python scripts/knowledge/render_navigation.py` | indexes regenerated |
| `python scripts/ci/check_documentation.py` | passed |

## Not proven

- Nothing was built or run.
- Whether Vortex builds on Windows, Linux and macOS under this workspace's pins.

## Roadmap and POA&M

POAM-090 added, High, Open. No other status change.
