# RRFlow storage reference

**Status:** active storage-reference index
**Coordinate:** `rrflow://rrflow-instance/data/reference-index/storage`
**Owner:** physical storage contract discovery; linked from `docs/reference/README.md`

This boundary records implemented wire and disk contracts. It does not define
the target architecture, delivery status, or benchmark conclusions.

| Subject | Durable warp | Checkout record | State |
|---|---|---|---|
| rrflowKV current physical format | [`rrflow://rrflow-instance/data/reference/storage/rrflowkv-current-format`](rrflow://rrflow-instance/data/reference/storage/rrflowkv-current-format) | [`rrflowkv-current-format.md`](rrflowkv-current-format.md) | pre-alpha row-segment implementation; not the accepted 1.0 target |
| rrflowKV benchmark harness | [`rrflow://rrflow-instance/data/reference/storage/rrflowkv-benchmark-harness`](rrflow://rrflow-instance/data/reference/storage/rrflowkv-benchmark-harness) | [`rrflowkv-benchmark-harness.md`](rrflowkv-benchmark-harness.md) | implemented storage diagnostics and retained historical-artifact parser; output is not release evidence |
| Tiered persistence | [`rrflow://rrflow-instance/data/reference/storage/tiered-persistence`](rrflow://rrflow-instance/data/reference/storage/tiered-persistence) | [`tiered-persistence.md`](tiered-persistence.md) | local I/O and snapshot/object movement primitives exist; automated hot-to-cold placement and hibernation remain open |
| Logical archive and application backup | [`rrflow://rrflow-instance/data/reference/storage/logical-archive`](rrflow://rrflow-instance/data/reference/storage/logical-archive) | [`logical-archive.md`](logical-archive.md) | resumable logical recovery exists; physical Arrow pages, derived indexes, and a complete authorized operator workflow remain outside this contract |
