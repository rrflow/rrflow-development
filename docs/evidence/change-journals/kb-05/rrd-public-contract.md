# KB-05 journal: rrd-public-contract

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/kb-05/rrd-public-contract`
**Owner:** the unchanged historical package receipt identified by its legacy heading and payload digest
**Legacy source:** `docs/roadmap/rrflow-1.0-execution-map.md@b7b061900b67d535ffec2710e6e4c30815d0f216#L1176`
**Legacy payload SHA-256:** `820ba51932cef9d39187b68eea21e1c407b7a2e759c1457a33bb25b76bbbb1c9`

This record refracts one completed package receipt out of the former
monolithic execution map. Its fenced payload is preserved byte for byte.
It reports historical evidence and cannot change roadmap completion or
POA&M lifecycle status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
for acceptance and the [change-journal index](../) for discovery.

## Journal payload

```text
gate/package: A-06 / KB-05 / rrd-public-contract
revision: parent 9c3d50b; result is the commit containing this entry
baseline files/digests: docs/rrd-public-contract.md=9b344bb4926c608fb21f60b6e65fa1f34f0a2b8a7ccbc78654c9d20d6373a1d0; rrd-contract/src/lib.rs=2a9b9c23126bcc7793057d64c2b7fad8937473b2376aeb72d1394c3f22812a19; public_contract.rs=606a771a31deffc083dd37fe693fbaf8139819fdc35cd47d6d92b2a00a274e08; public-contract-v1.json=08095dcd45f54845c3043e3797c60e1d88c7efcaf1fce70575554f3400eccbfa
files read in full: root README; flat public-contract record; knowledge/reference/protocol indexes; protocol server and subscription owners; objective, POA&M, canonical roadmap, and execution map; rrd-contract manifest, lib.rs, capability-surface and SDK-conformance modules, public golden, deployment/SDK corpora, and public-contract tests; engine and server capability builders; Rust client manifest, implementation, and SDK-conformance test
files changed/created/deleted/moved: create docs/reference/protocol/public-contract.md; update protocol index, POA&M, this resolved-review/queue/journal, and generated file inventory; delete docs/rrd-public-contract.md; no Rust, wire fixture, generated SDK, server, client, engine, or runtime file changed
contract or behavior changed: none; the canonical reference now distinguishes exported representation, catalogued exposure, runtime discovery, and actual single-engine acceptance, and maps each contract family to its required rrflowMX/rrflowKV/native-index/rrflowQL/Arrow/DataFusion/reasoning proof
smallest test command and result: cargo test -p rrd-contract --test public_contract --locked — 31 passed before editing and 31 passed after editing
owning package command and result: cargo test -p rrd-contract --all-targets --locked — 58 passed; cargo clippy -p rrd-contract --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrd-engine --test workspace_architecture public_contract_and_client_stay_implementation_free --locked — 1 passed; deterministic inventory check reported 755 records; documentation policy, generated-surface parity at 33 HTTP operations/OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715, workflow policy, frozen 1.0.0 version policy, Cargo formatting, and diff checks passed
failure/crash/differential evidence: the passing baseline did not reject the frozen sample's nonexistent POST /v1/backups/create binding; POAM-015 records that defect plus retired runtime capability labels and alternate successful pre-release branches; no new crash, reopen, engine differential, or resource evidence was created
not run and reason: engine/server/client real-process suites, external SDK conformance, full workspace tests, crash matrices, DataFusion streaming, native-index, reasoning, and deployment qualification do not prove a documentation-only KB-05 classification and remain owned by their named gates
remaining known errors: 23 KB-05 records remain; A-06/A-07 are incomplete; public capability drift in POAM-015 remains; the contract types do not establish persistent rrflowKV, equivalent rrflowMX semantics, streamed Arrow/DataFusion, native graph/BM25/vector execution, persisted reasoning, installation, or cross-surface qualification
roadmap checkbox changed: no
```
