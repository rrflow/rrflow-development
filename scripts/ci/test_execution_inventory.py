#!/usr/bin/env python3
"""Prove parent inventory never depends on an optional submodule checkout."""

from __future__ import annotations

import contextlib
import hashlib
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import build_execution_inventory as inventory
import check_workflow as ci_policy
import check_documentation as docs_policy


class GitlinkInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.git("init", "--quiet")
        self.output = self.root / "inventory.jsonl"
        self.output.write_text("", encoding="utf-8")
        for name, value in (("ROOT", self.root), ("OUTPUT", self.output), ("PLANNED_PATHS", {})):
            patcher = patch.object(inventory, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def git(self, *args: str, input: bytes | None = None) -> bytes:
        return subprocess.run(
            ["git", *args], cwd=self.root, input=input, check=True, capture_output=True
        ).stdout

    def pin(self, revision: str) -> None:
        self.git("update-index", "--add", "--cacheinfo", f"160000,{revision},connectome")

    def test_uninitialized_populated_and_dirty_child_have_identical_parent_records(self) -> None:
        self.pin("1" * 40)
        absent = inventory.records()
        child = self.root / "connectome"
        child.mkdir()
        self.assertEqual(absent, inventory.records())
        (child / "README.md").write_text("child source", encoding="utf-8")
        (child / "secret.txt").write_text("must not enter parent inventory", encoding="utf-8")
        self.assertEqual(absent, inventory.records())
        (child / "README.md").write_text("uncommitted child edit", encoding="utf-8")
        self.assertEqual(absent, inventory.records())
        link = next(record for record in absent if record["path"] == "connectome")
        self.assertEqual("git-submodule", link["kind"])
        self.assertEqual("1" * 40, link["git_revision"])
        self.assertIsNone(link["lines"])

    def test_pin_change_changes_revision_and_digest(self) -> None:
        self.pin("1" * 40)
        before = inventory.current_record("connectome", inventory.gitlinks())
        self.pin("2" * 40)
        after = inventory.current_record("connectome", inventory.gitlinks())
        self.assertNotEqual(before["git_revision"], after["git_revision"])
        self.assertNotEqual(before["sha256"], after["sha256"])
        self.assertEqual(before["bytes"], after["bytes"])

    def test_regular_file_still_tracks_bytes(self) -> None:
        path = self.root / "README.md"
        path.write_bytes(b"first\n")
        before = inventory.current_record("README.md", inventory.gitlinks())
        path.write_bytes(b"second\n")
        after = inventory.current_record("README.md", inventory.gitlinks())
        self.assertEqual("documentation", before["kind"])
        self.assertEqual(1, before["lines"])
        self.assertNotEqual(before["sha256"], after["sha256"])

    def test_unmerged_gitlink_is_rejected(self) -> None:
        conflict = f"160000 {'1' * 40} 1\tconnectome\n160000 {'2' * 40} 2\tconnectome\n"
        self.git("update-index", "--index-info", input=conflict.encode())
        with self.assertRaisesRegex(ValueError, "unmerged index entry: connectome"):
            inventory.gitlinks()


class CIInventoryPolicyTests(unittest.TestCase):
    def test_ci_binds_and_retains_the_generated_inventory(self) -> None:
        workflow = (Path(__file__).resolve().parents[2] / ".github/workflows/ci-reusable.yml").read_text(encoding="utf-8")
        verify = ci_policy.job_blocks(workflow)["verify"]
        ci_policy.verify_execution_inventory_ci(verify)
        marker = "      - name: Enforce exhaustive RRFlow 1.0 file planning\n"
        prefix, owned = verify.split(marker, 1)
        for required in (
            "python3 scripts/ci/build_execution_inventory.py\n",
            "python3 scripts/ci/build_execution_inventory.py --check",
            "set -euo pipefail",
            "git diff --exit-code",
            "git diff --cached --exit-code",
            'test -z "$(git ls-files --others --exclude-standard)"',
            "git rev-parse --verify HEAD > target/rrflow-inventory-revision.txt",
            'test "$(cat target/rrflow-inventory-revision.txt)" = "$GITHUB_SHA"',
            "target/rrflow-inventory-revision.txt",
            "target/rrflow-1.0-file-plan.jsonl",
            "if-no-files-found: error",
        ):
            with self.subTest(required=required):
                self.assertIn(required, owned)
                with self.assertRaises(ci_policy.PolicyError):
                    ci_policy.verify_execution_inventory_ci(prefix + marker + owned.replace(required, "", 1))
        checked = "          git diff --exit-code\n"
        generated = "          python3 scripts/ci/build_execution_inventory.py\n"
        reordered = owned.replace(checked, "__INVENTORY_TEST_SWAP__", 1).replace(generated, checked, 1).replace("__INVENTORY_TEST_SWAP__", generated, 1)
        with self.assertRaises(ci_policy.PolicyError):
            ci_policy.verify_execution_inventory_ci(prefix + marker + reordered)


class InventoryDescriptorTests(unittest.TestCase):
    def test_descriptor_is_explicit_and_preserves_previous_blob(self) -> None:
        root = Path(__file__).resolve().parents[2]
        source = (root / "docs/roadmap/rrflow-1.0-file-plan.jsonl").read_text(encoding="utf-8")
        self.assertTrue(docs_policy.valid_inventory_descriptor(source))
        descriptor = json.loads(source)
        for key, value in (("status", "current"), ("current_output", "docs/roadmap/rrflow-1.0-file-plan.jsonl"), ("previous_git_blob", "invalid")):
            with self.subTest(key=key):
                bad = descriptor | {key: value}
                self.assertFalse(docs_policy.valid_inventory_descriptor(json.dumps(bad)))
        self.assertFalse(docs_policy.valid_inventory_descriptor("not-json"))
        self.assertFalse(docs_policy.valid_inventory_descriptor(json.dumps(descriptor | {"unexpected": True})))
        prior = subprocess.check_output(["git", "rev-parse", "1f8fa02cf9df74ad5dab5b7481c49f68eeca96af:docs/roadmap/rrflow-1.0-file-plan.jsonl"], cwd=root, text=True).strip()
        self.assertEqual(descriptor["previous_git_blob"], prior)


class DefaultOutputPolicyTests(unittest.TestCase):
    def test_default_is_local_ignored_artifact(self) -> None:
        self.assertEqual(inventory.OUTPUT, inventory.ROOT / "target/rrflow-1.0-file-plan.jsonl")


class ConflictFreeInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.git("init", "--quiet")
        (self.root / ".gitignore").write_text("/target/\n", encoding="utf-8")
        (self.root / "README.md").write_text("baseline\n", encoding="utf-8")
        descriptor = self.root / "docs/roadmap/rrflow-1.0-file-plan.jsonl"
        descriptor.parent.mkdir(parents=True)
        descriptor.write_text('{"status":"retired"}\n', encoding="utf-8")
        self.output = self.root / "target/rrflow-1.0-file-plan.jsonl"
        self.output.parent.mkdir()
        roadmap = self.output.parent / "roadmap-fixture.md"
        roadmap.write_text(inventory.ROADMAP.read_text(encoding="utf-8").replace("| [ ] |", "| [x] |"), encoding="utf-8")
        for name, value in (("ROOT", self.root), ("OUTPUT", self.output), ("ROADMAP", roadmap), ("PLANNED_PATHS", {})):
            patcher = patch.object(inventory, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)
        self.git("add", ".gitignore", "README.md", "docs/roadmap/rrflow-1.0-file-plan.jsonl")

    def git(self, *args: str) -> str:
        return subprocess.run(["git", *args], cwd=self.root, check=True, capture_output=True, text=True).stdout.strip()

    def command(self, *args: str) -> int:
        with patch.object(sys, "argv", ["build_execution_inventory.py", *args]), contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return inventory.main()

    def commit(self, message: str) -> None:
        self.git("-c", "user.name=Inventory Test", "-c", "user.email=inventory@example.invalid", "commit", "--quiet", "-m", message)

    def test_default_output_is_ignored_local_artifact(self) -> None:
        self.assertEqual(inventory.OUTPUT, self.output)
        self.assertNotIn("target/rrflow-1.0-file-plan.jsonl", inventory.git_files())
        self.assertEqual(self.git("check-ignore", "target/rrflow-1.0-file-plan.jsonl"), "target/rrflow-1.0-file-plan.jsonl")

    def test_absent_stale_and_tampered_output_fail_then_regenerate(self) -> None:
        self.assertEqual(self.command("--check"), 1)
        self.assertEqual(self.command(), 0)
        self.assertEqual(self.command("--check"), 0)
        records = [json.loads(line) for line in self.output.read_text(encoding="utf-8").splitlines()]
        actual = {row["path"] for row in records if row["state"] in {"current", "generated"}}
        expected = set(self.git("ls-files").splitlines())
        self.assertEqual(actual, expected)
        row = next(row for row in records if row["path"] == "README.md")
        self.assertEqual(row["sha256"], hashlib.sha256((self.root / "README.md").read_bytes()).hexdigest())
        row["sha256"] = "0" * 64
        self.output.write_text("".join(json.dumps(item, sort_keys=True, separators=(",", ":")) + "\n" for item in records), encoding="utf-8", newline="")
        self.assertEqual(self.command("--check"), 1)
        self.assertEqual(self.command(), 0)
        self.output.write_text("tampered\n", encoding="utf-8")
        self.assertEqual(self.command("--check"), 1)
        self.assertEqual(self.command(), 0)
        (self.root / "untracked.txt").write_text("new path\n", encoding="utf-8")
        self.assertEqual(self.command("--check"), 1)
        self.assertEqual(self.command(), 0)
        records = [json.loads(line) for line in self.output.read_text(encoding="utf-8").splitlines()]
        self.assertIn("untracked.txt", {row["path"] for row in records})
        (self.root / "README.md").write_text("changed\n", encoding="utf-8")
        self.assertEqual(self.command("--check"), 1)
        self.assertEqual(self.command(), 0)
        self.assertEqual(self.command("--check"), 0)

    def test_independent_branches_merge_without_inventory_conflict(self) -> None:
        self.commit("baseline")
        base = self.git("rev-parse", "HEAD")
        self.git("checkout", "--quiet", "-b", "one")
        (self.root / "one.txt").write_text("one\n", encoding="utf-8")
        self.git("add", "one.txt")
        self.assertEqual(self.command(), 0)
        self.assertNotIn("rrflow-1.0-file-plan.jsonl", self.git("diff", "--cached", "--name-only"))
        self.commit("one")
        self.git("checkout", "--quiet", "-b", "two", base)
        (self.root / "two.txt").write_text("two\n", encoding="utf-8")
        self.git("add", "two.txt")
        self.assertEqual(self.command(), 0)
        self.assertNotIn("rrflow-1.0-file-plan.jsonl", self.git("diff", "--cached", "--name-only"))
        self.commit("two")
        self.git("-c", "user.name=Inventory Test", "-c", "user.email=inventory@example.invalid", "merge", "--quiet", "--no-edit", "one")
        self.assertEqual(self.command(), 0)
        self.assertEqual(self.command("--check"), 0)
        paths = {json.loads(line)["path"] for line in self.output.read_text(encoding="utf-8").splitlines()}
        self.assertTrue({"one.txt", "two.txt"}.issubset(paths))


if __name__ == "__main__":
    unittest.main()
