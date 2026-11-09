# Distinguish rejected terminal responses from unknown worker outcomes

**Status:** active completion repair; installed and candidate qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-terminal-response`
**Owner:** G-01b/G-02a / POAM-009; PR229; driver: active SWAG session

## Scope and reproduced defect

The pre-edit [PR229 brief](https://github.com/rrflow/rrflow-development/pull/229)
binds this completion repair to clean `e1b5ea465f41fb30096c54dc99fe5b2ad6c6a9b7`.
The common transport mapped every post-dispatch failure to potentially continuing
execution. A new engine regression first failed on a matching HTTP 200 reply
whose generation ended with `finish_reason: length`: rejected output incorrectly
retained unknown execution. The original failure log is preserved locally.

`endpoint_chat.rs` now owns one typed observation: not dispatched, unknown, or
response complete. A bounded, matching single assistant response can establish
completed generation only with `stop`, `length` or `content_filter` and no
tool/function request. Output validation remains strict. The common engine
`endpoint_failure` mapping uses uncertainty rather than dispatch history, so the
existing guarded completion transaction releases only known-completed capacity.
It still records failed output as absent, charges effort and stops the budget.
Lost, oversized, malformed, mismatched and unknown-reason responses retain
uncertainty. Final cancellation checks reject content even after decoding a
completed response. No public engine operation or persisted format changes.

The serialized inference diagnostic now carries typed `outcome` instead of the
ambiguous dispatch boolean. Dispatch history remains a derived Rust accessor;
historical raw diagnostic fixtures are not rewritten. Offline decoding is not
engine authority and cannot clear retained reservations.

## Ownership and research

Review covered repository instructions/README and existing change-authoring/CI
ownership, Gate G's task effort sequence, POAM-009's uncertainty gap, the
inference reference, complete common transport, engine failure mapping,
completion/capacity implementation and worker test helpers. Changes are confined
to those transport/mapping boundaries, their protocol tests, one engine regression
module, this journal/index and the existing reference/POAM detail.
No owner, lifecycle, task budget, provider ranking or model identity is added.

The primary [Chat Completions reference](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create)
defines terminal generation reasons. RRFlow deliberately retains uncertainty for
unsupported effect requests and unrecognized envelopes. This is protocol evidence,
not provider qualification or weight attestation; no upstream algorithm is copied.
Existing bounded logs and digests supply diagnostics. No production content trace
or resource measurement is needed for this correctness repair.

## Acceptance and limits

The engine oracle sends controlled HTTP bytes, checks both the completed-but-
rejected response and a socket closed after dispatch, then reopens the actual
engine store. It checks capacity ownership, missing output, retained attempts,
two spent steps, charged time and the original deadline. These are protocol
correctness fixtures, not fake model results or training/evaluation records.
The transport corpus also covers malformed/oversized envelopes, wrong model,
unknown/nonterminal reasons, tool/function requests, invalid JSON content,
inconsistent usage and an invalid selector choice after completed generation.

Validation logs and original failure remain under
`D:/projects/swag/.swag/evidence/20261031/worker-terminal-response/`.
The initial engine oracle failed as intended (exit 101); its log SHA-256 is
`03a59fe78f29cca6b931bf615964ac480f9c5393c271b2b592ca8f600b01309b`.
After repair, both transport targets pass 15 tests, and the focused worker
module passes eight tests including the new durable completion/loss comparison.
The full engine command `cargo test -p rrd-engine --all-targets --all-features
--locked --no-fail-fast` passes 362 tests with four existing ignored helpers
across 13 targets. Its log SHA-256 is
`9607cd9b9a678d2b39d8adff26b0831f171c32ff9bb2442c96f26718541eaec8`.
The full inference command `cargo test -p rrd-inference --all-targets
--all-features --locked` passes 42 tests, none ignored, across eight targets;
log SHA-256 `bd24a8b1b05d683fabae25cc138ce873071558a41f697909622d48063da748db`.

Navigation generation/check passes with 863 nodes and 3,113 edges. Execution
inventory generation/check passes with 2,307 paths. Documentation policy passes
with 864 statuses and 862 coordinates; whitespace checks pass. The independent
GitHub status query initially had a PowerShell/jq quoting error; a structured
JSON read corrected the observation without restarting CI or changing source.
Final source/diff review retains the declared scope and historical receipt bytes.
Strict `cargo clippy -p rrd-engine -p rrd-inference --all-targets --all-features
--locked -- -D warnings` and `cargo fmt --all -- --check` both pass. The local
validation record binds command arrays, source-file hashes and log digests.
No acceptance check failed after repair. Installed and final candidate checks
are not substituted by these component tests; they remain pending as stated below.

No old estate is cleared and no request is retried. Explicit resume/reconciliation,
complete ordinary-edit UI replanning, governed project effects, RROFLOW outcomes,
reviewed learning and native/container/model qualification remain unfinished.
Earlier installed trials and the Connectome preview retain their original binary
identities. A newly built installed trial is required before claiming this repair
works through the shipped process. Candidate/protected CI and dependency landing
remain required. No roadmap gate closes; version remains `1.0.0`.
