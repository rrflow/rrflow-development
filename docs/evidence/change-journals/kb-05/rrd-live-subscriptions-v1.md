# KB-05 journal: rrd-live-subscriptions-v1

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/kb-05/rrd-live-subscriptions-v1`
**Owner:** the unchanged historical package receipt identified by its legacy heading and payload digest
**Legacy source:** `docs/roadmap/rrflow-1.0-execution-map.md@b7b061900b67d535ffec2710e6e4c30815d0f216#L1194`
**Legacy payload SHA-256:** `1e95528dacb90b24196ed5a18c6bb8039bda50a1be465ead779a7e16e021b28d`

This record refracts one completed package receipt out of the former
monolithic execution map. Its fenced payload is preserved byte for byte.
It reports historical evidence and cannot change roadmap completion or
POA&M lifecycle status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
for acceptance and the [change-journal index](../) for discovery.

## Journal payload

```text
gate/package: A-06 / KB-05 / rrd-live-subscriptions-v1
revision: parent 369f8fe; result is the commit containing this entry
baseline files/digests: docs/rrd-live-subscriptions-v1.md=f54ea77b13fa4c615f2486d268b313783020ee35b3f9eb64d2bcc7f2b97260d4; unchanged implementation anchors verified by SHA-256 before editing
files read in full: flat subscription record; protocol index and server owner; query live reference; engine subscription implementation/tests; query live implementation/tests; server subscription handler/WebSocket adapter; prior complete contract/client reviews reused only after unchanged hashes and relevant contract, client, loopback, and mutual-TLS spans were revalidated
files changed/created/deleted/moved: create docs/reference/protocol/subscriptions.md; update protocol index, query cross-links, server evidence row, execution map, and generated file inventory; delete docs/rrd-live-subscriptions-v1.md
contract or behavior changed: none; documentation now separates implemented durable delivery from B-04 multiplexing and H-03 commit-impact work
smallest test command and result: cargo test -p rrd-contract durable_subscription_contract_bounds_retention_backpressure_and_stream_shape --locked — 1 passed
owning package command and result: cargo test -p rrd-engine subscription --locked — 5 passed; cargo test -p rrd-query --test live_query --locked — 2 passed
cross-boundary command and result: cargo test -p rrd-client --test real_server --locked — 3 passed; documentation, generated-surface, inventory, workflow, version, and formatting checks run after editing
failure/crash/differential evidence: existing engine corpus passed durable close/reopen and replay; existing query corpus passed rrflowMX/rrflowKV semantic-delta comparison; the first inventory regeneration failed because its tracked-path input reads the Git index and the intended deletion was not staged, so only this package was staged before a clean rerun; no new runtime evidence created
not run and reason: full workspace tests, SDK conformance, crash matrix, and release qualification are not substitutes for a documentation-only KB-05 classification
remaining known errors: dedicated heartbeat-polled subscription socket is not B-04 multiplexing; live query still materializes two snapshots instead of H-03 commit-impact evaluation; complete trace and generated-SDK qualification remain open
roadmap checkbox changed: no
```
