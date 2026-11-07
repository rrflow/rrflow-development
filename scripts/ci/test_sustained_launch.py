#!/usr/bin/env python3
"""Pure outer-receipt tests: no process allocation, engine or soak is launched."""
import json
from pathlib import Path
import tempfile
import unittest
from run_sustained_capacity import NotRun, retain_launch
from sustained_progress import MAX_READ_BYTES, ProgressError, ProgressObserver, ProgressReader, cgroup_counters


class ProgressTests(unittest.TestCase):
    def test_incremental_partial_unknown_and_sensitive_values_are_not_copied(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "run"
            reader = ProgressReader(root)
            self.assertEqual(reader.poll(), (False, 0, False))
            root.mkdir()
            source = root / "progress.jsonl"
            source.write_bytes(b'{"kind":"corpus_acknowledged","secret":"private payload"}\n{"kind":"future')
            self.assertEqual(reader.poll(), (True, 0, True))
            with source.open("ab") as output:
                output.write(b'_event","secret":"private payload"}\n')
            self.assertEqual(reader.poll(), (True, 0, False))
            self.assertEqual(dict(reader.counts), {"corpus_acknowledged": 1, "other": 1})
            self.assertEqual(reader.poll(), (True, 0, False))
            self.assertNotIn("private payload", str(reader.counts))

    def test_truncation_and_replacement_are_rejected(self):
        for replacement in (False, True):
            with self.subTest(replacement=replacement), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary) / "run"
                root.mkdir()
                source = root / "progress.jsonl"
                source.write_bytes(b'{"kind":"campaign"}\n')
                reader = ProgressReader(root)
                reader.poll()
                if replacement:
                    moved = root / "old.jsonl"
                    source.rename(moved)
                    source.write_bytes(b'{"kind":"campaign"}\n')
                else:
                    source.write_bytes(b'')
                with self.assertRaises(ProgressError):
                    reader.poll()

    def test_bounded_read_reports_backlog_and_catches_up_without_double_counting(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "run"
            root.mkdir()
            record = b'{"kind":"campaign","padding":"' + b'x' * 1000 + b'"}\n'
            count = MAX_READ_BYTES // len(record) + 10
            (root / "progress.jsonl").write_bytes(record * count)
            reader = ProgressReader(root)
            available, backlog, _ = reader.poll()
            self.assertTrue(available)
            self.assertGreater(backlog, 0)
            self.assertEqual(reader.offset, MAX_READ_BYTES)
            self.assertEqual(reader.poll(), (True, 0, False))
            self.assertEqual(reader.counts["campaign"], count)

    def test_malformed_input_reports_only_a_fixed_error(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "run"
            root.mkdir()
            (root / "progress.jsonl").write_text('private-payload\n', encoding="utf-8")
            observer = ProgressObserver(root, None, 1000, 4096)
            with observer:
                observer.thread.join(timeout=5)
            self.assertEqual(observer.error, "progress_record_invalid")
            self.assertNotIn("private-payload", observer.output_path.read_text(encoding="utf-8"))

    def test_observer_stops_on_exception_and_preserves_canonical_receipts(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "run"
            root.mkdir()
            (root / "progress.jsonl").write_text('{"kind":"campaign","secret":"hidden"}\n', encoding="utf-8")
            observer = ProgressObserver(root, None, 1000, 4096)
            self.assertIsNone(observer.started)
            with self.assertRaises(KeyboardInterrupt):
                with observer:
                    self.assertIsNotNone(observer.started)
                    raise KeyboardInterrupt()
            self.assertFalse(observer.thread.is_alive())
            rows = [json.loads(row) for row in observer.output_path.read_text(encoding="utf-8").splitlines()]
            self.assertTrue(rows[-1]["final"])
            self.assertTrue(rows[-1]["source_available"])
            self.assertEqual(rows[-1]["counts"], {"campaign": 1})
            self.assertNotIn("hidden", observer.output_path.read_text(encoding="utf-8"))
            self.assertFalse((root / "result.json").exists())

    def test_limits_exclusive_output_and_unavailable_cgroup(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "run"
            with self.assertRaises(ProgressError):
                ProgressObserver(root, None, 999, 4096)
            with self.assertRaises(ProgressError):
                ProgressObserver(root, None, 1000, 4095)
            observer = ProgressObserver(root, None, 1000, 4096)
            observer.output_path.write_text("existing", encoding="utf-8")
            with self.assertRaises(FileExistsError):
                with observer:
                    pass
            self.assertEqual(observer.output_path.read_text(), "existing")
            self.assertEqual(cgroup_counters(None), {"status": "unavailable"})

    def test_output_exhaustion_is_bounded_and_does_not_escape_context(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "run"
            root.mkdir()
            (root / "progress.jsonl").write_text('{"kind":"campaign"}\n', encoding="utf-8")
            observer = ProgressObserver(root, None, 1000, 4096)
            with observer:
                observer.max_bytes = 1
                observer.stop_event.set()
                observer.thread.join(timeout=5)
            self.assertEqual(observer.error, "observer_output_limit")
            self.assertLessEqual(observer.output_path.stat().st_size, 4096)


class LaunchTests(unittest.TestCase):
    def test_failure_interruption_and_unavailable_preserve_unsealed_evidence(self):
        for error, status in [(RuntimeError("cleanup failure"), "failed"),
                              (KeyboardInterrupt(), "interrupted"), (NotRun("no delegated controller"), "not_run")]:
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary) / "run"
                with self.assertRaises(type(error)):
                    with retain_launch(root, {"frozen": True}, "a" * 64):
                        raise error
                receipt = json.loads((root / "launch.json").read_text())
                self.assertEqual(receipt["status"], status)
                self.assertFalse(receipt["cleanup_confirmed"])
                self.assertIsNone(receipt["exit_code"])
                self.assertFalse((root / "result.json").exists())
                self.assertEqual(json.loads((root / "campaign.json").read_text()), {"frozen": True})

    def test_inner_success_does_not_override_outer_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "run"
            with self.assertRaises(RuntimeError):
                with retain_launch(root, {"frozen": True}, "a" * 64) as state:
                    root.mkdir()
                    (root / "result.json").write_text('{"status":"completed"}')
                    state["exit_code"] = 0
                    raise RuntimeError("native cleanup failed")
            receipt = json.loads((root / "launch.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertEqual(receipt["reason"], "native-cleanup-unconfirmed")
            self.assertEqual(json.loads((root / "result.json").read_text())["status"], "completed")

    def test_final_receipt_is_exclusive_and_preserves_existing_campaign(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "run"
            root.mkdir()
            (root / "campaign.json").write_text('{"retained":true}')
            with retain_launch(root, {"frozen": True}, "a" * 64) as state:
                state.update(status="completed", exit_code=0, cleanup_confirmed=True, reason=None)
            before = (root / "launch.json").read_bytes()
            self.assertEqual(json.loads(before)["status"], "completed")
            with self.assertRaises(FileExistsError):
                with retain_launch(root, {}, "b" * 64):
                    pass
            self.assertEqual((root / "launch.json").read_bytes(), before)
            self.assertEqual((root / "campaign.json").read_text(), '{"retained":true}')


if __name__ == "__main__":
    unittest.main()
