"""Boundary tests for explicit source-bound label-review evidence."""
import copy
from datetime import datetime, timedelta, timezone
from pathlib import Path
import tempfile
import unittest

from review_outcomes import build_packet, canonical, digest, publish, strict_json
from review_decisions import finalize, validate_review


class ReviewDecisionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.manifest_path = self.root / "manifest.json"
        tasks = [{"taskId": str(i), "project": "test-project", "sourceSha256": "a" * 64,
                  "input": f"Fixture task {i}", "labelOrigin": "test-author",
                  "provisionalLabel": "implementation"} for i in range(3)]
        outcomes = [{"taskId": str(i), "status": "error", "output": "fixture timeout",
                     "verification": {"state": "unknown", "basis": "test fixture"}} for i in range(3)]
        def write(name, values):
            content = b"".join(canonical(v) for v in values)
            (self.root / name).write_bytes(content)
            return {"path": name, "sha256": digest(content)}
        manifest = {"schemaVersion": 1, "purpose": "synthetic test fixture",
                    "tasks": write("tasks.jsonl", tasks),
                    "candidates": [{"candidateId": name, "artifactSha256": "b" * 64,
                                    "outcomes": write(name + ".jsonl", outcomes)}
                                   for name in ("base", "custom")]}
        self.manifest_path.write_bytes(canonical(manifest))
        self.packet = build_packet(self.manifest_path)
        self.packet_path = publish(self.packet, self.root / "packets")
        self.sha = self.packet_path.stem
        self.review = {"schemaVersion": 1, "purpose": "development_label_review",
                       "packetSha256": self.sha, "reviewerId": "fixture-reviewer",
                       "reviewerKind": "assistant", "labelAuthorId": "fixture-author",
                       "reviewedAt": (datetime.now(timezone.utc) - timedelta(seconds=1)).isoformat(),
                       "decisions": [{"taskId": str(i), "decision": kind, "label": label,
                                      "rationale": "Synthetic fixture only", "evidenceRefs": ["fixture:source"]}
                                     for i, kind, label in ((0, "accept", "implementation"),
                                                           (1, "correct", "documentation"),
                                                           (2, "exclude", None))]}

    def validate(self):
        return validate_review(self.packet, self.sha, self.review)

    def test_complete_review_preserves_corrections_exclusions_and_authority(self):
        result = self.validate()
        self.assertEqual(result["counts"], {"accept": 1, "correct": 1, "exclude": 1})
        self.assertEqual(result["labels"][1]["review"]["label"], "documentation")
        for name in ("identityAuthenticated", "trainingAdmitted", "acceptanceDataset", "deploymentQualified"):
            self.assertIs(result[name], False)

    def test_wrong_packet_and_author_identity_are_rejected(self):
        self.review["packetSha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "different packet"):
            self.validate()
        self.review["packetSha256"] = self.sha
        self.review["reviewerId"] = " FIXTURE-AUTHOR "
        with self.assertRaisesRegex(ValueError, "differ"):
            self.validate()

    def test_missing_duplicate_and_unknown_tasks_are_rejected(self):
        original = copy.deepcopy(self.review["decisions"])
        for decisions in (original[:2], [original[0]] * 3,
                          [original[0], original[1], {**original[2], "taskId": "unknown"}]):
            self.review["decisions"] = decisions
            with self.assertRaises(ValueError):
                self.validate()

    def test_invalid_decision_semantics_and_evidence_are_rejected(self):
        original = copy.deepcopy(self.review["decisions"][0])
        for changes in ({"decision": "pending"}, {"label": "documentation"},
                        {"decision": "correct"}, {"decision": "exclude"},
                        {"rationale": " "}, {"evidenceRefs": []},
                        {"evidenceRefs": ["x"] * 17}, {"trainingAdmitted": True}):
            self.review["decisions"][0] = {**original, **changes}
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.validate()

    def test_timestamp_kind_and_version_are_explicit(self):
        original = copy.deepcopy(self.review)
        for changes in ({"reviewedAt": "2026-01-01T00:00:00"},
                        {"reviewedAt": (datetime.now(timezone.utc) + timedelta(days=1)).isoformat()},
                        {"reviewerKind": "verified"}, {"schemaVersion": True}):
            self.review = {**original, **changes}
            with self.assertRaises(ValueError):
                self.validate()

    def test_full_files_roundtrip_is_idempotent(self):
        reviews = self.root / "reviews.json"
        reviews.write_bytes(canonical(self.review))
        first = finalize(self.manifest_path, self.packet_path, reviews, self.root / "final")
        second = finalize(self.manifest_path, self.packet_path, reviews, self.root / "final")
        self.assertEqual(first, second)
        result = strict_json(first.read_bytes())
        self.assertEqual(result["reviewFileSha256"], digest(reviews.read_bytes()))
        self.assertEqual(result["packetSha256"], self.sha)

    def test_modified_source_or_packet_creates_no_review_output(self):
        reviews = self.root / "reviews.json"
        reviews.write_bytes(canonical(self.review))
        original = self.packet_path.read_bytes()
        self.packet_path.write_bytes(original + b" ")
        with self.assertRaisesRegex(ValueError, "packet differs"):
            finalize(self.manifest_path, self.packet_path, reviews, self.root / "final")
        self.packet_path.write_bytes(original)
        (self.root / "tasks.jsonl").write_bytes(b"{}\n")
        with self.assertRaisesRegex(ValueError, "bytes changed"):
            finalize(self.manifest_path, self.packet_path, reviews, self.root / "final")
        self.assertFalse((self.root / "final").exists())


if __name__ == "__main__":
    unittest.main()
