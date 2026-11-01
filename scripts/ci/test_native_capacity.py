"""Capacity orchestration/cleanup checks; no native disk is created or filled."""
from contextlib import contextmanager, redirect_stdout, redirect_stderr
import errno
import io
import json
import os
import plistlib
import subprocess
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch

import run_native_capacity as runner
import run_filesystem_containment as containment
import check_workflow
from test_offline_lifecycle import end_holder, held_output_command


class NativeCapacityCases(unittest.TestCase):
    def native_child(self, command, verb):
        """Run a real child under an owned native command's identity."""
        real = runner.run

        def run(*arguments, **kwargs):
            self.assertEqual(arguments[:2], ("hdiutil", verb))
            return real(*command, **kwargs)
        return patch.object(runner, "run", run)

    def test_failed_create_with_held_output_retains_real_status_and_stderr(self):
        for held, stdout, stderr in [("stdout", True, False), ("stderr", False, True), ("both", True, True)]:
            with self.subTest(held=held), tempfile.TemporaryDirectory(prefix="rrflow-capacity-") as temporary:
                root = Path(temporary).resolve(strict=True)
                diagnostics = root / "rrflow-capacity-diagnostics"
                pid_file = root / "holder.pid"
                environment = {"RUNNER_TEMP": str(root), "RRFLOW_CAPACITY_DIAGNOSTICS": str(diagnostics)}
                begin = time.monotonic()
                try:
                    with patch.dict(os.environ, environment), patch.object(runner.sys, "platform", "darwin"), \
                            self.native_child(held_output_command(pid_file, stdout, stderr), "create"):
                        with self.assertRaises(runner.NativeCommandFailure) as caught:
                            runner.cases(self.config, root)
                finally:
                    end_holder(pid_file)
                self.assertLess(time.monotonic() - begin, 30, "the output-close wait must stay bounded")
                error = caught.exception
                self.assertEqual((error.tool, error.verb, error.returncode), ("hdiutil", "create", 1))
                self.assertIn("Resource busy", str(error))
                self.assertIsInstance(error.__cause__, runner.CommandOutputNotClosed)
                files = list(diagnostics.glob("*.json"))
                self.assertEqual(len(files), 1)
                receipt = json.loads(files[0].read_text(encoding="utf8"))
                self.assertEqual((receipt["tool"], receipt["verb"], receipt["returncode"]), ("hdiutil", "create", 1))
                self.assertEqual((receipt["mode"], receipt["point"], receipt["case_index"]),
                                 ("fresh", runner.POINTS[0], 1))
                self.assertIn("Resource busy", receipt["stderr_tail"])
                self.assertEqual((receipt["stdout_open"], receipt["stderr_open"]), (stdout, stderr))

    def test_failed_attach_with_held_output_reports_status_and_keeps_closed_receipt(self):
        pid_file = self.root / "holder.pid"
        diagnostics = self.root / "rrflow-capacity-diagnostics"
        environment = {"RUNNER_TEMP": str(self.root), "RRFLOW_CAPACITY_DIAGNOSTICS": str(diagnostics)}
        try:
            with self.native_child(held_output_command(pid_file, True, False), "attach"):
                with self.assertRaisesRegex(runner.NativeCommandFailure, "Resource busy") as caught:
                    runner.native_command("hdiutil", "attach", timeout=30)
        finally:
            end_holder(pid_file)
        with patch.dict(os.environ, environment):
            runner.retain_native_failure(self.config, self.root, "fresh", runner.POINTS[0], 1, caught.exception)
        raw = next(diagnostics.glob("*.json")).read_text(encoding="utf8")
        receipt = json.loads(raw)
        self.assertEqual(receipt["kind"], "command-output-not-closed")
        self.assertEqual(receipt["child_returncode"], 1)
        self.assertEqual((receipt["stdout_open"], receipt["stderr_open"]), (True, False))
        self.assertNotIn("Resource busy", raw)

    def test_successful_native_command_with_held_output_remains_a_closed_failure(self):
        pid_file = self.root / "holder.pid"
        try:
            with self.native_child(held_output_command(pid_file, True, False, 0, "partial native output"), "attach"):
                with self.assertRaises(runner.CommandOutputNotClosed) as caught:
                    runner.native_command("hdiutil", "attach", timeout=30)
        finally:
            end_holder(pid_file)
        self.assertEqual(caught.exception.child_returncode, 0)
        self.assertIsNone(caught.exception.captured)

    def test_native_deadline_with_held_output_stays_bounded_and_reports_status(self):
        pid_file = self.root / "holder.pid"
        try:
            with self.native_child(held_output_command(pid_file, True, False, status=None), "create"):
                with self.assertRaises(runner.NativeCommandFailure) as caught:
                    runner.native_command("hdiutil", "create", timeout=3)
        finally:
            end_holder(pid_file)
        self.assertNotEqual(caught.exception.returncode, 0)
        self.assertIn("Resource busy", caught.exception.stderr_tail)

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="rrflow-capacity-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve(strict=True)
        self.events = []
        self.generation = 0
        self.config = {"size_mib": 256, "modes": ["fresh", "existing"],
                       "points": list(runner.POINTS), "executable": "unused", "harness_sha256": "test"}

    def test_open_output_capture_retains_only_bounded_case_metadata(self):
        diagnostics = self.root / "rrflow-capacity-diagnostics"
        environment = {"RUNNER_TEMP": str(self.root), "RRFLOW_CAPACITY_DIAGNOSTICS": str(diagnostics),
                       "GITHUB_SHA": "abc", "GITHUB_RUN_ID": "42", "GITHUB_RUN_ATTEMPT": "2"}
        try:
            try:
                raise runner.CommandOutputNotClosed("hdiutil-attach", 0, True, False)
            except runner.CommandOutputNotClosed:
                raise RuntimeError("private cleanup path and product output")
        except RuntimeError as error:
            with patch.dict(os.environ, environment):
                runner.retain_native_failure(self.config, self.root, "existing", runner.POINTS[1], 22, error)
        files = list(diagnostics.glob("*.json"))
        self.assertEqual(len(files), 1)
        raw = files[0].read_text(encoding="utf8")
        receipt = json.loads(raw)
        self.assertEqual((receipt["mode"], receipt["point"], receipt["case_index"]),
                         ("existing", runner.POINTS[1], 22))
        self.assertEqual(receipt["command_category"], "hdiutil-attach")
        self.assertEqual(receipt["child_returncode"], 0)
        self.assertTrue(receipt["stdout_open"])
        self.assertFalse(receipt["stderr_open"])
        self.assertEqual(receipt["cleanup_state"], "unknown")
        self.assertNotIn("private", raw)
        self.assertNotIn(str(self.root), raw)

    def test_open_output_metadata_denies_untrusted_identifiers(self):
        diagnostics = self.root / "rrflow-capacity-diagnostics"
        environment = {"RUNNER_TEMP": str(self.root), "RRFLOW_CAPACITY_DIAGNOSTICS": str(diagnostics),
                       "GITHUB_SHA": "private-secret", "GITHUB_RUN_ID": "private-secret"}
        config = {"size_mib": "private-secret", "harness_sha256": "private-secret"}
        error = runner.CommandOutputNotClosed("private-secret", "private-secret", True, False)
        with patch.dict(os.environ, environment):
            runner.retain_native_failure(config, self.root, "private-secret", "private-secret", 22, error)
        raw = next(diagnostics.glob("*.json")).read_text(encoding="utf8")
        receipt = json.loads(raw)
        self.assertNotIn("private-secret", raw)
        self.assertEqual(receipt["command_category"], "other")
        self.assertEqual((receipt["mode"], receipt["point"]), ("unknown", "unknown"))
        self.assertIsNone(receipt["child_returncode"])

    @contextmanager
    def volume(self, fixture, size):
        self.assertEqual(fixture, self.root)
        self.assertEqual(size, 256)
        self.assertFalse((fixture / "capacity.img").exists(), "prior image was reused")
        self.generation += 1
        self.events.append(("attach", self.generation))
        (fixture / "capacity.img").write_bytes(b"owned image placeholder")
        (fixture / "volume").mkdir()
        descriptor = fixture / "volume.json"
        descriptor.write_text("{}", encoding="utf-8")
        try:
            yield descriptor, {"volume_uuid": str(self.generation)}
        finally:
            self.events.append(("detach", self.generation))

    def execute(self, executable, descriptor, point, mode):
        self.assertEqual(self.events[-1], ("attach", self.generation))
        self.assertTrue(descriptor.is_file())
        self.events.append(("execute", self.generation))
        return {"build": "test", "expected_sha256": f"{mode}:{point}"}

    def test_each_case_gets_a_new_image_and_identity_after_prior_cleanup(self):
        output = io.StringIO()
        with patch.object(runner, "volume", self.volume), patch.object(runner, "execute", self.execute), redirect_stdout(output):
            runner.cases(self.config, self.root)
        receipt = json.loads(output.getvalue().removeprefix("native_capacity_suite="))
        self.assertEqual(receipt["version"], 2)
        self.assertEqual(len(receipt["cases"]), 8)
        self.assertEqual([case["filesystem"]["volume_uuid"] for case in receipt["cases"]], list(map(str, range(1, 9))))
        self.assertEqual(self.events, [(event, index) for index in range(1, 9) for event in ("attach", "execute", "detach")])
        self.assertEqual(list(self.root.iterdir()), [])

    def test_failure_stops_later_cases_and_retains_failed_image(self):
        def fail(executable, descriptor, point, mode):
            if self.generation == 2:
                raise RuntimeError("selected native error missing")
            return self.execute(executable, descriptor, point, mode)
        with patch.object(runner, "volume", self.volume), patch.object(runner, "execute", fail), redirect_stdout(io.StringIO()) as output:
            with self.assertRaisesRegex(RuntimeError, "selected native error missing"):
                runner.cases(self.config, self.root)
        self.assertEqual(self.generation, 2)
        self.assertEqual(self.events[-1], ("detach", 2))
        self.assertTrue((self.root / "capacity.img").is_file())
        self.assertNotIn("native_capacity_suite=", output.getvalue())

    def test_cleanup_refuses_mounted_nonempty_or_invalid_artifacts_before_deletion(self):
        image = self.root / "capacity.img"
        image.write_bytes(b"preserve")
        mount = self.root / "volume"
        mount.mkdir()
        with patch.object(runner.os.path, "ismount", return_value=True):
            with self.assertRaisesRegex(Exception, "attached mount"):
                runner.cleanup_detached_volume(self.root)
        self.assertEqual(image.read_bytes(), b"preserve")
        marker = mount / "not-detached"
        marker.write_bytes(b"preserve")
        with self.assertRaisesRegex(Exception, "not empty"):
            runner.cleanup_detached_volume(self.root)
        self.assertEqual(image.read_bytes(), b"preserve")
        marker.unlink()
        (self.root / "volume.json").mkdir()
        with self.assertRaisesRegex(Exception, "escaped fixture"):
            runner.cleanup_detached_volume(self.root)
        self.assertEqual(image.read_bytes(), b"preserve")

    def test_detachment_failure_retains_image_and_prevents_next_case(self):
        @contextmanager
        def failed_detach(fixture, size):
            with self.volume(fixture, size) as owned:
                yield owned
            raise RuntimeError("native detachment failed")
        with patch.object(runner, "volume", failed_detach), patch.object(runner, "execute", self.execute), redirect_stdout(io.StringIO()) as output:
            with self.assertRaisesRegex(RuntimeError, "native detachment failed"):
                runner.cases(self.config, self.root)
        self.assertEqual(self.generation, 1)
        self.assertTrue((self.root / "capacity.img").is_file())
        self.assertNotIn("native_capacity_suite=", output.getvalue())

    def test_failed_native_create_retains_bounded_case_receipt_without_retry(self):
        @contextmanager
        def failed_create(fixture, size):
            self.generation += 1
            result = subprocess.CompletedProcess(["hdiutil", "create"], 1, "x" * 20000,
                                                 "verbose context\nhdiutil: create failed - Resource busy\n")
            raise runner.NativeCommandFailure("hdiutil", "create", result)
            yield  # pragma: no cover
        diagnostics = self.root / "rrflow-capacity-diagnostics"
        environment = {"RUNNER_TEMP": str(self.root), "RRFLOW_CAPACITY_DIAGNOSTICS": str(diagnostics),
                       "GITHUB_SHA": "abc", "GITHUB_RUN_ID": "42", "GITHUB_RUN_ATTEMPT": "2"}
        with patch.dict(os.environ, environment), patch.object(runner, "volume", failed_create):
            with self.assertRaisesRegex(runner.NativeCommandFailure, "Resource busy"):
                runner.cases(self.config, self.root)
        self.assertEqual(self.generation, 1)
        files = list(diagnostics.glob("*.json"))
        self.assertEqual(len(files), 1)
        receipt = json.loads(files[0].read_text(encoding="utf8"))
        self.assertEqual((receipt["mode"], receipt["point"]), ("fresh", runner.POINTS[0]))
        self.assertEqual(receipt["case_index"], 1)
        self.assertEqual((receipt["tool"], receipt["verb"], receipt["returncode"]), ("hdiutil", "create", 1))
        self.assertEqual(receipt["github_sha"], "abc")
        self.assertEqual(receipt["cleanup_state"], "unknown")
        self.assertEqual(len(receipt["stdout_head"]), 8192)
        self.assertEqual(len(receipt["stdout_tail"]), 8192)
        self.assertIn("Resource busy", receipt["stderr_tail"])

    def test_macos_volume_requests_verbose_creation_once_and_propagates_busy(self):
        result = subprocess.CompletedProcess([], 1, "native detail", "Resource busy")
        calls = []
        def native(*args, **kwargs):
            calls.append((args, kwargs))
            raise runner.NativeCommandFailure(args[0], args[1], result)
        with patch.object(runner.sys, "platform", "darwin"), patch.object(runner, "native_command", side_effect=native):
            with self.assertRaisesRegex(runner.NativeCommandFailure, "Resource busy"):
                with runner.volume(self.root, 256):
                    self.fail("failed creation must not yield a volume")
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][0][:3], ("hdiutil", "create", "-verbose"))
        self.assertFalse((self.root / "capacity.dmg").exists())

    def test_unrelated_error_does_not_save_product_output_and_bad_path_is_refused(self):
        diagnostics = self.root / "rrflow-capacity-diagnostics"
        env = {"RUNNER_TEMP": str(self.root), "RRFLOW_CAPACITY_DIAGNOSTICS": str(diagnostics)}
        with patch.dict(os.environ, env):
            runner.retain_native_failure(self.config, self.root, "fresh", runner.POINTS[0], 1, RuntimeError("private product output"))
        self.assertEqual(list(diagnostics.iterdir()), [])
        bad = self.root / "outside" / "rrflow-capacity-diagnostics"
        bad.parent.mkdir()
        with patch.dict(os.environ, {**env, "RRFLOW_CAPACITY_DIAGNOSTICS": str(bad)}):
            with self.assertRaisesRegex(Exception, "directly inside runner temp"):
                runner.retain_native_failure(self.config, self.root, "fresh", runner.POINTS[0], 1, RuntimeError("private"))

    def test_diagnostic_failure_does_not_replace_original_or_enable_later_case(self):
        @contextmanager
        def fail(fixture, size):
            self.generation += 1
            raise RuntimeError("original fixture failure")
            yield  # pragma: no cover
        with patch.object(runner, "volume", fail), patch.object(runner, "retain_native_failure", side_effect=RuntimeError("diagnostic unavailable")), redirect_stderr(io.StringIO()) as output:
            with self.assertRaisesRegex(RuntimeError, "original fixture failure"):
                runner.cases(self.config, self.root)
        self.assertEqual(self.generation, 1)
        self.assertIn("diagnostic retention failed", output.getvalue())

    def test_cleanup_error_preserves_preceding_native_create_diagnostic(self):
        diagnostics = self.root / "rrflow-capacity-diagnostics"
        env = {"RUNNER_TEMP": str(self.root), "RRFLOW_CAPACITY_DIAGNOSTICS": str(diagnostics)}
        try:
            try:
                raise runner.NativeCommandFailure("hdiutil", "create", subprocess.CompletedProcess([], 1, "", "Resource busy"))
            except runner.NativeCommandFailure:
                raise RuntimeError("cleanup failed")
        except RuntimeError as cleanup:
            with patch.dict(os.environ, env):
                runner.retain_native_failure(self.config, self.root, "existing", runner.POINTS[2], 7, cleanup)
        receipt = json.loads(next(diagnostics.glob("*.json")).read_text(encoding="utf8"))
        self.assertEqual(receipt["mode"], "existing")
        self.assertEqual(receipt["point"], runner.POINTS[2])
        self.assertEqual(receipt["case_index"], 7)
        self.assertEqual(receipt["cleanup_state"], "unknown")
        self.assertIn("Resource busy", receipt["stderr_tail"])

    def test_ci_policy_rejects_lost_diagnostics_or_native_repetitions(self):
        workflow = check_workflow.REUSABLE.read_text(encoding="utf8")
        job = check_workflow.job_blocks(workflow)["installed-crash"]
        check_workflow.verify_native_capacity_diagnostics(job)
        for altered in [
            job.replace("RRFLOW_CAPACITY_DIAGNOSTICS:", "REMOVED_DIAGNOSTICS:", 1),
            job.replace("if: always() && matrix.os == 'macos-latest'", "if: success()", 1),
            job.replace("rrflow-capacity-diagnostics/*.json", "**/*", 1),
            job.replace("--mode fresh --mode existing\n", "", 1),
        ]:
            with self.assertRaises(check_workflow.PolicyError):
                check_workflow.verify_native_capacity_diagnostics(altered)

    def test_manual_preflight_cannot_replace_candidate_or_hide_failures(self):
        workflow = check_workflow.NATIVE_PREFLIGHT.read_text(encoding="utf8")
        check_workflow.verify_native_capacity_preflight(workflow)
        for altered in [
            workflow.replace("  workflow_dispatch:\n", "  pull_request:\n", 1),
            workflow.replace("--mode fresh --mode existing\n", "", 1),
            workflow.replace("    runs-on: macos-latest", "    runs-on: ubuntu-latest", 1),
            workflow.replace("      - name: Bind exact revision", "      - continue-on-error: true\n      - name: Bind exact revision", 1),
            workflow.replace("          submodules: recursive", "          submodules: recursive\n          ref: main", 1),
            workflow.replace("python3 scripts/ci/macos_native_failure_preflight.py", "python3 omitted.py", 1),
            workflow.replace("rrflow-native-failure-probe/rrflow-capacity-diagnostics/*.json", "**/*", 1),
            workflow.replace("if-no-files-found: error", "if-no-files-found: ignore", 1),
            workflow.replace("      - name: Retain injected native failure receipt\n        if: always()", "      - name: Retain injected native failure receipt\n        if: always() && false", 1),
            workflow.replace("      - name: Retain native capacity create diagnostics\n        if: always()", "      - name: Retain native capacity create diagnostics\n        if: success()", 1),
        ]:
            with self.assertRaises(check_workflow.PolicyError):
                check_workflow.verify_native_capacity_preflight(altered)


