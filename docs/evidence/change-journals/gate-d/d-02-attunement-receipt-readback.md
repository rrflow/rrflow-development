# D-02: read committed attunement control receipts through the engine

**Status:** active bounded implementation evidence; public operation and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-attunement-receipt-readback`
**Owner:** OBJ-14 / D-02a / POAM-040 / [PR181](https://github.com/rrflow/rrflow-development/pull/181)

## Change brief and traceability

Starting public development/main `81fce08e14497bbc280790beefe4666716355aec`,
tree `0d92234da88dc5d4c0f14e15b01e6efa13a11f4f`; bootstrap commit
`28c681e0f11d16e43709e495eac9e81803c3e1c7`. The PR description specified
owner, scope, identity, failure oracles, tests and stop conditions before the
receipt code or tests. It was amended before the POAM edit to include that
record and the correction to the existing module's outdated transport comment;
the comment itself was corrected just before that amendment.

Previously a principal could recover an attunement control receipt only by
resubmitting its mutation. `RrdEngine::read_attunement_receipt` now authorizes
`EstateRead`, derives the same existing principal/job/idempotency-scoped receipt
key, reads the stored receipt without executing the command, verifies its
principal, job, digest shape, original plan and installed estate, and returns the
original status plus operation digest. It resolves the plan named by that
historical receipt rather than the current job's plan. The existing receipt
format, control-WAL/CAS writes, grant and engine authority are preserved. The
key-construction code was factored into one shared function; an independently
constructed expected on-disk key checks equivalence. Authorization can still
append its ordinary audit event. No public operation, background worker, project
inventory, fresh-source check, effect plan, memory receipt or new job state was
added. A future replanning implementation still needs its own end-to-end proof.

The changed implementation is `crates/authority/rrd-engine/src/engine/
attunement.rs`, its exports in `engine/mod.rs` and `src/lib.rs`, and the
existing `engine/tests/attunement.rs` fixture. POAM-040 is updated without
changing its open status. This journal, its generated Gate D index and the
execution inventory are the evidence paths. Nothing is moved or deleted.
Reviewed README, ADR-0002, D-02, POAM-039/040, the change-authoring procedure,
attunement job/plan/receipt control and validation, security authorization and
policy fixture, source diffs and tests. The existing engine/control contracts
and independently computed key provide the research and trace oracles; no
upstream source was adapted and no benchmark or resource improvement is claimed.

## Evidence

Windows x64; rustc 1.99.1; offline locked dependencies; isolated
`target/attunement-receipt-read-build`; dev/test debug info disabled.

- Before the implementation, `cargo test --offline --locked -p rrd-engine
  --lib engine::tests::attunement::attunement_receipt_readback_preserves_original_result_after_reopen
  -- --exact` failed to compile with E0599: `read_attunement_receipt` absent.
- The same focused oracle passed after implementation. Three receipt-specific
  tests pass: original status and digest after later job failure and reopen;
  another authorized principal cannot retrieve the first principal's receipt;
  missing `EstateRead` and invalid credentials are denied; malformed stored
  digest is rejected. The test checks no new attunement receipt is created and
  the live job remains unchanged. It reconstructs the original key independently.
- `cargo test --offline --locked -p rrd-engine --lib engine::tests::attunement::`
  passed 12; the native child entry point is explicitly ignored as a standalone
  test and is exercised by its parent kill/reopen test.
- `cargo test --offline --locked -p rrd-engine --test workspace_architecture`
  passed all 31 tests. `cargo test --offline --locked -p rrd-engine --all-targets`
  passed, with its explicitly ignored cases still ignored.
- `cargo clippy --offline --locked -p rrd-engine --all-targets -- -D warnings`
  and `cargo fmt --all -- --check` passed.

No source moved or semantic behavior was removed. The module comment was
corrected to acknowledge existing public read/cancel; it does not change the
wire catalogue. The initial test-first compile failure is retained above, not
counted as a passing test. Final navigation, documentation, workflow and
inventory checks and full diff review precede the commit. Public HTTP/SDK
receipt readback, source freshness/replan, external provider flow, other native
platforms and exact-head candidate CI are not proved. D-02, POAM-040, C-07 and
alpha status remain open. Checklist: explicit brief and owner; failing oracle;
existing authority/key retained; auth, actor isolation, corruption and reopen
checks; owning suite and lint; linked record and generated navigation; candidate
CI and protected merge still required.
