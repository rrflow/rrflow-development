# POAM-063 — no SDK computes a commit's `operation_sha256`

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-063`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. A client must call `transaction-preview` for every batch to learn the digest it must then commit; previews took 28 s of a 36.5 s load of this repository.

## Owning gates

H-09 SDKs; D-02 plan P6.

## Closure evidence

Closed when every generated SDK computes the digest locally from the catalogue's canonical encoding, verified by the shared conformance corpus.