class MountedFixtureCleanup(unittest.TestCase):
    def test_mounted_fixture_never_uses_recursive_removal(self):
        with tempfile.TemporaryDirectory(prefix="rrflow-mount-oracle-") as directory:
            base = Path(directory).resolve()
            fixture = base / "fixture"
            fixture.mkdir()
            executable = base / "filesystem-test"
            executable.write_text("stub", encoding="utf8")
            case = "mounted_volume_crossing_is_denied_before_any_effect"
            def fake_run(*args, **kwargs):
                if args[0] == "cargo":
                    artifact = {"reason": "compiler-artifact", "executable": str(executable),
                                "target": {"name": "filesystem"}, "profile": {"test": True}}
                    return subprocess.CompletedProcess(args, 0, json.dumps(artifact), "")
                if args[0] == str(executable) and "--list" in args:
                    return subprocess.CompletedProcess(args, 0, f"{case}: test\n", "")
                if args[0] == "/usr/bin/hdiutil":
                    (fixture / "fixture.sparseimage").write_bytes(b"owned")
                    return subprocess.CompletedProcess(args, 0, "", "")
                return subprocess.CompletedProcess(args, 0, f"test {case} ... ok\n1 passed; 0 failed; 0 ignored;\n", "")
            with patch.object(containment.sys, "platform", "darwin"), \
                 patch.object(containment.tempfile, "mkdtemp", return_value=str(fixture)), \
                 patch.object(containment, "run", side_effect=fake_run), \
                 patch.object(containment, "detach_owned_image"), \
                 patch("shutil.rmtree", side_effect=AssertionError("recursive removal called")), \
                 redirect_stdout(io.StringIO()):
                containment.main()
            self.assertFalse(fixture.exists())


    def fixture(self, directory):
        root = Path(directory).resolve() / "fixture"
        mount = root / "project" / "mounted"
        mount.mkdir(parents=True)
        image = root / "fixture.sparseimage"
        image.write_bytes(b"owned")
        return root, mount, image

    def test_known_native_pending_file_is_cleaned_after_mountpoint(self):
        with tempfile.TemporaryDirectory(prefix="rrflow-mount-pending-") as directory:
            root, mount, image = self.fixture(directory)
            pending = root / "project" / "pending"
            pending.write_bytes(b"owned")
            original = Path.rmdir
            def rmdir(path):
                if path == mount:
                    self.assertEqual(pending.read_bytes(), b"owned")
                    self.assertEqual(image.read_bytes(), b"owned")
                return original(path)
            with patch.object(Path, "rmdir", rmdir):
                containment.cleanup_owned_mount_fixture(root)
            self.assertFalse(root.exists())

    def test_busy_mount_settles_without_recursion(self):
        with tempfile.TemporaryDirectory(prefix="rrflow-mount-busy-") as directory:
            root, mount, image = self.fixture(directory)
            original = Path.rmdir
            attempts = []
            def rmdir(path):
                if path == mount:
                    attempts.append(path)
                    if len(attempts) <= 2:
                        raise OSError(errno.EBUSY, "still mounted")
                return original(path)
            with patch.object(Path, "rmdir", rmdir), patch.object(containment.time, "sleep") as sleep:
                containment.cleanup_owned_mount_fixture(root)
            self.assertEqual(len(attempts), 3)
            self.assertEqual([call.args[0] for call in sleep.call_args_list], [1, 2])
            self.assertFalse(root.exists())
            self.assertFalse(image.exists())

    def test_permanently_busy_mount_retains_fixture_and_image(self):
        with tempfile.TemporaryDirectory(prefix="rrflow-mount-permanent-") as directory:
            root, mount, image = self.fixture(directory)
            pending = root / "project" / "pending"
            pending.write_bytes(b"owned pending")
            attempts = []
            original = Path.rmdir
            def rmdir(path):
                if path == mount:
                    attempts.append(path)
                    raise OSError(errno.EBUSY, "still mounted")
                return original(path)
            with patch.object(Path, "rmdir", rmdir), patch.object(containment.time, "sleep"):
                with self.assertRaises(OSError) as result:
                    containment.cleanup_owned_mount_fixture(root)
            self.assertEqual(result.exception.errno, errno.EBUSY)
            self.assertEqual(len(attempts), 5)
            self.assertTrue(mount.is_dir())
            self.assertEqual(image.read_bytes(), b"owned")
            self.assertEqual(pending.read_bytes(), b"owned pending")

    def test_mount_settlement_deadline_retains_fixture(self):
        with tempfile.TemporaryDirectory(prefix="rrflow-mount-deadline-") as directory:
            root, mount, image = self.fixture(directory)
            original = Path.rmdir
            def rmdir(path):
                if path == mount:
                    raise OSError(errno.EBUSY, "still mounted")
                return original(path)
            with patch.object(Path, "rmdir", rmdir), \
                 patch.object(containment.time, "monotonic", side_effect=[0.0, 0.0, 31.0]), \
                 patch.object(containment.time, "sleep") as sleep:
                with self.assertRaisesRegex(RuntimeError, "cleanup deadline exceeded"):
                    containment.cleanup_owned_mount_fixture(root)
            sleep.assert_not_called()
            self.assertTrue(mount.is_dir())
            self.assertEqual(image.read_bytes(), b"owned")

    def test_nonempty_mount_never_loses_its_contents_or_image(self):
        with tempfile.TemporaryDirectory(prefix="rrflow-mount-nonempty-") as directory:
            root, mount, image = self.fixture(directory)
            sentinel = mount / "sentinel"
            sentinel.write_bytes(b"must survive")
            with patch.object(containment.time, "sleep"):
                with self.assertRaises(OSError) as result:
                    containment.cleanup_owned_mount_fixture(root)
            self.assertEqual(result.exception.errno, errno.ENOTEMPTY)
            self.assertEqual(sentinel.read_bytes(), b"must survive")
            self.assertEqual(image.read_bytes(), b"owned")

    def test_invalid_pending_entry_is_preserved(self):
        with tempfile.TemporaryDirectory(prefix="rrflow-mount-invalid-pending-") as directory:
            root, mount, image = self.fixture(directory)
            pending = root / "project" / "pending"
            pending.mkdir()
            with self.assertRaisesRegex(RuntimeError, "invalid pending file"):
                containment.cleanup_owned_mount_fixture(root)
            self.assertTrue(mount.is_dir())
            self.assertTrue(pending.is_dir())
            self.assertEqual(image.read_bytes(), b"owned")

    def test_unexpected_entry_is_preserved_before_any_deletion(self):
        with tempfile.TemporaryDirectory(prefix="rrflow-mount-unexpected-") as directory:
            root, mount, image = self.fixture(directory)
            unexpected = root / "unknown"
            unexpected.write_bytes(b"preserve")
            with self.assertRaisesRegex(RuntimeError, "unexpected fixture entries"):
                containment.cleanup_owned_mount_fixture(root)
            self.assertTrue(mount.is_dir())
            self.assertEqual(image.read_bytes(), b"owned")
            self.assertEqual(unexpected.read_bytes(), b"preserve")

    def test_captured_test_output_precedes_cleanup_failure(self):
        with tempfile.TemporaryDirectory(prefix="rrflow-mount-output-") as directory:
            base = Path(directory).resolve()
            fixture = base / "fixture"
            fixture.mkdir()
            executable = base / "filesystem-test"
            executable.write_text("stub", encoding="utf8")
            case = "mounted_volume_crossing_is_denied_before_any_effect"
            def fake_run(*args, **kwargs):
                if args[0] == "cargo":
                    artifact = {"reason": "compiler-artifact", "executable": str(executable),
                                "target": {"name": "filesystem"}, "profile": {"test": True}}
                    return subprocess.CompletedProcess(args, 0, json.dumps(artifact), "")
                if args[0] == str(executable) and "--list" in args:
                    return subprocess.CompletedProcess(args, 0, f"{case}: test\n", "")
                if args[0] == "/usr/bin/hdiutil":
                    (fixture / "fixture.sparseimage").write_bytes(b"owned")
                    return subprocess.CompletedProcess(args, 0, "", "")
                return subprocess.CompletedProcess(args, 1, "native test stdout\n", "native test stderr\n")
            stdout, stderr = io.StringIO(), io.StringIO()
            with patch.object(containment.sys, "platform", "darwin"), \
                 patch.object(containment.tempfile, "mkdtemp", return_value=str(fixture)), \
                 patch.object(containment, "run", side_effect=fake_run), \
                 patch.object(containment, "detach_owned_image"), \
                 patch.object(containment, "cleanup_owned_mount_fixture", side_effect=RuntimeError("cleanup failed")), \
                 redirect_stdout(stdout), redirect_stderr(stderr):
                with self.assertRaisesRegex(RuntimeError, "cleanup failed"):
                    containment.main()
            self.assertIn("native test stdout", stdout.getvalue())
            self.assertIn("native test stderr", stderr.getvalue())
            self.assertTrue((fixture / "fixture.sparseimage").exists())


