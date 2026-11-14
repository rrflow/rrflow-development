"""Model the macOS failure preflight without creating or mounting a disk."""
from contextlib import redirect_stdout
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import macos_native_failure_preflight as preflight
import run_native_capacity as runner


class NativeFailurePreflightTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()

    def test_failure_is_attributed_and_separate_from_capacity_artifacts(self):
        calls = []

        def fail(*args, **_kwargs):
            calls.append(args)
            raise runner.NativeCommandFailure("hdiutil", "create", subprocess.CompletedProcess(args, 1, "", "invalid fs"))

        with patch.dict(os.environ, {"RUNNER_TEMP": str(self.root), "GITHUB_SHA": "abc"}), \
                patch.object(runner.sys, "platform", "darwin"), patch.object(runner, "native_command", side_effect=fail), \
                redirect_stdout(io.StringIO()) as output:
            preflight.main()
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][calls[0].index("-fs") + 1], "RRFLOW_INVALID_FILESYSTEM")
        self.assertIn("capacity_pass=false", output.getvalue())
        probe = self.root / "rrflow-native-failure-probe"
        files = list((probe / "rrflow-capacity-diagnostics").glob("*.json"))
        self.assertEqual(len(files), 1)
        self.assertEqual(json.loads(files[0].read_text(encoding="utf-8"))["github_sha"], "abc")
        self.assertEqual(list(probe.glob("rrflow-capacity-*")), [probe / "rrflow-capacity-diagnostics"])
        self.assertFalse((self.root / "rrflow-capacity-diagnostics").exists())

    def test_unexpected_image_is_retained_and_not_reported_as_success(self):
        def fail(*args, **_kwargs):
            (Path(args[-1]).parent / "capacity.dmg").write_bytes(b"unexpected")
            raise runner.NativeCommandFailure("hdiutil", "create", subprocess.CompletedProcess(args, 1, "", "invalid fs"))

        with patch.dict(os.environ, {"RUNNER_TEMP": str(self.root), "GITHUB_SHA": "abc"}), \
                patch.object(runner.sys, "platform", "darwin"), patch.object(runner, "native_command", side_effect=fail), \
                patch.object(runner, "detach_owned_image"), redirect_stdout(io.StringIO()) as output:
            with self.assertRaisesRegex(RuntimeError, "unexpected native image"):
                preflight.main()
        self.assertNotIn("capacity_pass=false", output.getvalue())
        images = list((self.root / "rrflow-native-failure-probe").glob("rrflow-capacity-*/capacity.*"))
        self.assertEqual(len(images), 1)
        self.assertEqual(images[0].read_bytes(), b"unexpected")

    def test_non_macos_is_rejected_before_fixture_creation(self):
        with patch.dict(os.environ, {"RUNNER_TEMP": str(self.root)}), patch.object(runner.sys, "platform", "linux"):
            with self.assertRaisesRegex(RuntimeError, "requires macOS"):
                preflight.main()
        self.assertEqual(list(self.root.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
