# POAM-053 — named HNSW generations lack an authenticated source identity

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-053`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

On public main `b555d22c891c511775b5e54a9f9d00c8a51da48e`, a real engine
fixture demonstrated two related failures. Sources `(a-b, c)` and `(a, b-c)`
both produced `hnsw-a-b-c`. An empty HNSW generation built for `documents`
could also answer a named search for populated `mirror`, returning no hits
where exact search returned `mirror-point`. The generic HNSW catalogue did
not authenticate the structured source; an empty graph cannot establish it.
Reproduction commands and the proposed repair are recorded in the
[change journal](../../evidence/change-journals/gate-e/e-04-vector-source-identity-20261107.md).

## Impact

An approximate read can silently miss records, or two independently named
indexes can collide. The active candidate binds a versioned source to the
generation's authenticated operation receipt, checks it on publication and
read, and uses a domain-separated structured identity. Existing v2 entries
remain readable but are ineligible for named approximate selection until a
bound generation is built. This is an unmerged candidate, not closure evidence.

The collection and payload-index deletion guards conservatively refuse an old
unbound v2 HNSW name that could refer to the source. This can also refuse a
colliding source, and the current engine exposes quantization retirement but
no corresponding public HNSW retirement operation. Rebuilding therefore does
not by itself make those old guards releasable. A safe authorized HNSW
retirement or other non-destructive resolution must be designed and proven;
do not present the candidate as a complete legacy lifecycle.

## Owning gates

E-04 owns source identity, generation maintenance and vector recall.

Roadmap owner: [Gate E](../../roadmap/rrflow-1.0/gate-e.md).

## Closure evidence

Closure requires the two baseline failures and same-source reopen/replay to
pass at the merged revision, along with publication corruption/authorization
checks, a usable authorized lifecycle for old and new HNSW artifacts, the
owning package suite and candidate and protected CI. Installed
real-project recall and comparative quality remain separate alpha evidence.