class OwnedImageDetachment(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="rrflow-detach-")
        self.addCleanup(self.temp.cleanup)
        self.image = Path(self.temp.name).resolve() / "capacity.dmg"
        self.image.write_bytes(b"owned fixture")
        self.clock = 0.0
        self.calls = []
        self.sleeps = []

    def info(self, devices=("/dev/disk7",), duplicate=False):
        rows = [{"image-path": str(self.image), "system-entities": [
            {"dev-entry": device} for device in devices]}] if devices else []
        if duplicate:
            rows *= 2
        return subprocess.CompletedProcess([], 0, plistlib.dumps({"images": rows}).decode(), "")

    def detach(self, code=0):
        return subprocess.CompletedProcess([], code, "", "native busy" if code == 16 else "native failure")

    def invoke(self, responses, advance=0.1):
        if callable(responses):
            respond = responses
        else:
            iterator = iter(responses)
            def respond(*_):
                return next(iterator)
        def run(*args, **kwargs):
            self.calls.append(args)
            self.assertTrue(kwargs["process_group"])
            self.assertFalse(kwargs["check"])
            self.assertGreater(kwargs["timeout"], 0)
            self.assertLessEqual(kwargs["timeout"], 30)
            self.clock += advance
            return respond(*args)
        def sleep(seconds):
            self.sleeps.append(seconds)
            self.clock += seconds
        with patch.object(containment, "run", side_effect=run), \
             patch.object(containment.time, "monotonic", side_effect=lambda: self.clock), \
             patch.object(containment.time, "sleep", side_effect=sleep):
            containment.detach_owned_image(self.image)

    def test_success_requires_verified_absence(self):
        self.invoke([self.info(), self.detach(), self.info(())])
        self.assertEqual([x[1] for x in self.calls], ["info", "detach", "info"])
        self.assertEqual(self.image.read_bytes(), b"owned fixture")

    def test_busy_rechecks_ownership_before_retry(self):
        self.invoke([self.info(), self.detach(16), self.info(), self.detach(), self.info(())])
        self.assertEqual([x[1] for x in self.calls], ["info", "detach", "info", "detach", "info"])
        self.assertEqual([x[-1] for x in self.calls if x[1] == "detach"], ["/dev/disk7"] * 2)

    def test_busy_response_can_be_resolved_by_confirmed_absence(self):
        self.invoke([self.info(), self.detach(16), self.info(())])
        self.assertEqual(len(self.calls), 3)

    def test_busy_attachment_can_settle_within_existing_deadline(self):
        def response(*args):
            if args[1] == "info":
                return self.info(()) if self.clock >= 2.5 else self.info()
            self.assertEqual(args[1:], ("detach", "-force", "/dev/disk7"))
            return self.detach(16)
        self.invoke(response)
        self.assertEqual(self.sleeps, [1, 2])
        self.assertEqual(sum(x[1] == "detach" for x in self.calls), 2)
        self.assertLess(self.clock, 30)
        self.assertTrue(self.image.exists(), "settlement never authorizes image deletion here")

    def test_nonbusy_failure_keeps_native_diagnostics_and_never_retries(self):
        with self.assertRaisesRegex(RuntimeError, r"hdiutil detach \(5\): native failure"):
            self.invoke([self.info(), self.detach(5)])
        self.assertEqual(len(self.calls), 2)
        self.assertTrue(self.image.exists())

    def test_changed_or_ambiguous_device_is_never_detached(self):
        with self.assertRaisesRegex(RuntimeError, "identity changed"):
            self.invoke([self.info(), self.detach(16), self.info(("/dev/disk9",))])
        self.assertEqual(len(self.calls), 3)
        self.calls.clear()
        with self.assertRaisesRegex(RuntimeError, "ambiguous"):
            self.invoke([self.info(duplicate=True)])
        self.assertEqual(len(self.calls), 1)

    def test_busy_retries_are_finite_and_keep_fixture(self):
        with self.assertRaisesRegex(RuntimeError, "bounded cleanup.*native busy"):
            self.invoke([item for _ in range(5) for item in (self.info(), self.detach(16))] + [self.info()])
        self.assertEqual(sum(x[1] == "detach" for x in self.calls), 5)
        self.assertEqual(self.sleeps, [1, 2, 4, 8])
        self.assertLess(self.clock, 30)
        self.assertTrue(self.image.exists())

    def test_deadline_is_shared_across_all_commands(self):
        with self.assertRaisesRegex(RuntimeError, "cleanup deadline"):
            self.invoke([self.info(), self.detach(16), self.info()], advance=15)
        self.assertEqual(len(self.calls), 2)

    def test_success_with_attachment_still_present_is_denied(self):
        with self.assertRaisesRegex(RuntimeError, "remains attached"):
            self.invoke([self.info(), self.detach(), self.info()])
        self.assertEqual(len(self.calls), 3)
        self.assertTrue(self.image.exists())

    def test_missing_device_and_failed_inspection_never_detach(self):
        with self.assertRaisesRegex(RuntimeError, "whole-device identity"):
            self.invoke([self.info(("/dev/disk7s1",))])
        self.calls.clear()
        with self.assertRaisesRegex(RuntimeError, "inspection failed"):
            self.invoke([self.detach(5)])
        self.assertEqual(len(self.calls), 1)


if __name__ == "__main__":
    unittest.main()
