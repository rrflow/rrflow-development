# H-04 draft attunement Rust-client gap

**Status:** active verified gap; implementation and conformance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-attunement-rust-client-gap`
**Owner:** OBJ-14 / H-04 / POAM-011; [PR202](https://github.com/rrflow/rrflow-development/pull/202)

At draft source `8f91d6801dab8c35c4bc9291ff75248ec819867b`, the [public lease recovery package](../gate-d/d-02-public-lease-recovery.md) exposes eight additional authenticated engine operations through the HTTP catalogue and generates five language endpoint maps. Inspection of `crates/transport/rrd-client/src/operation.rs` found existing typed attunement read, cancellation and control-receipt read methods, but no typed methods for start, step, pause, resume, expired-lease recovery, replan preview, replan accept or replan receipt read. A search for those eight operation names in that Rust client file returned no matches. The PR brief recorded this gap before this POAM and journal edit.

This is not a claim that the five endpoint maps or the HTTP server are unusable. It is a missing Rust-client boundary and missing live multi-language conformance for the new actions. The existing [generated-surfaces owner](../../../roadmap/rrflow-1.0-execution/generated-surfaces.md) already states that the internal Rust client is not yet generated from the full operation model. H-04 must generate and validate the internal bindings from the executable catalogue, preserve existing response/identity validation and prove the new operations against the installed engine; adding a handwritten parallel operation registry would not meet that requirement. No engine implementation or gate status changes in this record.

Reviewed the PR202 contract/HTTP additions, Rust client `operation.rs`, its `rrflowdb_local` real-server tests, generated-surfaces owner and POAM-011. No external research, runtime trace or resource measurement was necessary to establish the missing symbols; actual runtime conformance remains unrun. Regenerate navigation and the Git-backed inventory, check documentation and the staged diff. Full candidate and protected merge-group CI remain required.

- [x] Record the verified missing boundary under its existing POAM owner without inventing SDK support.
- [ ] Implement catalogue-owned Rust bindings and prove the new operations and response identity across real installed execution.
