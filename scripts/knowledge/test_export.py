#!/usr/bin/env python3
"""Focused KB-03/KB-04 tests for deterministic knowledge export and CI drift."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPORT_PATH = Path(__file__).with_name("export.py")
SPEC = importlib.util.spec_from_file_location("rrflow_knowledge_export", EXPORT_PATH)
assert SPEC is not None and SPEC.loader is not None
EXPORT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EXPORT)
CHECK_PATH = ROOT / "scripts/ci/check_documentation.py"
CHECK_SPEC = importlib.util.spec_from_file_location(
    "rrflow_documentation_policy", CHECK_PATH
)
assert CHECK_SPEC is not None and CHECK_SPEC.loader is not None
CHECK = importlib.util.module_from_spec(CHECK_SPEC)
CHECK_SPEC.loader.exec_module(CHECK)

REVISION = "0123456789abcdef0123456789abcdef01234567"


def write(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8", newline="")


def fixture_repository(root: Path) -> None:
    write(root / "README.md", "# Example product\n\nSee [memory](docs/).\n")
    write(
        root / "docs/README.md",
        """# Knowledge index

**Status:** active documentation index
**Coordinate:** `rrflow://example/data/documentation-index/knowledge`
**Owner:** bootstrap memory

See [architecture](architecture/).
""",
    )
    write(
        root / "docs/architecture/README.md",
        """# Architecture index

**Status:** active architecture index
**Coordinate:** `rrflow://example/data/architecture-index/architecture`
**Owner:** architecture discovery

See [flow](flow.md).
""",
    )
    write(
        root / "docs/architecture/flow.md",
        """# Flow

**Status:** active accepted architecture
**Coordinate:** `rrflow://example/data/architecture/flow`
**Owner:** flow semantics

One engine owns the transaction.
""",
    )
    write(
        root / "docs/unclassified.md",
        "# Draft\n\nStatus: supporting unclassified note\n",
    )


class KnowledgeExportTests(unittest.TestCase):
    def test_python_digest_matches_the_rust_kb02_golden(self) -> None:
        fixture = json.loads(
            (
                ROOT
                / "crates/transport/rrd-contract/fixtures/knowledge-package-v1.json"
            ).read_text(encoding="utf-8")
        )
        package = fixture["package"]
        for record in package["records"]:
            self.assertEqual(EXPORT.record_digest(record), record["record_sha256"])
        self.assertEqual(EXPORT.package_digest(package), package["package_sha256"])

    def test_two_exports_are_identical_complete_ordered_and_non_mutating(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "repository"
            first_output = Path(directory) / "first/package.jsonl"
            second_output = Path(directory) / "second/package.jsonl"
            fixture_repository(root)
            before = {
                path.relative_to(root): path.read_bytes()
                for path in sorted(root.rglob("*.md"))
            }

            first = EXPORT.build_package(root, repository="example", revision=REVISION)
            second = EXPORT.build_package(root, repository="example", revision=REVISION)
            EXPORT.write_package(root, first_output, first)
            EXPORT.write_package(root, second_output, second)

            self.assertEqual(first_output.read_bytes(), second_output.read_bytes())
            self.assertTrue(first_output.read_bytes().endswith(b"\n"))
            decoded = json.loads(first_output.read_text(encoding="utf-8"))
            self.assertEqual(decoded["package_sha256"], EXPORT.package_digest(decoded))
            self.assertEqual(len(decoded["records"]), 3)
            self.assertEqual(len(decoded["exclusions"]), 2)
            paths = [entry["source_path"] for entry in decoded["manifest"]]
            self.assertEqual(paths, sorted(paths, key=str.encode))
            self.assertEqual(len(paths), len(set(paths)))
            self.assertEqual(
                set(paths),
                {path.as_posix() for path in before},
            )
            self.assertEqual(
                [
                    (record["coordinate"], record["source_path"])
                    for record in decoded["records"]
                ],
                sorted(
                    (
                        (record["coordinate"], record["source_path"])
                        for record in decoded["records"]
                    ),
                    key=lambda value: (value[0].encode(), value[1].encode()),
                ),
            )
            after = {
                path.relative_to(root): path.read_bytes()
                for path in sorted(root.rglob("*.md"))
            }
            self.assertEqual(before, after)

    def test_current_repository_exports_twice_with_complete_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            first = EXPORT.build_package(ROOT, repository="rrflow", revision=REVISION)
            second = EXPORT.build_package(ROOT, repository="rrflow", revision=REVISION)
            first_output = Path(directory) / "first.jsonl"
            second_output = Path(directory) / "second.jsonl"
            EXPORT.write_package(ROOT, first_output, first)
            EXPORT.write_package(ROOT, second_output, second)

            self.assertEqual(first_output.read_bytes(), second_output.read_bytes())
            discovered = {
                path.relative_to(ROOT).as_posix() for path in EXPORT._discover(ROOT)
            }
            manifested = {entry["source_path"] for entry in first["manifest"]}
            self.assertEqual(manifested, discovered)
            self.assertEqual(
                len(first["manifest"]),
                len(first["records"]) + len(first["exclusions"]),
            )
            self.assertEqual(CHECK.knowledge_package_drift_failures(ROOT), [])

    def test_duplicate_coordinate_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_repository(root)
            body = (root / "docs/architecture/flow.md").read_text(encoding="utf-8")
            write(root / "docs/architecture/duplicate.md", body)
            index = root / "docs/architecture/README.md"
            write(
                index,
                index.read_text(encoding="utf-8") + "See [duplicate](duplicate.md).\n",
            )
            with self.assertRaisesRegex(EXPORT.ExportError, "duplicate Coordinate"):
                EXPORT.build_package(root, repository="example", revision=REVISION)

    def test_missing_owner_link_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_repository(root)
            index = root / "docs/architecture/README.md"
            write(
                index,
                index.read_text(encoding="utf-8").replace("[flow](flow.md)", "flow"),
            )
            with self.assertRaisesRegex(EXPORT.ExportError, "does not link the record"):
                EXPORT.build_package(root, repository="example", revision=REVISION)

    def test_ci_policy_rejects_unclassified_eligible_record(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_repository(root)
            write(
                root / "docs/architecture/unclassified.md",
                """# Unclassified architecture

