# The complete alignment plan is written into the structure record

**Status:** active planning evidence; no code changed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/engine-alignment-plan-20261125`
**Owner:** [Code structure and conventions](../../../architecture/code-structure-and-conventions.md), section 6, and kernel package K13 in the [kernel contract](../../../roadmap/kernel-contract.md).

## Change brief

Baseline `7f16ed4be`. The owner ordered on 2026-11-24 that existing code is aligned before any new code, and on 2026-11-25 that the alignment is planned completely, with no assumptions, before further moves: canonical names, platform terms for files and folders, true component boundaries, clean separate processes with pause and stop, development and trace code captured for the internal build, and patterns a tool enforces with hard errors.

Current behavior: the structure record proposed tiers under the `rrd-*` naming rule and awaited approval; it did not name the component crates, place the engine's modules, or say what is unmeasured. Target: section 6 of that record is the plan. Scope: `code-structure-and-conventions.md`, the naming sentence in `system-overview.md`, this journal. No code. Stop if an objective, roadmap or POA&M status would have to change.

## What the plan fixes

- One naming rule: every crate is `rrflow-<name>`; a component's crate carries its name. This replaces rule R10 and the system overview's `rrd-*` sentence.
- The target folder, tier and name of all 32 crates, with the three-way split of `rrd-store` into `rrflow-kv`, `rrflow-tx` and `rrflow-db`.
- A folder for each of the engine's 54 modules, named for the crate it pulls.
- One layout inside every crate, a `dev/` folder and feature for development code, and a `lifecycle/` folder for start, serve, pause and stop.
- Fourteen packages in order, the proof each must give, and one manifest with one checker to enforce the result.
- Seven things not yet measured, each tied to the package that settles it.

## Measurements

Taken on Windows at the baseline:

- Crate sizes and workspace dependencies: `cargo metadata --no-deps --locked` and a line count of each crate's `.rs` files.
- Module sizes: a line count of each top-level entry under `src`.
- Module dependencies inside `rrd-store` and its `access` layer: a text scan for `crate::<module>` and `super::<module>` paths. No mutually dependent pair was found. The scan does not see glob imports, which the plan states.
- The two reasoning-tree files: their public item names compared, and a line diff ignoring indentation (269 differing lines).
- Lifecycle verbs: a search for pause, resume, quiesce and drain in the engine's serving, instance, registry and control modules and the CLI found one unrelated word.
- Cargo features of the engine, CLI, server, store and kernel read from their manifests.
- 29 tracked source and tooling files still contain "estate".

Not measured: listed in section 6.9 of the plan.

## Verification

- `python scripts/ci/check_documentation.py`: passed.
- `python scripts/knowledge/render_navigation.py`: indexes regenerated.

No objective, roadmap or POA&M status changes. Kernel package K13 already owns repository convergence.
