# Inventory reference and integrated lint follow-up

**Status:** active draft evidence; OBJ-14, D-02, D-03 and C-07 remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-inventory-reference-correction`
**Owner:** PR #225, dependent on #224.

Starting local merge revision `ff8a88c9dc4837d732cdd656726ec948d69270a7`
contains readable-plan source and #224 Inventory source. The prior integration
record describes the change brief, reviewed implementation and limitations.
The user-facing lifecycle reference also said that only Connect executes, even
though the integrated engine dispatches the separately authorized Inventory
step and #224 supplies installed Inventory evidence. Correct the current
reference to name Connect and Inventory; other steps remain unavailable.
Clarify that the POA&M's Connect-only sentence describes its earlier candidate,
not a new completion claim. No objective, roadmap or gap status is changed.

After the source integration, `cargo fmt --all --check` passed and
`cargo clippy --offline --locked -p rrd-contract -p rrd-engine -p rrflow-cli
--all-targets -- -D warnings` passed on the isolated Windows target. The tested
source, runtime traces and retained executable hashes are in
[the integration record](d-02-readable-inventory-integration.md); this
documentation-only follow-up changes no runtime behavior. No new upstream
source, external effect, performance claim or native qualification is implied.
The documentation changes and generated index require their policy checks;
complete changed-file review, #224's protected landing and #225's own
candidate/merge-group CI remain outstanding.
