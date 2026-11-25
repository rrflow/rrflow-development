# Prompt flight readback in Connectome

**Status:** active candidate evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/prompt-flight-devtools-readback`
**Owner:** H-05 / H-06 prompt evidence for POAM-032

Starting revision `b10cd07b0372f1601de44f20ef816d94f02c3d0d` with Connectome
`a54f2f92d28145dda6f94a2445f1197d22517cbb`. On this revision the internal
launcher builds rrflow.exe and stages it under target/dev-loop. Its installed
Observe operation wrote a rank-two document edge with a rank-one claim ahead
of it. The public query traversed that edge. Flight detail returned HTTP 500
both before and after process restart. Local observations are under
`rrflow-flight-main-2e7c2780eb564eeca338ff9124516246` in the temporary directory.

The change is bounded to engine-owned stamped flight readback, the pinned
client projection and the shared flight screen. Use the existing storage
port and remaining read budget. Keep packet rank, flight identity and the
original context stamp. Charge detail-read I/O to that later operation.
Observe-mode execution supplies no model-provider measurement. Verify the
same live graph, flight and replay after restart, then inspect the development
desktop. No storage format or controller change is required. The case advances
the prompt evidence prerequisite without closing the gate.

## Change

`read_flight_versions` named each use edge by `flight_id.rank` for ranks 1 to
`use_count`. A packet item that is a claim takes a rank and gets no edge, so
those names skip numbers and the read found fewer edges than the record
counts. The read now takes the flight row's outgoing use edges by adjacency at
the same stamp. The context-assembled event also keeps the packet's read
counters.

## Verification

Added on 2026-11-25 after merging `main` at `7f16ed4be`, on Windows:

- New test `a_flight_whose_context_ranks_a_claim_among_records_reads_back_whole`
  commits two notes and a claim, launches a flight and requires the use ranks
  to have a gap and the flight to read back equal.
- Against `main`'s read path the test fails with `prompt flight ... has 1 edges
  at this stamp but records 2`. With the change it passes.
- `cargo test -p rrd-engine --lib --locked -- engine::tests::flight`: 7 passed.
- `cargo test -p rrd-engine --lib --locked`: 532 passed, 0 failed, 12 ignored.
- Not run: the installed launcher path that first showed the HTTP 500, the
  Connectome flight screen, Linux, macOS.

No objective, roadmap or POA&M status changes; POAM-032 stays open.
