# Project map for imported projects

**Status:** active implementation evidence; owner-directed import experience
**Coordinate:** `rrflow://rrflow-instance/data/change-journal/project-map`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), installed lifecycle ([contract](../../../reference/deployment/installed-lifecycle.md#project-map))

## Outcome and baseline

Starting main `6c069e14`. On 2026-10-31 the repository owner asked that
importing a project automatically create a mapping file, use it, and evolve it
over time, at `.rrflow/project-map.md`. Before this change installation wrote
only `bootstrap.md`, the locator, credentials and storage; nothing described
the project in plain language.

## Change

- `installation/inventory.rs`: `project_entries` lists admitted ordinary files
  and folders using the same compiled inventory policy and Git inputs as the
  version-three precondition, without reading content.
- `engine/project_map.rs`: `RrdEngine::refresh_project_map` renders a
  deterministic map (kind, size, main languages, start-here files, folder
  table), carries forward the owner's notes section, and writes through a
  pending file only when the content changed.
- `rrflow install apply` and `rrflow serve` refresh the map and print one line
  naming it; a failure is reported and never blocks.
- Recovery admits `project-map.md` and its pending file below `.rrflow`.

## Evidence

- `the_project_map_describes_the_project_and_keeps_the_owners_notes` checks:
  - install writes the map, and Git-ignored folders and the lease file are
    excluded;
  - the owner's notes survive a refresh;
  - an unchanged project is not rewritten;
  - an added file updates the folder table and still keeps the notes.
- Unit tests cover notes extraction, damaged-map recovery and number/size
  formatting.

## Not done

- The map is not yet indexed as a recall record or shown in Connectome.
- Language detection is by file extension only.

## Correction, 2026-11-11

At main `ab32ec16` `RrdEngine::refresh_project_map` has no caller: the `install apply` and `serve` hook this record describes is absent from `crates/adapters/rrflow-cli/src/installed.rs`, and the test `the_project_map_describes_the_project_and_keeps_the_owners_notes` is not in the tree. The map is therefore not produced by the shipped product on this revision. Re-wiring it as a projection of the committed inventory records is package P2 of the [D-02 evidence-driven attunement plan](../../../roadmap/d02-evidence-driven-attunement-engineering-plan.md).
