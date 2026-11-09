# Installed recovery on the reconciled worker candidate

**Status:** active installed developer evidence; candidate CI and UI integration pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-budget-resume-current-installed`
**Owner:** G-01b/G-02a / POAM-009; PR230; SWAG integration driver

## Source, consumer and scope

This follows the [worker reconciliation](task-budget-resume-current-worker.md)
and the pre-edit brief in [PR230](https://github.com/rrflow/rrflow-development/pull/230).
Connectome's Tasks workspace consumes the resume operation through UI PR29;
UI PR30 then supplies the source-replanning controls. This package proves the
native prerequisite, not that complete UI journey.

The clean source is `a99d577aedcf05c9dc435114912ee666a6d0875b`, tree
`8208acbfca35de2a2bbe83a2d63d47ef1bd1933f`. The canonical builder's two
capture/embedding passes succeeded with `--profile dev` and one Cargo job.
The frozen executable SHA-256 is
`efc0115997c32056987937f17797876abaf8c255fd9ee4df6b9f205f51749dfe`;
the same-source contract helper is
`fd0230a70c4513199a2de770b0f3258e29772927e06fc0a2c206fc4215514b7f`.
This is an unsigned Windows developer build, not distribution qualification.

The task is the captured public PR230 title/body/head. The existing driver
projects its body to 4,096 UTF-8 bytes and records that truncation. The explicit
operator profile supplies one candidate; this cannot measure comparative
routing suitability. The worker output allowance changes from 512 to one token
for the terminal-response experiment, then returns to 512 through normal
configuration preview and approval. The two profiles differ only in that field.

## First failure and connection repair

The first installed attempt failed at the selector with HTTP 412 and
`execution_may_continue: true`. The relay recorded request intent and `URLError`
without a provider response. Observation found no local forwarding listener;
the existing tunnel log reported a connection reset. These facts do not prove
the original remote outcome.

Separate credential-free recovery retained that attempt as stopped, with one
reserved step and unknown execution. No retry, resume or budget reset was
performed on that estate. After a successful SSH probe, the existing launcher
started only the stopped model tunnel. Health and authenticated discovery
returned 200; provider metrics reported zero running and waiting requests.
That is point-in-time headroom, not old-lease clearance or proof of cross-estate
capacity admission. A new exclusive estate then ran the bounded experiment.

## Actual recovery history

`scripts/inference/installed_controller_proof.py --worker
--worker-terminal-resume --resume-profile <supplied-original-profile>` ran with
the frozen build receipt, contract helper, actual task and credential references.
The trial and a separate `--recover-proof` invocation both exited zero.

Four real provider responses ended with `stop`, `length`, `stop`, and `stop`:

1. The original selector was accepted. While the actual second response was
   held, resume was denied with `UnresolvedAttempt` and retained the active budget.
2. The length-terminated worker was rejected. After reopen, its stopped budget
   held two steps and 2,741 ms, with no active attempt or unknown execution.
3. Unauthenticated resume was denied. Authenticated resume preserved the two
   steps, 2,741 ms, original allowance and deadline. A stale digest conflicted;
   original-attempt replay was denied. A fresh selection on the same task passed.
4. A new worker draft was accepted. The final budget held four steps and
   21,855 ms with the original deadline. Both earlier attempts remained unchanged;
   replay of either new attempt dispatched nothing.

Independent review matched all four raw-response hashes with their relay
observations, and matched engine content identities for accepted responses
one, three and four. The second response has a length finish and terminal denial,
not an accepted engine content identity. Review checked denial payloads, budget
transitions and the complete credential-free recovery payloads. All eight
trial/recovery-owned process IDs were absent at review. The trial and recovery
managed only their own child processes. The earlier unknown attempt remains separate.

## Publication, validation and remaining work

The [manifest](task-budget-resume-current-worker-receipts/manifest.json) binds
42 selected evidence files. Raw captures retain their bytes; three explicit
projections omit private source/proof locators and retain source hashes. Both
the failed and successful histories are represented. Publication scanned actual
provider/operator secrets and private-location patterns before writing. An
initial location regex incorrectly matched HTTPS URLs and stopped before writes;
the corrected drive-letter boundary passed the selected set.

This increment adds this journal, selected receipts, its nearest generated index
and a POAM-009 evidence link. Runtime, SDK, harness and historical evidence remain
unchanged. The preceding journal owns local engine/SDK/lint acceptance.
Navigation and inventory generation/checks, documentation policy and authored
whitespace checks pass. Final staged-byte verification remains required before
this evidence commit; full candidate and protected CI remain required before
landing. Private commands, profiles, process records and logs
remain in SWAG's ignored evidence directory.

Review covered the existing build/driver/resume boundaries, actual response
contents and selected receipt set. No new algorithm or external source adaptation
is introduced. Existing task/attempt/configuration identities supply diagnostics;
there is no new trace or lifecycle authority and no roadmap status change.

The accepted content is a draft, and the task remains open. This does not prove
project context through the UI, source-edit successor approval, project-file
effects, independently reviewed model outcomes, automatic learning, native
binary/score/choice bursts, loaded weights or a serving-memory target. Continue
through UI PR29/PR30 and the existing source/effect owners. Preserve failures and
stop on changed source, lost lineage, failed checks or unknown external outcomes.
