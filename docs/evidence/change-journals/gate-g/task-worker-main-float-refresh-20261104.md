# Worker draft refresh after JSON float repair

**Status:** active local integration evidence; final base and CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-main-float-refresh-20261104`
**Owner:** OBJ-06 / G-01b/G-02a and POAM-009; PR229

The worker draft head `5185abf90204330619815c99ef30983dac07b923` is
refreshed with public main `5075f49c73518712209cee58a75b6bba835aebcf`
after #263 landed. The PR brief was updated before the merge. Read the
main-to-main diff of `Cargo.toml` and the new contract float test: main enables
`serde_json`'s `float_roundtrip` feature and adds round-trip/context tests.
This changes JSON parsing correctness and may affect digests; a conflict-free
merge is not itself behavioral proof. No source behavior was removed or
replaced by this merge.

The combined locked offline contract exporter produced 79 paths and the
same OpenAPI SHA-256
`f14071c71318062cd2b1a9fca491a16d91b4481e98c3e0bcefeeaf98359ed580`
as the earlier worker reconciliation.
`cargo test --offline --locked -j 1 -p rrd-contract --test json_float_round_trip`
passed both tests and `python scripts/ci/check_generated_surfaces.py --check`
passed. Those checks validate the combined contract and float fixture, not
the complete engine or installed execution. The earlier installed worker
receipt stays bound to source `30d2304c3b45904c5e468f94c03eb23e74d02c26`;
it is not relabeled as evidence for this refresh.

The next acceptance work is the full worker/context and engine suites plus
installed readback on the final combined base, followed by candidate and
merge-group CI. PR275 and PR246 were still in the protected merge queue when
this refresh began, so final main must be rechecked before readying the draft.
No upstream adaptation or new trace authority is introduced; existing
attempt/configuration digests and journaled receipts remain the debug path.
No roadmap or POAM status changes.
