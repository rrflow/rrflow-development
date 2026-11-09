# C-07g proof-read native outcome

**Status:** active immutable failure evidence; full-size setup incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-proof-read-native-outcome`
**Owner:** OBJ-02 / C-07g-h / POAM-035 / PR134

## Baseline and brief

PR134 published the outcome-record brief before these edits. Starting repository
head is `d0c446df1d7244212fe56e9daff8c82ec98d6d4b`; driver is the current Codex
session. The [launch record](c-07g-proof-read-native-preflight.md) froze actual
workload source `05817485c1b083e4bf64eeb902314f47eac5f4c0`, tree
`1fac463ab180bddebed45b99d4f0b09fe16e2574`, binary and allocation identities.
This successor records its terminal outcome without modifying the old receipt,
source, failed estate, workload, original setup/operation deadlines or gates.

## Actual result and retained identity

Read-only inspection found the workload service FAILED with exit status 1.
The driver acknowledged 100000 corpus operations and independently verified
38960 before `corpus setup exceeded its frozen deadline`. Launcher elapsed
7202224 ms; original setup bound 7200000 ms. No measured load segment or actual
24-hour soak started. Continuation and qualification services for this frozen
revision are absent. The original estate and all evidence remain retained.
This is incomplete readback, not demonstrated lost acknowledgements.

The final sample reports current/peak segment handles 128/128 and zero handle
denials, zero failed flushes and compactions, resident bytes 616640512, source
page bytes 67098553 and no outstanding/queued requests. These are final samples,
not whole-run peaks or proof of the only bottleneck. The component regression
still proves fewer proof-node backing reads; this run does not establish a
complete native operating point or an end-to-end performance improvement.

Stage in the owned Linux guest:
`/home/devcontainers/rrflow-c07-native-79663cdd/desktop-integrated-preflight-05817485`.
The outer campaign file still matches the launch receipt. The evidence campaign
is a separate emitted projection; its file digest is not the semantic campaign
identity reported in result/launch (`4ea5b25a244d888be083b4f1a5c9431f45173c795243f4460853a6c14bd7035e`).

| Retained artifact | Bytes | File SHA-256 |
|---|---:|---|
| campaign.json | 9423 | `ddeffa4e7b5e888444579fa1f855d4a2d81687ffb23b947c11a47fa2b01da543` |
| evidence/campaign.json | 6874 | `b5519604403e4f2e9aa6fc188caab3162d53f9f9ca343200fb90bd8645c5ceb6` |
| evidence/progress.jsonl | 50966026 | `ffb68b73ea93dcb13ae5431e7ddf24d9e8a5c90bcf4679b678ea39112834c9f1` |
| evidence/result.json | 427 | `51f543fddef98f5d7ea90af328ed7e156c21384cc73af573a7e5b61a4019627c` |
| evidence/launch.json | 505 | `b19c0370d3027bd3188c540c70d124e10eb30d33c295251d1f2834e3b0e9c985` |
| launcher.stderr.log | 603 | `ea58cc8a917ea958dd8ce1ae3ffbf41003ae61efbbffd39642e03bb07adfa4a8` |
| operator-observations.jsonl | 234777 | `f8614e1d74c62b87f0dbea86416e36d2500c783b28ab998dbd47f17234cdb708` |

## Inspection and next failure oracle

Complete reads: proof_nodes.rs and tests, new direct-read oracle, native launch
journal, existing read-only inspection script, outcome-hashing helper authored
under target/. Reviewed deltas: versioned_read.rs and handle integration source;
relevant workload verify_key, corpus seed/verification loops and query pipeline.
Earlier package journals own their broader code review and component results.
This is not a full-codebase review.

The corpus setup performs independent verification sequentially, although the
operating point declares four clients; the measured workload already keeps
verification inside its admitted workers. That is an inspected scheduling
fact, not yet a measured explanation for the failed setup. Any successor must
preserve complete independent record/edge/vector checks, canonical receipt
ordering, bounded admitted work, session rotation, original deadlines and
failure cleanup. Do not replace the readback oracle with ACK counts or widen
the deadline. Profile actual costs before claiming engine optimization.

Research/source-adaptation decision: use retained exact-source execution and
repository code; no new external algorithm is adopted. Preserve current bounded
signals and original evidence. Scope is this successor, POAM-035, the current
C-07 result reference and generated navigation/inventory. No runtime change,
new effect, source relocation or new status authority. Stop for weakened
qualification, evidence loss or an unowned implementation change.

## Commands, limits and checklist

`wsl -d Ubuntu -- python3 /mnt/d/localdev/devlabs/rrflow/target/inspect_c07_desktop_integrated.py 05817485c1b083e4bf64eeb902314f47eac5f4c0`
returned the terminal service/progress/result/launch/error evidence successfully.
`inspect_c07_outcome_receipt.py` streamed and hashed only the named evidence
files; local receipt is `target/c07-proof-outcome-identities.json`. No secrets
were printed and no service, source or estate was changed. The Windows CI job's
live log could not be fetched while running; that command is not passing test
evidence. Source exploration included an invalid guessed core directory; no
missing-code conclusion was drawn from it.

No new Cargo, provider, UI, native campaign or soak execution was performed for
this result-only correction. PR130 passed full candidate 40212146064 and entered
the normal merge queue, with merge-group run40265304881. PR134 retains that exact
source dependency and can enter its own candidate checks after this evidence
update. Queue and CI state are observed in GitHub, not inferred from this receipt.

Checklist: owner/brief/baseline before edits; source and evidence coverage
explicit; actual failure and hashes retained; resource/debug/research decisions
recorded; original acceptance preserved; current owner status corrected without
rewriting history; documentation/projection/whitespace and source-preservation
checks required before commit. C-07 and POAM-035 remain open.

Local acceptance passed: documentation (615 statuses/613 coordinates),
navigation (17 indexes/614 nodes/2430 edges), inventory (1742 records),
workflow policy and whitespace. Exact comparison confirmed no runtime, SDK or
Cargo dependency bytes changed. Full result-only diff reviewed before commit.
