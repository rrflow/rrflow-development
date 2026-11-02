# LFG advisory transport toward engine-owned selection

**Status:** active intermediate implementation; engine integration and acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/lfg-engine-admission`
**Owner:** G-01b/G-03, POAM-009

Starting main `e6f3632b32f1c9be33c1951886a9d9a2be648ddf`, tree `85c2e6e27c77e0dc02abc7c31c509e4fa2655bdc`; [draft PR #249](https://github.com/rrflow/rrflow-development/pull/249). This intermediate work is not completion evidence for Gate G, C-07, or alpha. PR #246 is queued for its required merge-group check; PR #235 is a separate pending outbound-observation change. Neither establishes engine intake of LFG observations.

## Current and target

Current main constructs an engine-owned selection packet containing task and endpoint allowance but no LFG observation. LFG implements `lfg.observation.v1`; SWAG has an explicit local MCP client. This package targets optional, explicitly configured engine-admitted analysis, bounded and source/configuration-bound packet construction, durable output, invalidation and replay, and controlled selection. No caller extension or model observation grants authority. The existing proposal/attempt authority and tests remain the canonical destination. Source mapping, admission, persistence and all failure-boundary integration remain to be implemented and reviewed; no existing path was removed or relocated.

The current implementation adds `lfg_observation.rs` and an optional `endpoint-chat`-gated `lfg_transport.rs` to the existing `rrd-inference` boundary. The decoder accepts only supported v1 advisory semantics and expected request/input/assessment/rules identities, retains known fields, and bounds the encoded response. The transport requires an explicitly supplied literal loopback origin, configured byte/time limits, pinned digests, and caller-provided network/deadline controls. It disables redirects and proxies; it reports a possibly continuing external effect after dispatch. Loopback is an intentional current restriction because the examined LFG service does not authenticate callers; it does not authenticate another local process. This is not engine authorization, an installed integration, model dispatch, real SWAG routing, or a provider identity attestation. The decoder and transport do not persist or log prompt text. Unknown v1 fields are discarded, not treated as accepted new semantics.

The captured fixture contains a synthetic request identity and actual response from an isolated LFG binary, SHA-256 `9e6b6bd30af82c94f1d2997b11aa6d08e7bfbdcfe895cbc65f3f3f6ccaab1558`. Its checked-out source was `1dc7e710fcc76a136d073a5d97d450b34d634338`; this test did not establish that the preexisting executable was built from that revision. The current-main Windows RRFlow source build produced executable SHA-256 `185e4eb9ab1819b02c4d1a6815a11d9619da529b93d291ff7c5a06bafec21c34`. SWAG checkout was `22558aa3b2f211854c8e5df86af02e0829e5432f`. A disposable fresh RRFlow installation and separate LFG service were contacted by two independent calls from one SWAG MCP process: authenticated RRFlow readback and LFG advisory response both passed. RRFlow was not sent that observation and SWAG did not route from it. The local executable characterization and report are retained under `target/rrflow-lfg-swag-proof-20261102/` (not release or CI evidence).

## Review, research, and diagnostics

Read README, AGENTS, Gate G and POAM-009, the LFG/SWAG and independent-engine references, change-authoring and traceability; relevant sections of endpoint profile/policy, engine task proposal/reservation, inference endpoint-chat and execution control; actual LFG observation v1 implementation and SWAG client. Large shared files were reviewed by relevant sections rather than exhaustively; the new adapter and tests were reread in full. LFG protocol values describe its v1 wire contract; no upstream code was copied. No remote authentication or model-quality claim follows from a loopback response. Response and request byte digests and elapsed time are available to a future engine receipt; no telemetry or memory write was added. Full-file review and engine security/admission analysis remain required.

## Evidence and failures

- `cargo test --locked --offline -p rrd-inference --test lfg_observation`: **2 passed**, including real captured response, partial/inconclusive and identity/invalid-value cases.
- `cargo test --locked --offline -p rrd-inference --features endpoint-chat --test lfg_transport`: **3 passed, 1 ignored** by default; the ignored real-service test requires explicit service configuration.
- Explicitly launched a separate native LFG process, then ran `cargo test --locked --offline -p rrd-inference --features endpoint-chat --test lfg_transport real_lfg_service_returns_bound_advisory_data -- --ignored --exact --nocapture`: **1 passed**. The first service start exited before readiness; stdout/stderr were suppressed, so cause is unknown. A diagnostic launch and explicit retry succeeded. No claim of a diagnosed cause.
- `cargo test --locked --offline -p rrd-inference --features endpoint-chat`: package suite **passed**; the separately invoked real-service case is ignored in its normal run.
- `python -m unittest discover -s tests -p test_swag_lfg.py -v` in the separate SWAG checkout: **4 passed**. Those tests include mocks; the distinct real-process call above provides separate wire evidence.
- `git diff --cached --check` initially found CRLF line endings in the captured JSON fixture; converted those line endings to LF and repeated the check. The parsed fixture values are unchanged.
- `cargo clippy --locked --offline -p rrd-inference --all-targets --features endpoint-chat -- -D warnings`: **passed**. `cargo fmt --all --check` and navigation check passed.
- `cargo build --locked --offline -p rrflow-cli --bin rrflow` at the starting main revision: **passed**. The disposable two-call connectivity check described above passed.

Remaining: extend the existing provider-neutral semantic/endpoint contracts without introducing LFG-owned authority; bind the analyzer configuration and selected task input; reserve/revalidate before LFG and controller calls; preserve failures, observation and provenance in the engine-owned receipt; test stale inputs, authorization changes, interruption/restart/replay, disabled mode and actual controller consumption. Full inference/engine suites after further edits, generated projections, installed end-to-end behavior, candidate CI and merge remain open. No roadmap or POAM status changes.

## Checklist

- [x] Selected engine integration boundary and recorded proposed PR brief before source edits.
- [x] Tested a bounded adapter with actual LFG v1 response and a separately launched service.
- [ ] Complete provider-neutral engine contract, explicit binding and controlled attempt integration.
- [ ] Prove the real installed RRFlow-to-LFG-to-controller flow, stale denial, interruption/restart and audit readback.
- [ ] Complete full-file/diff review, generated checks, final package evidence and candidate CI before ready or merge.
