#!/usr/bin/env python3
"""Regression tests for POA&M identity and non-mutating merge inspection."""
import unittest
import os
import subprocess
import tempfile
from pathlib import Path
from unittest import mock

import check_documentation
import inspect_merge_conflicts


class PoamIdentityTests(unittest.TestCase):
    def test_sparse_unique_matching_ids_are_valid(self):
        self.assertTrue(check_documentation.poam_ids_valid(
            ["POAM-001", "POAM-004", "POAM-049"],
            ["POAM-001", "POAM-004", "POAM-049"],
        ))

    def test_duplicates_missing_extra_unsorted_and_invalid_ids_are_rejected(self):
        valid = ["POAM-001", "POAM-004", "POAM-049"]
        for observed in (
            ["POAM-001", "POAM-004", "POAM-004", "POAM-049"],
            ["POAM-001", "POAM-049"],
            ["POAM-001", "POAM-004", "POAM-049", "POAM-050"],
            ["POAM-004", "POAM-001", "POAM-049"],
        ):
            with self.subTest(observed=observed):
                self.assertFalse(check_documentation.poam_ids_valid(valid, observed))
        for records in ([], ["POAM-000"], ["POAM-001", "POAM-001"], ["POAM-001", "bad"]):
            with self.subTest(records=records):
                self.assertFalse(check_documentation.poam_ids_valid(records, records))

    def test_deleting_a_historical_record_and_row_is_rejected(self):
        retained = ["POAM-001", "POAM-049"]
        self.assertFalse(check_documentation.poam_ids_valid(
            retained, retained, {"POAM-001", "POAM-004", "POAM-049"},
        ))
        self.assertFalse(check_documentation.poam_ids_valid(
            ["POAM-001"], ["POAM-001"], {"POAM-001", "POAM-049"},
        ))
        self.assertTrue(check_documentation.poam_ids_valid(
            retained, retained, {"POAM-001"},
        ))


class MergeInspectionTests(unittest.TestCase):
    def setUp(self):
        config = mock.patch.dict(os.environ, {
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_CONFIG_NOSYSTEM": "1",
        })
        config.start()
        self.addCleanup(config.stop)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git("init", "-q")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "user.name", "Fixture")
        self.git("config", "commit.gpgsign", "false")
        self.git("config", "core.hooksPath", str(self.root / "disabled-hooks"))
        self.generated = "sdks/python/src/rrd_client/generated/endpoints.py"
        self.semantic = "contract.txt"
        self.write("original\n")
        self.git("add", ".")
        self.git("commit", "-qm", "common")
        self.common = self.git("rev-parse", "HEAD")
        self.write("left\n")
        self.git("commit", "-qam", "left")
        self.left = self.git("rev-parse", "HEAD")
        self.git("checkout", "-q", "--detach", self.common)
        self.write("right\n")
        self.git("commit", "-qam", "right")
        self.right = self.git("rev-parse", "HEAD")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, text=True).strip()

    def write(self, value):
        for name in (self.generated, self.semantic):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(value, encoding="utf-8")

    def test_reports_real_conflicts_with_exact_ids_without_touching_worktree_or_refs(self):
        before = self.git("status", "--porcelain")
        head = self.git("rev-parse", "HEAD")
        refs = self.git("show-ref")
        report = inspect_merge_conflicts.inspect(self.root, self.left, self.right)
        self.assertEqual((report["base"], report["head"]), (self.left, self.right))
        self.assertEqual(report["status"], "conflicted")
        self.assertTrue(report["conflict_paths_may_be_incomplete"])
        paths = {item["path"]: item["generator"] for item in report["conflicts"]}
        self.assertEqual(paths[self.generated], "sdks/python/scripts/generate.py")
        self.assertIsNone(paths[self.semantic])
        self.assertEqual(self.git("status", "--porcelain"), before)
        self.assertEqual(self.git("rev-parse", "HEAD"), head)
        self.assertEqual(self.git("show-ref"), refs)

    def test_clean_and_unknown_revision(self):
        self.assertEqual(inspect_merge_conflicts.inspect(self.root, self.left, self.left)["status"], "clean")
        with self.assertRaises(ValueError):
            inspect_merge_conflicts.inspect(self.root, "no-such-revision", self.right)

    def test_configured_external_merge_driver_is_refused(self):
        self.git("config", "merge.fixture.driver", "must-not-run")
        with self.assertRaisesRegex(ValueError, "external merge drivers"):
            inspect_merge_conflicts.inspect(self.root, self.left, self.right)

    def test_malformed_merge_tree_output_is_rejected(self):
        for status, output in ((0, b"not-a-tree\0"),
                               (0, b"a" * 40 + b"\0unexpected\0")):
            with self.subTest(status=status, output=output):
                with self.assertRaises(ValueError):
                    inspect_merge_conflicts.parse_merge_tree(status, output)

    def test_conflict_without_file_entries_remains_a_conflict(self):
        self.assertEqual(
            inspect_merge_conflicts.parse_merge_tree(1, b"a" * 40 + b"\0"),
            ("a" * 40, []),
        )

    def test_first_parent_history_retains_deleted_record_identity(self):
        record = self.root / "docs/poam/rrflow-1.0-alpha/poam-001.md"
        record.parent.mkdir(parents=True)
        record.write_text("historical record\n", encoding="utf-8")
        self.git("add", ".")
        self.git("commit", "-qm", "record")
        record.unlink()
        self.git("add", "-u")
        self.git("commit", "-qm", "delete")
        self.assertEqual(check_documentation.historical_poam_ids(self.root), {"POAM-001"})

    def test_shallow_history_is_rejected(self):
        with mock.patch.object(check_documentation.subprocess, "run") as run:
            run.return_value.stdout = "true\n"
            with self.assertRaises(ValueError):
                check_documentation.historical_poam_ids(self.root)


if __name__ == "__main__":
    unittest.main()
