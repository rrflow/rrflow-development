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
