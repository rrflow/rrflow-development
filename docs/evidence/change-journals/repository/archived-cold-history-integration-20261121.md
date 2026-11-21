# Archived cold-history integration

**Status:** active integration evidence; cold lifecycle qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-cold-history-integration-20261121`
**Owner:** F-06 derived cold history and E-04 residency diagnostics; alpha recovery prerequisite

## Before-edit brief

Baseline 7651d973f641265f49412c75d4723aba4762047c. Integrate original sources
9dc7d8ac670e5b444106eda8e62af643cae382c0 (staged cold-history publication),
a39cf273ce774c3994101c9edbe8fbb5c7316faa (SDK cold readback), and
e6755305f5582a50fe8b73161239f8ac3c8a3f7d (cold vector residency observations).
The SDK stash has an empty third-parent tree f83fe299ce7d302f588a0e4c6e5178f7431928cf.
The later broad reconciliation snapshots are separate work, not silently covered.

Current engine cold_history/{maintain,state,files,tests}, query cold_history,
store bounded history paging, public ColdHistoryReadResult and all supported
SDK conformance clients already provide derived compressed history and bounded,
authenticated generation reads. Source history remains authoritative. The
README still describes the whole background lifecycle as unimplemented; clarify
the existing derived path while retaining all unproven F-06 requirements.
POAM-004 and Gate F own coverage, reclamation, other audit streams, backup,
resource/latency measurement and hot/cold qualifications that remain open.

Read current client/storage owners, original SDK corpus/readback changes,
original residency trigger implementation and source inventory before the brief.
Read original cold publication and affected conflict regions before resolution.
Keep current clock failure handling, scoped configuration, row/field grant
refusal, receipt-chain/digest/length checks, output and decode budgets, interruption
recovery and generated SDK identity. Preserve current vector observation time;
the old trigger used the query's historical valid time and must not return.

Scope: canonical cold-history, residency/diagnostic and SDK owners if a missing
behavior is found, README's current-status paragraph, this journal/index. No
new format, second implementation, provider or UI path. Failure oracle: engine
cold publication/restart/corruption/admission tests, query compressed history,
store scoped paging, installed configured cold readback across restart, Rust
shared SDK conformance and vector memory-tier timestamps. Then architecture,
docs/navigation and candidate/queue. No model-quality, storage-performance or
visual comparison; no new research/trace mechanism is needed. Stop on an
unmapped source behavior, discarded assertion, invalid generated contract or
red CI. The UI pin is owned by PR478 and must not be staged here independently.
