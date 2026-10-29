# D-02: correct the proposed replan for immutable job identity

**Status:** active draft design evidence; runtime behavior not implemented
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-immutable-job-replan-design`
**Owner:** OBJ-14 / D-02 / D-03 / POAM-039 / POAM-041 / [PR148](https://github.com/rrflow/rrflow-development/pull/148)

## Baseline, scope and traceability

Starting PR148 head `e3eaa78b3272bd19b58f93cfd6c8a6365ac4a01d`;
[PR193](https://github.com/rrflow/rrflow-development/pull/193) has since
merged as `291fee511887932572ecc9935e9786e098727677`. The PR148 description
recorded this correction before editing. The existing contract
`rrd-contract/src/attunement.rs::AttunementJob::validate_transition` rejects
changes to a job's `plan_id` and `plan_sha256`. The engine's existing
`read_attunement_receipt` resolves the receipt's plan, while its idempotent
`control_attunement` replay validates against the current job plan. Because
supported transitions cannot retarget that job, rewriting the same job to a
new plan would be an invalid test fixture rather than proof of a supported
replan failure. [PR197](https://github.com/rrflow/rrflow-development/pull/197)
was closed before implementation for this reason; it ran no candidate CI and
changed no engine source.

The supporting `docs/architecture/source-bound-plan-design.md` now states that,
under the current contract, a replacement plan needs a distinct engine-owned
job and a durable successor relation; it cannot silently alter the original
job's plan. Atomic successor publication, old-worker fencing, explicit
authorization and handling in-flight uncertain effects are requirements, not
implemented behavior. A future change to the immutable-job invariant would
need a separate explicit contract design and evidence. The proposal and prior
journals remain; no implementation is moved or removed.

Reviewed the full source-bound design, `AttunementJob` validation and
transition rules, engine plan/job and receipt paths, current D-02/D-03,
OBJ-14 and POAM-041. The correction follows repository code rather than
external algorithm adaptation; no runtime trace or resource measurement
applies. The ordinary-edit, independent mutation, source snapshot, restart
and cross-job replay oracles remain unimplemented. No POAM, roadmap, D-02,
C-07 or alpha status changed. Checklist: pre-edit PR brief, current-code
traceability, invalid hypothesis stopped rather than encoded as a test,
corrected supporting design, linked successor journal, generated navigation
and inventory, and documentation/workflow checks. Exact candidate and
protected merge-group CI and real implementation are still required.

Local checks: documentation policy passed (734 document statuses and 732
classified coordinates), navigation check passed 18 indexes, workflow policy
passed and inventory tests passed 10. Current ignored inventory is regenerated
and checked before commit; staged and unstaged whitespace checks pass.
