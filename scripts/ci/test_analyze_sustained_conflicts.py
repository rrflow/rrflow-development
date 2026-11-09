from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import analyze_sustained_conflicts as subject
import check_workflow as ci_policy


def digest(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def attempt(name: str, start: int, end: int, outcome: str = "acknowledged") -> dict:
    return {
        "logical_id": name,
        "sequence": start,
        "kind": "mixed_write",
        "started_ns": start,
        "finished_ns": end,
        "outcome": outcome,
    }


def trace(name: str, read: int = 4, expected: int = 4, actual: int = 11) -> str:
    return f"c07_runtime_conflict_trace request_sha256={digest(name)} read_cursor={read} expected_cursor={expected} actual_cursor={actual}\n"


class AnalyzeSustainedConflicts(unittest.TestCase):
    def evidence(
        self, root: Path, attempts: list[dict], lines: str
    ) -> tuple[Path, Path]:
        segments = root / "evidence"
        segments.mkdir()
        (segments / "segment-00000000000000000000.json").write_text(
            json.dumps(
                {
                    "format_version": 1,
                    "campaign_sha256": "a" * 64,
                    "point": "fixture",
                    "seed": 7,
                    "index": 0,
                    "phase": "measured",
                    "attempts": attempts,
                }
            ),
            encoding="utf-8",
        )
        log = root / "stderr.log"
        log.write_text(lines, encoding="utf-8")
        return segments, log

    def test_correlates_overlaps_but_never_exposes_raw_logical_ids(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            segments, log = self.evidence(
                root,
                [
                    attempt("private-rejected", 10, 30, "rejected"),
                    attempt("private-accepted", 5, 25),
                    attempt("nonoverlap", 31, 45),
                ],
                trace("private-rejected"),
            )
            result = subject.analyze(segments, log)
            item = result["conflicts"][0]
            self.assertEqual(item["logical_id_sha256"], digest("private-rejected"))
            self.assertEqual(
                (item["read_cursor"], item["expected_cursor"], item["actual_cursor"]),
                (4, 4, 11),
            )
            self.assertEqual(item["overlap_count"], 1)
            self.assertEqual(
                item["overlaps"][0]["logical_id_sha256"], digest("private-accepted")
            )
            self.assertNotIn("private-", json.dumps(result))
            self.assertEqual(result["unmatched_trace_digests"], [])

    def test_no_overlap_and_missing_trace_are_explicit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            segments, log = self.evidence(
                root,
                [attempt("first", 10, 20, "rejected"), attempt("other", 21, 30)],
                trace("unknown"),
            )
            result = subject.analyze(segments, log)
            self.assertEqual(result["conflicts"], [])
            self.assertEqual(result["unmatched_trace_digests"], [digest("unknown")])
            self.assertEqual(result["rejected_without_trace"], 1)
            log.write_text(trace("first"), encoding="utf-8")
            self.assertEqual(
                subject.analyze(segments, log)["conflicts"][0]["overlap_count"], 0
            )

    def test_invalid_or_duplicate_traces_and_bad_attempts_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            segments, log = self.evidence(
                root, [attempt("a", 1, 2, "rejected")], trace("a") + trace("a")
            )
            with self.assertRaises(subject.AnalysisError):
                subject.analyze(segments, log)
            log.write_text(
                "c07_runtime_conflict_trace request_sha256=bad read_cursor=4 expected_cursor=4 actual_cursor=11\n",
                encoding="utf-8",
            )
            with self.assertRaises(subject.AnalysisError):
                subject.analyze(segments, log)
            log.write_text(trace("a"), encoding="utf-8")
            (segments / "segment-00000000000000000000.json").write_text(
                json.dumps(
                    {
                        "format_version": 1,
                        "campaign_sha256": "a" * 64,
                        "point": "fixture",
                        "seed": 7,
                        "index": 0,
                        "phase": "measured",
                        "attempts": [attempt("a", 3, 2, "rejected")],
                    }
                ),
                encoding="utf-8",
            )
            with self.assertRaises(subject.AnalysisError):
                subject.analyze(segments, log)

    def test_different_points_do_not_create_false_overlaps(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            segments, log = self.evidence(
                root, [attempt("a", 1, 5, "rejected")], trace("a")
            )
            (segments / "segment-00000000000000000001.json").write_text(
                json.dumps(
                    {
                        "format_version": 1,
                        "campaign_sha256": "a" * 64,
                        "point": "other",
                        "seed": 7,
                        "index": 1,
                        "phase": "measured",
                        "attempts": [attempt("b", 2, 4)],
                    }
                ),
                encoding="utf-8",
            )
            result = subject.analyze(segments, log)
            self.assertEqual(result["conflicts"][0]["overlap_count"], 0)
            self.assertNotIn("fixture", json.dumps(result))
            self.assertNotIn("other", json.dumps(result))

    def test_preserves_uncertain_outcome_without_inference(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            segments, log = self.evidence(
                root,
                [attempt("a", 1, 5, "rejected"), attempt("b", 2, 4, "uncertain")],
                trace("a"),
            )
            result = subject.analyze(segments, log)
            self.assertEqual(
                result["conflicts"][0]["overlaps"][0]["outcome"], "uncertain"
            )
            self.assertTrue(result["analysis_only"])
            self.assertIn("do not establish", result["limitation"])

    def test_overlap_output_is_bounded_and_reports_truncation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rows = [attempt("conflict", 1, 10, "rejected")]
            rows.extend(attempt(f"overlap-{index}", 2, 9) for index in range(70))
            segments, log = self.evidence(root, rows, trace("conflict"))
            item = subject.analyze(segments, log)["conflicts"][0]
            self.assertEqual(item["overlap_count"], 70)
            self.assertEqual(len(item["overlaps"]), subject.MAX_OVERLAPS_PER_CONFLICT)
            self.assertTrue(item["overlaps_truncated"])

    def test_candidate_policy_requires_executed_analysis_tests(self) -> None:
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/ci-reusable.yml").read_text(
            encoding="utf-8"
        )
        verify = ci_policy.job_blocks(workflow)["verify"]
        ci_policy.verify_execution_inventory_ci(verify)
        command = "          python3 scripts/ci/test_analyze_sustained_conflicts.py\n"
        with self.assertRaises(ci_policy.PolicyError):
            ci_policy.verify_execution_inventory_ci(verify.replace(command, "", 1))
        with self.assertRaises(ci_policy.PolicyError):
            ci_policy.verify_execution_inventory_ci(
                verify.replace(
                    command, command + "        continue-on-error: true\n", 1
                )
            )

    def test_file_and_attempt_bounds_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            segments, log = self.evidence(
                root, [attempt("a", 1, 2, "rejected")], trace("a")
            )
            with self.assertRaises(subject.AnalysisError):
                subject.analyze(segments, log, max_file_bytes=8)
            with self.assertRaises(subject.AnalysisError):
                subject.analyze(segments, log, max_attempts=0)


if __name__ == "__main__":
    unittest.main()
