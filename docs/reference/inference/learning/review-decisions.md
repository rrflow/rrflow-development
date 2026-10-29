# Explicit controller label reviews

**Status:** active offline review validator; authenticated admission and runtime integration remain separate
**Coordinate:** `rrflow://rrflow-instance/data/reference/inference/learning/review-decisions`
**Owner:** G-06a explicit label-review evidence

The [paired-outcome exchange](outcome-review.md) produces immutable packets.
This validator checks decisions against the original export and packet bytes.
It emits immutable development-label evidence; it does not authenticate a
reviewer, establish consent, admit a dataset or launch training. The
[learning owner](controller-training.md) retains those requirements.

## Review exchange

A browser or editor may author a UTF-8 JSON review with exactly these fields:

```json
{
  "schemaVersion": 1,
  "purpose": "development_label_review",
  "packetSha256": "<exact packet digest>",
  "reviewerId": "operator-declared-reviewer",
  "reviewerKind": "human",
  "labelAuthorId": "original-label-author",
  "reviewedAt": "2026-10-28T20:00:00Z",
  "decisions": [
    {"taskId": "example-task", "decision": "accept",
     "label": "implementation", "rationale": "Source establishes a behavior fix.",
     "evidenceRefs": ["source:<source-record-digest>"]}
  ]
}
```

There must be exactly one decision for every packet task. `accept` must retain
the nonempty provisional label; `correct` must supply a different nonempty
label; `exclude` must supply null. Every decision needs a nonempty rationale
and 1–16 evidence references. Pending drafts cannot finalize. Named text
fields/references have the packet tool's 2,048-character bound; review input
is at most 10 MiB. References are declarations retained for inspection, not
fetched or independently authenticated evidence.

`reviewerKind` is `human` or `assistant`, and reviewer/label-author identifiers
must differ after trimming and case folding. This mechanical distinction does
not prove independent people or independent judgment. The timestamp requires
a timezone and cannot be in the future. The reviewer must check source
material and uncertainty rather than accepting model agreement as ground truth.

## Validate against the original export

```sh
python scripts/inference/review_decisions.py --manifest export/manifest.json --packet packets/PACKET_SHA.json --reviews review.json --output-dir reviewed
python scripts/inference/test_review_decisions.py
```

The tool reconstructs the complete packet using the retained manifest, task
and outcome bytes and the packet-tool identity. A different packet, source
edit, missing outcome or changed packet-tool version requires an explicitly
new packet/review. No prior decision is silently migrated. Previous evidence
remains inspectable and unrelated work remains available.

The result retains packet/review/validator digests, exact review-file digest,
all decisions and source task identities, including exclusions. Publication
uses the existing packet publisher and does not overwrite an earlier record.
Repeated identical input produces the same output. The declared author kind
and identity remain visible; `identityAuthenticated`, `trainingAdmitted`,
`acceptanceDataset` and `deploymentQualified` always remain false.

This is development evidence. Previously evaluated tasks cannot become a fresh
acceptance set by changing labels. Training/evaluation consent, leakage and
split admission, actual outcome/preferences and engine authorization remain
separate. Optional UI authors exchange files only; no browser lifecycle,
canonical state or sibling-repository runtime dependency is introduced.
