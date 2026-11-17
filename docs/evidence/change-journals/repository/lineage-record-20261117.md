# Lineage record for the earlier product name

**Status:** package journal, 2026-11-17
**Owner:** repository

## Change brief

- **Current behaviour:** the architecture test forbids the earlier product name in every tracked file, so the mapping from that name to rrflowLSM, rrflowQL and the engine could not be written down, and two history records already on a pull request failed the test.
- **Target:** one lineage record that states what was read to confirm the mapping, and a test that forbids the name everywhere except the records it lists.
- **Scope:** `docs/architecture/rrflowlsm-lineage.md`, its index row, and `retired_and_inputs.rs`.
- **Failure oracle:** the name added to any unlisted file fails `retired_pre_release_identity_is_absent_from_the_active_repository`; a listed record that does not name it, or is missing, fails `lineage_records_exist_and_name_the_earlier_product`.

## Result

Both tests pass (`cargo test -p rrd-engine --test workspace_architecture -- lineage_records retired_pre_release`). The record maps crates by role, confirms the storage format by its magics and one evidence file, and lists what is not read. It corrects an earlier guess: the old native application identity is not the snapshot bundle magic.

## Not run

The full workspace suite and the documentation checks beyond those named in the pull request.
