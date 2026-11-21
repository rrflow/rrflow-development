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

## Integration and replacement evidence

Original sources were merged sequentially, without rewriting their history:
9dc7d8ac into 00da10c7, a39cf273 into 8bce969a, and e6755305 into
7f01fdcd. The empty SDK third-parent tree was inspected explicitly. Green
development main through PR478 was joined in 52eefe09. There is no engine,
SDK, or test-code delta in this package; this is source-history integration
with a correction to the current-status prose, not a new implementation claim.

The source publication implementation was read through its planning, staged
file, receipt, head, digest, and restart boundaries. Its canonical destination
is engine/cold_history/{maintain,state,files,tests}. Current bounded source
paging lives in store/repository/runtime/snapshots.rs and access/runtime_state.rs.
The old installed-lifecycle monolith maps to
tests/installed_lifecycle/engrams_and_history.rs. Current module splits, typed
operation catalogue, deployment boundaries, generated SDK identities, and
centralized configuration admission were retained. A duplicate old Parquet
development dependency was not reintroduced.

Read original cold_history.rs publication/control sections (260–644 and
761–918), current maintain.rs (1–220), state.rs, module imports, affected
conflict regions, original SDK corpus/client changes, and the complete original
residency-trigger/search/diagnostic change. This is a bounded source review,
not a claim to have read every line of every affected monolith.

Preserved current publication invariants include foreground-transaction
exclusion, alternating native/cold maintenance, pending-quantum coalescing,
scoped row/field authorization, length/digest/receipt-chain checks, read and
decode budgets, and differentiated deadline/resource failures. The source's
old vector trigger used historical query valid-time; current
engine/vector/search/operations.rs instead uses observed_at_unix_ms. The
memory-tier test explicitly rejects historical time as observation time.
Residency counters remain observations, not proof of a fully qualified CLD
storage implementation. The cold path remains derived Parquet history.

README and POAM-004 now distinguish existing derived publication and bounded
readback from the still-open retention, reclamation, stream coverage, recovery,
and performance qualifications. No gate or POA&M item was closed.

## Verification

Commands used this checkout with CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target
and CARGO_BUILD_JOBS=4, after all three source merges:

- cargo test -p rrd-engine --lib cold_history --locked: 14 passed, including
  process termination at each publication boundary, prepared restart,
  corruption, configuration changes, authorization refusal, and bounded failure.
- cargo test -p rrd-query --test cold_history --locked: 1 passed.
- cargo test -p rrd-store --test cold_history_paging --locked: 1 passed.
- cargo test -p rrflow-cli --test installed_lifecycle
  configured_cold_history_publishes_and_reads_after_process_restart --locked:
  1 passed, 16 filtered.
- cargo test -p rrd-engine --lib
  engine_vector_memory_tiers_are_physical_bounded_and_restart_safe --locked:
  1 passed.
- cargo test -p rrd-server --lib maintenance --locked: 3 passed.
- cargo build -p rrd-client --example sdk_conformance_server --profile test
  --locked, then cargo test -p rrd-client --test sdk_conformance --locked --
  rust_sdk_passes_shared_real_daemon_conformance --exact --ignored --nocapture:
  1 passed against the owned real daemon. The existing runner's private
  workspace, manifest staging, and manifest-read helpers supplied the fixture;
  the daemon was signalled to shut down in a finally block. The emitted marker
  was SDK conformance OK: language=rust with corpus SHA-256
  bfab8094980fce9cbe8d41476fe4780018f2eda2b65471027916fba49537e245.
- cargo test -p rrd-engine --test workspace_architecture --locked: 35 passed.
- python scripts/ci/check_documentation.py and git diff --check passed.
  An initial architecture invocation named a nonexistent Python script; it
  failed without running tests and was corrected to the Rust target above.

The all-language SDK preflight failed because mvn is unavailable locally.
Other SDK languages were not run locally and no success is claimed for them.
The Rust-only driver invoked the canonical Rust command; it did not replace
other language commands with empty commands or weaken the shared corpus.
Logs are under D:/.usr/Temp/rrflow-project-policy-20261121/cold-recovery-*.log.
Candidate and merge-queue checks remain the landing gate. No live UI, provider,
full cold-lifecycle qualification, performance comparison, or visual comparison
was run. Broad reconciliation snapshots remain separate recovery work.
