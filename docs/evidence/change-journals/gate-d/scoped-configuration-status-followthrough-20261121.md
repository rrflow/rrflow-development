# Align configuration status in architecture discovery

**Status:** active documentation correction; runtime implementation unchanged
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/scoped-configuration-status-followthrough-20261121`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), PR456 configuration reference consistency

At candidate `10658d7b1d648aeb5f5704e53f0ed26c51f11f29`, a final cross-reference
search found two active discovery rows still describing the complete global
binding and resolver as unimplemented: `docs/decisions/README.md` and the user
policies row in `docs/architecture/system-overview.md`. Those rows contradicted
the bounded implementation described in the
[foundation journal](scoped-configuration-foundation-20261121.md).

The correction updates only those two rows to name numeric query/recall
admission and local user-source binding, link the implementation reference and
retain broader obligations, project editing and UI gaps. ADR-0005's historical
description remains historical. No runtime, schema, generator or test changes.
The PR brief named this scope before mutation. The decision index was read in
full; the architecture overview's entry map and component table were inspected,
with the complete two-row diff reviewed. No new research, trace or resource
change is needed; stop on a claim beyond the installed evidence.

The failure oracle is textual: active discovery must not state that the entire
binding/resolver is absent while the owning implementation reference describes
the shipped numeric subset. The targeted search across README, architecture,
decisions, reference, Gate D and POAM-077 found these two stale active rows;
historical ADR wording is not rewritten. Navigation, documentation policy and
whitespace checks pass before commit. No roadmap gate or POA&M item closes.

Candidate [run 44880027465](https://github.com/rrflow/rrflow-development/actions/runs/44880027465)
passed all selected jobs, including the repaired inventory, engine tests,
strict lint, dependent compilation and six-language live SDK conformance.
Queue revision `dc50f16b37b91af6f7efc6f676da336a9aa030b7` matched that candidate's
tree exactly, but was still running when these stale rows were found. It is
superseded by the documentation correction; no completed queue result is
claimed for it. The corrected revision must pass fresh candidate and queue
gates before landing.
