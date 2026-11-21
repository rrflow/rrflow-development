# POAM-065 — a physical write batch is capped at 16 MiB and reported only by failure

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-065`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Found by the 2026-11-08 dogfooding run of RRFlow on its own repository against
SurrealDB 4.0.0 and ArangoDB 3.12.13 (harness and results retained outside the
repository at `D:/localdev/devlabs/dogfood-corpus`; RRFlow 9 of 13 by default
and 10 of 13 at maximum limits against 11 and 12). Verified still present on
main `ab32ec16` on 2026-11-11. About 900 KB of mutations reached the 16 MiB physical batch limit, so loader batches had to drop to 800 mutations by trial.

The 2026-11-12 real run of Parse on this repository hit it again: the default parse policy gathered about 16 MiB of fact JSON, the store refused the batch at the 16 MiB write-batch ceiling, and every later Step retried the same batch. A batch's physical size is several times its JSON: 1.34 MiB of parse facts measured 10.5 MiB once the records, their versions, the change log, the per-index source deltas (each carrying the indexed text) and the BM25 postings were counted, a ratio of 7.8 ([journal](../../evidence/change-journals/gate-d/d-02-code-knowledge-slice-20261112.md)).

The 2026-11-21 original parser stack recovery reproduced another admission
regression: a first file exceeding the project's selected `max_commit_bytes`
was published as parsed. The limit constrained gathering only; final measured
admission used the 16 MiB physical ceiling without the smaller selected limit.
The restored original oversized-file oracle fails before the repair. The
[recovery journal](../../evidence/change-journals/repository/archived-parser-stacks-integration-20261121.md)
records its bounded repair and acceptance; SDK bound discovery and the broader
closure requirements below remain open.

## Impact

Batch sizing by trial and error is not a product contract; a loader cannot plan its commits from the capabilities document.

## Owning gates

C-02 transaction admission; D-02 plan P6.

Roadmap owner: [Gate C](../../roadmap/rrflow-1.0/gate-c.md), [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Remediation in progress

- `RuntimeRepository::measure_commit_with_control` stages a commit and its control transitions in a measuring transaction that is never committed and reports the exact physical bytes the payload ceiling admits (`PHYSICAL_BATCH_PAYLOAD_BYTES`). Parse measures each batch before it publishes and trims it to what fits, so its commits are no longer refused. The SDKs and the capabilities document do not yet expose the bound, and the amplification itself is unchanged.

## Closure evidence

Closed when admission evidence names the physical bound before commit and the SDK can size batches from the capabilities document.
