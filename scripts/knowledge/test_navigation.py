#!/usr/bin/env python3
"""Focused tests for generated Markdown navigation and relationship discovery."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("render_navigation.py")
SPEC = importlib.util.spec_from_file_location("rrflow_navigation", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
NAVIGATION = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(NAVIGATION)


def write(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8", newline="")


def record(title: str, coordinate: str, body: str = "") -> str:
    return f"""# {title}

**Status:** active test record
**Coordinate:** `{coordinate}`
**Owner:** test ownership

{body}
"""


class NavigationTests(unittest.TestCase):
    def test_inline_marker_examples_do_not_opt_in(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            index = root / "docs/topic/README.md"
            original = record(
                "Topic index",
                "rrflow://example/data/topic-index/topic",
                "Use `<!-- rrflow:generated-index:start -->` and "
                "`<!-- rrflow:generated-index:end -->` on separate lines.",
            )
            write(index, original)
            write(
                root / "docs/topic/alpha.md",
                record("Alpha", "rrflow://example/data/topic/alpha"),
            )

            self.assertEqual(NAVIGATION.marked_indexes(root), [])
            self.assertEqual(NAVIGATION.write_indexes(root), [])
            self.assertEqual(index.read_text(encoding="utf-8"), original)

    def test_marked_region_is_generated_and_authored_prose_is_preserved(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            index = root / "docs/topic/README.md"
            write(
                index,
                record(
                    "Topic index",
                    "rrflow://example/data/topic-index/topic",
                    "Authored introduction.\n\n"
                    f"{NAVIGATION.INDEX_START}\nold\n{NAVIGATION.INDEX_END}\n\n"
                    "Authored conclusion.",
                ),
            )
            write(
                root / "docs/topic/alpha.md",
                record("Alpha", "rrflow://example/data/topic/alpha"),
            )

            self.assertEqual(len(NAVIGATION.write_indexes(root)), 1)
            rendered = index.read_text(encoding="utf-8")
            self.assertIn("Authored introduction.", rendered)
            self.assertIn("Authored conclusion.", rendered)
            self.assertIn(
                "[Alpha](alpha.md) — "
                "[`rrflow://example/data/topic/alpha`]"
                "(rrflow://example/data/topic/alpha)",
                rendered,
            )
            self.assertEqual(NAVIGATION.index_drift_failures(root), [])

    def test_new_child_causes_drift_until_regenerated(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            index = root / "docs/topic/README.md"
            write(
                index,
                record(
                    "Topic index",
                    "rrflow://example/data/topic-index/topic",
                    f"{NAVIGATION.INDEX_START}\nseed\n{NAVIGATION.INDEX_END}",
                ),
            )
            write(
                root / "docs/topic/a.md",
                record("A", "rrflow://example/data/topic/a"),
            )
            NAVIGATION.write_indexes(root)
            write(
                root / "docs/topic/b.md",
                record("B", "rrflow://example/data/topic/b"),
            )
            self.assertEqual(
                NAVIGATION.index_drift_failures(root),
                ["docs/topic/README.md: generated index is stale"],
            )
            NAVIGATION.write_indexes(root)
            self.assertEqual(NAVIGATION.index_drift_failures(root), [])

    def test_uncoordinated_draft_is_not_forced_into_generated_index(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            index = root / "docs/topic/README.md"
            write(
                index,
                record(
                    "Topic index",
                    "rrflow://example/data/topic-index/topic",
                    f"{NAVIGATION.INDEX_START}\nseed\n{NAVIGATION.INDEX_END}",
                ),
            )
            write(
                root / "docs/topic/alpha.md",
                record("Alpha", "rrflow://example/data/topic/alpha"),
            )
            write(
                root / "docs/topic/notes.md",
                "# Working notes\n\n**Status:** draft exploratory note\n",
            )

            NAVIGATION.write_indexes(root)
            rendered = index.read_text(encoding="utf-8")
            self.assertIn("[Alpha](alpha.md)", rendered)
            self.assertNotIn("notes.md", rendered)
            self.assertEqual(NAVIGATION.index_drift_failures(root), [])

    def test_relationship_graph_derives_local_and_coordinate_edges(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            index = root / "docs/topic/README.md"
            write(
                index,
                record(
                    "Topic index",
                    "rrflow://example/data/topic-index/topic",
                    f"{NAVIGATION.INDEX_START}\nseed\n{NAVIGATION.INDEX_END}",
                ),
            )
            write(
                root / "docs/topic/a.md",
                record(
                    "A",
                    "rrflow://example/data/topic/a",
                    "See [B](b.md#detail) and "
                    "[its warp](rrflow://example/data/topic/b).",
                ),
            )
            write(
                root / "docs/topic/b.md",
                record("B", "rrflow://example/data/topic/b", "## Detail"),
            )
            NAVIGATION.write_indexes(root)

            first = NAVIGATION.relationship_graph(root)
            second = NAVIGATION.relationship_graph(root)
            self.assertEqual(first, second)
            a_to_b = [
                edge
                for edge in first["edges"]
                if edge["source"] == "rrflow://example/data/topic/a"
                and edge["target"] == "rrflow://example/data/topic/b"
            ]
            self.assertEqual(len(a_to_b), 2)
            self.assertEqual({edge["fragment"] for edge in a_to_b}, {"detail", None})

    def test_duplicate_child_coordinate_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            index = root / "docs/topic/README.md"
            write(
                index,
                record(
                    "Topic index",
                    "rrflow://example/data/topic-index/topic",
                    f"{NAVIGATION.INDEX_START}\nseed\n{NAVIGATION.INDEX_END}",
                ),
            )
            duplicate = "rrflow://example/data/topic/duplicate"
            write(root / "docs/topic/a.md", record("A", duplicate))
            write(root / "docs/topic/b.md", record("B", duplicate))
            with self.assertRaisesRegex(
                NAVIGATION.NavigationError, "child Coordinate.*duplicated"
            ):
                NAVIGATION.write_indexes(root)


if __name__ == "__main__":
    unittest.main()
