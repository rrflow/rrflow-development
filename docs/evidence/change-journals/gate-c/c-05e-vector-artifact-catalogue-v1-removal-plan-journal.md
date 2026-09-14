# C-05e vector artifact catalogue v1 removal plan journal

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-05e-vector-artifact-catalogue-v1-removal-plan-journal`
**Owner:** the unchanged historical package receipt identified by its legacy heading and payload digest
**Legacy source:** `docs/roadmap/rrflow-1.0-execution-map.md@b7b061900b67d535ffec2710e6e4c30815d0f216#L2602`
**Legacy payload SHA-256:** `75aea64644b47f4acc3a4d6955c595959e2ba3bf881f61a2ecb895ae870f9d85`

This record refracts one completed package receipt out of the former
monolithic execution map. Its fenced payload is preserved byte for byte.
It reports historical evidence and cannot change roadmap completion or
POA&M lifecycle status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
for acceptance and the [change-journal index](../) for discovery.

## Journal payload

```text
gate/package: C-05e / remove the successful vector artifact catalogue v1 reader and alternate digest; planned, no completion claim
starting revision: a905c04156689cebb28f8f3f47dad01ae7759431; tree 0b2b361159cba8620003e2bf50f7d8289a6f57e1
implementation-requirements trace: VectorArtifactCatalogEntry currently publishes v2 but validate explicitly admits v1, forbids only v1 build evidence, and identity_bytes hashes a shorter v1 tuple. Engine catalogue replay deserializes entry_json, calls validate, rebuilds the canonical record, and can therefore reopen a correctly hashed v1 record. The useful semantics are the typed descriptor/object/revision/digest validation and current v2 publication; the second version and digest are not required behavior
baseline files/digests: rrd-vector catalog.rs=ffc615f98ca84dc1aa59abb8f9c06ba5923d65630608d440f52b2ad6cd93120c; rrd-vector runtime.rs=f732e195282b32fbe5cbd5fa351ad09ddd401e5210ed471390c02b7f977a00a6; rrd-engine runtime/vector_catalog.rs=e541160c0d8ea84e0a45c7d5b5820dd90339166ecfe416c711c41cf77e8a7465; execution map=04d59655c187b735b5b4b11eb5a2bd1140764ac8b8df4f5715d1e46433ee3212; canonical roadmap=22d2dd5825f25a87fdc3cbd83a82cd39ca24b8fc2a06404c5cee4eacc2f3c236; POA&M=e3bdac1ee793c8fd1ff5857f8e9420359a4270758164d33b22e6462f4996e589
files read in full before editing: repository instructions; README and C-05 owner/status/POA&M chain; complete rrd-vector src/catalog.rs; complete rrd-engine tests/runtime_vector_artifact_catalog.rs; complete rrflowKV current-format and vector-search references; all constructors, serializers, deserializers, validators, record encoders, public projections, reopen callers, tests, and source hits for VectorArtifactCatalogEntry, contract_version, entry_digest, build_evidence, and the v1 constant/branch. Relevant sections of rrd-engine runtime/vector_catalog.rs, engine/vector/index.rs, and engine/diagnostic.rs were read; none needs behavior changed for this direct removal
baseline characterization: current constructors always emit VECTOR_ARTIFACT_CATALOG_VERSION 2. The reader accepts 1 or 2; version 1 with no build evidence receives a separate six-field identity encoding and succeeds when entry_digest matches it. Current v2 publication is already proven identical on rrflowMX and rrflowKV, and rrflowKV close/reopen reconstructs the catalogue before missing artifact bytes fail closed
planned files changed/created/deleted/moved: change rrd-vector src/catalog.rs to accept only v2 and use one identity encoding; add a focused negative unit regression in that same fully read module; extend the existing workspace architecture guard against revival; update vector-search current state, POAM-003, canonical roadmap C-05 evidence, this journal, and deterministic inventory. No file move, format alias, decoder fallback, physical segment, dependency, API, SDK, provider adapter, version, or release state is planned
planned contract and behavior: a VectorArtifactCatalogEntry is valid only when contract_version equals VECTOR_ARTIFACT_CATALOG_VERSION. identity_bytes always includes build_evidence under one canonical v2 tuple. A forged, internally digest-consistent v1 entry fails before artifact decoding or catalogue replay. Current v2 exact/HNSW publication, object binding, CAS revision, MX/KV equality, and reopen semantics remain unchanged
planned smallest proof: add the exact prior-version rejection test first and run it against the current reader, where it must fail because v1 still validates; then remove the branch and require that test to pass. Run rrd-vector all targets, the rrd-engine runtime_vector_artifact_catalog target including rrflowKV reopen, workspace architecture, strict affected Clippy, and exact source absence. Then run the complete default workspace tests/Clippy/check and repository policies before recording completion
scope boundary: this package removes only vector artifact catalogue contract v1. It does not yet remove missing schema tables, missing RuntimeVector.collection, the second VectorCatalog/TurboQuant suppression path, or current optional build evidence; those remain separately visible under C-05. It does not qualify E-04 HNSW lifecycle/recall, F Arrow/DataFusion, or C-06 hybrid segments
roadmap checkbox changed: no; C-05 remains unchecked and Gate C remains 4/7
```
