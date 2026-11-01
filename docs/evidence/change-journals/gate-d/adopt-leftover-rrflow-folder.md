# Adopt a leftover `.rrflow` folder in place

**Status:** active implementation evidence; owner-directed import unblocker
**Coordinate:** `rrflow://rrflow-instance/data/change-journal/adopt-leftover-rrflow-folder`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), installed lifecycle ([contract](../../../reference/deployment/installed-lifecycle.md))

## Outcome and baseline

Starting main `f256c8e1`. On 2026-10-31 the repository owner reported that
importing a project from Connectome "fails protected". Reproduced with the
engine bundled in the owner's build (`8d47500`): any project holding a
`.rrflow` folder with ordinary inherited permissions and no locator failed the
no-write preview with
`Storage("private object requires the current owner and a protected non-null DACL")`.
The owner's own checkout had such a folder from September development runs.
The owner decided the folder must be adopted in place, not refused.

## Change

- `rrd-contract`: `InstallationPlan.adopted_estate_entries` (optional, sorted,
  bounded, plain top-level names; `previous` excluded). Plans without it keep
  their established digest; plans with it bind the list into the digest.
- `rrd-engine` planning lists the leftover folder's entries instead of failing;
  admission tolerates an unadmitted non-private `.rrflow` only before a locator
  exists. An installed estate stays strict and is never adopted.
- Apply (new installs only, before the install intent) refuses changed
  entries, makes `.rrflow` private in place when the current user owns a real
  folder (Windows protected owner/SYSTEM DACL via the retained handle; Unix
  `fchmod 0700`), and moves each listed entry with no-replace renames into
  `.rrflow/previous/<plan id>/`. Recovery admits `previous`.
- The superseded test that asserted refusal of `.rrflow/instance.toml` is
  replaced by tests for the new owner-decided behavior.

## Evidence

- `cargo test -p rrd-contract --test attunement_contract`: 20 passed, including
  `adopted_estate_entries_are_bound_only_when_present`.
- `cargo test -p rrd-engine --lib`: 259 passed, 4 ignored, 0 failed, including
  `a_leftover_rrflow_folder_is_adopted_in_place_and_keeps_every_file`,
  `a_changed_leftover_rrflow_folder_refuses_the_stale_plan_without_effects` and
  `an_installed_estate_is_never_adopted`.
- Windows CLI, the owner's failing shape (`.rrflow/dev`, `.rrflow/rrd`, no
  locator): plan lists `["dev","rrd"]`, apply installs, `verify` passes,
  `serve` plus `ready` authenticate. A clean project passes the same flow.

## Not done

- Moved entries keep their original permissions; they were already readable
  as before adoption.
- The install inventory still ignores `.gitignore`; projects with large
  generated folders can exceed its file or byte bound. Separate package.