**Status:** active architecture record
**Owner:** architecture behavior

This active record has not received its stable coordinate.
""",
            )
            failures = CHECK.knowledge_package_drift_failures(root)
            self.assertTrue(
                any(
                    "active or historical record is unclassified" in failure
                    for failure in failures
                ),
                failures,
            )

    def test_ci_policy_rejects_changed_body_without_digest_change(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_repository(root)
            package = EXPORT.build_package(
                root, repository="example", revision=REVISION
            )
            package["records"][0]["body"] += "Undigested change.\n"
            failures = CHECK.knowledge_package_integrity_failures(package)
            self.assertTrue(
                any(
                    "body changed without a matching body digest" in failure
                    for failure in failures
                ),
                failures,
            )

    def test_ci_policy_rejects_unstable_record_order(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_repository(root)
            package = EXPORT.build_package(
                root, repository="example", revision=REVISION
            )
            package["records"].reverse()
            failures = CHECK.knowledge_package_integrity_failures(package)
            self.assertTrue(
                any("records are not ordered" in failure for failure in failures),
                failures,
            )

    def test_ci_policy_rejects_missing_exclusion(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_repository(root)
            package = EXPORT.build_package(
                root, repository="example", revision=REVISION
            )
            package["exclusions"].pop()
            discovered = {
                path.relative_to(root).as_posix() for path in EXPORT._discover(root)
            }
            failures = CHECK.knowledge_package_integrity_failures(package, discovered)
            self.assertTrue(
                any("has no ledger entry" in failure for failure in failures),
                failures,
            )

    def test_non_utf8_source_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_repository(root)
            (root / "docs/binary.md").write_bytes(b"\xff\xfe")
            with self.assertRaisesRegex(EXPORT.ExportError, "source is not UTF-8"):
                EXPORT.build_package(root, repository="example", revision=REVISION)

    def test_tracked_output_path_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_repository(root)
            package = EXPORT.build_package(
                root, repository="example", revision=REVISION
            )
            with self.assertRaisesRegex(EXPORT.ExportError, "must be ignored by Git"):
                EXPORT.write_package(root, root / "package.jsonl", package)


if __name__ == "__main__":
    unittest.main()
