# C-07: retained ARM transaction-begin timings

**Status:** active diagnostic evidence; no performance or release gate passed  
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-arm-begin-phase-evidence-20261107`  
**Owner:** C-07g, OBJ-10/11, [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)

The installed 100,000-record ARM characterization at public source `40870e5c5f8f79dfebc7126bf721458ccf789634`, tree `fd041325a1861ad3c7180541c0e563a02ab6b31c`, failed after its warmup; PR #294 retains the full failure and independently verified evidence. Its retained launcher log SHA-256 is `881d53dff39607e36f703a3e87161a378003a471a7e51d5652be0db9550adb47`. The existing test driver emits at most 64 slow write records, hashes their synthetic logical IDs, and records client-observed begin, commit and total call durations. Matching SHA-256 of the retained synthetic IDs to the log yielded:

| Warmup operation | Begin ns | Commit ns | Total ns | Outcome |
|---|---:|---:|---:|---|
| 14 | 296989401 | 11252125 | 308241638 | commit error |
| 18 | 518348497 | 12180444 | 530529053 | accepted |
| 34 | 378090179 | 10264016 | 388354339 | accepted |
| 38 | 500190212 | 16153670 | 516344186 | accepted |

These four are threshold-selected observations, not p99 or a representative latency distribution. The begin interval includes client transport, server admission and execution. It does not identify a slow internal phase. A separately retained analyzer tied two rejected attempts to stale cursors and overlapping acknowledged client intervals; the intervals do not prove commit ordering or cause. The campaign remains failed and does not qualify C-07 or the 24-hour soak.

Code inspection: `RrdEngine::begin_transaction_with_intent` validates and admits through `TransactionGate`, loads/authenticates the session, resolves replay, obtains a `ReadStamp` and persists session/control state. The exact cause of the client begin duration is not yet established. `runtime/trace.rs::record_runtime_trace` itself commits runtime mutations and advances the shared cursor; using it inside a transaction timing investigation can perturb the behavior under observation. The installed CLI already offers opt-in `RRFLOW_TRACE` and stderr JSON. [tracing event enablement](https://docs.rs/tracing/latest/tracing/macro.event_enabled.html) permits avoiding diagnostic work when disabled; [OpenTelemetry sampling guidance](https://opentelemetry.io/docs/concepts/sampling/) explains why sampled diagnostic events are not complete audit evidence. No upstream code was copied.

This is evidence only and changes no engine or performance claim. A proposed follow-up must bind redacted client/server request identity and measure admission wait, session/auth/replay, read-stamp and session publication without changing canonical state, transaction ordering, authorization, deadlines or replay semantics. It must test disabled and enabled operation, success/rejection and installed process correlation, measure overhead, and retain any missed or unavailable attribution explicitly. Preserve #294's backup-lane repair and the original failed ARM run. C-07 and alpha remain open.
