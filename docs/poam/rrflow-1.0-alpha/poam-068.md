# POAM-068 — `rrflow install status` is documented but absent

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-068`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. The installed lifecycle reference documents `rrflow install status`; the command does not exist in the CLI.

## Impact

Documentation promises an operation the product does not have.

## Owning gates

D-01 installed lifecycle; D-02 plan P6.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when the command exists with the documented output or the documentation is corrected, with a CLI test either way.
