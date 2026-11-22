# POAM-071 — independent writes can still conflict across models

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-071`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On 2026-11-11, a live probe opened two transactions and wrote unrelated rows through the public API. The second commit was refused because both had to publish immediately after their shared begin cursor. The original eight-writer run recorded 370 commits, 1,056 conflicts and 30.8 commits per second, against 51.3 commits per second for one writer.

On 2026-11-22, eight independent sessions writing open-ended records from one snapshot committed eight of eight disjoint records. The baseline committed one of eight under the same driver. Fresh processes read back exactly the acknowledged keys in both runs. The installed test also exercises an overlapping write, historical preview and exact retry. Mixed transactions containing edges or vectors still require a current publication cursor. [Implementation and verification](../../evidence/change-journals/gate-c/disjoint-document-transactions.md)

## Impact

Agents can commit independent document edits. Unrelated mixed-model edits can still conflict. The single-wave experiment measures conflict behavior. Sustained multi-writer throughput and Postgres-compatible transaction semantics still need qualification.

## Owning gates

C-02 owns the transaction primitives.

Roadmap owner: [Gate C](../../roadmap/rrflow-1.0/gate-c.md).

## Closure evidence

Closed when commits conflict only on overlapping data, unrelated concurrent transactions both commit, a lost update remains impossible, and a multi-writer run shows the conflict rate falling on disjoint keys.
