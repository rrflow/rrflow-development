# Bounded local controller observations

**Status:** active offline collection contract; no engine or training authority
**Coordinate:** `rrflow://rrflow-instance/data/reference/inference/learning/capture-observations`
**Owner:** optional local observation exchange under G-06a

`scripts/inference/capture_observations.py` polls an explicitly selected trusted
Git repository and a local task JSON file. It uses Python's standard library
and repository-owned review-exchange helpers. It is optional tooling, not an
installation dependency, engine routine or accepted source read stamp.
The [training owner](controller-training.md) governs later review and admission.

## Explicit input and execution

Create a local task file separate from the output log:

```json
{"schemaVersion":1,"issueTitle":"Fix the observed build failure","issueId":"project#123","userOverride":null}
```

Allowed fields are exactly `schemaVersion`, `issueTitle`, optional `issueId`,
`intentText`, `userOverride` and `route`. Version is integer 1. Title is nonempty
and at most 2,000 characters; issue ID is at most 200 and intent at most 8,000.
Override is a boolean or null. A supplied route has exactly `provider`, `modelId`,
`effort`, `accessMode` and `bindingId`; values are null or nonempty strings up to
200 characters, with access mode restricted to `local`, `api` or `subscription`.
Do not include credentials. Unknown fields, duplicate JSON keys, non-finite
values and assertions of training consent are rejected.

```sh
python scripts/inference/capture_observations.py --repo /projects/app --context /captures/task.json --output /captures/observations.jsonl --duration-seconds 300 --interval-seconds 5 --max-records 100
```

Select the repository root with an existing commit. Output must be outside it
to prevent capture feedback. Use `--once` for one observation. A producer can
atomically replace the task file when its title or explicit preference changes.
This tool does not poll remote issues or infer user overrides.

## Identity, recovery and JSONL output

Each line is a `schemaVersion: 1`, `purpose: unreviewed_local_observation` record.
`source` binds HEAD, branch digest, status, staged and unstaged diff digests,
exact context bytes, collector/helper source hashes and Python version.
`streamSha256` identifies selected repository/context paths and scope;
`snapshotSha256` identifies source content. `recordId` hashes the canonical
record without its own field. `previousRecordId` links observed transitions.
Task text, redacted changed paths and UTC time are retained. Raw diffs and
local absolute paths are not in the record.

Repeated edits to the same filename create distinct source identities. Two
consecutive equal observations are required; a moving input defers the poll,
and a corrected input can recover later. Unchanged polls deduplicate across
restart. Returning to an old state after an observed change creates a new record.
These sampled observations are not an atomic Git snapshot or proof every edit
was seen. Untracked files are excluded; submodule contents need their own
explicit collection. Changed collector bytes produce a new identity.

Every record retains `labelStatus: unreviewed`, `trainingAdmitted: false`,
`engineReceipt: null` and an unknown task outcome. Preferences and hashes are
not independently verified labels, consent, authorization or authenticity proof.
The paired-outcome review exchange accepts its own explicit export contract;
this collector does not silently transform observations into that data.

Append locking serializes readers/writers. Retained identities and predecessor
order are checked before appending. Partial tails, modified records, reordered
chains and occupied locks are preserved. No stale-lock stealing, truncation,
repair or deletion happens automatically. Recovery is an explicit operator task.
A copy of the log is not an authenticated engine audit trail.

## Resource and privacy boundary

Diffs are limited to 16 MiB each, status to 1 MiB, context to 64 KiB, changed
paths to 1,000, each record to 128 KiB and a log to 64 MiB. Select a new retained
output explicitly when full. Polling has duration and record limits; an active
poll can finish after the scheduling duration. Each Git output read has a
20-second limit plus bounded cleanup. No process remains after normal completion.
This is not a sandbox for hostile Git configuration.

SWAG experimental redaction patterns were adapted for titles, intent, issue IDs
and paths. Sensitive-pattern route identities are rejected rather than changed.
Redaction is best effort and hashes are not anonymization. CLI diagnostics
report error classes, not payloads or Git stderr. Files stay local; there is no
upload, model invocation or training.

Run `python scripts/inference/test_capture_observations.py` for real temporary-Git
and process tests. A clean-checkout collection proves optional tooling only;
installed governed learning remains subject to G-01 through G-06 and I-03.
