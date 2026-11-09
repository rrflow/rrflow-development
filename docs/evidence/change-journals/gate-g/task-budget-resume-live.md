# Actual installed recovery on the same task

**Status:** active installed recovery evidence; UI and release acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-budget-resume-live`
**Owner:** G-01b/G-02a / POAM-009; PR230

## Source and intervention

Clean [harness](task-budget-resume-proof.md) revision
`ce20dc7935a347445eb90218b677f6f120383f9e` ran against the independently
frozen [resume engine](task-budget-resume.md) at
`6a4cbb042a13f5a7c41b53d11d62b54a85df3bed`, executable SHA-256
`d4cc99d8527b97ba145023e0b1cc4a7c26a0682f6edc8c2d5d8d9ff1ad4ffec0`.
The actual task is PR230's captured title/body/revision. Its body projection is
bounded by the existing runner; the full captured source and profile-change
identities are retained. No task, model response or correctness label was made up.

The operator supplied a profile reducing the configured worker output allowance
from 512 tokens to one, and its original profile restoring 512. Only that field
differs. Both use the same configured one-call binding. The unchanged relay
forwarded actual provider bytes, holding only the second response before delivery
so the active attempt and denied resume could be observed. The provider reported
`length` on that response and `stop` on the other three. This is a controlled
recovery experiment, not a performance or model-quality benchmark. A service
alias is not independent weight attestation.

## Verified history

The installed trial and separate read-only recovery both exited zero. Exactly
four actual provider calls occurred:

1. The original selector proposal was accepted.
2. The actual length-terminated worker was rejected with HTTP 412 and no accepted
   output. Resume while its response was held was denied with `UnresolvedAttempt`
   and left the active budget unchanged. After delivery and reopen, its stopped
   budget retained two steps, 2,507 ms and the original deadline, with no active
   attempt or unknown execution.
3. After unauthenticated resume was denied, authenticated resume at the exact
   stopped budget digest preserved two steps, 2,507 ms and the original deadline.
   Stale resume conflicted; replay of the original worker was denied without
   dispatch. Normal configuration preview/approval restored the supplied worker
   allowance. A fresh selector on the same task was accepted.
4. A fresh worker draft was accepted under that new selection. The final budget
   contains four steps and 21,529 ms, retaining the original deadline. Both old
   attempts remain unchanged. Replaying either new attempt was denied without
   another provider call.

Credential-free reopen compared all four attempts and the final budget exactly.
A separate recovery invocation repeated those readbacks with no provider operation.
The task remains open: the output is a draft, not an executed project change.
All seven trial servers and the independent recovery server were stopped and
waited for; subsequent OS process lookup found none of those eight PIDs running.
Their termination exit codes are retained separately from the successful runner
exit codes. No historical estate, unknown lease or original budget was reset.

## Publication and limits

The [manifest and 37 byte-preserved receipts](task-budget-resume-live-receipts/manifest.json)
include four provider responses/observations, active-resume denial, terminal
rejection, original and resumed budgets, authorization/staleness/replay denials,
fresh accepted attempts, original retained attempts, final task and independent
recovery. Every copied response matches its recorded relay digest. Publication
scanned JSON fields and raw bytes against the actual private credential; no
credential, credential mapping, estate or secret file is published. Full commands,
profile inputs, process exits and logs remain under SWAG
`.swag/evidence/20261031/task-budget-resume-installed/`.

The pre-edit PR230 brief covers this publication. Review inspected the actual
four provider contents, public payload identities/budgets, recovery equality,
process termination and complete selected receipt set. No trial failed or was
retried. The harness's 14 tests and compilation/repository checks passed before
its clean commit; no Rust or provider rerun is needed for evidence publication.
Navigation passes with 869 nodes and 3,130 edges; inventory generation/check
passes with 2,378 paths. Documentation policy passes with 870 statuses and 868
coordinates. Staged receipt hashes must match all 37 original captures before
commit. Authored whitespace and receipt-scoped CRLF-aware checks are retained.
Runtime, SDK and relay code are unchanged here. No external
research, new trace authority, resource claim, roadmap closure or version change
is introduced.

This supplies installed same-task recovery evidence missing from the preceding
harness and engine increment. Connectome controls and its pinned executable still
need composition. Unknown-outcome reconciliation, ordinary-edit UI replanning,
governed project effects, correlated RROFLOW outcomes, independently reviewed
learning, candidate/dependency landing and native/container/model qualification
remain open. Neither automatic learning nor a public release is established.
