# POAM-051 — no public read joins a committed change to the request that made it

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-051`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Found by source review while checking what Connectome's restored Stream and
Activity views can read. `RuntimeChangeSnapshot`
(`crates/transport/rrd-contract/src/lib.rs`) carries the cursor, commit digest,
scope, time, actor, mutation and change digest. `AuditRecordSnapshot` carries the
sequence, time, principal, action, request and operation ids, decision, status
and digests of the request and response. The two share no field, and audit runs
on its own sequence with no link to runtime cursors. A reader can only match them
by time and actor, which is a guess.

This is a source finding. No client has been run against it.

## Impact

A view cannot show who asked for a change, or which changes a request made, as
evidence. Connectome must label any link between Stream and Activity "around this
time" rather than state it. The causal chain H-05 requires cannot cross from an
authorized request to its commit through public records.

## Owning gates

H-05 (one bounded causal chain across ingress, authorization and commit). The
related records are [POAM-016](poam-016.md) and [POAM-049](poam-049.md).

Roadmap owner: [Gate H](../../roadmap/rrflow-1.0/gate-h.md).

## Closure evidence

A public field or read links a committed change to its authorized request, and a
request to its changes, without exposing anything the reader's authorization
withholds. A test through the real engine commits through an audited request and
follows the link in both directions. A request that committed nothing, and a
change with no audited request, each report that plainly rather than matching by
time.
