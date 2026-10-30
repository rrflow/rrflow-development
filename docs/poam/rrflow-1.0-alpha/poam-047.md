# POAM-047 — concurrent identical immutable-object publication can be rejected

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-047`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

## Observed deficiency

On Windows, the E-04d integration's concurrent quantization-build test returned
`object store: Access is denied. (os error 5)` for a racing writer. An
independent synchronized probe at the same object-store implementation observed
29 errors in 200 equal-content two-writer trials, and a second 200-trial run
observed 29 errors before the object store's `AfterPublish` hook. The exact
failing filesystem operation was not identified by those observations. The
current publication code checks for the target and later performs a replacing
rename, leaving a concurrent-publication window.

The candidate exclusive-publication repair passes synchronized local tests,
but a winner can become visible before its publication call returns. On
Windows the current directory-sync helper is a no-op, so those tests do not
establish that a losing publisher's acknowledgement survives a crash. The
repair stays draft until that ordering and native durability are addressed or
otherwise supported by appropriate evidence.

The follow-up draft coordinates cooperating publication, quarantine and
reclamation through OS-backed locks keyed by the canonical content shard. A
waiting writer no longer passes the guard until the holder's publication call
returns, and a test kills the holder process before retrying. The lock currently
has no cancellation or configurable wait budget. Windows power-loss durability,
concurrent corruption/reclamation, resource impact and final candidate CI are
not established by those local tests; this record remains open.

## Impact

The immutable object authority can reject a correct concurrent publication,
including a quantization build. The evidence does not establish corruption or
loss of acknowledged state. This affects the OBJ-02 persistent-engine and
E-04c prerequisites and the C-07/J-02 concurrent and native qualification;
[Gate C](../../roadmap/rrflow-1.0/gate-c.md) and
[Gate J](../../roadmap/rrflow-1.0/gate-j.md) own their respective acceptance.

## Owning gates

C-07, E-04c, J-02.

Roadmap owners: [Gate C](../../roadmap/rrflow-1.0/gate-c.md),
[Gate E](../../roadmap/rrflow-1.0/gate-e.md), and
[Gate J](../../roadmap/rrflow-1.0/gate-j.md).

## Closure evidence

Publish through an atomic non-overwriting boundary; recognize a competing
winner only when its exact content is independently authenticated and preserve
all directory-sync and unrelated failure semantics. Exercise synchronized
identical and distinct writers, corruption, fault boundaries and native
Windows and Linux. The candidate must pass the existing racing engine test,
full candidate CI and protected merge-group CI. An implementation or a single
local test does not close C-07, J-02 or this deficiency; the parent ledger
owns status and the named gates own acceptance.

## Evidence

- [Repair pull request](https://github.com/rrflow/rrflow-development/pull/219)
- [Vector integration candidate](https://github.com/rrflow/rrflow-development/pull/190)
