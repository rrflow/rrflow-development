# POAM-016 — security initialization and effect-complete authorization

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-016`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Security policy, sessions, and audit have useful validation and denial behavior, but
`rrd-security` still exposes a direct `StorageEngine` repository; policy and runtime data
use separate observations; transaction authorization is operation-wide rather than
effect-complete; missing policy enables anonymous loopback application access; and
authorized/domain/completed audit writes are split commits. Initial security is additionally
a rrflowKV-only `rrd-security-bootstrap` path used by CLI and Kubernetes code: it opens
caller-selected storage before input validation, accepts caller time and arbitrary absolute
secret paths, resolves then reopens files, performs no non-Unix privacy enforcement, derives
idempotency from incomplete coordinates, and lets the development supervisor manufacture a
policy outside installation.

## Impact

Code can bypass `RrdEngine`, a concurrent policy change can race a data read, a permitted
transaction can contain unauthorized graph/index/vector or field effects, an uninitialized
instance can grant application access, a domain mutation can commit without final
audit/outbox evidence, and cold-start races or adapter-specific paths can initialize the
wrong estate, ingest replaced secret bytes, duplicate credentials, or persist an unreviewed
grant set.

## Owning gates

A-07, C-01 through C-04, D-01, D-02, D-06, F-01, F-05, H-04, H-05, H-07, J-01 through J-03,
J-05

## Closure evidence

Make security a pure vocabulary and decision dependency of `RrdEngine`; transact policy,
sessions, state, indexes, audit, outbox, and cursors through one accepted rrflowMX/rrflowKV
boundary; bind authorization to the data stamp; authorize every semantic effect before
commit; and replace both absence-driven access and the standalone bootstrap with D-01's
local-only, fresh-target, exact-plan `initialize_instance` action. Use engine time,
capability-scoped/versioned secret adapters, typed verifiers, one atomic
installed-binding/policy/checkpoint/audit commit, and prepared/effect receipts for
credential delivery; then delete every old helper/caller/shape and pass MX/KV differential,
conflict, effect-boundary crash/reopen, path/link/ACL/provider-drift, denial, redaction,
cross-surface, secret-accounting, and clean-deployment tests.
