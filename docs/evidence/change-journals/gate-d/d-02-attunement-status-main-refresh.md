# D-02: integrate the public attunement read with the current catalogue

**Status:** active integration evidence; fresh candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-attunement-status-main-refresh`
**Owner:** OBJ-07 / D-02a / POAM-006 / PR146

## Integration and behavior

The existing status-read candidate `5526fe2bb8a5f109883c0deb6b5a4255c35bf6ef`
was built from main `04fae276`. Its linked [implementation journal](d-02-attunement-public-read.md)
records the original first-failure oracle, local tests and limitations.
The owner decision in PR145 has since merged. This continuation ordinary-merges
development main `86adada6dab6411cc58cb9aaa931e35472c0f452` into
[PR146](https://github.com/rrflow/rrflow-development/pull/146), whose brief was
updated before the merge. Main's PR147 added mandatory operation reference
metadata and changed OpenAPI. The 20 conflicts involved that digest, reference
citations, generated SDK/signal projections, and the generated inventory.
Both engine status functionality and current main's catalogue were preserved.

`operation_reference.rs` now describes the existing `attunement-read`
operation under Identity and control and explicitly states its source check is
not performed. All five endpoint generators, the TypeScript schema generator
and the existing signal-projection generator were run. The six current digest
citations and contract constant were updated to the new generated OpenAPI
digest `eb6e2e724814dc1c8465a650310e7b15c96788835c2357f81331de6a2084ffda`.
`rrflow-eval`'s existing synthetic storage-contract refresher updated its three
fixture identities; it did not run a native workload. The source-based
navigation and inventory projections are regenerated for the merged tree.
No mutation control, source scan, current-source guarantee or replan was added.

## Local evidence and limitations

The full `cargo test --offline --locked -p rrd-contract --quiet` suite passed;
the `rrflow-eval` synthetic `storage_contract` target passed 11 tests. The real
installed HTTP/Rust client source-edit and reopen test passed, as did the real
server security test for no session, denied EstateRead and audit readback.
`python scripts/ci/check_generated_surfaces.py --write` and `--check` passed
with 56 operations and seven signal projections. TypeScript's six client tests
and `go test ./...` passed. The engine attunement suite passed nine tests with
its subprocess entry ignored as a direct test; strict Clippy passed for all
targets of the four affected crates. Navigation and inventory regeneration and
checks, documentation and workflow policy, formatting and whitespace checks
passed. The complete code and conflict diff was reviewed. No Python, Java,
.NET or shared SDK conformance result is claimed locally.

This was a routine preservation and regeneration merge, not a source move,
implementation deletion or second authority. The exact refreshed revision
needs its own candidate CI and merge-queue checks; prior CI and local results
for other revisions cannot be transferred. C-07, source revalidation, public
job control, per-prompt planning and memory-effect audit remain independently
open. No roadmap or POA&M status changes in this record.
