# POAM-049 — an audit read's journal scan grows with the journal's length

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-049`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Found by source review during [POAM-030](poam-030.md) (pull request #144).
`SecurityRepository::audit_since` (`crates/authority/rrd-security/src/lib.rs`)
limits the audit records it returns, but not the control-journal entries it
examines. It keeps fetching pages of `limit` control entries until it holds
`limit` audit records or reaches the tail, so a long run of other control
entries makes the work of one read grow with the journal's length.

The public `AuditPage` has a continuation sequence but no completion flag. A
scan budget that returned an empty, nonterminal page could be mistaken for the
end of history, so a fix needs a compatible continuation and completion
contract plus chain validation.

This is a source finding. No performance incident has been measured.

## Impact

`audit-read`, `audit-export` and `diagnostics-read` have no bound on the work
of one page when the journal holds few audit entries, so the record limit does
not bound their resources. POAM-030's fix neither causes nor worsens this.

## Owning gates

H-05 (complete redacted causal evidence, including bounded audit read and
export). The related records are [POAM-030](poam-030.md) and
[POAM-016](poam-016.md).

Roadmap owner: [Gate H](../../roadmap/rrflow-1.0/gate-h.md).

## Closure evidence

One audit read examines at most a stated number of control entries and reports
through an explicit continuation and completion contract whether it reached the
tail, so an empty page cannot be mistaken for the end of history. A test over a
journal with a long run of non-audit entries proves the bound, and paged read
and export still return the same chain prefix as an unbounded read at every
page size.
