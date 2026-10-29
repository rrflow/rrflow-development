"""Validate explicit label reviews against original paired-outcome export bytes."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path

from review_outcomes import (MAX_TOTAL_BYTES, build_packet, canonical, digest,
                             fields, identity, publish, read_bounded, strict_json, text)


def validate_review(packet: dict, packet_sha: str, review: object) -> dict:
    value = fields(review, {"schemaVersion", "purpose", "packetSha256", "reviewerId",
                           "reviewerKind", "labelAuthorId", "reviewedAt", "decisions"})
    if (type(value["schemaVersion"]) is not int or value["schemaVersion"] != 1
            or value["purpose"] != "development_label_review"):
        raise ValueError("unsupported review exchange")
    if identity(value["packetSha256"]) != packet_sha:
        raise ValueError("review names a different packet")
    reviewer = text(value["reviewerId"])
    author = text(value["labelAuthorId"])
    if reviewer.strip().casefold() == author.strip().casefold():
        raise ValueError("reviewer must differ from declared label author")
    if value["reviewerKind"] not in ("human", "assistant"):
        raise ValueError("reviewer kind must be explicit")
    reviewed = datetime.fromisoformat(text(value["reviewedAt"]).replace("Z", "+00:00"))
    if reviewed.tzinfo is None or reviewed > datetime.now(timezone.utc):
        raise ValueError("review needs a timezone and cannot be future-dated")
    decisions = value["decisions"]
    if not isinstance(decisions, list) or len(decisions) != len(packet["tasks"]):
        raise ValueError("review must decide every packet task")
    tasks = {entry["task"]["taskId"]: entry["task"] for entry in packet["tasks"]}
    seen = set()
    labels = []
    counts = {"accept": 0, "correct": 0, "exclude": 0}
    for decision in decisions:
        fields(decision, {"taskId", "decision", "label", "rationale", "evidenceRefs"})
        task_id = text(decision["taskId"])
        if task_id in seen or task_id not in tasks:
            raise ValueError("duplicate or unknown reviewed task")
        seen.add(task_id)
        kind = decision["decision"]
        if kind not in ("accept", "correct", "exclude"):
            raise ValueError("pending or unsupported review decision")
        text(decision["rationale"])
        references = decision["evidenceRefs"]
        if not isinstance(references, list) or not 1 <= len(references) <= 16:
            raise ValueError("require bounded explicit review evidence references")
        for reference in references:
            text(reference)
        label = decision["label"]
        if kind == "exclude":
            if label is not None:
                raise ValueError("excluded tasks cannot carry a training target")
        else:
            text(label)
            unchanged = label == tasks[task_id]["provisionalLabel"]
            if (kind == "accept") != unchanged:
                raise ValueError("accept/correct must agree with the original label")
        counts[kind] += 1
        labels.append({"task": tasks[task_id], "review": decision})
    return {"schemaVersion": 1, "purpose": "reviewed_development_label_evidence",
            "packetSha256": packet_sha, "reviewSha256": digest(canonical(value)),
            "review": value, "counts": counts, "labels": labels,
            "identityAuthenticated": False, "trainingAdmitted": False,
            "acceptanceDataset": False, "deploymentQualified": False}


def finalize(manifest: Path, packet_path: Path, review_path: Path, output: Path) -> Path:
    packet_bytes = read_bounded(packet_path, MAX_TOTAL_BYTES * 6)
    rebuilt = build_packet(manifest)
    if canonical(rebuilt) != packet_bytes:
        raise ValueError("packet differs from original export or packet-tool version")
    review_bytes = read_bounded(review_path)
    result = validate_review(rebuilt, digest(packet_bytes), strict_json(review_bytes))
    result["reviewFileSha256"] = digest(review_bytes)
    result["validatorSha256"] = digest(Path(__file__).read_bytes())
    return publish(result, output)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("manifest", "packet", "reviews", "output-dir"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    try:
        result = finalize(args.manifest, args.packet, args.reviews, args.output_dir)
    except (ValueError, OSError) as error:
        parser.exit(1, f"label review rejected: {error}\n")
    print(f"review evidence: {result.name}; trainingAdmitted=false")


if __name__ == "__main__":
    main()
