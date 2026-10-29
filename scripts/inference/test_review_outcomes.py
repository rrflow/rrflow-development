"""Failure-boundary tests for the offline paired-outcome review exchange."""
from __future__ import annotations

import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import review_outcomes as review


class OutcomeReviewTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.manifest_path = self.root / "manifest.json"
        self.tasks = [{"taskId": "one", "project": "project-a", "sourceSha256": "a" * 64,
                       "input": "Review a change", "labelOrigin": "assistant-inferred",
                       "provisionalLabel": "implementation"}]
        self.outcomes = [{"taskId": "one", "status": "error", "output": {"error": "timeout"},
                          "verification": {"state": "unknown", "basis": "no independent review"}}]
        self.manifest = {"schemaVersion": 1, "purpose": "development review",
                         "tasks": self.write_rows("tasks.jsonl", self.tasks),
                         "candidates": [{"candidateId": name, "artifactSha256": char * 64,
                                         "outcomes": self.write_rows(name + ".jsonl", self.outcomes)}
                                        for name, char in (("base", "b"), ("custom", "c"))]}
        self.save_manifest()

    def write_rows(self, name, rows):
        content = b"".join(review.canonical(row) for row in rows)
        (self.root / name).write_bytes(content)
        return {"path": name, "sha256": review.digest(content)}

    def save_manifest(self):
        self.manifest_path.write_bytes(review.canonical(self.manifest))

    def build(self):
        self.save_manifest()
        return review.build_packet(self.manifest_path)

    def test_errors_and_uncertainty_survive_pairing(self):
        packet = self.build()
        self.assertFalse(packet["trainingAdmitted"])
        self.assertFalse(packet["deploymentQualified"])
        self.assertEqual(packet["tasks"][0]["task"], self.tasks[0])
        self.assertEqual(packet["tasks"][0]["outcomes"]["custom"], self.outcomes[0])
        self.assertEqual(packet["reviewState"], "pending_independent_review")

    def test_changed_input_is_rejected_then_new_identity_preserves_old_packet(self):
        old = review.publish(self.build(), self.root / "packets")
        old_bytes = old.read_bytes()
        self.tasks[0]["input"] = "Review an ordinary edit"
        new_ref = self.write_rows("tasks.jsonl", self.tasks)
        with self.assertRaisesRegex(ValueError, "bytes changed"):
            self.build()
        self.manifest["tasks"] = new_ref
        new = review.publish(self.build(), self.root / "packets")
        self.assertNotEqual(old, new)
        self.assertEqual(old.read_bytes(), old_bytes)
        self.assertEqual(review.publish(self.build(), self.root / "packets"), new)

    def test_missing_extra_and_duplicate_outcomes_are_rejected(self):
        for values in ([], self.outcomes * 2, [{**self.outcomes[0], "taskId": "other"}]):
            with self.subTest(values=values):
                self.manifest["candidates"][0]["outcomes"] = self.write_rows("base.jsonl", values)
                with self.assertRaises(ValueError):
                    self.build()

    def test_duplicate_tasks_and_candidates_are_rejected(self):
        self.manifest["tasks"] = self.write_rows("tasks.jsonl", self.tasks * 2)
        with self.assertRaisesRegex(ValueError, "duplicate task"):
            self.build()
        self.manifest["tasks"] = self.write_rows("tasks.jsonl", self.tasks)
        self.manifest["candidates"][1] = copy.deepcopy(self.manifest["candidates"][0])
        with self.assertRaisesRegex(ValueError, "duplicate candidate"):
            self.build()

    def test_same_task_ids_with_changed_candidate_bytes_need_new_digest(self):
        (self.root / "base.jsonl").write_bytes(b"{}\n")
        with self.assertRaisesRegex(ValueError, "bytes changed"):
            self.build()

    def test_paths_are_contained(self):
        for name in ("../escape", "/absolute", "C:/absolute", "a\\b", "a/../b", "./tasks.jsonl", "a//b"):
            with self.subTest(name=name):
                self.manifest["tasks"]["path"] = name
                with self.assertRaisesRegex(ValueError, "relative POSIX"):
                    self.build()

    def test_symlink_cannot_escape_export_root(self):
        with tempfile.TemporaryDirectory() as outside:
            target = Path(outside) / "tasks.jsonl"
            target.write_bytes((self.root / "tasks.jsonl").read_bytes())
            link = self.root / "link.jsonl"
            try:
                link.symlink_to(target)
            except OSError:
                self.skipTest("OS does not permit symlink creation")
            self.manifest["tasks"]["path"] = "link.jsonl"
            with self.assertRaisesRegex(ValueError, "escapes"):
                self.build()

    def test_json_ambiguity_and_truncation_are_rejected(self):
        for content in (b'{"a":1,"a":2}', b'{"a":NaN}', b'{"a":1e999}', b'\xff'):
            with self.subTest(content=content), self.assertRaises(ValueError):
                review.strict_json(content)
        with self.assertRaisesRegex(ValueError, "newline-terminated"):
            review.rows(b'{"taskId":"one"}')

    def test_row_file_and_total_bounds(self):
        with self.assertRaisesRegex(ValueError, "byte limit"):
            review.read_bounded(self.manifest_path, 1)
        with patch.object(review, "MAX_ROW_BYTES", 1), self.assertRaisesRegex(ValueError, "oversized"):
            self.build()
        with patch.object(review, "MAX_TOTAL_BYTES", 1), self.assertRaisesRegex(ValueError, "aggregate"):
            self.build()
        with patch.object(review, "MAX_TASKS", 0), self.assertRaises(ValueError):
            self.build()

    def test_claims_do_not_become_independent_review(self):
        self.outcomes[0]["verification"] = {"state": "pass", "basis": "self-reported"}
        self.manifest["candidates"][0]["outcomes"] = self.write_rows("base.jsonl", self.outcomes)
        packet = self.build()
        self.assertEqual(packet["tasks"][0]["reviewState"], "pending_independent_review")
        self.assertFalse(packet["trainingAdmitted"])

    def test_pairing_uses_identity_instead_of_row_order(self):
        self.tasks.append({**self.tasks[0], "taskId": "two", "provisionalLabel": None})
        self.manifest["tasks"] = self.write_rows("tasks.jsonl", self.tasks)
        second = {**self.outcomes[0], "taskId": "two", "status": "abstained", "output": None}
        for index, candidate in enumerate(self.manifest["candidates"]):
            values = [self.outcomes[0], second] if index == 0 else [second, self.outcomes[0]]
            candidate["outcomes"] = self.write_rows(candidate["candidateId"] + ".jsonl", values)
        packet = self.build()
        self.assertEqual(packet["tasks"][1]["outcomes"]["custom"], second)
        self.assertIsNone(packet["tasks"][1]["task"]["provisionalLabel"])

    def test_invalid_outcome_status_and_verification_rejected(self):
        for row in ({**self.outcomes[0], "status": "approved"},
                    {**self.outcomes[0], "verification": {"state": "approved", "basis": "claim"}}):
            self.manifest["candidates"][0]["outcomes"] = self.write_rows("base.jsonl", [row])
            with self.assertRaises(ValueError):
                self.build()

    def test_unsupported_fields_states_versions_and_identities_rejected(self):
        original = copy.deepcopy(self.manifest)
        for key, value in (("schemaVersion", True), ("schemaVersion", 2), ("trainingAdmitted", True)):
            self.manifest = {**original, key: value}
            with self.assertRaises(ValueError):
                self.build()
        self.manifest = original
        self.manifest["candidates"][0]["artifactSha256"] = "moving-model-name"
        with self.assertRaisesRegex(ValueError, "SHA-256"):
            self.build()

    def test_corrupted_existing_packet_is_not_overwritten(self):
        packet = self.build()
        path = review.publish(packet, self.root / "packets")
        path.write_bytes(b"corrupt")
        with self.assertRaisesRegex(ValueError, "collision or corruption"):
            review.publish(packet, self.root / "packets")
        self.assertEqual(path.read_bytes(), b"corrupt")
        self.assertEqual(list(path.parent.glob(".pending-*")), [])

    def test_failed_publication_leaves_no_partial_packet(self):
        with patch.object(review.os, "link", side_effect=OSError("unavailable")):
            with self.assertRaises(OSError):
                review.publish(self.build(), self.root / "packets")
        self.assertEqual(list((self.root / "packets").iterdir()), [])


if __name__ == "__main__":
    unittest.main()
