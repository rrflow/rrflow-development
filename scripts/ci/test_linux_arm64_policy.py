#!/usr/bin/env python3
"""Regression checks for the native Linux ARM64 qualification boundary."""

import unittest

from check_workflow import CALLER, REUSABLE, PolicyError, verify_linux_arm64_ci


class LinuxArm64PolicyTests(unittest.TestCase):
    def setUp(self):
        self.caller = CALLER.read_text(encoding="utf-8")
        self.reusable = REUSABLE.read_text(encoding="utf-8")

    def test_checked_in_native_arm64_lane_is_required(self):
        verify_linux_arm64_ci(self.caller, self.reusable)

    def test_wrong_or_missing_native_architecture_is_rejected(self):
        for old in [
            'test "$RUNNER_ARCH" = ARM64',
            'test "$(uname -m)" = aarch64',
            "host: aarch64-unknown-linux-gnu",
        ]:
            with self.subTest(old=old):
                self.assertIn(old, self.reusable)
                with self.assertRaises(PolicyError):
                    verify_linux_arm64_ci(self.caller, self.reusable.replace(old, "true", 1))

    def test_installed_lifecycle_cannot_be_removed(self):
        command = "cargo test -p rrflow-cli --test installed_lifecycle --locked -- --nocapture"
        marker = "  native-arm64:\n"
        before, after = self.reusable.split(marker, 1)
        self.assertIn(command, after)
        with self.assertRaises(PolicyError):
            verify_linux_arm64_ci(
                self.caller, before + marker + after.replace(command, "true", 1)
            )

    def test_missing_job_or_gate_dependency_is_rejected(self):
        for old in [
            "  native-arm64:\n",
            "      - native-arm64\n",
            "needs.native-arm64.result",
            'expect "$FULL_LANE" "$ARM64_RESULT" native-arm64',
        ]:
            with self.subTest(old=old):
                self.assertIn(old, self.reusable)
                with self.assertRaises(PolicyError):
                    verify_linux_arm64_ci(self.caller, self.reusable.replace(old, "", 1))

    def test_arm_job_or_test_cannot_be_skipped(self):
        marker = "  native-arm64:\n"
        before, after = self.reusable.split(marker, 1)
        for insertion in ["    if: false\n", "    continue-on-error: true\n",
                          "    if: ${{ needs.changes.outputs.docs == 'true' }}\n"]:
            with self.subTest(insertion=insertion):
                with self.assertRaises(PolicyError):
                    verify_linux_arm64_ci(self.caller, before + marker + insertion + after)

    def test_fork_pr_cannot_route_to_self_hosted_arm_runner(self):
        marker = "      linux-arm64-runner: >-\n"
        before, after = self.caller.split(marker, 1)
        trust = "github.event.pull_request.head.repo.full_name == github.repository"
        self.assertIn(trust, after)
        with self.assertRaises(PolicyError):
            verify_linux_arm64_ci(
                before + marker + after.replace(trust, "true", 1), self.reusable
            )


if __name__ == "__main__":
    unittest.main()
