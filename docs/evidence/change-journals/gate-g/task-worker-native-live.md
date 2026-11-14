# Installed native selector and worker observations

**Status:** active installed native evidence; CI, UI and full workflow pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-native-live`
**Owner:** G-01b / G-02a / H-04; POAM-009 / POAM-011; PR229 driver: active SWAG session

## Exact components and scope

This follows the [native-client composition](task-worker-native-client.md).
The receipt-publication brief was recorded on PR229 before adding these files.
The SDK example and runner come from clean commit
`43d18059fa5387fac91125b28a7a10d612965f26`, tree
`f27a0aafb78d0bf73e2afafbb03b5a3f6d439842`. The separately installed primary
remains clean engine `c75c189612db7e7c8efe016fd92fb1052a3fb056`, executable SHA-256
`60a708117899a022b04818c096af17a925c60eae06c554c39d00e5fe09e028fa`.
Do not label that engine as the new SDK revision or a qualified release.

The example was built with
`cargo build --locked -p rrd-client --example task_model_call --all-features`.
The canonical source-snapshot helper verified clean, unchanged source before
and after build. Its copied executable is 9,657,344 bytes with SHA-256
`06a8b717ffae2cdce3a2af44eabcccefc7f07e6ec17064fc55b2d1ba5ad1862f`.
The [build observation](task-worker-native-receipts/client-build.json) is external
provenance, not an embedded or signed release attestation. The contract helper
was built at the same commit with
`cargo build --locked -p rrd-contract --example endpoint_selection --all-features`.
Runner SHA-256: `e7e80112ddaa78b5e50a08d752964d6f00a49d7ec48524d452b605c8920c8ae7`.

The actual task and initial profile are unchanged from the preceding installed
trial: captured PR229 title/body and one explicitly configured candidate.
Discovery observed the configured model alias; it did not attest loaded weights.
No routing-quality labels, training records or consent were created.

## Failed native worker and retained uncertainty

The first native selector succeeded. Its worker then returned engine HTTP 412:
`chat response is incomplete or requests unsupported effects`.
The [native output](task-worker-native-receipts/withheld/native-worker.json)
records the actual error and successful session closure. The client exited 1;
it did not retry. The public error does not identify which decoding predicate
failed, so token truncation is not asserted as the cause.

[Credential-free recovery](task-worker-native-receipts/withheld/recovery.json)
retained the accepted selector, stopped worker, two spent steps,
`attempt_failed`, `execution_may_continue: true` and no accepted worker output.
Its estate/capacity remain intact. Public reconciliation of uncertain work is
still a deployment requirement; a new test estate does not clear this one.

## Second isolated experiment

Only the operator worker prompt changed: bound the draft to at most three short
items in each requested section and avoid repeating the task/test counts.
The task, candidate, generation bounds, provider and validators stayed unchanged.
This was an explicit new isolated experiment, not replay of the uncertain attempt
or a production fallback. Full inputs/command arrays are retained locally under
SWAG `.swag/evidence/20261031/worker-native-client/`.

The native example carried both `task-endpoint-propose` and
`task-worker-execute` through the public Rust client. The
[selector output](task-worker-native-receipts/available/native-proposal.json) and
[worker output](task-worker-native-receipts/available/native-worker.json)
record successful session closure. The existing installed harness passed:

- Unauthenticated and [missing-credential worker denial](task-worker-native-receipts/available/credential-denial.json)
  without additional budget spending or a worker attempt.
- Accepted draft identity/content digest and retained-attempt equality; two
  total spent steps under the original deadline, no active or uncertain attempt.
- Unchanged task record: the model draft did not complete the task.
- Restart equality and [replay rejection](task-worker-native-receipts/available/replay.json)
  specifically as `ReasoningAttemptRecorded`, without additional spending.
- A separate [credential-free recovery](task-worker-native-receipts/available/recovery.json)
  read both attempts; its decoded worker record equals the successful result.
  Every owned server was stopped; no provider was called during recovery.

The [result](task-worker-native-receipts/available/result.json) has SHA-256
`38014d0cddf8ea1763d93465a35cb8a1be36b3c0fc56c9315e878024f151c3ca`.
Recovery SHA-256: `084645d8785a13a13f58d909aad23245e25ebaf439c826e61fe0adf2d8f6833e`.
The selector reported 1,934 ms. The worker reported HTTP 200, 1,149 content bytes,
1,419 prompt / 216 completion / 1,635 total tokens and 16,902 ms.
These individual observations are not comparative performance or quality results.

## Reviewable evidence and limits

The [manifest](task-worker-native-receipts/manifest.json) binds nine exact copied
receipts by byte count, hash and source. Copies were checked against the actual
provider key and both estates' operator credentials; none occurs in these files.
Configuration/credential files are omitted. Captured newline bytes are preserved
using the existing repository fixture convention. The failed and successful
cases remain distinct, inspectable observations.

Read/reviewed the changed runner/example, build/source checks, actual result and
recovery fields, native errors, receipt manifest and preceding package journal.
This package adds only this journal, receipt subset and reference/POAM links,
plus generated navigation/inventory. No executable code changes, new research,
tracing or duplicate authority are needed. Broad Rust suites remain the preceding
package's evidence, not rerun claims for this documentation increment.

All nine staged receipt blobs match the manifest's original byte counts and
SHA-256 values. `python scripts/knowledge/render_navigation.py`,
`python scripts/ci/build_execution_inventory.py` and its `--check`,
`python scripts/ci/check_documentation.py`, and working/staged whitespace checks
passed. Navigation reports 854 nodes / 3,094 edges; inventory 2,215 paths;
documentation 855 statuses / 853 coordinates. Exact exits are retained in
`worker-native-client/live-repository-validation.json`. The final diff contains
only the stated evidence/reference paths. The authoring checklist preserves
unchanged execution/authorization behavior and explicit source/claim limits.

No complete Connectome flow, assembled-context native trial, project effect,
ordinary-edit end-to-end replan, worker process-death recovery, RROFLOW correlation,
reviewed learning, model comparison/quantization, native/container release or
protected landing is established. Required CI and remaining acceptance still
gate readiness; PR229 stays draft and no roadmap/POAM closes.
